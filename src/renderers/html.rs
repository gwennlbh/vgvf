use crate::{Frame, InitializationParameters, renderers::Renderer};
use anyhow::{Result, anyhow};
use base64::prelude::*;
use diff_match_patch_rs::{Compat, PatchInput};
use serde::Serialize;
use std::time::Duration;

#[derive(Default)]
pub struct HTMLRenderer {
    pub svg_attributes: String,
    pub stylesheet: String,
    pub svg_content: String,
    pub frame_duration: std::time::Duration,
    pub dmp: diff_match_patch_rs::DiffMatchPatch,
}

#[derive(Serialize)]
pub struct HTMLFrame {
    pub svg: String,
    pub style: String,
}

impl HTMLRenderer {
    fn step(&mut self, frame: &Frame) -> Result<()> {
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
}

impl Renderer<HTMLFrame, String> for HTMLRenderer {
    fn new() -> Self {
        Self::default()
    }

    fn encode(&mut self, frames: impl IntoIterator<Item = Frame>) -> Result<String> {
        let mut encoded_frames = Vec::new();

        for frame in frames {
            self.step(&frame)?;

            match frame {
                Frame::Full(_) | Frame::Delta(_) => {
                    encoded_frames.push(HTMLFrame {
                        style: self.stylesheet.to_base64(),
                        svg: format!("<svg {}>{}</svg>", self.svg_attributes, self.svg_content)
                            .to_base64(),
                    });
                }
                _ => {}
            }
        }

        Ok(format!(
            "
            <!DOCTYPE html>
            <html>
                <head>
                    <meta charset=\"UTF-8\">
                    <title>VGV Player</title>
                    <script>{script}</script>
                    <style>{initial_style}</style>
                </head>
                <body>
                    {initial_body}
                </body>
            </html>",
            initial_body = encoded_frames.first().unwrap().svg.from_base64(),
            initial_style = encoded_frames.first().unwrap().style.from_base64(),
            script = format!(
                r#"
                    window.frameNo = 1;
                    const frames = {frames_array};
                    const decoder = new TextDecoder('utf-8');
                    function fromBase64(str) {{
                        return decoder.decode(Uint8Array.fromBase64(str));
                    }}
                    setInterval(() => {{
                        window.frameNo = (window.frameNo + 1) % frames.length;
                        const frame = frames[window.frameNo];
                        document.body.innerHTML = fromBase64(frame.svg);
                        document.querySelector('style').innerHTML = fromBase64(frame.style);
                    }}, {frame_duration});
                "#,
                frames_array = serde_json::to_string(&encoded_frames).unwrap(),
                frame_duration = self.frame_duration.as_millis()
            )
        ))
    }
}

trait Base64Encodable {
    fn to_base64(&self) -> String;
    fn from_base64(&self) -> Self;
}

impl Base64Encodable for String {
    fn to_base64(&self) -> String {
        BASE64_STANDARD.encode(self.as_bytes())
    }

    fn from_base64(&self) -> Self {
        String::from_utf8(BASE64_STANDARD.decode(self).unwrap()).unwrap()
    }
}
