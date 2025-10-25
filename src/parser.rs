use crate::{Frame, InitializationParameters};
use anyhow::{Result, anyhow};

pub const MAGIC: &str = "vgv1";

#[derive(Default)]
pub struct Parser;



impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, raw_frames: &str) -> Result<Vec<Frame>> {
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

                let frame_width = parts
                    .next()
                    .ok_or(anyhow!("Missing frame width"))?
                    .parse::<u32>()?;

                let frame_height = parts
                    .next()
                    .ok_or(anyhow!("Missing frame height"))?
                    .parse::<u32>()?;

                let backdrop_color = parts
                    .next()
                    .ok_or(anyhow!("Missing backdrop color"))?
                    .to_string();

                Ok(Frame::Initialization(
                    InitializationParameters {
                        d: frame_duration,
                        w: frame_width,
                        h: frame_height,
                        bg: backdrop_color,
                    },
                    parts.last().unwrap_or("").to_string(),
                ))
            }
            _ => Err(anyhow!("Unknown frame type {}", first_char)),
        }
    }
}

pub trait VGVParsable {
    fn parse_as_vgv(&self) -> Result<Vec<Frame>>;
}

impl VGVParsable for String {
    fn parse_as_vgv(&self) -> Result<Vec<Frame>> {
        Parser::new().parse(self)
    }
}
