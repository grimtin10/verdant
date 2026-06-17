// TODO: optimize the case where someone loads the same image a bunch

use std::{fmt::{self, Debug, Formatter}, sync::{Arc, RwLockReadGuard}};

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};

use crate::{GpuContext, RendererResult, errors::Error, types::ByteSource};

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

#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,

    pub(crate) data: Option<Arc<ImageData>>,

    image: Arc<Vec<u8>>,
    dirty_zone: Option<Bounds>,
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
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data", &self.data)
            .field("image", &format!("[{} bytes of image data]", self.image.len()))
            .field("dirty_zone", &self.dirty_zone)
            .finish()
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.image, &other.image)
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

        Ok(Self {
            width,
            height,

            data: None,

            image: Arc::new(data),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        })
    }

    /// Creates a completely transparent image with the given width and height.
    pub fn new_empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,

            data: None,

            image: Arc::new(vec![0; (width * height * 4) as usize]),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        }
    }

    /// Loads an image from the given source (byte array or path to image file).
    #[cfg(feature = "image")]
    pub fn load(data: impl ByteSource) -> RendererResult<Self> {
        use image::load_from_memory;

        let bytes = data.load()?;

        let image = load_from_memory(&bytes)?.to_rgba8();
        let (width, height) = image.dimensions();

        Ok(Self {
            width,
            height,

            data: None,

            image: Arc::new(image.to_vec()),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        })
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

        let row_bytes = (src_width * 4) as usize;
        let required_bytes = src_height as usize * row_bytes;
        if src_data.len() < required_bytes {
            return Err(Error::BlitBufferTooSmall {
                expected: required_bytes,
                actual: src_data.len(),
            });
        }

        let is_shared = Arc::strong_count(&self.image) > 1;
        let image_mut = Arc::make_mut(&mut self.image);
        let image_mut: &mut [u8] = image_mut;

        if is_shared {
            self.data = None;
        }

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
        self.dirty_zone = Some(self.dirty_zone.map_or(new_bounds, |old| old.union(new_bounds)));

        Ok(())
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

        let image_ref: &[u8] = &self.image;
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

    pub(crate) fn submit_to_gpu(&mut self, ctx: &GpuContext) -> RendererResult<Arc<ImageData>> {
        if let Some(data) = self.data.clone() {
            self.write_texture(ctx, data.clone());
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
            self.write_texture(ctx, data.clone());
            self.data = Some(data.clone());

            Ok(data)
        }
    }

    pub(crate) fn write_texture(&mut self, ctx: &GpuContext, data: Arc<ImageData>) {
        if let Some(zone) = self.dirty_zone.take() {
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
                &self.image,
                TexelCopyBufferLayout {
                    offset,
                    bytes_per_row: Some(4 * self.width),
                    rows_per_image: Some(self.height),
                },
                size,
            );
        }
    }
}
