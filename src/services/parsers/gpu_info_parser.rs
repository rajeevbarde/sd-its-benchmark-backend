use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedGpuInfo {
    pub device: Option<String>,
    pub driver: Option<String>,
    pub gpu_chip: Option<String>,
}

pub struct GpuInfoParser;

impl GpuInfoParser {
    /// Parse GPU information from the device_info string
    /// 
    /// The device_info string format is typically: "device:NVIDIA driver:470.82.01 NVIDIA GeForce RTX 3080"
    /// 
    /// # Arguments
    /// * `device_info_string` - The raw device info string to parse
    /// 
    /// # Returns
    /// * `ParsedGpuInfo` - Structured GPU information
    pub fn parse(device_info_string: &str) -> ParsedGpuInfo {
        let parts: Vec<&str> = device_info_string.split(' ').collect();
        let mut parsed_gpu_info = ParsedGpuInfo {
            device: None,
            driver: None,
            gpu_chip: None,
        };

        let mut in_gpu_chip = false;
        let mut gpu_chip_parts = Vec::new();

        for part in parts {
            let colon_index = match part.find(':') {
                Some(index) => index,
                None => {
                    // Handle non-colon parts
                    if part.contains("GB") {
                        // Append GB value to the device if it's a memory size
                        if let Some(ref mut device) = parsed_gpu_info.device {
                            device.push(' ');
                            device.push_str(part);
                        }
                    } else if in_gpu_chip {
                        gpu_chip_parts.push(part);
                    } else if let Some(ref mut device) = parsed_gpu_info.device {
                        device.push(' ');
                        device.push_str(part);
                    }
                    continue;
                }
            };

            let key = &part[..colon_index];
            let value = &part[colon_index + 1..];

            match key {
                "device" => {
                    parsed_gpu_info.device = Some(value.to_string());
                }
                "driver" => {
                    parsed_gpu_info.driver = Some(value.to_string());
                    // After driver, everything else goes to gpu_chip
                    in_gpu_chip = true;
                }
                _ => {
                    // Any other colon-separated part goes to gpu_chip
                    in_gpu_chip = true;
                    gpu_chip_parts.push(part);
                }
            }
        }

        parsed_gpu_info.gpu_chip = if gpu_chip_parts.is_empty() {
            None
        } else {
            Some(gpu_chip_parts.join(" "))
        };

        parsed_gpu_info
    }

    /// Validate if the parsed GPU info contains valid data
    /// 
    /// # Arguments
    /// * `gpu_info` - The parsed GPU info to validate
    /// 
    /// # Returns
    /// * `bool` - True if the GPU info is valid
    pub fn is_valid(gpu_info: &ParsedGpuInfo) -> bool {
        gpu_info.device.is_some() || 
        gpu_info.driver.is_some() || 
        gpu_info.gpu_chip.is_some()
    }

    /// Get a summary of the parsed GPU info
    /// 
    /// # Arguments
    /// * `gpu_info` - The parsed GPU info
    /// 
    /// # Returns
    /// * `String` - A summary string
    pub fn get_summary(gpu_info: &ParsedGpuInfo) -> String {
        let mut parts = Vec::new();
        
        if let Some(device) = &gpu_info.device {
            parts.push(format!("device:{}", device));
        }
        
        if let Some(driver) = &gpu_info.driver {
            parts.push(format!("driver:{}", driver));
        }
        
        if let Some(gpu_chip) = &gpu_info.gpu_chip {
            parts.push(gpu_chip.clone());
        }
        
        parts.join(" ")
    }

    /// Determine if a GPU is in a laptop based on device string
    /// 
    /// # Arguments
    /// * `device_string` - The device string to check
    /// 
    /// # Returns
    /// * `bool` - True if the GPU is in a laptop
    pub fn is_laptop_gpu(device_string: &str) -> bool {
        device_string.contains("Laptop") || device_string.contains("Mobile")
    }

    /// Get the brand name from a device string
    /// 
    /// # Arguments
    /// * `device_string` - The device string to analyze
    /// 
    /// # Returns
    /// * `String` - The brand name (nvidia, amd, intel, or unknown)
    pub fn get_brand_name(device_string: &str) -> String {
        let lowercase_device = device_string.to_lowercase();

        if lowercase_device.contains("nvidia") || 
           lowercase_device.contains("quadro") || 
           lowercase_device.contains("geforce") ||
           lowercase_device.contains("tesla") {
            "nvidia".to_string()
        } else if lowercase_device.contains("amd") || 
                  lowercase_device.contains("radeon") {
            "amd".to_string()
        } else if lowercase_device.contains("intel") {
            "intel".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gpu_info_basic() {
        let device_info_string = "device:NVIDIA driver:470.82.01 NVIDIA GeForce RTX 3080";
        let result = GpuInfoParser::parse(device_info_string);
        
        assert_eq!(result.device, Some("NVIDIA".to_string()));
        assert_eq!(result.driver, Some("470.82.01".to_string()));
        assert_eq!(result.gpu_chip, Some("NVIDIA GeForce RTX 3080".to_string()));
    }

    #[test]
    fn test_parse_gpu_info_with_memory() {
        let device_info_string = "device:NVIDIA 8GB driver:470.82.01 NVIDIA GeForce RTX 3080";
        let result = GpuInfoParser::parse(device_info_string);
        
        assert_eq!(result.device, Some("NVIDIA 8GB".to_string()));
        assert_eq!(result.driver, Some("470.82.01".to_string()));
        assert_eq!(result.gpu_chip, Some("NVIDIA GeForce RTX 3080".to_string()));
    }

    #[test]
    fn test_parse_gpu_info_partial() {
        let device_info_string = "device:NVIDIA";
        let result = GpuInfoParser::parse(device_info_string);
        
        assert_eq!(result.device, Some("NVIDIA".to_string()));
        assert_eq!(result.driver, None);
        assert_eq!(result.gpu_chip, None);
    }

    #[test]
    fn test_parse_gpu_info_empty() {
        let device_info_string = "";
        let result = GpuInfoParser::parse(device_info_string);
        
        assert_eq!(result.device, None);
        assert_eq!(result.driver, None);
        assert_eq!(result.gpu_chip, None);
    }

    #[test]
    fn test_is_valid() {
        let valid_info = ParsedGpuInfo {
            device: Some("NVIDIA".to_string()),
            driver: None,
            gpu_chip: None,
        };
        assert!(GpuInfoParser::is_valid(&valid_info));

        let invalid_info = ParsedGpuInfo {
            device: None,
            driver: None,
            gpu_chip: None,
        };
        assert!(!GpuInfoParser::is_valid(&invalid_info));
    }

    #[test]
    fn test_get_summary() {
        let gpu_info = ParsedGpuInfo {
            device: Some("NVIDIA".to_string()),
            driver: Some("470.82.01".to_string()),
            gpu_chip: Some("NVIDIA GeForce RTX 3080".to_string()),
        };
        
        let summary = GpuInfoParser::get_summary(&gpu_info);
        assert_eq!(summary, "device:NVIDIA driver:470.82.01 NVIDIA GeForce RTX 3080");
    }

    #[test]
    fn test_is_laptop_gpu() {
        assert!(GpuInfoParser::is_laptop_gpu("NVIDIA GeForce RTX 3080 Laptop"));
        assert!(GpuInfoParser::is_laptop_gpu("NVIDIA GeForce RTX 3080 Mobile"));
        assert!(!GpuInfoParser::is_laptop_gpu("NVIDIA GeForce RTX 3080"));
    }

    #[test]
    fn test_get_brand_name() {
        assert_eq!(GpuInfoParser::get_brand_name("NVIDIA GeForce RTX 3080"), "nvidia");
        assert_eq!(GpuInfoParser::get_brand_name("AMD Radeon RX 6800"), "amd");
        assert_eq!(GpuInfoParser::get_brand_name("Intel UHD Graphics"), "intel");
        assert_eq!(GpuInfoParser::get_brand_name("Unknown GPU"), "unknown");
    }
} 