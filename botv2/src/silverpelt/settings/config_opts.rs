#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ColumnType {
    /// A single valued column (scalar)
    Scalar {
        /// The value type
        column_type: InnerColumnType,
    },
    /// An array column
    Array {
        /// The inner type of the array
        inner: InnerColumnType,
    },
}

impl ColumnType {
    /// Returns whether the column type is an array
    #[allow(dead_code)]
    pub fn is_array(&self) -> bool {
        matches!(self, ColumnType::Array { .. })
    }

    /// Returns whether the column type is a scalar
    #[allow(dead_code)]
    pub fn is_scalar(&self) -> bool {
        matches!(self, ColumnType::Scalar { .. })
    }

    pub fn new_scalar(inner: InnerColumnType) -> Self {
        ColumnType::Scalar { column_type: inner }
    }

    pub fn new_array(inner: InnerColumnType) -> Self {
        ColumnType::Array { inner }
    }
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Scalar { column_type } => write!(f, "{}", column_type),
            ColumnType::Array { inner } => write!(f, "Array<{}>", inner),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InnerColumnType {
    Uuid {},
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allowed_values: Vec<&'static str>, // If empty, all values are allowed
    },
    Timestamp {},
    Integer {},
    Float {},
    BitFlag {
        /// The bit flag values
        values: indexmap::IndexMap<&'static str, u64>,
    },
    Boolean {},
    User {},
    Channel {},
    Role {},
    Emoji {},
    Message {},
    Json {},
}

impl std::fmt::Display for InnerColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerColumnType::Uuid {} => write!(f, "Uuid"),
            InnerColumnType::String {
                min_length,
                max_length,
                allowed_values,
            } => {
                write!(f, "String")?;
                if let Some(min) = min_length {
                    write!(f, " (min length: {})", min)?;
                }
                if let Some(max) = max_length {
                    write!(f, " (max length: {})", max)?;
                }
                if !allowed_values.is_empty() {
                    write!(f, " (allowed values: {:?})", allowed_values)?;
                }
                Ok(())
            }
            InnerColumnType::Timestamp {} => write!(f, "Timestamp"),
            InnerColumnType::Integer {} => write!(f, "Integer"),
            InnerColumnType::Float {} => write!(f, "Float"),
            InnerColumnType::BitFlag { values } => {
                write!(f, "BitFlag (values: ")?;
                for (i, (key, value)) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, ")")
            }
            InnerColumnType::Boolean {} => write!(f, "Boolean"),
            InnerColumnType::User {} => write!(f, "User"),
            InnerColumnType::Channel {} => write!(f, "Channel"),
            InnerColumnType::Role {} => write!(f, "Role"),
            InnerColumnType::Emoji {} => write!(f, "Emoji"),
            InnerColumnType::Message {} => write!(f, "Message"),
            InnerColumnType::Json {} => write!(f, "Json"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnSuggestion {
    Static {
        suggestions: Vec<&'static str>,
    },
    Dynamic {
        table_name: &'static str,
        column_name: &'static str,
    },
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnAction {
    /// Adds a column/row to the state map
    CollectColumnToMap {
        /// The table to use
        table: &'static str,

        /// The column to fetch
        column: &'static str,

        /// The key to store the record under
        key: &'static str,

        /// Whether to fetch all or only one rows
        fetch_all: bool,
    },
    /// Executes a lua script, the *last* result will be stored in result
    ///
    /// Note that the lua script must return true or false
    ExecLuaScript {
        script: &'static str,
        on_success: Vec<ColumnAction>,
        on_failure: Vec<ColumnAction>,
    },
    IpcPerModuleFunction {
        /// The module to use
        module: &'static str,

        /// The function to execute
        function: &'static str,

        /// The arguments to pass to the function
        ///
        /// In syntax: {key_on_function} -> {key_on_map}
        arguments: indexmap::IndexMap<&'static str, &'static str>,
    },
    /// Return an error thus failing the configuration view/create/update/delete
    Error {
        /// The error message to return, {key_on_map} can be used here in the message
        message: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// The ID of the column
    pub id: &'static str,

    /// The friendly name of the column
    pub name: &'static str,

    /// The type of the column
    pub column_type: ColumnType,

    /// Whether or not the column is nullable
    pub nullable: bool,

    /// Suggestions to display
    pub suggestions: ColumnSuggestion,

    /// Whether or not the column is unique
    pub unique: bool,

    /// The read-only status of each operation
    ///
    /// Only applies to create and update
    pub readonly: indexmap::IndexMap<OperationType, bool>,

    /// Pre-execute checks
    pub pre_checks: indexmap::IndexMap<OperationType, Vec<ColumnAction>>,

    /// Default pre-execute checks to fallback to if the operation specific ones are not set
    pub default_pre_checks: Vec<ColumnAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationSpecific {
    /// The corresponding command for ACL purposes
    pub corresponding_command: &'static str,

    /// Which column ids should be usable for this operation
    ///
    /// E.g, create does not need to show created_at or id while view should
    ///
    /// If empty, all columns are usable
    pub column_ids: Vec<&'static str>,

    /// Any columns to set. For example, a last_updated column should be set on update
    ///
    /// Variables:
    /// - {user_id} => the user id of the user running the operation
    /// - {now} => the current timestamp
    ///
    /// Note: only applies to create, update and delete
    ///
    /// Key should be of form `table_name.column_name` and value should be the value to set
    pub columns_to_set: indexmap::IndexMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(dead_code)]
pub enum OperationType {
    View,
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigOption {
    /// The ID of the option
    pub id: &'static str,

    /// The name of the option
    pub name: &'static str,

    /// The description of the option
    pub description: &'static str,

    /// The table name for the config option
    pub table: &'static str,

    /// The column name refering to the guild id of the config option    
    pub guild_id: &'static str,

    /// The primary key of the table
    pub primary_key: &'static str,

    /// The columns for this option
    pub columns: Vec<Column>,

    /// Operation specific data
    pub operations: indexmap::IndexMap<OperationType, OperationSpecific>,
}