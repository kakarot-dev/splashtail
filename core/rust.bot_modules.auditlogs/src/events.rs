use gwevent::field::CategorizedField;
use include_dir::{include_dir, Dir};
use log::warn;
use poise::serenity_prelude::FullEvent;
use serenity::all::{ChannelId, CreateMessage};
use silverpelt::EventHandlerContext;

static DEFAULT_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

pub fn load_embedded_event_template(event: &str) -> Result<String, silverpelt::Error> {
    let template = match DEFAULT_TEMPLATES.get_file(format!("{}.art", event)) {
        Some(template) => template,
        None => {
            // Load default.art
            DEFAULT_TEMPLATES
                .get_file("default.art")
                .ok_or("Failed to load default template")?
        }
    };

    let template_str = template.contents_utf8().ok_or("Failed to load template")?;

    Ok(template_str.to_string())
}

#[inline]
pub const fn not_audit_loggable_event() -> &'static [&'static str] {
    &[
        "CACHE_READY",         // Internal
        "INTERACTION_CREATE",  // Spams too much / is useless
        "RATELIMIT",           // Internal
        "GUILD_CREATE",        // Internal
        "GUILD_MEMBERS_CHUNK", // Internal
    ]
}

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    if not_audit_loggable_event().contains(&event.into()) {
        return Ok(());
    }

    // (hopefully temporary) work around to reduce spam
    match event {
        FullEvent::GuildAuditLogEntryCreate { .. } => {}
        _ => match gwevent::core::get_event_user_id(event) {
            Ok(user_id) => {
                if user_id == ctx.cache.current_user().id {
                    return Ok(());
                }
            }
            Err(Some(e)) => {
                return Err(e);
            }
            Err(None) => {}
        },
    }

    let Some(expanded_event) = gwevent::core::expand_event(event.clone()) else {
        // Event cannot be expanded, ignore
        return Ok(());
    };

    // Convert to titlecase by capitalizing the first letter of each word
    let event_titlename = event
        .snake_case_name()
        .split('_')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    let event_name: &'static str = event.into();

    dispatch_audit_log(
        ctx,
        &ectx.data,
        event_name,
        &event_titlename,
        expanded_event,
        ectx.guild_id,
    )
    .await
}

/// Check if an event matches a list of filters
///
/// Rules:
/// - If filter is empty, return true unless a special case applies
/// - If filter matches the event_name, return true unless a special case applies
///
/// Special cases:
/// - If filter starts with `R/`, treat it as a regex
/// - If event_name is MESSAGE, then it must be an exact match to be dispatched AND must have a custom template declared for it. This is to avoid spam
pub async fn should_dispatch_event(
    event_name: &str,
    filters: &[String],
    uses_custom_template: bool,
    silverpelt_cache: &silverpelt::cache::SilverpeltCache,
) -> Result<bool, silverpelt::Error> {
    if event_name == "MESSAGE" {
        if !filters.contains(&event_name.to_string()) {
            return Ok(false);
        }

        if !uses_custom_template {
            return Ok(false);
        }

        return Ok(true);
    }

    // If empty, always return Ok
    if filters.is_empty() {
        return Ok(true);
    }

    let mut regexes = Vec::new();

    for filter in filters.iter() {
        if filter.starts_with("R/") && filter.len() > 2 {
            regexes.push(&filter[2..]);
        }

        if event_name == filter {
            return Ok(true);
        }
    }

    for regex in regexes {
        match silverpelt_cache.regex_match(regex, event_name).await {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(e) => {
                log::warn!("Failed to match regex: {}", e);
            }
        };
    }

    Ok(false)
}

pub async fn dispatch_audit_log(
    ctx: &serenity::all::client::Context,
    data: &silverpelt::data::Data,
    event_name: &str,
    event_titlename: &str,
    expanded_event: indexmap::IndexMap<String, CategorizedField>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), silverpelt::Error> {
    let sinks = super::cache::get_sinks(guild_id, &data.pool).await?;

    if sinks.is_empty() {
        return Ok(());
    }

    let event_json = serde_json::to_string(&serde_json::json! {
        {
            "event": expanded_event,
            "event_name": event_name,
            "event_titlename": event_titlename,
        }
    })?;

    for sink in sinks.iter() {
        // Verify event dispatch
        if !should_dispatch_event(
            event_name,
            {
                // False positive, unwrap_or_default cannot be used here as it moves the event out of the sink
                #[allow(clippy::manual_unwrap_or_default)]
                if let Some(ref events) = sink.events {
                    events
                } else {
                    &[]
                }
            },
            {
                if let Some(ref embed_template) = sink.embed_template {
                    !embed_template.is_empty()
                } else {
                    false
                }
            },
            &data.silverpelt_cache,
        )
        .await?
        {
            continue;
        }

        let template = {
            if let Some(ref embed_template) = sink.embed_template {
                if !embed_template.is_empty() {
                    embed_template.clone()
                } else {
                    // Load default template
                    load_embedded_event_template(event_name)?
                }
            } else {
                load_embedded_event_template(event_name)?
            }
        };

        let discord_reply = match templating::render_message_template(
            guild_id,
            &template,
            data.pool.clone(),
            templating::core::MessageTemplateContext {
                event_titlename: event_titlename.to_string(),
                event_name: event_name.to_string(),
                fields: expanded_event.clone(),
            },
            templating::CompileTemplateOptions {
                ignore_cache: false,
                cache_result: true,
            },
        )
        .await
        {
            Ok(reply) => reply,
            Err(e) => {
                let embed = serenity::all::CreateEmbed::default()
                    .description(format!("Failed to render template: {}", e));

                templating::core::DiscordReply {
                    embeds: vec![embed],
                    ..Default::default()
                }
            }
        };

        match sink.typ.as_str() {
            "channel" => {
                let channel: ChannelId = sink.sink.parse()?;

                let mut message = CreateMessage::default().embeds(discord_reply.embeds);

                if let Some(content) = discord_reply.content {
                    message = message.content(content);
                }

                if sink.send_json_context {
                    message = message.add_file(serenity::all::CreateAttachment::bytes(
                        event_json.clone().into_bytes(),
                        "event_data.json",
                    ))
                }

                match channel.send_message(&ctx.http, message).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!(
                            "Failed to send audit log event to channel: {} [sink id: {}]",
                            e, sink.id
                        );

                        if let serenity::Error::Http(
                            serenity::http::HttpError::UnsuccessfulRequest(ref err),
                        ) = e
                        {
                            match err.status_code {
                                reqwest::StatusCode::FORBIDDEN
                                | reqwest::StatusCode::UNAUTHORIZED
                                | reqwest::StatusCode::NOT_FOUND
                                | reqwest::StatusCode::GONE => {
                                    sqlx::query!(
                                        "UPDATE auditlogs__sinks SET broken = true WHERE id = $1",
                                        sink.id
                                    )
                                    .execute(&data.pool)
                                    .await?;
                                }
                                _ => {}
                            }
                        }
                    }
                };
            }
            "discord_webhook" => {
                let parsed_sink = sink.sink.parse()?;
                let Some((id, token)) = serenity::utils::parse_webhook(&parsed_sink) else {
                    warn!(
                        "Failed to parse webhook URL: {} [sink id: {}]",
                        sink.sink, sink.id
                    );
                    continue;
                };

                let mut files = Vec::new();
                if sink.send_json_context {
                    files.push(serenity::all::CreateAttachment::bytes(
                        event_json.clone().into_bytes(),
                        "event_data.json",
                    ));
                }

                if let Err(serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(
                    ref err,
                ))) = ctx
                    .http
                    .execute_webhook(id, None, token, false, files, &discord_reply)
                    .await
                {
                    match err.status_code {
                        reqwest::StatusCode::FORBIDDEN
                        | reqwest::StatusCode::UNAUTHORIZED
                        | reqwest::StatusCode::NOT_FOUND
                        | reqwest::StatusCode::GONE => {
                            sqlx::query!(
                                "UPDATE auditlogs__sinks SET broken = true WHERE id = $1",
                                sink.id
                            )
                            .execute(&data.pool)
                            .await?;
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                warn!("Unknown sink type: {} [sink id: {}]", sink.typ, sink.id);
            }
        }

        log::info!("Dispatched audit log event: {}", event_name);
    }

    Ok(())
}