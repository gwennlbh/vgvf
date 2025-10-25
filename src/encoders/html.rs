use crate::{Encoder, Frame, Renderer};
use anyhow::{Result, anyhow};
use base64::prelude::*;
use serde::Serialize;

#[derive(Default)]
pub struct HTMLEncoder {
    pub renderer: Renderer,
}

#[derive(Serialize)]
struct HTMLFrame {
    pub svg: String,
    pub style: String,
}

impl HTMLEncoder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Encoder<String> for HTMLEncoder {
    fn encode(&mut self, frames: impl IntoIterator<Item = Frame>) -> Result<String> {
        let mut encoded_frames = Vec::new();

        for frame in frames {
            self.renderer.step(&frame)?;

            if frame.triggers_new_image() {
                encoded_frames.push(HTMLFrame {
                    style: self.renderer.stylesheet.to_base64(),
                    svg: self.renderer.svg_tag().to_base64(),
                });
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
                    <style>{player_styles}</style>
                    <style id=framestyles>{initial_style}</style>
                </head>
                <body>
                    {initial_body}
                </body>
            </html>",
            player_styles = r#"
                body, html {
                    margin: 0;
                    padding: 0;
                }

                html {
                    background-color: black;
                }

                body {
                    height: 100vh;
                    width: 100vw;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                }

                svg {
                    background-color: white;
                    width: 100%;
                    height: 100%;
                    object-fit: contain;
                }
            "#,
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
                        document.querySelector('style#framestyles').innerHTML = fromBase64(frame.style);
                    }}, {frame_duration});
                "#,
                frames_array = serde_json::to_string(&encoded_frames).unwrap(),
                frame_duration = self.renderer.frame_duration.as_millis()
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
