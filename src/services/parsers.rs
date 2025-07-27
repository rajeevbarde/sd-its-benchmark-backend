// Parser modules for data transformation
pub mod app_details_parser;
pub mod system_info_parser;
pub mod gpu_info_parser;
pub mod libraries_parser;
pub mod performance_parser;

// Re-export all parsers for easy access
pub use app_details_parser::*;
pub use system_info_parser::*;
pub use gpu_info_parser::*;
pub use libraries_parser::*;
pub use performance_parser::*; 