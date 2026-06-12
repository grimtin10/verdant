use std::{fmt::{self, Display, Formatter}, io, sync::PoisonError};

#[cfg(feature = "image")]
use image::ImageError;
use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError};
use winit::error::{EventLoopError, RequestError};

#[derive(Debug)]
pub enum Error {
    // init errors
    NoAdapterFound(RequestAdapterError),
    DeviceRequestFailed(RequestDeviceError),

    // window errors
    WindowCreationFailed(RequestError),
    SurfaceCreationFailed(CreateSurfaceError),

    // image errors
    BlitOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        img_width: u32,
        img_height: u32,
    },
    BlitBufferTooSmall {
        expected: usize,
        actual: usize,
    },
    ReadOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        img_width: u32,
        img_height: u32,
    },
    ReadBufferTooSmall {
        expected: usize,
        actual: usize,
    },
    ImageBufferTooSmall {
        expected: usize,
        actual: usize,
    },

    #[cfg(feature = "image")]
    ImageError(ImageError),

    // text errors
    TextTooBig,

    // misc errors
    IOError(io::Error),
    FontError(&'static str),
    EventLoopError(EventLoopError),
    PoisonError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapterFound(e) => write!(f, "Failed to find a GPU adapter: {e}"),
            Self::DeviceRequestFailed(e) => write!(f, "Failed to request a device: {e}"),

            Self::WindowCreationFailed(e) => write!(f, "Failed to create window: {e}"),
            Self::SurfaceCreationFailed(e) => write!(f, "Failed to create surface: {e}"),

            Self::BlitOutOfBounds { x, y, width, height, img_width, img_height }
                => write!(f, "Blit region (at x: {x}, y: {y}, size: {width}x{height}) exceeds target dimensions ({img_width}x{img_height})"),
            Self::BlitBufferTooSmall { expected, actual }
                => write!(f, "Blit source buffer is too small: expected at least {expected} bytes, but got {actual} bytes"),
            Self::ReadOutOfBounds { x, y, width, height, img_width, img_height }
                => write!(f, "Read region (at x: {x}, y: {y}, size: {width}x{height}) exceeds source dimensions ({img_width}x{img_height})"),
            Self::ReadBufferTooSmall { expected, actual }
                => write!(f, "Read destination buffer is too small: expected at least {expected} bytes, but got {actual} bytes"),
            Self::ImageBufferTooSmall { expected, actual }
                => write!(f, "Image source buffer is too small: expected at least {expected} bytes, but got {actual} bytes"),

            #[cfg(feature = "image")]
            Self::ImageError(e) => write!(f, "Image error: {e}"),

            Self::TextTooBig => write!(f, "Text was too big to fit in atlas"),

            Self::IOError(e) => write!(f, "IO error: {e}"),
            Self::FontError(e) => write!(f, "Font error: {e}"),
            Self::EventLoopError(e) => write!(f, "Event loop error: {e}"),
            Self::PoisonError(e) => write!(f, "Poisoned lock error: {e}"),
        }
    }
}

impl From<RequestAdapterError> for Error {
    fn from(error: RequestAdapterError) -> Self {
        Self::NoAdapterFound(error)
    }
}

impl From<RequestDeviceError> for Error {
    fn from(error: RequestDeviceError) -> Self {
        Self::DeviceRequestFailed(error)
    }
}

impl From<RequestError> for Error {
    fn from(error: RequestError) -> Self {
        Self::WindowCreationFailed(error)
    }
}

impl From<CreateSurfaceError> for Error {
    fn from(error: CreateSurfaceError) -> Self {
        Self::SurfaceCreationFailed(error)
    }
}

#[cfg(feature = "image")]
impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Self::ImageError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<EventLoopError> for Error {
    fn from(error: EventLoopError) -> Self {
        Self::EventLoopError(error)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Self {
        Error::PoisonError(error.to_string())
    }
}
