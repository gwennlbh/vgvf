use anyhow::Result;

use crate::Frame;

pub mod html;
pub use html::HTMLTranscoder;
pub mod mp4;
pub use mp4::MP4Transcoder;

pub trait Transcoder<T> {
    fn encode(&mut self, frames: impl IntoIterator<Item = Frame>) -> Result<T>;
}
