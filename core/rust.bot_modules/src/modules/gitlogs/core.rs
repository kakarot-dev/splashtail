use log::error;
use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, CreateMessage},
    CreateReply,
};
use rand::distributions::{Alphanumeric, DistString};
use splashcore_rs::value::Value;

use crate::{Context, Error};

/// Gitlogs base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_cooldown = 10,
    subcommands("list", "newhook", "delhook")
)]
pub async fn gitlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lsts all webhooks in a guild with their respective repos and channel IDs
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, return an error
        sqlx::query!(
            "INSERT INTO gitlogs__guilds (guild_id) VALUES ($1)",
            ctx.guild_id().unwrap().to_string()
        )
        .execute(&data.pool)
        .await?;

        ctx.say("This guild doesn't have any webhooks yet. Get started with ``/gitlogs newhook`` (or ``%gitlogs newhook``)").await?;
    } else {
        // Get all webhooks
        let webhooks = sqlx::query!(
            "SELECT id, comment, created_at FROM gitlogs__webhooks WHERE guild_id = $1",
            ctx.guild_id().unwrap().to_string()
        )
        .fetch_all(&data.pool)
        .await;

        match webhooks {
            Ok(webhooks) => {
                let mut embeds = Vec::new();

                let api_url = config::CONFIG.sites.api.get();

                for webhook in webhooks {
                    let webhook_id = webhook.id;
                    embeds.push(
                        CreateEmbed::new()
                            .title(format!("Webhook \"{}\"", webhook.comment))
                            .field("Webhook ID", webhook_id.clone(), false)
                            .field(
                                "Hook URL (visit for hook info, add to Github to recieve events)",
                                api_url.clone()
                                    + "/integrations/gitlogs/kittycat?id="
                                    + &webhook_id,
                                false,
                            )
                            .field("Created at", webhook.created_at.to_string(), false),
                    );
                }

                let mut cr =
                    CreateReply::default().content("Here are all the webhooks in this guild:");

                for embed in embeds {
                    cr = cr.embed(embed);
                }

                ctx.send(cr).await?;
            }
            Err(e) => {
                error!("Error fetching webhooks: {:?}", e);
                ctx.say("This guild doesn't have any webhooks yet. Get started with ``/gitlogs newhook`` (or ``%gitlogs newhook``)").await?;
            }
        }
    }

    Ok(())
}

/// Creates a new webhook in a guild for sending Github notifications
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn newhook(
    ctx: Context<'_>,
    #[description = "The comment for the webhook"] comment: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, create it
        sqlx::query!(
            "INSERT INTO gitlogs__guilds (guild_id) VALUES ($1)",
            ctx.guild_id().unwrap().to_string()
        )
        .execute(&data.pool)
        .await?;
    }

    // Check webhook count
    let webhook_count = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__webhooks WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if webhook_count.count.unwrap_or_default() >= 5 {
        ctx.say("You can't have more than 5 webhooks per guild")
            .await?;
        return Ok(());
    }

    // Create the webhook
    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

    let webh_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 256);

    // Create a new dm channel with the user if not slash command
    let dm_channel = ctx.author().create_dm_channel(ctx.http()).await;

    let dm = match dm_channel {
        Ok(dm) => dm,
        Err(_) => {
            ctx.say(
                "I couldn't create a DM channel with you, please enable DMs from server members",
            )
            .await?;
            return Ok(());
        }
    };

    sqlx::query!(
        "INSERT INTO gitlogs__webhooks (id, guild_id, comment, secret) VALUES ($1, $2, $3, $4)",
        id,
        ctx.guild_id().unwrap().to_string(),
        comment,
        webh_secret
    )
    .execute(&data.pool)
    .await?;

    ctx.say("Webhook created! Trying to DM you the credentials...")
        .await?;

    dm.id.send_message(
        ctx.http(),
        CreateMessage::new()
        .content(
            format!(
                "
Next, add the following webhook to your Github repositories (or organizations): `{api_url}/integrations/gitlogs/kittycat?id={id}`

Set the `Secret` field to `{webh_secret}` and ensure that Content Type is set to `application/json`. 

When creating repositories, use `{id}` as the ID.
            
**Note that the above URL and secret is unique and should not be shared with others**

**Delete this message after you're done!**
                ",
                api_url=config::CONFIG.sites.api.get(),
                id=id,
                webh_secret=webh_secret
            )
        )
    ).await?;

    ctx.say("Webhook created! Check your DMs for the webhook information.")
        .await?;

    Ok(())
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

/// Deletes a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn delhook(
    ctx: Context<'_>,
    #[description = "The webhook ID"] id: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, return a error
        return Err("You don't have any webhooks in this guild! Use ``/newhook`` (or ``git!newhook``) to create one".into());
    }

    sqlx::query!(
        "DELETE FROM gitlogs__webhooks WHERE id = $1 AND guild_id = $2",
        id,
        ctx.guild_id().unwrap().to_string()
    )
    .execute(&data.pool)
    .await?;

    ctx.say("Webhook deleted if it exists!").await?;

    Ok(())
}

/// Resets a webhook secret. DMs must be open
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn resetsecret(
    ctx: Context<'_>,
    #[description = "The webhook ID"] id: String,
) -> Result<(), Error> {
    let data = ctx.data();

    // Check if the guild exists on our DB
    let guild = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__guilds WHERE guild_id = $1",
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // If it doesn't, return a error
        return Err("You don't have any webhooks in this guild! Use ``/gitlogs newhook`` (or ``%gitlogs newhook``) to create one".into());
    }

    // Check if the webhook exists
    let webhook = sqlx::query!(
        "SELECT COUNT(1) FROM gitlogs__webhooks WHERE id = $1 AND guild_id = $2",
        id,
        ctx.guild_id().unwrap().to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if webhook.count.unwrap_or_default() == 0 {
        return Err("That webhook doesn't exist! Use ``/gitlogs newhook`` (or ``git!newhook``) to create one".into());
    }

    let webh_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 256);

    // Try to DM the user
    // Create a new dm channel with the user if not slash command
    let dm_channel = ctx.author().create_dm_channel(ctx.http()).await;

    let dm = match dm_channel {
        Ok(dm) => dm,
        Err(_) => {
            ctx.say(
                "I couldn't create a DM channel with you, please enable DMs from server members",
            )
            .await?;
            return Ok(());
        }
    };

    sqlx::query!(
        "UPDATE gitlogs__webhooks SET secret = $1 WHERE id = $2 AND guild_id = $3",
        webh_secret,
        id,
        ctx.guild_id().unwrap().to_string()
    )
    .execute(&data.pool)
    .await?;

    dm.id.send_message(
        ctx.http(),
        CreateMessage::new()
        .content(
            format!(
                "Your new webhook secret is `{webh_secret}`. 

Update this webhooks information in GitHub settings now. Your webhook will not accept messages from GitHub unless you do so!

**Delete this message after you're done!**
                ",
                webh_secret=webh_secret
            )
        )
    ).await?;

    ctx.say("Webhook secret updated! Check your DMs for the webhook information.")
        .await?;

    Ok(())
}
