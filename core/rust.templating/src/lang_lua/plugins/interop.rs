use crate::lang_lua::state;
use mlua::prelude::*;

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    // Null
    module.set("null", lua.null())?;

    // Array metatable
    module.set("array_metatable", lua.array_metatable())?;

    module.set(
        "memusage",
        lua.create_function(|lua, _: ()| Ok(lua.used_memory()))?,
    )?;

    module.set(
        "guild_id",
        lua.create_function(|lua, _: ()| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            Ok(data.guild_id.to_string())
        })?,
    )?;

    module.set(
        "gettemplatedata",
        lua.create_function(|lua, token: String| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            match data.per_template.read(&token, |_, x| x.clone()) {
                Some(data) => {
                    let v = lua.to_value(&data)?;
                    Ok(v)
                }
                None => Ok(lua.null()),
            }
        })?,
    )?;

    module.set(
        "current_user",
        lua.create_function(|lua, _: ()| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let v = lua.to_value(&data.serenity_context.cache.current_user().clone())?;
            Ok(v)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
