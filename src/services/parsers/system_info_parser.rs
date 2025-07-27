use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSystemInfo {
    pub arch: Option<String>,
    pub cpu: Option<String>,
    pub system: Option<String>,
    pub release: Option<String>,
    pub python: Option<String>,
}

pub struct SystemInfoParser;

impl SystemInfoParser {
    /// Parse system information from the system_info string
    /// 
    /// The system_info string format is typically: "arch:x86_64 cpu:Intel system:Linux release:5.15.0 python:3.9.0"
    /// 
    /// # Arguments
    /// * `system_info_string` - The raw system info string to parse
    /// 
    /// # Returns
    /// * `ParsedSystemInfo` - Structured system information
    pub fn parse(system_info_string: &str) -> ParsedSystemInfo {
        let mut system_info = ParsedSystemInfo {
            arch: None,
            cpu: None,
            system: None,
            release: None,
            python: None,
        };

        // Split by spaces and process each part
        let parts: Vec<&str> = system_info_string.split(' ').collect();
        let mut i = 0;
        
        while i < parts.len() {
            let part = parts[i];
            
            if let Some(colon_index) = part.find(':') {
                let key = &part[..colon_index];
                let value = &part[colon_index + 1..];
                
                // Handle multi-word values
                let mut full_value = value.to_string();
                let mut next_i = i + 1;
                
                // Look ahead for multi-word values (especially for CPU)
                while next_i < parts.len() && !parts[next_i].contains(':') {
                    full_value.push(' ');
                    full_value.push_str(parts[next_i]);
                    next_i += 1;
                }
                
                match key {
                    "arch" => {
                        system_info.arch = Some(full_value);
                    }
                    "cpu" => {
                        system_info.cpu = Some(full_value);
                    }
                    "system" => {
                        system_info.system = Some(full_value);
                    }
                    "release" => {
                        system_info.release = Some(full_value);
                    }
                    "python" => {
                        system_info.python = Some(full_value);
                    }
                    _ => {}
                }
                
                i = next_i;
            } else {
                i += 1;
            }
        }

        system_info
    }

    /// Validate if the parsed system info contains valid data
    /// 
    /// # Arguments
    /// * `system_info` - The parsed system info to validate
    /// 
    /// # Returns
    /// * `bool` - True if the system info is valid
    pub fn is_valid(system_info: &ParsedSystemInfo) -> bool {
        system_info.arch.is_some() || 
        system_info.cpu.is_some() || 
        system_info.system.is_some() || 
        system_info.release.is_some() || 
        system_info.python.is_some()
    }

    /// Get a summary of the parsed system info
    /// 
    /// # Arguments
    /// * `system_info` - The parsed system info
    /// 
    /// # Returns
    /// * `String` - A summary string
    pub fn get_summary(system_info: &ParsedSystemInfo) -> String {
        let mut parts = Vec::new();
        
        if let Some(arch) = &system_info.arch {
            parts.push(format!("arch:{}", arch));
        }
        
        if let Some(cpu) = &system_info.cpu {
            parts.push(format!("cpu:{}", cpu));
        }
        
        if let Some(system) = &system_info.system {
            parts.push(format!("system:{}", system));
        }
        
        if let Some(release) = &system_info.release {
            parts.push(format!("release:{}", release));
        }
        
        if let Some(python) = &system_info.python {
            parts.push(format!("python:{}", python));
        }
        
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_info_basic() {
        let system_info_string = "arch:x86_64 cpu:Intel system:Linux release:5.15.0 python:3.9.0";
        let result = SystemInfoParser::parse(system_info_string);
        
        assert_eq!(result.arch, Some("x86_64".to_string()));
        assert_eq!(result.cpu, Some("Intel".to_string()));
        assert_eq!(result.system, Some("Linux".to_string()));
        assert_eq!(result.release, Some("5.15.0".to_string()));
        assert_eq!(result.python, Some("3.9.0".to_string()));
    }

    #[test]
    fn test_parse_system_info_multi_word_cpu() {
        let system_info_string = "arch:x86_64 cpu:Intel Core i7 system:Linux release:5.15.0 python:3.9.0";
        let result = SystemInfoParser::parse(system_info_string);
        
        assert_eq!(result.arch, Some("x86_64".to_string()));
        assert_eq!(result.cpu, Some("Intel Core i7".to_string()));
        assert_eq!(result.system, Some("Linux".to_string()));
        assert_eq!(result.release, Some("5.15.0".to_string()));
        assert_eq!(result.python, Some("3.9.0".to_string()));
    }

    #[test]
    fn test_parse_system_info_partial() {
        let system_info_string = "arch:x86_64 cpu:Intel";
        let result = SystemInfoParser::parse(system_info_string);
        
        assert_eq!(result.arch, Some("x86_64".to_string()));
        assert_eq!(result.cpu, Some("Intel".to_string()));
        assert_eq!(result.system, None);
        assert_eq!(result.release, None);
        assert_eq!(result.python, None);
    }

    #[test]
    fn test_parse_system_info_empty() {
        let system_info_string = "";
        let result = SystemInfoParser::parse(system_info_string);
        
        assert_eq!(result.arch, None);
        assert_eq!(result.cpu, None);
        assert_eq!(result.system, None);
        assert_eq!(result.release, None);
        assert_eq!(result.python, None);
    }

    #[test]
    fn test_parse_system_info_unknown_keys() {
        let system_info_string = "arch:x86_64 unknown:value cpu:Intel";
        let result = SystemInfoParser::parse(system_info_string);
        
        assert_eq!(result.arch, Some("x86_64".to_string()));
        assert_eq!(result.cpu, Some("Intel".to_string()));
        assert_eq!(result.system, None);
        assert_eq!(result.release, None);
        assert_eq!(result.python, None);
    }

    #[test]
    fn test_is_valid() {
        let valid_info = ParsedSystemInfo {
            arch: Some("x86_64".to_string()),
            cpu: None,
            system: None,
            release: None,
            python: None,
        };
        assert!(SystemInfoParser::is_valid(&valid_info));

        let invalid_info = ParsedSystemInfo {
            arch: None,
            cpu: None,
            system: None,
            release: None,
            python: None,
        };
        assert!(!SystemInfoParser::is_valid(&invalid_info));
    }

    #[test]
    fn test_get_summary() {
        let system_info = ParsedSystemInfo {
            arch: Some("x86_64".to_string()),
            cpu: Some("Intel Core i7".to_string()),
            system: Some("Linux".to_string()),
            release: Some("5.15.0".to_string()),
            python: Some("3.9.0".to_string()),
        };
        
        let summary = SystemInfoParser::get_summary(&system_info);
        assert_eq!(summary, "arch:x86_64 cpu:Intel Core i7 system:Linux release:5.15.0 python:3.9.0");
    }
} 