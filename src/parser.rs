use std::time::Duration;

use crate::{Frame, InitializationParameters};
use anyhow::{Result, anyhow};
use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};

pub const MAGIC: &str = "vgv1";

#[derive(Default)]
pub struct Parser {
    pub stylesheet: String,
    pub svg_content: String,
    pub audio_bytes: Vec<u8>,
    pub dmp: DiffMatchPatch,
    pub frame_duration: Duration,
    pub current_time: Duration,
    pub svg_attributes: String,
}

pub trait VGVParsable {
    fn parse_as_vgv(&self) -> Result<Vec<Frame>>;
}

impl VGVParsable for String {
    fn parse_as_vgv(&self) -> Result<Vec<Frame>> {
        Parser::new().parse_frames(self)
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_frames(&mut self, raw_frames: &str) -> Result<Vec<Frame>> {
        let mut out = Vec::new();
        for (i, line) in raw_frames.lines().enumerate() {
            match i {
                0 if line == MAGIC => continue,
                0 => return Err(anyhow!("Invalid magic header, expected {MAGIC}")),
                _ => match self.parse_frame(line) {
                    Ok(frame) => out.push(frame),
                    Err(e) => return Err(anyhow!("Could not parse line {i}: {e:?}")),
                },
            }
        }

        Ok(out)
    }

    pub fn parse_frame(&mut self, raw_frame: &str) -> Result<Frame> {
        let (first_char, others) = raw_frame.split_at(1);
        match first_char {
            "S" => Ok(Frame::Style(others.to_string())),
            "F" => Ok(Frame::Full(others.to_string())),
            "D" => Ok(Frame::Delta(others.to_string())),
            "A" => Err(anyhow!("Audio frames are not supported yet")),
            "I" => {
                let mut parts = others.split("\t");

                let frame_duration = parts
                    .next()
                    .ok_or(anyhow!("Missing frame duration"))?
                    .parse()?;

                Ok(Frame::Initialization(
                    InitializationParameters { d: frame_duration },
                    parts.last().unwrap_or("").to_string(),
                ))
            }
            _ => Err(anyhow!("Unknown frame type {}", first_char)),
        }
    }

    pub fn parse_incremental(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Style(rules) => self.stylesheet += &rules,
            Frame::Full(content) => self.svg_content = content.to_string(),
            Frame::Initialization(InitializationParameters { d }, svg_attributes) => {
                self.frame_duration = Duration::from_millis(*d);
                self.svg_attributes = svg_attributes.clone();
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

    pub fn output_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<style>");
        html.push_str(&self.stylesheet);
        html.push_str("\n</style>\n<svg ");
        html.push_str(&self.svg_attributes);
        html.push_str(">\n");
        html.push_str(&self.svg_content);
        html.push_str("\n</svg>\n");
        html
    }
}
