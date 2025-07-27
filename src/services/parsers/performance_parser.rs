use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPerformanceData {
    pub its_values: Vec<f64>,
    pub avg_its: Option<f64>,
    pub raw_vram_usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub min_its: f64,
    pub max_its: f64,
    pub avg_its: f64,
    pub count: usize,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            min_its: 0.0,
            max_its: 0.0,
            avg_its: 0.0,
            count: 0,
        }
    }
}

pub struct PerformanceParser;

impl PerformanceParser {
    /// Parse performance data from vram_usage string
    /// 
    /// The vram_usage string format is typically: "1.5/2.1/1.8" (ITS values)
    /// 
    /// # Arguments
    /// * `vram_usage_string` - The raw vram_usage string to parse
    /// 
    /// # Returns
    /// * `ParsedPerformanceData` - Structured performance data
    pub fn parse(vram_usage_string: &str) -> ParsedPerformanceData {
        let its_values: Vec<f64> = vram_usage_string
            .split('/')
            .map(|value| value.trim().parse::<f64>().ok())
            .filter_map(|value| value)
            .collect();

        let avg_its = if !its_values.is_empty() {
            Some(its_values.iter().sum::<f64>() / its_values.len() as f64)
        } else {
            None
        };

        ParsedPerformanceData {
            its_values,
            avg_its,
            raw_vram_usage: vram_usage_string.to_string(),
        }
    }

    /// Validate if the parsed performance data contains valid data
    /// 
    /// # Arguments
    /// * `performance_data` - The parsed performance data to validate
    /// 
    /// # Returns
    /// * `bool` - True if the performance data is valid
    pub fn is_valid(performance_data: &ParsedPerformanceData) -> bool {
        !performance_data.its_values.is_empty() && performance_data.avg_its.is_some()
    }

    /// Get performance statistics
    /// 
    /// # Arguments
    /// * `performance_data` - The parsed performance data
    /// 
    /// # Returns
    /// * `PerformanceStats` - Performance statistics
    pub fn get_statistics(performance_data: &ParsedPerformanceData) -> PerformanceStats {
        if performance_data.its_values.is_empty() {
            return PerformanceStats::default();
        }

        let values = &performance_data.its_values;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        PerformanceStats {
            min_its: min,
            max_its: max,
            avg_its: avg,
            count: values.len(),
        }
    }

    /// Get a summary of the parsed performance data
    /// 
    /// # Arguments
    /// * `performance_data` - The parsed performance data
    /// 
    /// # Returns
    /// * `String` - A summary string
    pub fn get_summary(performance_data: &ParsedPerformanceData) -> String {
        if performance_data.its_values.is_empty() {
            return "No valid ITS values".to_string();
        }

        let stats = Self::get_statistics(performance_data);
        format!(
            "ITS: {} (avg: {:.2}, min: {:.2}, max: {:.2})",
            performance_data.raw_vram_usage,
            stats.avg_its,
            stats.min_its,
            stats.max_its
        )
    }

    /// Enhanced validation with specific error types
    /// 
    /// # Arguments
    /// * `vram_usage_string` - The vram_usage string to validate
    /// 
    /// # Returns
    /// * `Result<ParsedPerformanceData, ParsingError>` - Parsed data or error
    pub fn validate_with_errors(vram_usage_string: &str) -> Result<ParsedPerformanceData, ParsingError> {
        if vram_usage_string.trim().is_empty() {
            return Err(ParsingError::EmptyInput);
        }

        let performance_data = Self::parse(vram_usage_string);
        
        if performance_data.its_values.is_empty() {
            return Err(ParsingError::NoValidValues);
        }

        // Check for reasonable ITS values (e.g., between 0.1 and 100)
        for value in &performance_data.its_values {
            if *value < 0.1 || *value > 100.0 {
                return Err(ParsingError::InvalidValue(*value));
            }
        }

        Ok(performance_data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("Empty input string")]
    EmptyInput,
    #[error("No valid numeric values found")]
    NoValidValues,
    #[error("Invalid ITS value: {0}")]
    InvalidValue(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_performance_data_basic() {
        let result = PerformanceParser::parse("1.5/2.1/1.8");
        assert_eq!(result.its_values, vec![1.5, 2.1, 1.8]);
        assert_eq!(result.avg_its, Some(1.8)); // (1.5 + 2.1 + 1.8) / 3 = 1.8
        assert_eq!(result.raw_vram_usage, "1.5/2.1/1.8");
    }

    #[test]
    fn test_parse_performance_data_single_value() {
        let result = PerformanceParser::parse("2.5");
        assert_eq!(result.its_values, vec![2.5]);
        assert_eq!(result.avg_its, Some(2.5));
    }

    #[test]
    fn test_parse_performance_data_with_invalid_values() {
        let result = PerformanceParser::parse("1.5/invalid/2.1");
        assert_eq!(result.its_values, vec![1.5, 2.1]);
        assert_eq!(result.avg_its, Some(1.8)); // (1.5 + 2.1) / 2 = 1.8
    }

    #[test]
    fn test_parse_performance_data_empty() {
        let result = PerformanceParser::parse("");
        assert_eq!(result.its_values, vec![] as Vec<f64>);
        assert_eq!(result.avg_its, None);
    }

    #[test]
    fn test_parse_performance_data_whitespace() {
        let result = PerformanceParser::parse(" 1.5 / 2.1 / 1.8 ");
        assert_eq!(result.its_values, vec![1.5, 2.1, 1.8]);
        assert_eq!(result.avg_its, Some(1.8));
    }

    #[test]
    fn test_is_valid() {
        let valid_data = ParsedPerformanceData {
            its_values: vec![1.5, 2.1, 1.8],
            avg_its: Some(1.8),
            raw_vram_usage: "1.5/2.1/1.8".to_string(),
        };
        assert!(PerformanceParser::is_valid(&valid_data));

        let invalid_data = ParsedPerformanceData {
            its_values: vec![],
            avg_its: None,
            raw_vram_usage: "".to_string(),
        };
        assert!(!PerformanceParser::is_valid(&invalid_data));
    }

    #[test]
    fn test_get_statistics() {
        let data = ParsedPerformanceData {
            its_values: vec![1.5, 2.1, 1.8],
            avg_its: Some(1.8),
            raw_vram_usage: "1.5/2.1/1.8".to_string(),
        };
        let stats = PerformanceParser::get_statistics(&data);
        assert_eq!(stats.min_its, 1.5);
        assert_eq!(stats.max_its, 2.1);
        assert_eq!(stats.avg_its, 1.8);
        assert_eq!(stats.count, 3);
    }

    #[test]
    fn test_get_summary() {
        let data = ParsedPerformanceData {
            its_values: vec![1.5, 2.1, 1.8],
            avg_its: Some(1.8),
            raw_vram_usage: "1.5/2.1/1.8".to_string(),
        };
        let summary = PerformanceParser::get_summary(&data);
        assert!(summary.contains("1.5/2.1/1.8"));
        assert!(summary.contains("avg: 1.80"));
        assert!(summary.contains("min: 1.50"));
        assert!(summary.contains("max: 2.10"));
    }

    #[test]
    fn test_validate_with_errors_valid() {
        let result = PerformanceParser::validate_with_errors("1.5/2.1/1.8");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_errors_empty() {
        let result = PerformanceParser::validate_with_errors("");
        assert!(matches!(result, Err(ParsingError::EmptyInput)));
    }

    #[test]
    fn test_validate_with_errors_no_valid_values() {
        let result = PerformanceParser::validate_with_errors("invalid/not_a_number");
        assert!(matches!(result, Err(ParsingError::NoValidValues)));
    }

    #[test]
    fn test_validate_with_errors_invalid_value() {
        let result = PerformanceParser::validate_with_errors("0.05/1.5"); // 0.05 < 0.1
        assert!(matches!(result, Err(ParsingError::InvalidValue(0.05))));
    }
} 