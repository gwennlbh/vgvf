use vgv::{
    self, MP4Encoder,
    encoders::{Encoder, html::HTMLEncoder},
    parser::VGVParsable,
};

pub fn main() {
    let mut args = pico_args::Arguments::from_env();

    let input_file: String = args.free_from_str().expect("Provide an input file");

    let frames = std::fs::read_to_string(&input_file)
        .expect(&format!("Failed to read input file {input_file}"))
        .parse_as_vgv()
        .expect("Failed to parse frames");

    std::fs::write(
        "output.html",
        HTMLEncoder::new()
            .encode(frames.clone())
            .expect("Couldn't render to HTML"),
    )
    .expect("Couldn't write file");

    MP4Encoder::new("output.mp4", 1920, 1080)
        .encode(frames)
        .expect("Couldn't render to MP4");
}
