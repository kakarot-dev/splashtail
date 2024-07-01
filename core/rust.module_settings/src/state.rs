use splashcore_rs::value::Value;

pub struct State {
    /// The state of the module
    pub state: indexmap::IndexMap<String, Value>,
    /// Columns that should not be ignore_for'd for a specific operation
    /// even if they are in the ignore_for list
    pub bypass_ignore_for: std::collections::HashSet<String>,
}

impl From<State> for indexmap::IndexMap<String, Value> {
    fn from(val: State) -> Self {
        val.state
    }
}

impl From<State> for indexmap::IndexMap<String, serde_json::Value> {
    fn from(val: State) -> Self {
        val.state
            .into_iter()
            .map(|(k, v)| (k, v.to_json()))
            .collect()
    }
}

impl State {
    pub fn get_variable_value(
        &self,
        author: serenity::all::UserId,
        guild_id: serenity::all::GuildId,
        variable: &str,
    ) -> Value {
        match variable {
            "__author" => Value::String(author.to_string()),
            "__guild_id" => Value::String(guild_id.to_string()),
            "__now" => Value::TimestampTz(chrono::Utc::now()),
            "__now_naive" => Value::Timestamp(chrono::Utc::now().naive_utc()),
            _ => self.state.get(variable).cloned().unwrap_or(Value::None),
        }
    }

    /// Given a template string, where state variables are surrounded by curly braces, return the
    /// template value (if a single variable) or a string if not
    pub fn template_to_string(
        &self,
        author: serenity::all::UserId,
        guild_id: serenity::all::GuildId,
        template: &str,
    ) -> Value {
        let mut result = template.to_string();

        if result.starts_with("{") && result.ends_with("}") {
            let var = template
                .chars()
                .skip(1)
                .take(template.len() - 2)
                .collect::<String>();

            return self.get_variable_value(author, guild_id, &var);
        }

        for (key, value) in &self.state {
            result = result.replace(&format!("{{{}}}", key), &value.to_string());
        }

        Value::String(result)
    }

    /// A public version of the internal state map, excluding any variables that start with __
    pub fn get_public(&self) -> indexmap::IndexMap<String, Value> {
        self.state
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // Creates a new state
    pub fn new() -> Self {
        State {
            state: indexmap::IndexMap::new(),
            bypass_ignore_for: std::collections::HashSet::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
