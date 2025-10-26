use crate::Frame;
use anyhow::{Result, anyhow};
use diff_match_patch_rs::{Compat, PatchInput};
use std::time::Duration;

#[derive(Default)]
pub struct Renderer {
    pub svg_attributes: String,
    pub backdrop: String,
    pub stylesheet: String,
    pub svg_content: String,
    pub frame_duration: std::time::Duration,
    pub frame_dimensions: (u32, u32),
    pub dmp: diff_match_patch_rs::DiffMatchPatch,
}

impl Renderer {
    pub fn step(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Unchanged(_) => {}
            Frame::Style(rules) => self.stylesheet += &rules,
            Frame::Full(content) => self.svg_content = content.to_string(),
            Frame::Initialization { d, w, h, bg, svg } => {
                self.frame_duration = Duration::from_millis(*d);
                self.frame_dimensions = (*w, *h);
                self.svg_attributes = svg.clone();
                self.backdrop = bg.clone();
            }
            Frame::Delta(delta) => {
                let (new_frame, _) = self
                    .dmp
                    .diff_from_delta::<Compat>(&self.svg_content, &delta)
                    .and_then(|diffs| self.dmp.patch_make(PatchInput::Diffs(&diffs)))
                    .and_then(|patches| self.dmp.patch_apply(&patches, &self.svg_content))
                    .map_err(|e| anyhow!("Failed to apply delta patch: {:?}", e))?;

                self.svg_content = new_frame;
            }
        };

        Ok(())
    }

    pub fn svg_tag(&self) -> String {
        format!(
            r#"<svg width="{w}" height="{h}" {attrs}><rect width="100%" height="100%" fill="{bg}"/>{content}</svg>"#,
            w = self.frame_dimensions.0,
            h = self.frame_dimensions.1,
            bg = self.backdrop,
            attrs = self.svg_attributes,
            content = self.svg_content
        )
    }
}

impl Frame {
    pub fn triggers_new_images(&self) -> u32 {
        match self {
            Frame::Full(_) => 1,
            Frame::Delta(_) => 1,
            Frame::Unchanged(count) => *count,
            _ => 0,
        }
    }
}
