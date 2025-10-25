use anyhow::Result;

use crate::Frame;

pub mod html;

pub trait Renderer<EncodedFrame, EncodedVideo> {
    fn new() -> Self;
    fn encode(&mut self, frames: impl IntoIterator<Item = Frame>) -> Result<EncodedVideo>;
}
