use std::io;

#[cfg(feature = "image")]
use image::ImageError;
use thiserror::Error;
use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError};
use winit::error::{EventLoopError, RequestError};

#[derive(Debug, Error)]
pub enum Error {
    // init errors
    #[error("Failed to find a GPU adapter: {0}")]
    NoAdapterFound(#[from] RequestAdapterError),

    #[error("Failed to request a device: {0}")]
    DeviceRequestFailed(#[from] RequestDeviceError),

    // window errors
    #[error("Failed to create window: {0}")]
    WindowCreationFailed(#[from] RequestError),

    #[error("Failed to create surface: {0}")]
    SurfaceCreationFailed(#[from] CreateSurfaceError),

    // image errors
    #[error("Blit region (at x: {x}, y: {y}, size: {width}x{height}) exceeds target dimensions ({img_width}x{img_height})")]
    BlitOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        img_width: u32,
        img_height: u32,
    },

    #[error("Blit source buffer is too small: expected at least {expected} bytes, but got {actual} bytes")]
    BlitBufferTooSmall {
        expected: usize,
        actual: usize,
    },

    #[error("Read region (at x: {x}, y: {y}, size: {width}x{height}) exceeds source dimensions ({img_width}x{img_height})")]
    ReadOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        img_width: u32,
        img_height: u32,
    },

    #[error("Read destination buffer is too small: expected at least {expected} bytes, but got {actual} bytes")]
    ReadBufferTooSmall {
        expected: usize,
        actual: usize,
    },

    #[error("Image source buffer is too small: expected at least {expected} bytes, but got {actual} bytes")]
    ImageBufferTooSmall {
        expected: usize,
        actual: usize,
    },

    #[cfg(feature = "image")]
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),

    // misc errors
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),

    #[error("Font error: {0}")]
    FontError(&'static str),

    #[error("Event loop error: {0}")]
    EventLoopError(#[from] EventLoopError),
}
