// Modern directory-based module declarations
pub mod data_processing;
pub mod parsers;

// Re-export main service types for easy access
pub use data_processing::*;
pub use parsers::*;
