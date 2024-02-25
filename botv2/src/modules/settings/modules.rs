type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

#[poise::command(
    prefix_command, 
    slash_command, 
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands(
        "modules_enable",
        "modules_disable",
    )
)]
pub async fn modules(
    _ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    prefix_command, 
    slash_command, 
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "enable",
)]
pub async fn modules_enable(
    ctx: Context<'_>,
    #[description = "The module to enable"] 
    #[autocomplete = "crate::silverpelt::poise_ext::module_list::autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    // Check that the module exists
    if !crate::silverpelt::SILVERPELT_CACHE.module_id_cache.contains_key(&module) {
        return Err(
            format!(
                "The module you are trying to disable ({}) does not exist",
                module
            ).into()
        );
    }    

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let data = ctx.data();
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && !disabled.disabled.unwrap_or_default() {
            return Err("Module is already enabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = false WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, false)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(e) = crate::silverpelt::SILVERPELT_CACHE.invalidate_for_guild(guild_id) {
            log::error!("Failed to invalidate cache for guild {}: {}", guild_id, e);
        }
    });

    ctx.say("Module enabled successfully!").await?;

    Ok(())
}

/// Disables a module. Note that certain modules may not be disablable
#[poise::command(
    prefix_command, 
    slash_command, 
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "disable",
)]
pub async fn modules_disable(
    ctx: Context<'_>,
    #[description = "The module to disable"] 
    #[autocomplete = "crate::silverpelt::poise_ext::module_list::autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    // Check that the module exists
    if !crate::silverpelt::SILVERPELT_CACHE.module_id_cache.contains_key(&module) {
        return Err(
            format!(
                "The module you are trying to disable ({}) does not exist",
                module
            ).into()
        );
    }

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let data = ctx.data();
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && disabled.disabled.unwrap_or_default() {
            return Err("Module is already disabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = true WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, true)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(e) = crate::silverpelt::SILVERPELT_CACHE.invalidate_for_guild(guild_id) {
            log::error!("Failed to invalidate cache for guild {}: {}", guild_id, e);
        }
    });

    ctx.say("Module disabled successfully!").await?;

    Ok(())
}
