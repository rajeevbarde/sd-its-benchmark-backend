use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLibraries {
    pub torch: Option<String>,
    pub xformers: Option<String>,
    pub diffusers: Option<String>,
    pub transformers: Option<String>,
}

pub struct LibrariesParser;

impl LibrariesParser {
    /// Parse library information from the model_info string
    /// 
    /// The model_info string format is typically: "torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0"
    /// 
    /// # Arguments
    /// * `model_info_string` - The raw model info string to parse
    /// 
    /// # Returns
    /// * `ParsedLibraries` - Structured library information
    pub fn parse(model_info_string: &str) -> ParsedLibraries {
        let parts: Vec<&str> = model_info_string.split(' ').collect();
        let mut parsed_libraries = ParsedLibraries {
            torch: None,
            xformers: None,
            diffusers: None,
            transformers: None,
        };

        let mut torch_flag = false;
        let mut torch_value = String::new();

        for part in parts {
            let colon_index = match part.find(':') {
                Some(index) => index,
                None => {
                    // If we're collecting torch info and this part has no colon, continue collecting
                    if torch_flag {
                        torch_value.push(' ');
                        torch_value.push_str(part);
                        parsed_libraries.torch = Some(torch_value.trim().to_string());
                    }
                    continue;
                }
            };

            let key = &part[..colon_index];
            let value = &part[colon_index + 1..];

            match key {
                "torch" => {
                    // Start collecting the torch value
                    torch_flag = true;
                    torch_value = value.to_string();
                    parsed_libraries.torch = Some(torch_value.clone());
                }
                "xformers" => {
                    torch_flag = false;
                    parsed_libraries.xformers = Some(value.to_string());
                }
                "diffusers" => {
                    torch_flag = false;
                    parsed_libraries.diffusers = Some(value.to_string());
                }
                "transformers" => {
                    torch_flag = false;
                    parsed_libraries.transformers = Some(value.to_string());
                }
                _ => {
                    // If we're collecting torch info, continue
                    if torch_flag {
                        torch_value.push(' ');
                        torch_value.push_str(part);
                        parsed_libraries.torch = Some(torch_value.trim().to_string());
                    }
                }
            }
        }

        parsed_libraries
    }

    /// Validate if the parsed libraries contain valid data
    /// 
    /// # Arguments
    /// * `libraries` - The parsed libraries to validate
    /// 
    /// # Returns
    /// * `bool` - True if the libraries are valid
    pub fn is_valid(libraries: &ParsedLibraries) -> bool {
        libraries.torch.is_some() || 
        libraries.xformers.is_some() || 
        libraries.diffusers.is_some() || 
        libraries.transformers.is_some()
    }

    /// Get a summary of the parsed libraries
    /// 
    /// # Arguments
    /// * `libraries` - The parsed libraries
    /// 
    /// # Returns
    /// * `String` - A summary string
    pub fn get_summary(libraries: &ParsedLibraries) -> String {
        let mut parts = Vec::new();
        
        if let Some(torch) = &libraries.torch {
            parts.push(format!("torch:{}", torch));
        }
        
        if let Some(xformers) = &libraries.xformers {
            parts.push(format!("xformers:{}", xformers));
        }
        
        if let Some(diffusers) = &libraries.diffusers {
            parts.push(format!("diffusers:{}", diffusers));
        }
        
        if let Some(transformers) = &libraries.transformers {
            parts.push(format!("transformers:{}", transformers));
        }
        
        parts.join(" ")
    }

    /// Get the version of a specific library
    /// 
    /// # Arguments
    /// * `libraries` - The parsed libraries
    /// * `library_name` - The name of the library to get version for
    /// 
    /// # Returns
    /// * `Option<String>` - The version if found
    pub fn get_version(libraries: &ParsedLibraries, library_name: &str) -> Option<String> {
        match library_name.to_lowercase().as_str() {
            "torch" => libraries.torch.clone(),
            "xformers" => libraries.xformers.clone(),
            "diffusers" => libraries.diffusers.clone(),
            "transformers" => libraries.transformers.clone(),
            _ => None,
        }
    }

    /// Check if all required libraries are present
    /// 
    /// # Arguments
    /// * `libraries` - The parsed libraries
    /// * `required_libraries` - List of required library names
    /// 
    /// # Returns
    /// * `bool` - True if all required libraries are present
    pub fn has_all_required(libraries: &ParsedLibraries, required_libraries: &[&str]) -> bool {
        required_libraries.iter().all(|lib| {
            LibrariesParser::get_version(libraries, lib).is_some()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_libraries_basic() {
        let model_info_string = "torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0";
        let result = LibrariesParser::parse(model_info_string);
        
        assert_eq!(result.torch, Some("2.0.0".to_string()));
        assert_eq!(result.xformers, Some("0.0.22".to_string()));
        assert_eq!(result.diffusers, Some("0.21.0".to_string()));
        assert_eq!(result.transformers, Some("4.30.0".to_string()));
    }

    #[test]
    fn test_parse_libraries_multi_word_torch() {
        let model_info_string = "torch:2.0.0+cu118 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0";
        let result = LibrariesParser::parse(model_info_string);
        
        assert_eq!(result.torch, Some("2.0.0+cu118".to_string()));
        assert_eq!(result.xformers, Some("0.0.22".to_string()));
        assert_eq!(result.diffusers, Some("0.21.0".to_string()));
        assert_eq!(result.transformers, Some("4.30.0".to_string()));
    }

    #[test]
    fn test_parse_libraries_partial() {
        let model_info_string = "torch:2.0.0 xformers:0.0.22";
        let result = LibrariesParser::parse(model_info_string);
        
        assert_eq!(result.torch, Some("2.0.0".to_string()));
        assert_eq!(result.xformers, Some("0.0.22".to_string()));
        assert_eq!(result.diffusers, None);
        assert_eq!(result.transformers, None);
    }

    #[test]
    fn test_parse_libraries_empty() {
        let model_info_string = "";
        let result = LibrariesParser::parse(model_info_string);
        
        assert_eq!(result.torch, None);
        assert_eq!(result.xformers, None);
        assert_eq!(result.diffusers, None);
        assert_eq!(result.transformers, None);
    }

    #[test]
    fn test_parse_libraries_unknown_keys() {
        let model_info_string = "torch:2.0.0 unknown:value xformers:0.0.22";
        let result = LibrariesParser::parse(model_info_string);
        
        assert_eq!(result.torch, Some("2.0.0".to_string()));
        assert_eq!(result.xformers, Some("0.0.22".to_string()));
        assert_eq!(result.diffusers, None);
        assert_eq!(result.transformers, None);
    }

    #[test]
    fn test_is_valid() {
        let valid_libraries = ParsedLibraries {
            torch: Some("2.0.0".to_string()),
            xformers: None,
            diffusers: None,
            transformers: None,
        };
        assert!(LibrariesParser::is_valid(&valid_libraries));

        let invalid_libraries = ParsedLibraries {
            torch: None,
            xformers: None,
            diffusers: None,
            transformers: None,
        };
        assert!(!LibrariesParser::is_valid(&invalid_libraries));
    }

    #[test]
    fn test_get_summary() {
        let libraries = ParsedLibraries {
            torch: Some("2.0.0".to_string()),
            xformers: Some("0.0.22".to_string()),
            diffusers: Some("0.21.0".to_string()),
            transformers: Some("4.30.0".to_string()),
        };
        
        let summary = LibrariesParser::get_summary(&libraries);
        assert_eq!(summary, "torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0");
    }

    #[test]
    fn test_get_version() {
        let libraries = ParsedLibraries {
            torch: Some("2.0.0".to_string()),
            xformers: Some("0.0.22".to_string()),
            diffusers: Some("0.21.0".to_string()),
            transformers: Some("4.30.0".to_string()),
        };
        
        assert_eq!(LibrariesParser::get_version(&libraries, "torch"), Some("2.0.0".to_string()));
        assert_eq!(LibrariesParser::get_version(&libraries, "xformers"), Some("0.0.22".to_string()));
        assert_eq!(LibrariesParser::get_version(&libraries, "unknown"), None);
    }

    #[test]
    fn test_has_all_required() {
        let libraries = ParsedLibraries {
            torch: Some("2.0.0".to_string()),
            xformers: Some("0.0.22".to_string()),
            diffusers: Some("0.21.0".to_string()),
            transformers: Some("4.30.0".to_string()),
        };
        
        assert!(LibrariesParser::has_all_required(&libraries, &["torch", "xformers"]));
        assert!(LibrariesParser::has_all_required(&libraries, &["torch", "xformers", "diffusers"]));
        assert!(!LibrariesParser::has_all_required(&libraries, &["torch", "unknown"]));
    }
} 