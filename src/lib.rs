pub mod parser;
pub mod encoders;
pub use encoders::{Encoder, HTMLEncoder, MP4Encoder};
pub mod renderer;
pub use renderer::Renderer;

#[derive(Debug, Clone)]
pub struct InitializationParameters {
    /// Frames duration in milliseconds
    pub d: u64,
    /// Frame width
    pub w: u32,
    /// Frame height
    pub h: u32,
}

#[derive(Debug, Clone)]
pub enum Frame {
    /// I frame 
    /// Contains vgv-specific params, svg attributes
    Initialization(InitializationParameters, String),
    /// S frame
    /// Contains CSS rules
    Style(String),
    /// F frame
    /// Contains SVG content
    Full(String),
    /// D frame
    /// Contains SVG content delta
    Delta(String)
}

pub use parser::Parser;
