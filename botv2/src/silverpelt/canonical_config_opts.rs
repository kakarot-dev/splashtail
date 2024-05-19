use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalColumnType {
    Uuid {},
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allowed_values: Vec<String>,
    },
    Timestamp {},
    Integer {},
    BitFlag {
        /// The bit flag values
        values: indexmap::IndexMap<String, u64>,
    },
    Boolean {},
    User {},
    Channel {},
    Role {},
    Emoji {},
    Message {},
}

impl From<super::config_opts::ColumnType> for CanonicalColumnType {
    fn from(column_type: super::config_opts::ColumnType) -> Self {
        match column_type {
            super::config_opts::ColumnType::Uuid {} => CanonicalColumnType::Uuid {},
            super::config_opts::ColumnType::String {
                min_length,
                max_length,
                allowed_values,
            } => CanonicalColumnType::String {
                min_length,
                max_length,
                allowed_values: allowed_values.iter().map(|s| s.to_string()).collect(),
            },
            super::config_opts::ColumnType::Timestamp {} => CanonicalColumnType::Timestamp {},
            super::config_opts::ColumnType::Integer {} => CanonicalColumnType::Integer {},
            super::config_opts::ColumnType::BitFlag { values } => CanonicalColumnType::BitFlag {
                values: values
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect::<indexmap::IndexMap<String, u64>>(),
            },
            super::config_opts::ColumnType::Boolean {} => CanonicalColumnType::Boolean {},
            super::config_opts::ColumnType::User {} => CanonicalColumnType::User {},
            super::config_opts::ColumnType::Channel {} => CanonicalColumnType::Channel {},
            super::config_opts::ColumnType::Role {} => CanonicalColumnType::Role {},
            super::config_opts::ColumnType::Emoji {} => CanonicalColumnType::Emoji {},
            super::config_opts::ColumnType::Message {} => CanonicalColumnType::Message {},
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
    None,
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
            super::config_opts::ColumnSuggestion::None => CanonicalColumnSuggestion::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalColumnAction {
    /// Adds a column/row to the state map
    CollectColumnToMap {
        /// The table to use
        table: String,

        /// The column to fetch
        column: String,

        /// The key to store the row under
        key: String,

        /// Whether to fetch all or only one rows
        fetch_all: bool,
    },
    /// Executes a lua script, the *last* result will be stored in result
    ///
    /// Note that the lua script must return true or false
    ExecLuaScript {
        script: String,
        on_success: Vec<CanonicalColumnAction>,
        on_failure: Vec<CanonicalColumnAction>,
    },
    IpcPerModuleFunction {
        /// The module to use
        module: String,

        /// The function to execute
        function: String,

        /// The arguments to pass to the function
        ///
        /// In syntax: {key_on_function} -> {key_on_map}
        arguments: indexmap::IndexMap<String, String>,
    },
    /// Return an error thus failing the configuration view/create/update/delete
    Error {
        /// The error message to return, {key_on_map} can be used here in the message
        message: String,
    },
}

impl From<super::config_opts::ColumnAction> for CanonicalColumnAction {
    fn from(column_action: super::config_opts::ColumnAction) -> Self {
        match column_action {
            super::config_opts::ColumnAction::CollectColumnToMap {
                table,
                column,
                key,
                fetch_all,
            } => CanonicalColumnAction::CollectColumnToMap {
                table: table.to_string(),
                column: column.to_string(),
                key: key.to_string(),
                fetch_all,
            },
            super::config_opts::ColumnAction::ExecLuaScript {
                script,
                on_success,
                on_failure,
            } => CanonicalColumnAction::ExecLuaScript {
                script: script.to_string(),
                on_success: on_success.into_iter().map(|c| c.into()).collect(),
                on_failure: on_failure.into_iter().map(|c| c.into()).collect(),
            },
            super::config_opts::ColumnAction::IpcPerModuleFunction {
                module,
                function,
                arguments,
            } => CanonicalColumnAction::IpcPerModuleFunction {
                module: module.to_string(),
                function: function.to_string(),
                arguments: arguments
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            super::config_opts::ColumnAction::Error { message } => CanonicalColumnAction::Error {
                message: message.to_string(),
            },
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

    /// Whether or not the column is an array
    pub array: bool,

    /// The read-only status of each operation
    ///
    /// Only applies to create and update
    pub readonly: indexmap::IndexMap<CanonicalOperationType, bool>,

    /// Pre-execute checks
    pub pre_checks: indexmap::IndexMap<CanonicalOperationType, Vec<CanonicalColumnAction>>,

    /// Default pre-execute checks to fallback to if the operation specific ones are not set
    pub default_pre_checks: Vec<CanonicalColumnAction>,
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
            array: column.array,
            readonly: column
                .readonly
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
            pre_checks: column
                .pre_checks
                .into_iter()
                .map(|(k, v)| (k.into(), v.into_iter().map(|c| c.into()).collect()))
                .collect(),
            default_pre_checks: column
                .default_pre_checks
                .into_iter()
                .map(|c| c.into())
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
                .map(|(k, v)| (k.to_string(), v.to_string()))
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
