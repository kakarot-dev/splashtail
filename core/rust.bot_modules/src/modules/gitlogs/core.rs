use crate::{Context, Error};
use poise::serenity_prelude::ChannelId;
use splashcore_rs::value::Value;

/// Gitlogs base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_cooldown = 10,
    subcommands(
        "webhooks_list",
        "webhooks_create",
        "webhooks_update",
        "webhooks_delete",
        "repo_list",
        "repo_create",
        "repo_update",
        "repo_delete",
        "eventmods_list",
        "eventmods_create",
        "eventmods_update",
        "eventmods_delete"
    )
)]
pub async fn gitlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all webhooks
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn webhooks_list(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::webhooks()).await
}

/// Creates a new webhook in a guild
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn webhooks_create(
    ctx: Context<'_>,
    #[description = "The comment for the webhook"] comment: String,
    #[description = "Custom secret for the webhook"] secret: Option<String>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::webhooks(),
        indexmap::indexmap! {
            "comment".to_string() => Value::String(comment),
            "secret".to_string() => {
                if let Some(secret) = secret {
                    Value::String(secret)
                } else {
                    Value::None // Settings_creator will autogenerate a secret if this is None
                }
            },
        },
    )
    .await
}

/// Updates a webhook in a guild
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn webhooks_update(
    ctx: Context<'_>,
    #[description = "The webhook ID"] id: String,
    #[description = "The comment for the webhook"] comment: String,
    #[description = "Custom secret for the webhook"] secret: Option<String>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::webhooks(),
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "comment".to_string() => Value::String(comment),
            "secret".to_string() => {
                if let Some(secret) = secret {
                    Value::String(secret)
                } else {
                    Value::None // Settings_creator will autogenerate a secret if this is None
                }
            },
        },
    )
    .await
}

/// Deletes a webhook in a guild
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn webhooks_delete(
    ctx: Context<'_>,
    #[description = "The webhook ID"] id: String,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::webhooks(),
        Value::String(id),
    )
    .await
}

/// Creates a new repository for a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn repo_list(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::repos()).await
}

/// Creates a new repository for a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn repo_create(
    ctx: Context<'_>,
    #[description = "The webhook ID to use"] webhook_id: String,
    #[description = "The repo owner or organization"] owner: String,
    #[description = "The repo name"] name: String,
    #[description = "The channel to send to"] channel: ChannelId,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::repos(),
        indexmap::indexmap! {
            "webhook_id".to_string() => Value::String(webhook_id),
            "repo_name".to_string() => Value::String((owner + "/" + &name).to_lowercase()),
            "channel_id".to_string() => Value::String(channel.to_string()),
        },
    )
    .await
}

/// Updates an existing repository for a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn repo_update(
    ctx: Context<'_>,
    #[description = "The repo ID"] id: String,
    #[description = "The webhook ID to use"] webhook_id: String,
    #[description = "The repo owner or organization"] owner: String,
    #[description = "The repo name"] name: String,
    #[description = "The channel to send to"] channel: ChannelId,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::repos(),
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "webhook_id".to_string() => Value::String(webhook_id),
            "repo_name".to_string() => Value::String((owner + "/" + &name).to_lowercase()),
            "channel_id".to_string() => Value::String(channel.to_string()),
        },
    )
    .await
}

/// Deletes a repo of a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn repo_delete(
    ctx: Context<'_>,
    #[description = "The repo ID"] id: String,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::repos(),
        Value::String(id),
    )
    .await
}

/// Lists all event modifiers
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn eventmods_list(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::event_modifiers())
        .await
}

/// Creates a event modifier
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::too_many_arguments)]
pub async fn eventmods_create(
    ctx: Context<'_>,
    #[description = "The webhook ID"] webhook_id: String,
    #[description = "The events to match against, comma/space seperated"] events: String,
    #[description = "Blacklist the events"] blacklisted: bool,
    #[description = "Whitelist the events. Other events will not be allowed"] whitelisted: bool,
    #[description = "Priority. Use 0 for normal priority"] priority: Option<i32>,
    // Lazy = "prefer to parse the current argument as the other params first"
    #[description = "Repository ID, will match all if unset"]
    #[lazy]
    repo_id: Option<String>,
    #[description = "Redirect channel ID"] redirect_channel: Option<ChannelId>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::event_modifiers(),
        indexmap::indexmap! {
            "webhook_id".to_string() => Value::String(webhook_id),
            "events".to_string() => {
                let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();

                let mut value_events = Vec::new();

                for evt in events {
                    value_events.push(Value::String(evt));
                }

                Value::List(value_events)
            },
            "blacklisted".to_string() => Value::Boolean(blacklisted),
            "whitelisted".to_string() => Value::Boolean(whitelisted),
            "priority".to_string() => Value::Integer(priority.unwrap_or_default() as i64),
            "repo_id".to_string() => {
                if let Some(repo_id) = repo_id {
                    Value::String(repo_id)
                } else {
                    Value::None
                }
            },
            "redirect_channel".to_string() => {
                if let Some(redirect_channel) = redirect_channel {
                    Value::String(redirect_channel.to_string())
                } else {
                    Value::None
                }
            }
        },
    )
    .await
}

/// Updates a event modifier
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::too_many_arguments)]
pub async fn eventmods_update(
    ctx: Context<'_>,
    #[description = "The modifier ID"] modifier_id: String,
    #[description = "The webhook ID"] webhook_id: String,
    #[description = "The events to match against, comma/space seperated"] events: String,
    #[description = "Blacklist the events"] blacklisted: bool,
    #[description = "Whitelist the events. Other events will not be allowed"] whitelisted: bool,
    #[description = "Priority. Use 0 for normal priority"] priority: Option<i32>,
    // Lazy = "prefer to parse the current argument as the other params first"
    #[description = "Repository ID, will match all if unset"]
    #[lazy]
    repo_id: Option<String>,
    #[description = "Redirect channel ID"] redirect_channel: Option<ChannelId>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::event_modifiers(),
        indexmap::indexmap! {
            "id".to_string() => Value::String(modifier_id),
            "webhook_id".to_string() => Value::String(webhook_id),
            "events".to_string() => {
                let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();

                let mut value_events = Vec::new();

                for evt in events {
                    value_events.push(Value::String(evt));
                }

                Value::List(value_events)
            },
            "blacklisted".to_string() => Value::Boolean(blacklisted),
            "whitelisted".to_string() => Value::Boolean(whitelisted),
            "priority".to_string() => Value::Integer(priority.unwrap_or_default() as i64),
            "repo_id".to_string() => {
                if let Some(repo_id) = repo_id {
                    Value::String(repo_id)
                } else {
                    Value::None
                }
            },
            "redirect_channel".to_string() => {
                if let Some(redirect_channel) = redirect_channel {
                    Value::String(redirect_channel.to_string())
                } else {
                    Value::None
                }
            }
        },
    )
    .await
}

/// Deletes a event modifier
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn eventmods_delete(
    ctx: Context<'_>,
    #[description = "The modifier ID"] modifier_id: String,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::event_modifiers(),
        Value::String(modifier_id),
    )
    .await
}
