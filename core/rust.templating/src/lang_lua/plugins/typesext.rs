use mlua::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct U64(u64);

impl FromLua for U64 {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(i) if i >= 0 => Ok(U64(i as u64)),
            LuaValue::String(s) => {
                let str_value = s.to_str()?;
                str_value
                    .parse::<u64>()
                    .map(U64)
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "string",
                        to: "U64".to_string(),
                        message: Some("Value must be a non-negative integer".to_string()),
                    })
            }
            LuaValue::UserData(u) => {
                let u64 = u
                    .borrow::<U64>()
                    .map_err(|_| LuaError::FromLuaConversionError {
                        from: "UserData",
                        to: "U64".to_string(),
                        message: Some("UserData must be a U64".to_string()),
                    })?;

                Ok(U64(u64.0))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "{integer | string}",
                to: "U64".to_string(),
                message: Some("Value must be a non-negative integer or a string".to_string()),
            }),
        }
    }
}

impl LuaUserData for U64 {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, value: U64| {
            let v = this
                .0
                .checked_add(value.0)
                .ok_or_else(|| LuaError::external("Overflow occurred during addition"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, value: U64| {
            let v = this
                .0
                .checked_sub(value.0)
                .ok_or_else(|| LuaError::external("Underflow occurred during subtraction"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, value: U64| {
            let v = this
                .0
                .checked_mul(value.0)
                .ok_or_else(|| LuaError::external("Overflow occurred during multiplication"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Div, |_, this, value: U64| {
            let v = this
                .0
                .checked_div(value.0)
                .ok_or_else(|| LuaError::external("Overflow occurred during division"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Mod, |_, this, value: U64| {
            let v = this
                .0
                .checked_rem(value.0)
                .ok_or_else(|| LuaError::external("Overflow occurred during modulo operation"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::Pow, |_, this, value: u32| {
            let v = this
                .0
                .checked_pow(value)
                .ok_or_else(|| LuaError::external("Overflow occurred during modulo operation"))?;
            Ok(U64(v))
        });

        methods.add_meta_method(LuaMetaMethod::IDiv, |_, this, value: U64| {
            // Same as Div
            let v = this
                .0
                .checked_div(value.0)
                .ok_or_else(|| LuaError::external("Overflow occurred during floor division"))?;
            Ok(U64(v))
        });

        // Comparison
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, value: U64| {
            Ok(this.0 == value.0)
        });

        methods.add_meta_method(
            LuaMetaMethod::Lt,
            |_, this, value: U64| Ok(this.0 < value.0),
        );

        methods.add_meta_method(LuaMetaMethod::Le, |_, this, value: U64| {
            Ok(this.0 <= value.0)
        });

        // Returns the string representation of the U64 value
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, _: ()| {
            Ok(this.0.to_string())
        });

        // Type
        methods.add_meta_method(LuaMetaMethod::Type, |_, _, _: ()| Ok("U64".to_string()));
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "U64",
        lua.create_function(|lua, initial_value: LuaValue| {
            match initial_value {
                LuaValue::Nil => Ok(U64(0)), // Default value
                _ => {
                    let u64_value = U64::from_lua(initial_value, lua)?;
                    Ok(u64_value)
                }
            }
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
