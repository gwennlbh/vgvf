use anyhow::Result;

use crate::Frame;

pub mod html;
pub use html::HTMLEncoder;
pub mod mp4;
pub use mp4::MP4Encoder;

pub trait Encoder<T> {
    fn encode(&mut self, frames: impl IntoIterator<Item = Frame>) -> Result<T>;
}
