use sqlx::PgPool;
use serenity::all::GuildId;
use super::{
    CommandExtendedData, 
    GuildCommandConfiguration, 
    GuildModuleConfiguration, 
    silverpelt_cache::SILVERPELT_CACHE,
    utils::permute_command_names
};

/// Returns whether or not a module is enabled based on cache and/or database
///
/// Note that fetching directly from database may be more reliable in certain cases
/// such as module_enable/disable and as such *SHOULD* be used there. This function
/// should only be called for cases where querying the database every time would be
/// too great a cost
#[allow(dead_code)] // This function is a useful utility function
pub async fn is_module_enabled(
    pool: &PgPool,
    guild_id: GuildId, 
    module: &str
) -> Result<bool, crate::Error> {
    if let Some(state) = SILVERPELT_CACHE.module_enabled_cache.get(&(guild_id, module.to_string())).await {
        Ok(state)
    } else {
        // Fetch from database
        let disabled = sqlx::query!(
            "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
            guild_id.to_string(),
            module
        )
        .fetch_optional(pool)
        .await?;

        if let Some(disabled) = disabled {
            if let Some(disabled) = disabled.disabled {
                SILVERPELT_CACHE.module_enabled_cache.insert((guild_id, module.to_string()), !disabled).await;
                Ok(!disabled)
            } else {
                // User wants to use the default value
                let module = SILVERPELT_CACHE.module_id_cache.get(module).ok_or::<crate::Error>(
                    format!("Could not find module {} in cache", module).into()
                )?;

                SILVERPELT_CACHE.module_enabled_cache.insert((guild_id, module.id.to_string()), module.is_default_enabled).await;
                Ok(module.is_default_enabled)
            }
        } else {
            // User wants to use the default value
            let module = SILVERPELT_CACHE.module_id_cache.get(module).ok_or::<crate::Error>(
                format!("Could not find module {} in cache", module).into()
            )?;

            SILVERPELT_CACHE.module_enabled_cache.insert((guild_id, module.id.to_string()), module.is_default_enabled).await;
            Ok(module.is_default_enabled)
        }
    }
}

/// Returns the configuration of a command
pub async fn get_command_configuration(
    pool: &PgPool,
    guild_id: &str,
    name: &str,
) -> Result<
    (
        CommandExtendedData,
        Option<GuildCommandConfiguration>,
        Option<GuildModuleConfiguration>,
    ),
    crate::Error,
> {
    let permutations = permute_command_names(name);
    let root_cmd = permutations.first().unwrap();

    let root_cmd_data = SILVERPELT_CACHE.command_extra_data_map.get(root_cmd);

    let Some(root_cmd_data) = root_cmd_data else {
        return Err(format!(
            "The command ``{}`` does not exist [command not found in cache?]",
            name
        )
        .into());
    };

    let module = SILVERPELT_CACHE.command_id_module_map.get(root_cmd).ok_or::<crate::Error>("Unknown error determining module of command".into())?;

    // Check if theres any module configuration
    let module_configuration = sqlx::query!(
        "SELECT id, guild_id, module, disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2",
        guild_id,
        module,
    )
    .fetch_optional(pool)
    .await?
    .map(|rec| GuildModuleConfiguration {
        id: rec.id.hyphenated().to_string(),
        guild_id: rec.guild_id,
        module: rec.module,
        disabled: rec.disabled,
    });

    let mut cmd_data = root_cmd_data
        .get("")
        .unwrap_or(
            &CommandExtendedData::kittycat_or_admin(root_cmd, "*")
        )
        .clone();

    for command in permutations.iter() {
        let cmd_replaced = command
            .replace(&root_cmd.to_string(), "")
            .trim()
            .to_string();
        if let Some(data) = root_cmd_data.get(&cmd_replaced.as_str()) {
            cmd_data = data.clone();
        }
    }

    let mut command_configuration = None;

    for permutation in permutations.iter() {
        let rec = sqlx::query!(
            "SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
            guild_id,
            permutation,
        )
        .fetch_optional(pool)
        .await?;

        // We are deeper in the tree, so we can overwrite the command configuration
        let mut _cmd_perms_overriden = false; // Not used currently but will be used in the future for module no_admin etc.
        if let Some(rec) = rec {
            command_configuration = Some(GuildCommandConfiguration {
                id: rec.id.hyphenated().to_string(),
                guild_id: rec.guild_id,
                command: rec.command,
                perms: {
                    if let Some(perms) = rec.perms {
                        _cmd_perms_overriden = true;
                        serde_json::from_value(perms).unwrap()
                    } else {
                        None
                    }
                },
                disabled: rec.disabled,
            });
        }
    }

    Ok((cmd_data, command_configuration, module_configuration))
}
