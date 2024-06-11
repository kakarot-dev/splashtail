use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalColumnType {
    /// A single valued column (scalar)
    Scalar {
        /// The value type
        column_type: CanonicalInnerColumnType,
    },
    /// An array column
    Array {
        /// The inner type of the array
        inner: CanonicalInnerColumnType,
    },
}

impl From<super::config_opts::ColumnType> for CanonicalColumnType {
    fn from(column_type: super::config_opts::ColumnType) -> Self {
        match column_type {
            super::config_opts::ColumnType::Scalar { column_type } => CanonicalColumnType::Scalar {
                column_type: column_type.into(),
            },
            super::config_opts::ColumnType::Array { inner } => CanonicalColumnType::Array {
                inner: inner.into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalInnerColumnType {
    Uuid {},
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allowed_values: Vec<String>,
    },
    Timestamp {},
    TimestampTz {},
    Integer {},
    Float {},
    BitFlag {
        /// The bit flag values
        values: indexmap::IndexMap<String, i64>,
    },
    Boolean {},
    User {},
    Channel {},
    Role {},
    Emoji {},
    Message {},
    Json {},
}

impl From<super::config_opts::InnerColumnType> for CanonicalInnerColumnType {
    fn from(column_type: super::config_opts::InnerColumnType) -> Self {
        match column_type {
            super::config_opts::InnerColumnType::Uuid {} => CanonicalInnerColumnType::Uuid {},
            super::config_opts::InnerColumnType::String {
                min_length,
                max_length,
                allowed_values,
            } => CanonicalInnerColumnType::String {
                min_length,
                max_length,
                allowed_values: allowed_values.iter().map(|s| s.to_string()).collect(),
            },
            super::config_opts::InnerColumnType::Timestamp {} => {
                CanonicalInnerColumnType::Timestamp {}
            }
            super::config_opts::InnerColumnType::TimestampTz {} => {
                CanonicalInnerColumnType::TimestampTz {}
            }
            super::config_opts::InnerColumnType::Integer {} => CanonicalInnerColumnType::Integer {},
            super::config_opts::InnerColumnType::Float {} => CanonicalInnerColumnType::Float {},
            super::config_opts::InnerColumnType::BitFlag { values } => {
                CanonicalInnerColumnType::BitFlag {
                    values: values
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect::<indexmap::IndexMap<String, i64>>(),
                }
            }
            super::config_opts::InnerColumnType::Boolean {} => CanonicalInnerColumnType::Boolean {},
            super::config_opts::InnerColumnType::User {} => CanonicalInnerColumnType::User {},
            super::config_opts::InnerColumnType::Channel {} => CanonicalInnerColumnType::Channel {},
            super::config_opts::InnerColumnType::Role {} => CanonicalInnerColumnType::Role {},
            super::config_opts::InnerColumnType::Emoji {} => CanonicalInnerColumnType::Emoji {},
            super::config_opts::InnerColumnType::Message {} => CanonicalInnerColumnType::Message {},
            super::config_opts::InnerColumnType::Json {} => CanonicalInnerColumnType::Json {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalColumnSuggestion {
    Static {
        suggestions: Vec<String>,
    },
    Dynamic {
        table_name: String,
        column_name: String,
    },
    None {},
}

impl From<super::config_opts::ColumnSuggestion> for CanonicalColumnSuggestion {
    fn from(column_suggestion: super::config_opts::ColumnSuggestion) -> Self {
        match column_suggestion {
            super::config_opts::ColumnSuggestion::Static { suggestions } => {
                CanonicalColumnSuggestion::Static {
                    suggestions: suggestions.iter().map(|s| s.to_string()).collect(),
                }
            }
            super::config_opts::ColumnSuggestion::Dynamic {
                table_name,
                column_name,
            } => CanonicalColumnSuggestion::Dynamic {
                table_name: table_name.to_string(),
                column_name: column_name.to_string(),
            },
            super::config_opts::ColumnSuggestion::None {} => CanonicalColumnSuggestion::None {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalColumn {
    /// The ID of the column
    pub id: String,

    /// The friendly name of the column
    pub name: String,

    /// The type of the column
    pub column_type: CanonicalColumnType,

    /// Whether or not the column is nullable
    pub nullable: bool,

    /// Suggestions to display
    pub suggestions: CanonicalColumnSuggestion,

    /// Whether or not the column is unique
    pub unique: bool,

    /// The read-only status of each operation
    ///
    /// Only applies to create and update
    pub readonly: indexmap::IndexMap<CanonicalOperationType, bool>,
}

impl From<super::config_opts::Column> for CanonicalColumn {
    fn from(column: super::config_opts::Column) -> Self {
        Self {
            id: column.id.to_string(),
            name: column.name.to_string(),
            column_type: column.column_type.into(),
            nullable: column.nullable,
            suggestions: column.suggestions.into(),
            unique: column.unique,
            readonly: column
                .readonly
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOperationSpecific {
    /// The corresponding command for ACL purposes
    pub corresponding_command: String,

    /// Which column ids should be usable for this operation
    ///
    /// E.g, create does not need to show created_at or id while view should
    ///
    /// If empty, all columns are usable
    pub column_ids: Vec<String>,

    /// Any columns to set. For example, a last_updated column should be set on update
    ///
    /// Variables:
    /// - {user_id} => the user id of the user running the operation
    /// - {now} => the current timestamp
    ///
    /// Note: only applies to create, update and delete
    ///
    /// Key should be of form `table_name.column_name` and value should be the value to set
    pub columns_to_set: indexmap::IndexMap<String, String>,
}

impl From<super::config_opts::OperationSpecific> for CanonicalOperationSpecific {
    fn from(operation_specific: super::config_opts::OperationSpecific) -> Self {
        Self {
            corresponding_command: operation_specific.corresponding_command.to_string(),
            column_ids: operation_specific
                .column_ids
                .iter()
                .map(|c| c.to_string())
                .collect(),
            columns_to_set: operation_specific
                .columns_to_set
                .iter()
                .map(|(k, v)| (format!("{}.{}", k.0, k.1), v.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalOperationType {
    #[serde(rename = "View")]
    View,
    #[serde(rename = "Create")]
    Create,
    #[serde(rename = "Update")]
    Update,
    #[serde(rename = "Delete")]
    Delete,
}

impl From<super::config_opts::OperationType> for CanonicalOperationType {
    fn from(operation_type: super::config_opts::OperationType) -> Self {
        match operation_type {
            super::config_opts::OperationType::View => CanonicalOperationType::View,
            super::config_opts::OperationType::Create => CanonicalOperationType::Create,
            super::config_opts::OperationType::Update => CanonicalOperationType::Update,
            super::config_opts::OperationType::Delete => CanonicalOperationType::Delete,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalConfigOption {
    /// The ID of the option
    pub id: String,

    /// The name of the option
    pub name: String,

    /// The description of the option
    pub description: String,

    /// The table name for the config option
    pub table: String,

    /// The column name refering to the guild id of the config option    
    pub guild_id: String,

    /// The primary key of the table
    pub primary_key: String,

    /// The columns for this option
    pub columns: Vec<CanonicalColumn>,

    /// Operation specific data
    pub operations: indexmap::IndexMap<CanonicalOperationType, CanonicalOperationSpecific>,
}

/// Given a module, return its canonical representation
impl From<super::config_opts::ConfigOption> for CanonicalConfigOption {
    fn from(module: super::config_opts::ConfigOption) -> Self {
        Self {
            id: module.id.to_string(),
            table: module.table.to_string(),
            guild_id: module.guild_id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            columns: module.columns.into_iter().map(|c| c.into()).collect(),
            primary_key: module.primary_key.to_string(),
            operations: module
                .operations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}
