use diff_match_patch_rs::{Compat, DiffMatchPatch, dmp::Diff};
use vgv::{self, Frame, parser::VGVParsable, renderers::Renderer, renderers::html::HTMLRenderer};

pub fn main() {
    let mut args = pico_args::Arguments::from_env();

    let input_file: String = args.free_from_str().expect("Provide an input file");

    let frames = std::fs::read_to_string(&input_file)
        .expect(&format!("Failed to read input file {input_file}"))
        .parse_as_vgv()
        .expect("Failed to parse frames");

    std::fs::write(
        "player.html",
        HTMLRenderer::new()
            .encode(frames)
            .expect("Couldn't render to HTML"),
    )
    .expect("Couldn't write file")
}
