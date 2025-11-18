/// Advanced filtering system for metadata queries
///
/// Supports compound queries (AND/OR/NOT), range queries, and regex matching

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter condition for metadata queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Exact match: field == value
    Eq { field: String, value: String },
    /// Not equal: field != value
    Ne { field: String, value: String },
    /// Greater than: field > value (numeric/string comparison)
    Gt { field: String, value: String },
    /// Greater than or equal: field >= value
    Gte { field: String, value: String },
    /// Less than: field < value
    Lt { field: String, value: String },
    /// Less than or equal: field <= value
    Lte { field: String, value: String },
    /// Range query: min <= field <= max
    Range {
        field: String,
        min: String,
        max: String,
    },
    /// Regex pattern match
    Regex { field: String, pattern: String },
    /// Field exists
    Exists { field: String },
    /// Field does not exist
    NotExists { field: String },
    /// AND compound query (all conditions must match)
    And { conditions: Vec<FilterCondition> },
    /// OR compound query (at least one condition must match)
    Or { conditions: Vec<FilterCondition> },
    /// NOT query (condition must not match)
    Not { condition: Box<FilterCondition> },
}

impl FilterCondition {
    /// Create an equality filter
    pub fn eq(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a not-equal filter
    pub fn ne(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Ne {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than filter
    pub fn gt(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Gt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than-or-equal filter
    pub fn gte(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Gte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than filter
    pub fn lt(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Lt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than-or-equal filter
    pub fn lte(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Lte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a range filter
    pub fn range(
        field: impl Into<String>,
        min: impl Into<String>,
        max: impl Into<String>,
    ) -> Self {
        Self::Range {
            field: field.into(),
            min: min.into(),
            max: max.into(),
        }
    }

    /// Create a regex filter
    pub fn regex(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::Regex {
            field: field.into(),
            pattern: pattern.into(),
        }
    }

    /// Create an exists filter
    pub fn exists(field: impl Into<String>) -> Self {
        Self::Exists {
            field: field.into(),
        }
    }

    /// Create a not-exists filter
    pub fn not_exists(field: impl Into<String>) -> Self {
        Self::NotExists {
            field: field.into(),
        }
    }

    /// Create an AND filter
    pub fn and(conditions: Vec<FilterCondition>) -> Self {
        Self::And { conditions }
    }

    /// Create an OR filter
    pub fn or(conditions: Vec<FilterCondition>) -> Self {
        Self::Or { conditions }
    }

    /// Create a NOT filter
    pub fn not(condition: FilterCondition) -> Self {
        Self::Not {
            condition: Box::new(condition),
        }
    }

    /// Evaluate filter against metadata
    pub fn matches(&self, metadata: &HashMap<String, String>) -> bool {
        match self {
            FilterCondition::Eq { field, value } => metadata.get(field) == Some(value),

            FilterCondition::Ne { field, value } => metadata.get(field) != Some(value),

            FilterCondition::Gt { field, value } => {
                if let Some(field_value) = metadata.get(field) {
                    // Try numeric comparison first
                    if let (Ok(a), Ok(b)) = (field_value.parse::<f64>(), value.parse::<f64>()) {
                        a > b
                    } else {
                        // Fall back to string comparison
                        field_value > value
                    }
                } else {
                    false
                }
            }

            FilterCondition::Gte { field, value } => {
                if let Some(field_value) = metadata.get(field) {
                    if let (Ok(a), Ok(b)) = (field_value.parse::<f64>(), value.parse::<f64>()) {
                        a >= b
                    } else {
                        field_value >= value
                    }
                } else {
                    false
                }
            }

            FilterCondition::Lt { field, value } => {
                if let Some(field_value) = metadata.get(field) {
                    if let (Ok(a), Ok(b)) = (field_value.parse::<f64>(), value.parse::<f64>()) {
                        a < b
                    } else {
                        field_value < value
                    }
                } else {
                    false
                }
            }

            FilterCondition::Lte { field, value } => {
                if let Some(field_value) = metadata.get(field) {
                    if let (Ok(a), Ok(b)) = (field_value.parse::<f64>(), value.parse::<f64>()) {
                        a <= b
                    } else {
                        field_value <= value
                    }
                } else {
                    false
                }
            }

            FilterCondition::Range { field, min, max } => {
                if let Some(field_value) = metadata.get(field) {
                    if let (Ok(v), Ok(min_v), Ok(max_v)) = (
                        field_value.parse::<f64>(),
                        min.parse::<f64>(),
                        max.parse::<f64>(),
                    ) {
                        v >= min_v && v <= max_v
                    } else {
                        // String comparison
                        field_value >= min && field_value <= max
                    }
                } else {
                    false
                }
            }

            FilterCondition::Regex { field, pattern } => {
                if let Some(field_value) = metadata.get(field) {
                    if let Ok(re) = Regex::new(pattern) {
                        re.is_match(field_value)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            FilterCondition::Exists { field } => metadata.contains_key(field),

            FilterCondition::NotExists { field } => !metadata.contains_key(field),

            FilterCondition::And { conditions } => {
                conditions.iter().all(|c| c.matches(metadata))
            }

            FilterCondition::Or { conditions } => conditions.iter().any(|c| c.matches(metadata)),

            FilterCondition::Not { condition } => !condition.matches(metadata),
        }
    }
}

/// Filter builder for constructing complex queries
#[derive(Debug, Clone, Default)]
pub struct FilterBuilder {
    conditions: Vec<FilterCondition>,
}

impl FilterBuilder {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an equality condition
    pub fn eq(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.conditions
            .push(FilterCondition::eq(field, value));
        self
    }

    /// Add a not-equal condition
    pub fn ne(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.conditions
            .push(FilterCondition::ne(field, value));
        self
    }

    /// Add a greater-than condition
    pub fn gt(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.conditions
            .push(FilterCondition::gt(field, value));
        self
    }

    /// Add a range condition
    pub fn range(
        mut self,
        field: impl Into<String>,
        min: impl Into<String>,
        max: impl Into<String>,
    ) -> Self {
        self.conditions
            .push(FilterCondition::range(field, min, max));
        self
    }

    /// Add a regex condition
    pub fn regex(mut self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.conditions
            .push(FilterCondition::regex(field, pattern));
        self
    }

    /// Build an AND filter (all conditions must match)
    pub fn build_and(self) -> FilterCondition {
        if self.conditions.len() == 1 {
            self.conditions.into_iter().next().unwrap()
        } else {
            FilterCondition::And {
                conditions: self.conditions,
            }
        }
    }

    /// Build an OR filter (at least one condition must match)
    pub fn build_or(self) -> FilterCondition {
        if self.conditions.len() == 1 {
            self.conditions.into_iter().next().unwrap()
        } else {
            FilterCondition::Or {
                conditions: self.conditions,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_metadata() -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "Alice".to_string());
        metadata.insert("age".to_string(), "30".to_string());
        metadata.insert("city".to_string(), "New York".to_string());
        metadata.insert("score".to_string(), "85.5".to_string());
        metadata
    }

    #[test]
    fn test_eq_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::eq("name", "Alice");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::eq("name", "Bob");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_ne_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::ne("name", "Bob");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::ne("name", "Alice");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_numeric_gt_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::gt("age", "25");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::gt("age", "35");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_range_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::range("age", "25", "35");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::range("age", "40", "50");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_regex_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::regex("name", "^A");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::regex("name", "^B");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_exists_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::exists("name");
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::exists("email");
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_and_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::and(vec![
            FilterCondition::eq("name", "Alice"),
            FilterCondition::gt("age", "25"),
        ]);
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::and(vec![
            FilterCondition::eq("name", "Alice"),
            FilterCondition::gt("age", "35"),
        ]);
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_or_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::or(vec![
            FilterCondition::eq("name", "Bob"),
            FilterCondition::gt("age", "25"),
        ]);
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::or(vec![
            FilterCondition::eq("name", "Bob"),
            FilterCondition::gt("age", "35"),
        ]);
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_not_filter() {
        let metadata = create_metadata();
        let filter = FilterCondition::not(FilterCondition::eq("name", "Bob"));
        assert!(filter.matches(&metadata));

        let filter = FilterCondition::not(FilterCondition::eq("name", "Alice"));
        assert!(!filter.matches(&metadata));
    }

    #[test]
    fn test_filter_builder() {
        let metadata = create_metadata();
        let filter = FilterBuilder::new()
            .eq("name", "Alice")
            .gt("age", "25")
            .build_and();

        assert!(filter.matches(&metadata));
    }

    #[test]
    fn test_complex_filter() {
        let metadata = create_metadata();
        // (name == "Alice" AND age > 25) OR (city == "Boston")
        let filter = FilterCondition::or(vec![
            FilterCondition::and(vec![
                FilterCondition::eq("name", "Alice"),
                FilterCondition::gt("age", "25"),
            ]),
            FilterCondition::eq("city", "Boston"),
        ]);

        assert!(filter.matches(&metadata));
    }
}
