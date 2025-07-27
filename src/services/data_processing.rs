// Data processing services for admin operations
pub mod save_data_service;
pub mod process_its_service;
pub mod process_app_details_service;
pub mod process_system_info_service;
pub mod process_libraries_service;
pub mod process_gpu_service;
pub mod update_gpu_brands_service;
pub mod update_gpu_laptop_info_service;
pub mod process_run_details_service;
pub mod analyze_app_details_service;
pub mod fix_app_names_service;
pub mod update_run_more_details_service;

// Re-export all services for easy access
pub use save_data_service::*;
pub use process_its_service::*;
pub use process_app_details_service::*;
pub use process_system_info_service::*;
pub use process_libraries_service::*;
pub use process_gpu_service::*;
pub use update_gpu_brands_service::*;
pub use update_gpu_laptop_info_service::*;
pub use process_run_details_service::*;
pub use analyze_app_details_service::*;
pub use fix_app_names_service::*;
pub use update_run_more_details_service::*; 