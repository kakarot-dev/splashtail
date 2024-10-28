use silverpelt::{Context, Error};

/// A TagContext is the context for executing in
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecContext {
    /// The user that triggered the captcha
    pub user: serenity::all::User,
    /// The guild ID that the user triggered the captcha in
    pub guild_id: serenity::all::GuildId,
    /// The channel ID that the user triggered the captcha in. May be None in some cases (tag not in channel)
    pub channel_id: Option<serenity::all::ChannelId>,
    /// The arguments passed to the tag. Note that it is up to the tag to parse these arguments
    pub args: Option<String>,
}

#[typetag::serde]
impl templating::Context for ExecContext {}

/// Execute a Lua template directly
#[poise::command(prefix_command, slash_command)]
pub async fn exec_template(
    ctx: Context<'_>,
    #[description = "The statement to execute"] expr: String,
    #[description = "The arguments to pass to the code"] args: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let mut msg = ctx
        .say("Executing tag... please wait")
        .await?
        .into_message()
        .await?;

    let discord_reply = templating::execute::<_, Option<serde_json::Value>>(
        guild_id,
        templating::Template::Raw(expr),
        ctx.data().pool.clone(),
        ctx.serenity_context().clone(),
        ctx.data().reqwest.clone(),
        ExecContext {
            user: ctx.author().clone(),
            guild_id,
            channel_id: Some(msg.channel_id),
            args,
        },
    )
    .await;

    let discord_reply = match discord_reply {
        Ok(reply) => {
            if let Some(reply) = reply {
                let reply = match serde_json::from_value::<templating::core::messages::CreateMessage>(
                    reply.clone(),
                ) {
                    Ok(templated_reply) => templated_reply,
                    Err(_) => templating::core::messages::CreateMessage {
                        content: Some(reply.to_string()),
                        embeds: vec![],
                    },
                };

                match templating::core::messages::to_discord_reply(reply) {
                    Ok(reply) => reply,
                    Err(e) => {
                        let embed = serenity::all::CreateEmbed::default()
                            .description(format!("Failed to render tag: {}", e));

                        templating::core::messages::DiscordReply {
                            embeds: vec![embed],
                            ..Default::default()
                        }
                    }
                }
            } else {
                let embed = serenity::all::CreateEmbed::default()
                    .description("Template returned no message");

                templating::core::messages::DiscordReply {
                    embeds: vec![embed],
                    ..Default::default()
                }
            }
        }
        Err(e) => {
            let embed = serenity::all::CreateEmbed::default()
                .description(format!("Failed to render template: {}", e));

            templating::core::messages::DiscordReply {
                embeds: vec![embed],
                ..Default::default()
            }
        }
    };

    let message = discord_reply.to_edit_message();

    msg.edit(ctx, message).await?;

    Ok(())
}
