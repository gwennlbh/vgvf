pub mod parser;
pub use parser::Parser;
pub mod encoder;
pub use encoder::Encoder;
pub mod transcoders;
pub use transcoders::{HTMLTranscoder, MP4Transcoder, Transcoder};
pub mod renderer;
pub use renderer::Renderer;

#[derive(Debug, Clone)]
pub enum Frame {
    /// I frame
    /// Contains vgv-specific params, svg attributes
    Initialization {
        /// Frames duration in milliseconds
        d: u64,
        /// Frame width
        w: u32,
        /// Frame height
        h: u32,
        /// Backdrop color as CSS color string
        bg: String,
        /// Attributes to add to the SVG element
        svg: String
    },
    /// S frame
    /// Contains CSS rules
    Style(String),
    /// F frame
    /// Contains SVG content
    Full(String),
    /// D frame
    /// Contains SVG content delta
    Delta(String),
    /// U frame
    /// Number of frames that are unchanged
    Unchanged(u32),
}
