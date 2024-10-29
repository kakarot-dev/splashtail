use poise::serenity_prelude::FullEvent;
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};
use splashcore_rs::field::Field;

#[inline]
pub(crate) const fn not_audit_loggable_event() -> &'static [&'static str] {
    &[
        "CACHE_READY",         // Internal
        "INTERACTION_CREATE",  // Spams too much / is useless
        "RATELIMIT",           // Internal
        "GUILD_CREATE",        // Internal
        "GUILD_MEMBERS_CHUNK", // Internal
    ]
}

pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
        silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((ref event_name, ref data)) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/TrustedWebEvent",
                "(Anti Raid) Trusted Web Event",
                indexmap::indexmap! {
                    "event_name".to_string() => event_name.clone().into(),
                    "data".to_string() => data.clone().into(),
                },
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::Discord(ref event) => {
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

            let Some(event_data) = gwevent::core::expand_event(event.clone()) else {
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
                event_data,
                ectx.guild_id,
            )
            .await
        }

        AntiraidEvent::Custom(ref event) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                &event.event_name,
                &event.event_titlename,
                event.event_data.clone(),
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::StingCreate(ref sting) => {
            let sting_val = serde_json::to_value(sting)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/StingCreate",
                "(Anti Raid) Created Sting For User",
                indexmap::indexmap! {
                    "target".to_string() =>
                        match &sting.target {
                            silverpelt::stings::StingTarget::User(user_id) => (*user_id).into(),
                            silverpelt::stings::StingTarget::System => "System".to_string().into(),
                        },
                    "reason".to_string() =>
                        match &sting.reason {
                            Some(reason) => reason.clone().into(),
                            None => Field::None,
                        },
                    "stings".to_string() => sting.stings.into(),
                    "state".to_string() => sting.state.to_string().into(),
                    "sting_val".to_string() => sting_val.into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentCreate(ref punishments) => {
            let punishment = serde_json::to_value(punishments)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/PunishmentCreate",
                "(Anti Raid) Created Punishment",
                indexmap::indexmap! {
                    "punishment".to_string() => punishment.into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::MemberVerify((user_id, ref data)) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/MemberVerify",
                "(Anti Raid) Member Verify",
                indexmap::indexmap! {
                    "user_id".to_string() => user_id.into(),
                    "data".to_string() => data.clone().into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
    }
}

/// Check if an event matches a list of filters
///
/// Rules:
/// - If filter is empty, return true unless a special case applies
/// - If filter matches the event_name, return true unless a special case applies
///
/// Special cases:
/// - If event_name is MESSAGE, then it must be an exact match to be dispatched AND must have a custom template declared for it. This is to avoid spam
pub(crate) async fn should_dispatch_event(
    event_name: &str,
    filters: &[String],
) -> Result<bool, silverpelt::Error> {
    if event_name == "MESSAGE" {
        // Message should only be fired if the template explicitly wants MESSAGE events
        if !filters.contains(&event_name.to_string()) {
            return Ok(false);
        }

        return Ok(true);
    }

    // If empty, always return Ok
    if filters.is_empty() {
        return Ok(true);
    }

    Ok(filters.contains(&event_name.to_string()))
}

async fn dispatch_audit_log(
    ctx: &serenity::all::client::Context,
    data: &silverpelt::data::Data,
    event_name: &str,
    event_titlename: &str,
    event_data: indexmap::IndexMap<String, Field>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), silverpelt::Error> {
    let sinks = super::cache::get_sinks(guild_id, &data.pool).await?;

    if sinks.is_empty() {
        return Ok(());
    }

    for sink in sinks.iter() {
        // Verify event dispatch
        if !should_dispatch_event(event_name, {
            // False positive, unwrap_or_default cannot be used here as it moves the event out of the sink
            #[allow(clippy::manual_unwrap_or_default)]
            if let Some(ref events) = sink.events {
                events
            } else {
                &[]
            }
        })
        .await?
        {
            continue;
        }

        templating::execute::<_, Option<()>>(
            guild_id,
            templating::Template::Named(sink.template.clone()),
            data.pool.clone(),
            ctx.clone(),
            data.reqwest.clone(),
            HookContext {
                event_titlename: event_titlename.to_string(),
                event_name: event_name.to_string(),
                event_data: event_data.clone(),
                sink_id: sink.id.to_string(),
                sink: sink.sink.clone(),
            },
        )
        .await?;
    }

    Ok(())
}

/// A HookContext is a context for message templates
/// that can be accessed in hook templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct HookContext {
    pub event_titlename: String,
    pub event_name: String,
    pub event_data: indexmap::IndexMap<String, Field>,
    pub sink_id: String,
    pub sink: String,
}

#[typetag::serde]
impl templating::Context for HookContext {}
