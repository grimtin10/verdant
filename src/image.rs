// TODO: optimize the case where someone loads the same image a bunch

use std::sync::Arc;

use image::{RgbaImage, load_from_memory};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};

use crate::{GpuContext, RendererResult, errors::Error, types::ByteSource};

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Bounds {
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn union(self, other: Self) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.w).max(other.x + other.w);
        let max_y = (self.y + self.h).max(other.y + other.h);

        Self {
            x: min_x,
            y: min_y,
            w: max_x - min_x,
            h: max_y - min_y,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImageData {
    pub(crate) bind_group: BindGroup,
    texture: Texture,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,

    pub(crate) data: Option<Arc<ImageData>>,

    image: Arc<RgbaImage>,
    dirty_zone: Option<Bounds>,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.data.as_ref().map(Arc::as_ptr) == other.data.as_ref().map(Arc::as_ptr)
    }
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> RendererResult<Self> {
        let required_bytes = (width * height) as usize;
        let actual_bytes = data.len();
        Ok(Self {
            width,
            height,

            data: None,

            image: Arc::new(
                RgbaImage::from_vec(width, height, data)
                    .ok_or(Error::ImageBufferTooSmall {
                        expected: required_bytes,
                        actual: actual_bytes,
                    })?
            ),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        })
    }

    pub fn new_empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,

            data: None,

            image: Arc::new(RgbaImage::new(width, height)),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        }
    }

    pub fn load(data: impl ByteSource) -> RendererResult<Self> {
        let bytes = data.load()?;

        let image = load_from_memory(&bytes)?.to_rgba8();
        let (width, height) = image.dimensions();

        Ok(Self {
            width,
            height,

            data: None,

            image: Arc::new(image),
            dirty_zone: Some(Bounds::new(0, 0, width, height)),
        })
    }

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

    pub fn read_rect(&self, bounds: Bounds, dst: &mut [u8]) -> RendererResult<()> {
        if bounds.w == 0 || bounds.h == 0 {
            return Ok(());
        }

        if bounds.x + bounds.w > self.width || bounds.y + bounds.h > self.height {
            return Err(Error::ReadOutOfBounds {
                x: bounds.x,
                y: bounds.y,
                width: bounds.w,
                height: bounds.h,
                img_width: self.width,
                img_height: self.height,
            });
        }

        let row_bytes = (bounds.w * 4) as usize;
        let required_bytes = bounds.h as usize * row_bytes;

        if dst.len() < required_bytes {
            return Err(Error::ReadBufferTooSmall {
                expected: required_bytes,
                actual: dst.len(),
            });
        }

        let image_ref: &[u8] = &self.image;
        let start_byte = ((bounds.y * self.width + bounds.x) * 4) as usize;

        if bounds.w == self.width {
            dst[..required_bytes].copy_from_slice(&image_ref[start_byte..start_byte + required_bytes]);
        } else {
            let src_stride = (self.width * 4) as usize;
            let src_rows = image_ref[start_byte..].chunks_exact(src_stride);
            let dst_rows = dst[..required_bytes].chunks_exact_mut(row_bytes);

            for (src_row, dst_row) in src_rows.zip(dst_rows).take(bounds.h as usize) {
                dst_row.copy_from_slice(&src_row[..row_bytes]);
            }
        }

        Ok(())
    }

    pub fn get_rect(&self, bounds: Bounds) -> RendererResult<Vec<u8>> {
        let size = (bounds.w * bounds.h * 4) as usize;
        let mut buffer = vec![0; size];
        self.read_rect(bounds, &mut buffer)?;
        Ok(buffer)
    }

    pub(crate) fn submit_to_gpu(&mut self, ctx: &GpuContext) -> RendererResult<Arc<ImageData>> {
        if let Some(data) = self.data.clone() {
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
                width: zone.w,
                height: zone.h,
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
