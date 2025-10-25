use diff_match_patch_rs::Compat;

use crate::{Frame, InitializationParameters};

pub struct Encoder {
    pub frames: Vec<Frame>,
    last_frame_svg: Option<String>,
}

impl Encoder {
    pub fn new(params: InitializationParameters, svg_attrs: String) -> Self {
        Self {
            frames: vec![Frame::Initialization(params, svg_attrs)],
            last_frame_svg: None,
        }
    }

    pub fn encode_svg(&mut self, svg_contents: String) -> () {
        // Add a new full frame every 100 frames
        if self.frames.len() % 100 == 0 {
            self.frames.push(Frame::Full(svg_contents.clone()));
        } else {
            if let None = self.last_frame_svg {
                self.frames.push(Frame::Full(svg_contents.clone()));
                return;
            }

            let dmp = diff_match_patch_rs::DiffMatchPatch::default();
            let diffs = dmp
                .diff_main::<Compat>(self.last_frame_svg.as_ref().unwrap(), &svg_contents.clone())
                .expect("Couldn't diff with previous full frame");

            let delta = dmp
                .diff_to_delta(&diffs)
                .expect("Couldn't crush diff into delta");

            self.frames.push(Frame::Delta(delta));
        }

        self.last_frame_svg = Some(svg_contents);
    }
}
