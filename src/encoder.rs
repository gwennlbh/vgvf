use crate::{Frame, Transcoder, parser::MAGIC};
use diff_match_patch_rs::{Compat, Ops};
use anyhow::Result;
use std::io::Write;

pub struct Encoder {
    dmp: diff_match_patch_rs::DiffMatchPatch,
    last_frame_svg: Option<String>,
    pub frames: Vec<Frame>,
    /// Every how many frames do we insert a full frame
    pub full_diff_ratio: usize,
}

impl Encoder {
    pub fn new(init_frame: Frame) -> Self {
        Self {
            frames: vec![init_frame],
            dmp: diff_match_patch_rs::DiffMatchPatch::new(),
            last_frame_svg: None,
            full_diff_ratio: 100,
        }
    }

    pub fn encode_svg(&mut self, svg_contents: String) -> () {
        // Add a new full frame every 100 frames
        if self.frames.len() % self.full_diff_ratio == 0 {
            self.push_full_frame(svg_contents);
        } else {
            self.push_diff_frame(svg_contents);
        }
    }

    pub fn dump(&self, writer: &mut impl Write) -> () {
        writeln!(writer, "{MAGIC}").expect("Couldn't write magic header");
        for frame in &self.frames {
            writeln!(writer, "{}", frame.encode()).expect("Couldn't write frame")
        }
    }

    fn push_full_frame(&mut self, svg_contents: String) {
        self.frames.push(Frame::Full(svg_contents.clone()));
        self.last_frame_svg = Some(svg_contents);
    }

    fn push_unchanged_frame(&mut self) {
        if let Some(Frame::Unchanged(count)) = self.frames.last_mut() {
            *count += 1;
        } else {
            self.frames.push(Frame::Unchanged(1));
        }
    }

    fn push_diff_frame(&mut self, svg_contents: String) {
        if let None = self.last_frame_svg {
            self.push_full_frame(svg_contents);
            return;
        }

        let diffs = self
            .dmp
            .diff_main::<Compat>(self.last_frame_svg.as_ref().unwrap(), &svg_contents)
            .expect("Couldn't diff with previous full frame");

        match (diffs.len(), diffs.first()) {
            (_, None) => {
                self.push_unchanged_frame();
            }
            (1, Some(diff)) if diff.op() == Ops::Equal => {
                self.push_unchanged_frame();
            }
            _ => {
                let delta = self
                    .dmp
                    .diff_to_delta(&diffs)
                    .expect("Couldn't crush diff into delta");

                self.frames.push(Frame::Delta(delta));
                self.last_frame_svg = Some(svg_contents);
            }
        }
    }
}


impl Encoder {
    pub fn transcode<T, U: Transcoder<T>>(self, transcoder: &mut U) -> Result<T> {
        transcoder.encode(self.frames)
    }
}

impl Frame {
    pub fn encode(&self) -> String {
        match self {
            Frame::Unchanged(count) => format!("U{}", count),
            Frame::Style(rules) => format!("S{}", rules.remove_newlines()),
            Frame::Full(content) => format!("F{}", content.remove_newlines()),
            Frame::Delta(delta) => format!("D{}", delta.remove_newlines()),
            Frame::Initialization { d, w, h, bg, svg } => format!(
                "I{}",
                [
                    d.to_string(),
                    w.to_string(),
                    h.to_string(),
                    bg.clone(),
                    svg.to_string()
                ]
                .join("\t")
            )
            .remove_newlines(),
        }
    }
}

trait RemoveNewlines {
    fn remove_newlines(&self) -> Self;
}

impl RemoveNewlines for String {
    fn remove_newlines(&self) -> Self {
        self.replace("\n", "")
    }
}
