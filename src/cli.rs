use base64::prelude::*;
use diff_match_patch_rs::{Compat, DiffMatchPatch, dmp::Diff};
use vgv::{self, Frame};

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

pub fn main() {
    let mut args = pico_args::Arguments::from_env();
    let mut parser = vgv::Parser::new();

    let input_file: String = args.free_from_str().expect("Provide an input file");

    let raw_frames = std::fs::read_to_string(&input_file)
        .expect(&format!("Failed to read input file {input_file}"));

    let dmp = DiffMatchPatch::new();
    println!(
        "{:?}",
        dmp.diff_to_delta::<Compat>(
            dmp.diff_main("<rect x=10 y=10>", "<rect x=22 y=10>")
                .unwrap()
                .as_slice()
        )
        .unwrap()
    );
    println!(
        "{:?}",
        dmp.diff_from_delta::<Compat>("<rect x=10 y=10>", "=8\t-2\t+20\t=6")
            .unwrap()
    );

    let frames = parser
        .parse_frames(&raw_frames)
        .expect("Failed to parse frames");

    let mut html_frames = Vec::new();

    for frame in frames.iter() {
        parser
            .parse_incremental(frame)
            .expect(&format!("Couldn't parse frame"));

        if let Frame::Delta(_) | Frame::Full(_) = frame {
            html_frames.push(parser.output_html().to_base64());
        }
    }

    std::fs::write(
        "player.html", 
        format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>VGV Player</title>\n<script>{script}</script></head>\n<body>\n{initial_body}\n</body>\n</html>",
            initial_body =  html_frames.first().unwrap().from_base64(),
            script = format!(
                r#"
                    window.frameNo = 0;
                    const frames = {frames_array};
                    const decoder = new TextDecoder('utf-8');
                    setInterval(() => {{
                        document.body.innerHTML = decoder.decode(Uint8Array.fromBase64(frames[window.frameNo]));
                        window.frameNo = (window.frameNo + 1) % frames.length;
                    }}, {frame_duration});
                "#, 
                frames_array = serde_json::to_string(&html_frames).unwrap(),
                frame_duration = parser.frame_duration.as_millis()
            ),
    )
).expect("Couldn't write player")
}
