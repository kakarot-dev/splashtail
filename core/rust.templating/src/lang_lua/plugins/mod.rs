pub mod actions;
pub mod r#async;
pub mod interop;
pub mod kv;
pub mod lune;
pub mod message;
pub mod permissions;

use mlua::prelude::*;
use std::sync::LazyLock;

// Modules can load their own plugins
pub static PLUGINS: LazyLock<indexmap::IndexMap<String, ModuleFn>> = LazyLock::new(|| {
    indexmap::indexmap! {
        "@antiraid/actions".to_string() => actions::init_plugin as ModuleFn,
        "@antiraid/async".to_string() => r#async::init_plugin as ModuleFn,
        "@antiraid/builtins".to_string() => builtins as ModuleFn,
        "@antiraid/interop".to_string() => interop::init_plugin as ModuleFn,
        "@antiraid/kv".to_string() => kv::init_plugin as ModuleFn,
        "@antiraid/message".to_string() => message::init_plugin as ModuleFn,
        "@antiraid/permissions".to_string() => permissions::init_plugin as ModuleFn,
        "@lune/datetime".to_string() => lune::datetime::init_plugin as ModuleFn,
        "@lune/regex".to_string() => lune::regex::init_plugin as ModuleFn,
        "@lune/serde".to_string() => lune::serde::init_plugin as ModuleFn,
    }
});

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;

/// Provides the lua builtins as a seperate table
pub fn builtins(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    module.set("require", lua.create_async_function(require)?)?;
    module.set_readonly(true); // Block any attempt to modify this table
    Ok(module)
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct RequirePluginArgs {
    pub plugin_cache: Option<bool>,
}

pub async fn require(lua: Lua, (plugin_name, args): (String, LuaValue)) -> LuaResult<LuaTable> {
    // Relative imports are special, they include the template stored in guild_templates

    match PLUGINS.get(plugin_name.as_str()) {
        Some(plugin) => {
            let args: RequirePluginArgs = lua
                .from_value::<Option<RequirePluginArgs>>(args)?
                .unwrap_or_default();

            if args.plugin_cache.unwrap_or(true) {
                // Get table from vm cache
                if let Ok(table) = lua.named_registry_value::<LuaTable>(&plugin_name) {
                    return Ok(table);
                }
            }

            let res = plugin(&lua);

            if args.plugin_cache.unwrap_or(true) {
                if let Ok(table) = &res {
                    lua.set_named_registry_value(&plugin_name, table.clone())?;
                }
            }

            res
        }
        None => {
            if let Ok(table) = lua.globals().get::<LuaTable>(plugin_name.clone()) {
                return Ok(table);
            }

            Err(LuaError::runtime(format!(
                "module '{}' not found",
                plugin_name
            )))
        }
    }
}

/// Resolves a path given the current_path and the path to resolve
///
/// Rules:
/// The path starts at current path and the 'instructions' in path are applied to it
///
/// The caller should then, if requested in args, make a new template token and inherit the pragma (or use Null for token if caller did not request) for the imported template
fn resolve_template_import_path(current_path: &str, path: &str, prefix: &str) -> String {
    /*
    Potentially useful in the future
    // Get cwd and the file
    let (cwd, cwf) = if current_path.is_empty() {
        ("", current_path) // If current_path is empty, we are at the root
    } else {
        // Split by prefix and take all but last element as cwd and last as file
        let (cwd, file) = current_path.rsplit_once(prefix).unwrap_or(("", current_path));
        (cwd, file)
    };*/

    let mut new_path = current_path.to_string();

    for (i, inst) in path.split(prefix).enumerate() {
        match (i, inst) {
            (0, "~") => new_path.clear(),
            (_, "") => {}
            (_, ".") => {
                // UNIX notes: Note that the current path is a file, everything else is considered a directory
                //
                // Using '/' as a prefix 'casts' the file path to a directory
                if new_path == current_path && !new_path.starts_with(prefix) {
                    new_path = new_path
                        .rsplit_once(prefix)
                        .map(|(parent, _)| parent)
                        .unwrap_or("")
                        .to_string();
                }
            }
            (_, "..") => {
                new_path = new_path
                    .rsplit_once(prefix)
                    .map(|(parent, _)| parent)
                    .unwrap_or("")
                    .to_string();
            }
            (_, _) => {
                if new_path.is_empty() {
                    new_path.push_str(inst);
                } else {
                    new_path.push_str(prefix);
                    new_path.push_str(inst);
                }
            }
        }
    }

    new_path
        .trim_start_matches(prefix)
        .replace("//", "/")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_template_import_path;

    #[test]
    fn test_resolve_template_import_path() {
        assert_eq!(resolve_template_import_path("myscripts/foo", "~/", "/"), "");
        assert_eq!(
            resolve_template_import_path("", "myscripts/abc", "/"),
            "myscripts/abc"
        );
        assert_eq!(
            resolve_template_import_path("myscripts", "../abc", "/"),
            "abc",
        );
        assert_eq!(
            resolve_template_import_path("myscripts", "../abcgh/d", "/"),
            "abcgh/d"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "../def", "/"),
            "myscripts/def"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "./def", "/"),
            "myscripts/def"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc", "def/..", "/"),
            "myscripts/abc"
        );
        assert_eq!(
            resolve_template_import_path("myscripts/abc/", "def", "/"),
            "myscripts/abc/def"
        );
        assert_eq!(
            resolve_template_import_path("/myscripts/abc", "def", "/"),
            "myscripts/abc/def"
        );
        assert_eq!(
            resolve_template_import_path("/myscripts/abc/", "/def", "/"),
            "myscripts/abc/def"
        );
    }
}
