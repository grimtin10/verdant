// TODO: optimize the case where someone loads the same image a bunch

#[cfg(feature = "image")]
use std::path::Path;
use std::{fmt::{self, Debug, Formatter}, sync::{Arc, Mutex, MutexGuard, RwLockReadGuard}};

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};

use crate::{GpuContext, RendererResult, errors::Error};
#[cfg(feature = "image")]
use crate::types::ByteSource;

/// A rectangular region defined by a position and size.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    /// An empty bounds at the origin.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Creates a new bounds with the given position and size.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Returns the smallest bounds that contains both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);

        Self {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImageData {
    pub(crate) bind_group: BindGroup,
    texture: Texture,
}

/// The parts of an [`Image`] that are shared between all of its clones.
///
/// Clones share the GPU texture as well as the pixels, so drawing a clone
/// (which canvases do for every draw batch) doesn't create and upload a new texture each time.
struct Shared {
    pixels: Vec<u8>,
    gpu: Option<Arc<ImageData>>,
    /// The region changed since the texture was last uploaded.
    /// Ignored until the texture is created, since creating it uploads the whole image.
    dirty_zone: Option<Bounds>,
}

#[derive(Clone)]
pub struct Image {
    width: u32,
    height: u32,

    shared: Arc<Mutex<Shared>>,
}

impl AsRef<Image> for Image {
    fn as_ref(&self) -> &Image {
        self
    }
}

impl AsRef<Image> for RwLockReadGuard<'_, Image> {
    fn as_ref(&self) -> &Image {
        self
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let shared = self.lock();
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("image", &format!("[{} bytes of image data]", shared.pixels.len()))
            .field("gpu", &shared.gpu)
            .field("dirty_zone", &shared.dirty_zone)
            .finish()
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl Image {
    /// Creates an image with the given data, width, and height.
    /// Expects data to be RGBA8.
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> RendererResult<Self> {
        let required_bytes = (width * height * 4) as usize;
        let actual_bytes = data.len();
        if actual_bytes < required_bytes {
            return Err(Error::ImageBufferTooSmall { expected: required_bytes, actual: actual_bytes });
        }

        Ok(Self::from_pixels(data, width, height))
    }

    fn from_pixels(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,

            shared: Arc::new(Mutex::new(Shared { pixels, gpu: None, dirty_zone: None })),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Creates a completely transparent image with the given width and height.
    pub fn new_empty(width: u32, height: u32) -> Self {
        Self::from_pixels(vec![0; (width * height * 4) as usize], width, height)
    }

    /// Loads an image from the given source (byte array or path to image file).
    #[cfg(feature = "image")]
    pub fn load(data: impl ByteSource) -> RendererResult<Self> {
        use image::load_from_memory;

        let bytes = data.load()?;

        let image = load_from_memory(&bytes)?.to_rgba8();
        let (width, height) = image.dimensions();

        Ok(Self::from_pixels(image.into_raw(), width, height))
    }

    #[cfg(feature = "image")]
    pub fn save(&self, path: impl AsRef<Path>) -> RendererResult<()> {
        use image::{DynamicImage, ImageBuffer, Rgba};

        Ok(
            DynamicImage::ImageRgba8(
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(self.width, self.height, self.lock().pixels.clone())
                    .expect("this should never happen")
            ).save(path)?
        )
    }

    /// Returns the width of this image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of this image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Copies raw RGBA8 pixel data into thie image at the given destination position.
    ///
    /// `src_data` must be at least `src_width * src_height * 4` bytes. The destination
    /// rectangle must fit entirely within the image bounds.
    pub fn blit(
        &mut self,
        dst_x: u32,
        dst_y: u32,
        src_width: u32,
        src_height: u32,
        src_data: &[u8],
    ) -> RendererResult<()> {
        self.check_blit(dst_x, dst_y, src_width, src_height, src_data)?;

        // images are values, so if clones still share our pixels we copy them first (copy-on-write)
        // so that the clones don't change. this also gives us our own texture
        if Arc::strong_count(&self.shared) > 1 {
            let pixels = self.lock().pixels.clone();
            *self = Self::from_pixels(pixels, self.width, self.height);
        }

        self.write_pixels(dst_x, dst_y, src_width, src_height, src_data);
        Ok(())
    }

    /// Like [`Image::blit`], but writes in place even if clones share this image,
    /// so they all see the change and no copy is made.
    ///
    /// Used for font atlases, which only ever gain glyphs: draw batches holding
    /// an older clone only sample regions that never change, so copying the
    /// whole atlas (and its texture) for every new glyph would be pure waste.
    #[cfg(feature = "text")]
    pub(crate) fn blit_shared(
        &self,
        dst_x: u32,
        dst_y: u32,
        src_width: u32,
        src_height: u32,
        src_data: &[u8],
    ) -> RendererResult<()> {
        self.check_blit(dst_x, dst_y, src_width, src_height, src_data)?;
        self.write_pixels(dst_x, dst_y, src_width, src_height, src_data);
        Ok(())
    }

    fn check_blit(
        &self,
        dst_x: u32,
        dst_y: u32,
        src_width: u32,
        src_height: u32,
        src_data: &[u8],
    ) -> RendererResult<()> {
        if src_width == 0 || src_height == 0 {
            return Ok(());
        }

        if dst_x + src_width > self.width || dst_y + src_height > self.height {
            return Err(Error::BlitOutOfBounds {
                x: dst_x,
                y: dst_y,
                width: src_width,
                height: src_height,
                img_width: self.width,
                img_height: self.height,
            });
        }

        let required_bytes = (src_width * 4) as usize * src_height as usize;
        if src_data.len() < required_bytes {
            return Err(Error::BlitBufferTooSmall {
                expected: required_bytes,
                actual: src_data.len(),
            });
        }

        Ok(())
    }

    /// Writes an already bounds-checked rectangle and marks it for upload.
    fn write_pixels(&self, dst_x: u32, dst_y: u32, src_width: u32, src_height: u32, src_data: &[u8]) {
        if src_width == 0 || src_height == 0 {
            return;
        }

        let row_bytes = (src_width * 4) as usize;
        let required_bytes = src_height as usize * row_bytes;

        let mut shared = self.lock();
        let image_mut: &mut [u8] = &mut shared.pixels;

        let dst_start_byte = ((dst_y * self.width) * 4) as usize;

        if src_width == self.width && dst_x == 0 {
            image_mut[dst_start_byte..dst_start_byte + required_bytes]
                .copy_from_slice(&src_data[..required_bytes]);
        } else {
            let dst_stride = (self.width * 4) as usize;
            let dst_x_byte = (dst_x * 4) as usize;

            let src_rows = src_data.chunks_exact(row_bytes);
            let dst_rows = image_mut[dst_start_byte..].chunks_exact_mut(dst_stride);

            for (src_row, dst_row) in src_rows.zip(dst_rows).take(src_height as usize) {
                dst_row[dst_x_byte..dst_x_byte + row_bytes].copy_from_slice(src_row);
            }
        }

        let new_bounds = Bounds::new(dst_x, dst_y, src_width, src_height);
        shared.dirty_zone = Some(shared.dirty_zone.map_or(new_bounds, |old| old.union(new_bounds)));
    }

    /// Copies a rectangular region from another image into this image at the given position.
    ///
    /// `src_bounds` defines the region to copy from `src`. The source rectangle must fit entire
    /// within this image's bounds.
    pub fn blit_image(
        &mut self,
        dst_x: u32,
        dst_y: u32,
        src: Image,
        src_bounds: Bounds,
    ) -> RendererResult<()> {
        let data = src.get_rect(src_bounds)?;
        self.blit(dst_x, dst_y, src_bounds.width, src_bounds.height, &data)?;
        Ok(())
    }

    /// Reads a rectangular region of this image into `dst` as raw RGBA8 pixel data.
    ///
    /// `dst` must be at least `bounds.width * bounds.height * 4` bytes. The requested
    /// region must fit entirely within the image bounds.
    pub fn read_rect(&self, bounds: Bounds, dst: &mut [u8]) -> RendererResult<()> {
        if bounds.width == 0 || bounds.height == 0 {
            return Ok(());
        }

        if bounds.x + bounds.width > self.width || bounds.y + bounds.height > self.height {
            return Err(Error::ReadOutOfBounds {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                img_width: self.width,
                img_height: self.height,
            });
        }

        let row_bytes = (bounds.width * 4) as usize;
        let required_bytes = bounds.height as usize * row_bytes;

        if dst.len() < required_bytes {
            return Err(Error::ReadBufferTooSmall {
                expected: required_bytes,
                actual: dst.len(),
            });
        }

        let shared = self.lock();
        let image_ref: &[u8] = &shared.pixels;
        let start_byte = ((bounds.y * self.width + bounds.x) * 4) as usize;

        if bounds.width == self.width {
            dst[..required_bytes].copy_from_slice(&image_ref[start_byte..start_byte + required_bytes]);
        } else {
            let src_stride = (self.width * 4) as usize;
            let src_rows = image_ref[start_byte..].chunks_exact(src_stride);
            let dst_rows = dst[..required_bytes].chunks_exact_mut(row_bytes);

            for (src_row, dst_row) in src_rows.zip(dst_rows).take(bounds.height as usize) {
                dst_row.copy_from_slice(&src_row[..row_bytes]);
            }
        }

        Ok(())
    }

    /// Returns a rectangular region of this image as a `Vec<u8>` of raw RGBA8 pixel data.
    ///
    /// The is a convenience wrapper around [`read_rect`] that allocates the buffer for you.
    /// The requested region must fit entirely within the image bounds.
    pub fn get_rect(&self, bounds: Bounds) -> RendererResult<Vec<u8>> {
        let size = (bounds.width * bounds.height * 4) as usize;
        let mut buffer = vec![0; size];
        self.read_rect(bounds, &mut buffer)?;
        Ok(buffer)
    }

    pub(crate) fn submit_to_gpu(&self, ctx: &GpuContext) -> RendererResult<Arc<ImageData>> {
        let mut shared = self.lock();
        if let Some(data) = shared.gpu.clone() {
            if let Some(zone) = shared.dirty_zone.take() {
                self.write_texture(ctx, &data, zone, &shared.pixels);
            }
            Ok(data)
        } else {
            let size = Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 };
            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: Some("image texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&TextureViewDescriptor::default());

            let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
                layout: &ctx.texture_group_layout,
                entries: &[
                    BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&view) },
                    BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&ctx.sampler) },
                ],
                label: Some("image bind group"),
            });

            let data = Arc::new(ImageData { bind_group, texture });
            self.write_texture(ctx, &data, Bounds::new(0, 0, self.width, self.height), &shared.pixels);
            shared.gpu = Some(data.clone());
            shared.dirty_zone = None;

            Ok(data)
        }
    }

    fn write_texture(&self, ctx: &GpuContext, data: &ImageData, zone: Bounds, pixels: &[u8]) {
        let size = Extent3d {
            width: zone.width,
            height: zone.height,
            depth_or_array_layers: 1
        };

        let offset = ((zone.y * self.width + zone.x) * 4) as u64;

        ctx.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &data.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: zone.x,
                    y: zone.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            pixels,
            TexelCopyBufferLayout {
                offset,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );
    }
}
