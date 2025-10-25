use std::io::Write;

use diff_match_patch_rs::Compat;

use crate::{Frame, InitializationParameters, parser::MAGIC};

pub struct Encoder {
    pub frames: Vec<Frame>,
    last_frame_svg: Option<String>,
    /// Every how many frames do we insert a full frame
    pub full_diff_ratio: usize,
}

impl Encoder {
    pub fn new(params: InitializationParameters, svg_attrs: String) -> Self {
        Self {
            frames: vec![Frame::Initialization(params, svg_attrs)],
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

    fn push_diff_frame(&mut self, svg_contents: String) {
        if let None = self.last_frame_svg {
            self.push_full_frame(svg_contents);
            return;
        }

        let dmp = diff_match_patch_rs::DiffMatchPatch::default();
        let diffs = dmp
            .diff_main::<Compat>(self.last_frame_svg.as_ref().unwrap(), &svg_contents)
            .expect("Couldn't diff with previous full frame");

        let delta = dmp
            .diff_to_delta(&diffs)
            .expect("Couldn't crush diff into delta");

        self.frames.push(Frame::Delta(delta));
        self.last_frame_svg = Some(svg_contents)
    }
}

impl Frame {
    pub fn encode(&self) -> String {
        match self {
            Frame::Style(rules) => format!("S{}", rules.remove_newlines()),
            Frame::Full(content) => format!("F{}", content.remove_newlines()),
            Frame::Delta(delta) => format!("D{}", delta.remove_newlines()),
            Frame::Initialization(params, svg_attrs) => format!(
                "I{}",
                [
                    params.d.to_string(),
                    params.w.to_string(),
                    params.h.to_string(),
                    params.bg.clone(),
                    svg_attrs.to_string()
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
