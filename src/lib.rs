pub mod parser;
pub mod renderers;

pub struct InitializationParameters {
    /// Frames duration in milliseconds
    pub d: u64
}

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
