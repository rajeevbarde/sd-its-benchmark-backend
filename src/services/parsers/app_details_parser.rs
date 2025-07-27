use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAppDetails {
    pub app_name: Option<String>,
    pub updated: Option<String>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

pub struct AppDetailsParser;

impl AppDetailsParser {
    /// Parse application details from the info string
    /// 
    /// The info string format is typically: "app:name updated:date hash:hash url:url"
    /// 
    /// # Arguments
    /// * `info_string` - The raw info string to parse
    /// 
    /// # Returns
    /// * `ParsedAppDetails` - Structured application details
    pub fn parse(info_string: &str) -> ParsedAppDetails {
        let parts = info_string.split(' ');
        let mut app_details = ParsedAppDetails {
            app_name: None,
            updated: None,
            hash: None,
            url: None,
        };

        for part in parts {
            let colon_index = match part.find(':') {
                Some(index) => index,
                None => continue,
            };

            let key = &part[..colon_index];
            let value = &part[colon_index + 1..];

            match key {
                "app" => app_details.app_name = Some(value.to_string()),
                "updated" => app_details.updated = Some(value.to_string()),
                "hash" => app_details.hash = Some(value.to_string()),
                "url" => app_details.url = Some(value.to_string()),
                _ => {
                    // Unknown key, skip it
                    continue;
                }
            }
        }

        app_details
    }

    /// Validate if the parsed app details contain valid data
    /// 
    /// # Arguments
    /// * `app_details` - The parsed app details to validate
    /// 
    /// # Returns
    /// * `bool` - True if the app details are valid
    pub fn is_valid(app_details: &ParsedAppDetails) -> bool {
        app_details.app_name.is_some() || 
        app_details.updated.is_some() || 
        app_details.hash.is_some() || 
        app_details.url.is_some()
    }

    /// Get a summary of the parsed app details
    /// 
    /// # Arguments
    /// * `app_details` - The parsed app details
    /// 
    /// # Returns
    /// * `String` - A summary string
    pub fn get_summary(app_details: &ParsedAppDetails) -> String {
        let mut parts = Vec::new();
        
        if let Some(app_name) = &app_details.app_name {
            parts.push(format!("app:{}", app_name));
        }
        
        if let Some(updated) = &app_details.updated {
            parts.push(format!("updated:{}", updated));
        }
        
        if let Some(hash) = &app_details.hash {
            parts.push(format!("hash:{}", hash));
        }
        
        if let Some(url) = &app_details.url {
            parts.push(format!("url:{}", url));
        }
        
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_app_details_basic() {
        let info_string = "app:test-app updated:2024-01-01 hash:abc123 url:https://example.com";
        let result = AppDetailsParser::parse(info_string);
        
        assert_eq!(result.app_name, Some("test-app".to_string()));
        assert_eq!(result.updated, Some("2024-01-01".to_string()));
        assert_eq!(result.hash, Some("abc123".to_string()));
        assert_eq!(result.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parse_app_details_partial() {
        let info_string = "app:test-app updated:2024-01-01";
        let result = AppDetailsParser::parse(info_string);
        
        assert_eq!(result.app_name, Some("test-app".to_string()));
        assert_eq!(result.updated, Some("2024-01-01".to_string()));
        assert_eq!(result.hash, None);
        assert_eq!(result.url, None);
    }

    #[test]
    fn test_parse_app_details_empty() {
        let info_string = "";
        let result = AppDetailsParser::parse(info_string);
        
        assert_eq!(result.app_name, None);
        assert_eq!(result.updated, None);
        assert_eq!(result.hash, None);
        assert_eq!(result.url, None);
    }

    #[test]
    fn test_parse_app_details_unknown_keys() {
        let info_string = "app:test-app unknown:value updated:2024-01-01";
        let result = AppDetailsParser::parse(info_string);
        
        assert_eq!(result.app_name, Some("test-app".to_string()));
        assert_eq!(result.updated, Some("2024-01-01".to_string()));
        assert_eq!(result.hash, None);
        assert_eq!(result.url, None);
    }

    #[test]
    fn test_is_valid() {
        let valid_details = ParsedAppDetails {
            app_name: Some("test".to_string()),
            updated: None,
            hash: None,
            url: None,
        };
        assert!(AppDetailsParser::is_valid(&valid_details));

        let invalid_details = ParsedAppDetails {
            app_name: None,
            updated: None,
            hash: None,
            url: None,
        };
        assert!(!AppDetailsParser::is_valid(&invalid_details));
    }

    #[test]
    fn test_get_summary() {
        let app_details = ParsedAppDetails {
            app_name: Some("test-app".to_string()),
            updated: Some("2024-01-01".to_string()),
            hash: Some("abc123".to_string()),
            url: Some("https://example.com".to_string()),
        };
        
        let summary = AppDetailsParser::get_summary(&app_details);
        assert_eq!(summary, "app:test-app updated:2024-01-01 hash:abc123 url:https://example.com");
    }
} 