//! Tag filtering for server-side event filtering.
//!
//! Allows clients to specify filters when subscribing to channels,
//! so that the server only sends events that match the filter criteria.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Filter operation for tag filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum FilterOp {
    /// Equality check: field == value
    #[serde(rename = "$eq")]
    Eq { field: String, value: String },

    /// Not equal: field != value
    #[serde(rename = "$neq")]
    Neq { field: String, value: String },

    /// Less than: field < value
    #[serde(rename = "$lt")]
    Lt { field: String, value: String },

    /// Less than or equal: field <= value
    #[serde(rename = "$lte")]
    Lte { field: String, value: String },

    /// Greater than: field > value
    #[serde(rename = "$gt")]
    Gt { field: String, value: String },

    /// Greater than or equal: field >= value
    #[serde(rename = "$gte")]
    Gte { field: String, value: String },

    /// In set: field in [values]
    #[serde(rename = "$in")]
    In { field: String, values: Vec<String> },

    /// Not in set: field not in [values]
    #[serde(rename = "$nin")]
    NotIn { field: String, values: Vec<String> },

    /// Field exists
    #[serde(rename = "$exists")]
    Exists { field: String },

    /// Field does not exist
    #[serde(rename = "$nexists")]
    NotExists { field: String },

    /// Logical AND of multiple filters
    #[serde(rename = "$and")]
    And { filters: Vec<FilterOp> },

    /// Logical OR of multiple filters
    #[serde(rename = "$or")]
    Or { filters: Vec<FilterOp> },
}

impl FilterOp {
    /// Create an equality filter
    pub fn eq(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a not-equal filter
    pub fn neq(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Neq {
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

    /// Create an IN filter
    pub fn in_set(field: impl Into<String>, values: Vec<String>) -> Self {
        Self::In {
            field: field.into(),
            values,
        }
    }

    /// Create a NOT IN filter
    pub fn not_in(field: impl Into<String>, values: Vec<String>) -> Self {
        Self::NotIn {
            field: field.into(),
            values,
        }
    }

    /// Create an EXISTS filter
    pub fn exists(field: impl Into<String>) -> Self {
        Self::Exists {
            field: field.into(),
        }
    }

    /// Create a NOT EXISTS filter
    pub fn not_exists(field: impl Into<String>) -> Self {
        Self::NotExists {
            field: field.into(),
        }
    }

    /// Create an AND filter
    pub fn and(filters: Vec<FilterOp>) -> Self {
        Self::And { filters }
    }

    /// Create an OR filter
    pub fn or(filters: Vec<FilterOp>) -> Self {
        Self::Or { filters }
    }

    /// Convert to JSON value for protocol
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Validate the filter
    pub fn validate(&self) -> Result<(), FilterValidationError> {
        match self {
            Self::Eq { field, .. }
            | Self::Neq { field, .. }
            | Self::Lt { field, .. }
            | Self::Lte { field, .. }
            | Self::Gt { field, .. }
            | Self::Gte { field, .. }
            | Self::Exists { field }
            | Self::NotExists { field } => {
                if field.is_empty() {
                    return Err(FilterValidationError::EmptyField);
                }
            }
            Self::In { field, values } | Self::NotIn { field, values } => {
                if field.is_empty() {
                    return Err(FilterValidationError::EmptyField);
                }
                if values.is_empty() {
                    return Err(FilterValidationError::EmptyValueSet);
                }
            }
            Self::And { filters } | Self::Or { filters } => {
                if filters.is_empty() {
                    return Err(FilterValidationError::EmptyFilterList);
                }
                for filter in filters {
                    filter.validate()?;
                }
            }
        }
        Ok(())
    }
}

/// Filter validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterValidationError {
    EmptyField,
    EmptyValueSet,
    EmptyFilterList,
}

impl std::fmt::Display for FilterValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField => write!(f, "Field name cannot be empty"),
            Self::EmptyValueSet => write!(f, "Value set cannot be empty"),
            Self::EmptyFilterList => write!(f, "Filter list cannot be empty"),
        }
    }
}

impl std::error::Error for FilterValidationError {}

/// Builder for creating complex filters
pub struct FilterBuilder {
    filters: Vec<FilterOp>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self { filters: vec![] }
    }

    pub fn eq(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.push(FilterOp::eq(field, value));
        self
    }

    pub fn neq(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.push(FilterOp::neq(field, value));
        self
    }

    pub fn in_set(mut self, field: impl Into<String>, values: Vec<String>) -> Self {
        self.filters.push(FilterOp::in_set(field, values));
        self
    }

    pub fn build_and(self) -> FilterOp {
        if self.filters.len() == 1 {
            self.filters.into_iter().next().unwrap()
        } else {
            FilterOp::and(self.filters)
        }
    }

    pub fn build_or(self) -> FilterOp {
        if self.filters.len() == 1 {
            self.filters.into_iter().next().unwrap()
        } else {
            FilterOp::or(self.filters)
        }
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// FFI-safe filter builder for use across language boundaries
#[cfg(all(not(feature = "wasm"), feature = "uniffi"))]
#[derive(uniffi::Object)]
pub struct FfiFilterBuilder {
    inner: FilterBuilder,
}

#[cfg(all(not(feature = "wasm"), feature = "uniffi"))]
#[uniffi::export]
impl FfiFilterBuilder {
    /// Create a new filter builder
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: FilterBuilder::new(),
        }
    }

    /// Add an equality filter
    pub fn eq(self: std::sync::Arc<Self>, field: String, value: String) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            inner: self.inner.clone().eq(field, value),
        })
    }

    /// Add a not-equal filter
    pub fn neq(self: std::sync::Arc<Self>, field: String, value: String) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            inner: self.inner.clone().neq(field, value),
        })
    }

    /// Add an IN filter
    pub fn in_set(
        self: std::sync::Arc<Self>,
        field: String,
        values: Vec<String>,
    ) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            inner: self.inner.clone().in_set(field, values),
        })
    }

    /// Build with AND logic
    pub fn build_and(&self) -> FilterOp {
        self.inner.clone().build_and()
    }

    /// Build with OR logic
    pub fn build_or(&self) -> FilterOp {
        self.inner.clone().build_or()
    }
}

#[cfg(not(feature = "wasm"))]
impl Clone for FilterBuilder {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_filter() {
        let filter = FilterOp::eq("type", "goal");
        let json = filter.to_json();
        assert!(json.is_object());
    }

    #[test]
    fn test_compound_filter() {
        let filter = FilterOp::and(vec![
            FilterOp::eq("type", "goal"),
            FilterOp::in_set("team", vec!["home".to_string(), "away".to_string()]),
        ]);
        filter.validate().unwrap();
    }

    #[test]
    fn test_builder() {
        let filter = FilterBuilder::new()
            .eq("priority", "high")
            .in_set("status", vec!["active".to_string(), "pending".to_string()])
            .build_and();
        filter.validate().unwrap();
    }

    #[test]
    fn test_validation_empty_field() {
        let filter = FilterOp::eq("", "value");
        assert!(matches!(
            filter.validate(),
            Err(FilterValidationError::EmptyField)
        ));
    }
}
