use gwevent::field::Field;
use mlua::prelude::*;

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "format_gwevent_field",
        lua.create_function(|lua, (field,): (LuaValue,)| {
            let field: Field = lua.from_value(field)?;
            lua.to_value(&field.template_format().map_err(LuaError::external)?)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
