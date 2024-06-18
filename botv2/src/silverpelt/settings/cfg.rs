use super::config_opts::SettingsError;
use super::config_opts::{ColumnType, ConfigOption, InnerColumnType, OperationType};
use super::state::State;
use crate::silverpelt::value::Value;
use sqlx::Row;

/// Validates the value against the schema's column type handling schema checks if `perform_schema_checks` is true
#[allow(dead_code)]
fn _validate_value(
    v: &Value,
    column_type: &ColumnType,
    column_id: &str,
    is_nullable: bool,
    perform_schema_checks: bool,
) -> Result<(), SettingsError> {
    match column_type {
        ColumnType::Scalar { column_type } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err(SettingsError::SchemaNullValueValidationError {
                        column: column_id.to_string(),
                    });
                }
            }

            if matches!(v, Value::List(_)) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Scalar".to_string(),
                    got_type: "Array".to_string(),
                });
            }

            match column_type {
                InnerColumnType::Uuid {} => {
                    if !matches!(v, Value::Uuid(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Uuid".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::String {
                    min_length,
                    max_length,
                    allowed_values,
                } => {
                    if !matches!(v, Value::String(_) | Value::Uuid(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "String".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        if let Some(min) = min_length {
                            if s.len() < *min {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "minlength".to_string(),
                                    value: v.to_json(),
                                    accepted_range: format!(">{}", min),
                                    error: "s.len() < *min".to_string(),
                                });
                            }
                        }

                        if let Some(max) = max_length {
                            if s.len() > *max {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "maxlength".to_string(),
                                    value: v.to_json(),
                                    accepted_range: format!("<{}", max),
                                    error: "s.len() > *max".to_string(),
                                });
                            }
                        }

                        if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "allowed_values".to_string(),
                                value: v.to_json(),
                                accepted_range: format!("{:?}", allowed_values),
                                error: "!allowed_values.is_empty() && !allowed_values.contains(&s.as_str())".to_string()
                            });
                        }
                    }
                }
                InnerColumnType::Timestamp {} => {
                    if !matches!(v, Value::Timestamp(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Timestamp".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // No further checks needed
                }
                InnerColumnType::TimestampTz {} => {
                    if !matches!(v, Value::TimestampTz(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "TimestampTz".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // No further checks needed
                }
                InnerColumnType::Integer {} => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Integer".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::Float {} => {
                    if !matches!(v, Value::Float(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Float".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::BitFlag { .. } => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Integer".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // TODO: Add value parsing for bit flags
                }
                InnerColumnType::Boolean {} => {
                    if !matches!(v, Value::Boolean(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Boolean".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::User {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "User (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a UserId
                        if let Err(err) = s.parse::<serenity::all::UserId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid user id".to_string(),
                                error: err.to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Channel {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Channel (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a ChannelId
                        if let Err(err) = s.parse::<serenity::all::ChannelId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid channel id".to_string(),
                                error: err.to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Role {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Role (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a RoleId
                        if let Err(err) = s.parse::<serenity::all::RoleId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid role id".to_string(),
                                error: err.to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Emoji {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Emoji (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to an EmojiId
                        if let Err(err) = s.parse::<serenity::all::EmojiId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid emoji id".to_string(),
                                error: err.to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Message {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Message (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // The format of a message on db should be channel_id/message_id
                        //
                        // So, split by '/' and check if the first part is a valid channel id
                        // and the second part is a valid message id
                        let parts: Vec<&str> = s.split('/').collect();

                        if parts.len() != 2 {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_plength".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                                error: "parts.len() != 2".to_string(),
                            });
                        }

                        // Try parsing to a ChannelId
                        if let Err(err) = parts[0].parse::<serenity::all::ChannelId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_0".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                                error: format!("p1: {}", err),
                            });
                        }

                        // Try parsing to a MessageId
                        if let Err(err) = parts[1].parse::<serenity::all::MessageId>() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_1".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                                error: format!("p2: {}", err),
                            });
                        }
                    }
                }
                InnerColumnType::Json {} => {
                    if !matches!(v, Value::Map(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Json".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
            }
        }
        ColumnType::Array { inner } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err(SettingsError::SchemaNullValueValidationError {
                        column: column_id.to_string(),
                    });
                }
            }

            if !matches!(v, Value::List(_)) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Array".to_string(),
                    got_type: format!("{:?}", v),
                });
            }

            let l = match v {
                Value::List(l) => l,
                _ => unreachable!(),
            };

            let column_type = ColumnType::new_scalar(inner.clone());
            for v in l {
                _validate_value(
                    v,
                    &column_type,
                    column_id,
                    is_nullable,
                    perform_schema_checks,
                )?;
            }
        }
    }

    Ok(())
}

/// Binds a value to a query
///
/// Note that Maps are binded as JSONs
fn _query_bind_value(
    query: sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: Value,
    column_type_hint: Option<ColumnType>,
) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match value {
        Value::Uuid(value) => query.bind(value),
        Value::String(value) => query.bind(value),
        Value::Timestamp(value) => query.bind(value),
        Value::TimestampTz(value) => query.bind(value),
        Value::Integer(value) => query.bind(value),
        Value::Float(value) => query.bind(value),
        Value::Boolean(value) => query.bind(value),
        Value::List(values) => {
            // Get the type of the first element
            let first = values.first();

            if let Some(first) = first {
                // This is hacky and long but sqlx doesn't support binding lists
                //
                // Loop over all values to make a Vec<T> then bind that
                match first {
                    Value::Uuid(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::Uuid(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::String(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::String(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::Timestamp(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::Timestamp(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::TimestampTz(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::TimestampTz(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::Integer(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::Integer(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::Float(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::Float(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    Value::Boolean(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            if let Value::Boolean(value) = value {
                                vec.push(value);
                            }
                        }

                        query.bind(vec)
                    }
                    // In all other cases (list/map)
                    Value::Map(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            vec.push(value.to_json());
                        }

                        query.bind(vec)
                    }
                    // TODO: Improve this, right now, we fallback to string
                    Value::List(_) => {
                        let mut vec = Vec::new();

                        for value in values {
                            vec.push(value.to_json());
                        }

                        query.bind(vec)
                    }
                    Value::None => {
                        let vec: Vec<String> = Vec::new();
                        query.bind(vec)
                    }
                }
            } else {
                let vec: Vec<String> = Vec::new();
                query.bind(vec)
            }
        }
        Value::Map(_) => query.bind(value.to_json()),
        Value::None => match column_type_hint {
            Some(ColumnType::Scalar {
                column_type: column_type_hint,
            }) => match column_type_hint {
                InnerColumnType::Uuid {} => query.bind(None::<sqlx::types::uuid::Uuid>),
                InnerColumnType::String { .. } => query.bind(None::<String>),
                InnerColumnType::Timestamp {} => query.bind(None::<chrono::NaiveDateTime>),
                InnerColumnType::TimestampTz {} => {
                    query.bind(None::<chrono::DateTime<chrono::Utc>>)
                }
                InnerColumnType::Integer {} => query.bind(None::<i64>),
                InnerColumnType::Float {} => query.bind(None::<f64>),
                InnerColumnType::BitFlag { .. } => query.bind(None::<i64>),
                InnerColumnType::Boolean {} => query.bind(None::<bool>),
                InnerColumnType::User {} => query.bind(None::<String>),
                InnerColumnType::Channel {} => query.bind(None::<String>),
                InnerColumnType::Role {} => query.bind(None::<String>),
                InnerColumnType::Emoji {} => query.bind(None::<String>),
                InnerColumnType::Message {} => query.bind(None::<String>),
                InnerColumnType::Json {} => query.bind(None::<serde_json::Value>),
            },
            Some(ColumnType::Array {
                inner: column_type_hint,
            }) => match column_type_hint {
                InnerColumnType::Uuid {} => query.bind(None::<Vec<sqlx::types::uuid::Uuid>>),
                InnerColumnType::String { .. } => query.bind(None::<Vec<String>>),
                InnerColumnType::Timestamp {} => query.bind(None::<Vec<chrono::NaiveDateTime>>),
                InnerColumnType::TimestampTz {} => {
                    query.bind(None::<Vec<chrono::DateTime<chrono::Utc>>>)
                }
                InnerColumnType::Integer {} => query.bind(None::<Vec<i64>>),
                InnerColumnType::Float {} => query.bind(None::<Vec<f64>>),
                InnerColumnType::BitFlag { .. } => query.bind(None::<Vec<i64>>),
                InnerColumnType::Boolean {} => query.bind(None::<Vec<bool>>),
                InnerColumnType::User {} => query.bind(None::<Vec<String>>),
                InnerColumnType::Channel {} => query.bind(None::<Vec<String>>),
                InnerColumnType::Role {} => query.bind(None::<Vec<String>>),
                InnerColumnType::Emoji {} => query.bind(None::<Vec<String>>),
                InnerColumnType::Message {} => query.bind(None::<Vec<String>>),
                InnerColumnType::Json {} => query.bind(None::<Vec<serde_json::Value>>),
            },
            None => query.bind(None::<String>),
        },
    }
}

/// Settings API: View implementation
pub async fn settings_view(
    setting: &ConfigOption,
    ctx: &serenity::all::Context,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
) -> Result<Vec<State>, SettingsError> {
    let cols = setting
        .columns
        .iter()
        .map(|c| c.id.to_string())
        .collect::<Vec<String>>();

    let row = sqlx::query(
        format!(
            "SELECT {} FROM {} WHERE {} = $1",
            cols.join(", "),
            setting.table,
            setting.guild_id
        )
        .as_str(),
    )
    .bind(guild_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_view [query fetch_all]".to_string(),
        typ: "internal".to_string(),
    })?;

    if row.is_empty() {
        return Ok(Vec::new());
    }

    let mut values: Vec<State> = Vec::new();

    for row in row {
        let mut state = State::new();

        // We know that the columns are in the same order as the row
        for (i, col) in setting.columns.iter().enumerate() {
            // Fetch and validate the value
            let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "_parse_row [Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?;

            _validate_value(&val, &col.column_type, col.id, col.nullable, false)?;

            let actions = col
                .pre_checks
                .get(&OperationType::View)
                .unwrap_or(&col.default_pre_checks);

            // Insert the value into the map
            state.state.insert(col.id.to_string(), val);

            crate::silverpelt::settings::action_executor::execute_actions(
                &mut state, actions, ctx, author, guild_id,
            )
            .await?;
        }

        // Apply columns_to_set in operation specific data
        if let Some(op_specific) = setting.operations.get(&OperationType::View) {
            // Only apply if there are columns to set
            if !op_specific.columns_to_set.is_empty() {
                let mut set_stmt = "".to_string();
                let mut values = Vec::new();

                let mut i = 0;
                for (column, value) in op_specific.columns_to_set.iter() {
                    set_stmt.push_str(&format!("{} = ${}, ", column, i + 1));

                    let value = state.template_to_string(author, guild_id, value);
                    values.push(value.clone());

                    // Add directly to state
                    state.state.insert(column.to_string(), value);

                    i += 1;
                }

                // Remove the trailing comma
                set_stmt.pop();

                let sql_stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${}",
                    setting.table,
                    set_stmt,
                    setting.guild_id,
                    i + 1
                );

                let mut query = sqlx::query(sql_stmt.as_str());

                for value in values {
                    query = _query_bind_value(query, value, None);
                }

                query
                    .bind(guild_id.to_string())
                    .execute(pool)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "_parse_row [query execute]".to_string(),
                        typ: "internal".to_string(),
                    })?;
            }
        }

        // Remove ignored columns now that the actions have been executed
        for col in &setting.columns {
            if col.ignored_for.contains(&OperationType::View) {
                state.state.shift_remove(col.id);
            }
        }

        values.push(state);
    }

    Ok(values)
}

pub async fn settings_create(
    setting: &ConfigOption,
    ctx: &serenity::all::Context,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<State, SettingsError> {
    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut state: State = State::new();
    for column in setting.columns.iter() {
        // If the column is ignored for create, skip
        let value = {
            if column.ignored_for.contains(&OperationType::Create) {
                Value::None
            } else {
                match fields.get(column.id) {
                    Some(val) => {
                        _validate_value(
                            val,
                            &column.column_type,
                            column.id,
                            column.nullable,
                            true,
                        )?;

                        val.clone()
                    }
                    None => Value::None,
                }
            }
        };

        // Insert the value into the state
        state.state.insert(column.id.to_string(), value.clone());

        // Execute actions
        let actions = column
            .pre_checks
            .get(&OperationType::Create)
            .unwrap_or(&column.default_pre_checks);

        crate::silverpelt::settings::action_executor::execute_actions(
            &mut state, actions, ctx, author, guild_id,
        )
        .await?;

        // Check if the column is nullable
        if !column.nullable && matches!(value, Value::None) {
            return Err(SettingsError::MissingOrInvalidField {
                field: column.id.to_string(),
            });
        }

        // Handle cases of uniqueness
        if column.unique || column.id == setting.primary_key {
            match value {
                Value::None => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} IS NULL",
                        setting.table, setting.guild_id, column.id
                    );

                    let query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    let row = query
                    .fetch_one(pool)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: format!("settings_create [unique check (null value), query.fetch_one] for column `{}`", column.id),
                        typ: "internal".to_string(),
                    })?;

                    let count = row.try_get::<i64, _>(0)
                    .map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: format!("settings_create [unique check (null value), row.try_get] for column `{}`", column.id),
                        typ: "internal".to_string(),
                    })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
                _ => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} = $2",
                        setting.table, setting.guild_id, column.id
                    );

                    let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    query = _query_bind_value(query, value, None);

                    let row = query
                        .fetch_one(pool)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!(
                                "settings_create [unique check, query.fetch_one] for column `{}`",
                                column.id
                            ),
                            typ: "internal".to_string(),
                        })?;

                    let count = row
                        .try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!(
                                "settings_create [unique check, row.try_get] for column `{}`",
                                column.id
                            ),
                            typ: "internal".to_string(),
                        })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
            }
        }
    }

    // Remove ignored columns now that the actions have been executed
    for col in &setting.columns {
        if col.ignored_for.contains(&OperationType::Create) {
            state.state.shift_remove(col.id);
        }
    }

    // Now insert all the columns_to_set into state
    // As we have removed the ignored columns, we can just directly insert the columns_to_set into the state
    // Insert columns_to_set seperately as we need to bypass ignored_for
    if let Some(op_specific) = setting.operations.get(&OperationType::Create) {
        for (column, value) in op_specific.columns_to_set.iter() {
            let value = state.template_to_string(author, guild_id, value);
            state.state.insert(column.to_string(), value);
        }
    }

    // Create the row

    // First create the $N's from the cols starting with 2 as 1 is the guild_id
    let mut n_params = "".to_string();
    let mut col_params = "".to_string();
    for (i, (col, _)) in state.state.iter().enumerate() {
        n_params.push_str(&format!("${}", i + 2));
        col_params.push_str(col);

        n_params.push(',');
        col_params.push(',');
    }

    // Remove the trailing comma
    n_params.pop();
    col_params.pop();

    // Execute the SQL statement
    let sql_stmt = format!(
        "INSERT INTO {} ({},{}) VALUES ($1,{}) RETURNING {}",
        setting.table, setting.guild_id, col_params, n_params, setting.primary_key
    );

    let mut query = sqlx::query(sql_stmt.as_str());

    // Bind the sql query arguments
    query = query.bind(guild_id.to_string());

    for (col, value) in state.state.iter() {
        // Get column type from schema for db query hinting, this is the only real place (other than update) where we need to hint the db
        let column_type = setting
            .columns
            .iter()
            .find(|c| c.id == col)
            .map(|c| c.column_type.clone());

        query = _query_bind_value(query, value.clone(), column_type);
    }

    // Execute the query
    let pkey_row = query
        .fetch_one(pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_create [query execute]".to_string(),
            typ: "internal".to_string(),
        })?;

    // Save pkey to state
    state.state.insert(
        setting.primary_key.to_string(),
        Value::from_sqlx(&pkey_row, 0).map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_create [Value::from_sqlx]".to_string(),
            typ: "internal".to_string(),
        })?,
    );

    Ok(state)
}
