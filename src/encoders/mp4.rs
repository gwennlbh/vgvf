use std::{io::Write, thread, time::Duration};

use crate::{Encoder, Frame, InitializationParameters, Renderer};

pub struct MP4Encoder {
    pub renderer: Renderer,
    pub dimensions: (u32, u32),
    pub encoder: std::process::Child,
    pub output_path: std::path::PathBuf,
}

impl MP4Encoder {
    pub fn new(
        output_file: impl Into<std::path::PathBuf>,
        audio_file: impl Into<std::path::PathBuf>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Self {
        let output_path = output_file.into();
        Self {
            output_path: output_path.clone(),
            dimensions: (width, height),
            renderer: Renderer::default(),
            encoder: std::process::Command::new("ffmpeg")
                .arg("-i")
                .arg(audio_file.into())
                .arg("-f")
                .arg("rawvideo")
                .arg("-pixel_format")
                .arg("rgba")
                .arg("-video_size")
                .arg(format!("{width}x{height}"))
                .arg("-framerate")
                .arg(format!("{}", fps))
                .arg("-i")
                .arg("-")
                .arg("-map")
                .arg("0:a")
                .arg("-map")
                .arg("1:v")
                .arg("-shortest")
                .arg(output_path)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("Couldn't start ffmpeg"),
        }
    }
}

impl Encoder<std::path::PathBuf> for MP4Encoder {
    fn encode(
        &mut self,
        frames: impl IntoIterator<Item = crate::Frame>,
    ) -> anyhow::Result<std::path::PathBuf> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<(String, (u32, u32))>(1_000);
        let mut encoder_stdin = self.encoder.stdin.take().unwrap();
        let (width, height) = self.dimensions;

        let encoder_thread = thread::spawn(move || {
            let mut pixels =
                tiny_skia::Pixmap::new(width, height).expect("Failed to create pixmap");
            let mut fonts = usvg::fontdb::Database::new();
            fonts.load_system_fonts();

            let usvg_options = usvg::Options {
                fontdb: fonts.into(),
                ..usvg::Options::default()
            };

            for (svg, (svg_width, svg_height)) in rx.iter() {
                if svg.is_empty() {
                    break;
                }

                let tree = usvg::Tree::from_str(&svg, &usvg_options).expect("Failed to parse SVG");
                resvg::render(
                    &tree,
                    tiny_skia::Transform::from_scale(
                        width as f32 / svg_width as f32,
                        height as f32 / svg_height as f32,
                    ),
                    &mut pixels.as_mut(),
                );

                encoder_stdin.write_all(&pixels.data());
            }

            encoder_stdin.flush().unwrap();
        });

        // Placeholder implementation
        // Actual MP4 encoding logic would go here
        for frame in frames {
            self.renderer.step(&frame)?;
            // Process the frame for MP4 encoding

            if frame.triggers_new_image() {
                tx.send((
                    self.renderer.svg_tag().replace(
                        "</svg>",
                        &format!("<style>{}</style></svg>", self.renderer.stylesheet),
                    ),
                    self.renderer.frame_dimensions,
                ))
                .expect("Failed to send frame to encoder thread");
            }
        }

        encoder_thread.join().expect("Encoder thread panicked");

        Ok(self.output_path.clone()) // Return an empty Vec as a placeholder
    }
}
