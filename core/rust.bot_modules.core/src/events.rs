use silverpelt::ar_event::EventHandlerContext;

pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
        silverpelt::ar_event::AntiraidEvent::Discord(ref fe) => {
            match fe {
                serenity::all::FullEvent::GuildCreate { guild, is_new } => {
                    if is_new.unwrap_or(false) == false {
                        return Ok(()); // Don't send welcome message to existing guilds
                    }

                    let owner_dm = guild.owner_id.create_dm_channel(ctx).await?;

                    owner_dm
                        .send_message(
                            &ctx.http, 
                            serenity::all::CreateMessage::new()
                            .embed(
                                serenity::all::CreateEmbed::new()
                                .title("Thank you for adding AntiRaid")
                                .description(r#"While you have successfully added AntiRaid to your server, it won't do much until you take some time to configure it to your needs.
        
Please check out both the `User Guide` and the `Website` to tailor AntiRaid to the needs of your server! And, if you need help, feel free to join our `Support Server`!  
                                "#)
                            )
                            .components(
                                vec![
                                    serenity::all::CreateActionRow::Buttons(
                                        vec![
                                            serenity::all::CreateButton::new_link(
                                                config::CONFIG.sites.docs.clone(),
                                            )
                                            .label("User Guide"),
                                            serenity::all::CreateButton::new_link(
                                                config::CONFIG.sites.frontend.clone(),
                                            )
                                            .label("Website"),
                                            serenity::all::CreateButton::new_link(
                                                config::CONFIG.meta.support_server_invite.clone(),
                                            )
                                            .label("Support Server")
                                        ]
                                    )
                                ]
                            )
                        )
                        .await?;

                    Ok(())
                }
                _ => Ok(())
            }
        },
        silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((ref event_name, ref data)) => {
            if event_name != "settings.clearModuleEnabledCache" {
                return Ok(()); // Ignore all other events
            }

            if ectx.guild_id == silverpelt::ar_event::SYSTEM_GUILD_ID {
                ectx.data
                    .silverpelt_cache
                    .module_enabled_cache
                    .invalidate_all();
            } else {
                // Check for module data
                #[derive(serde::Deserialize)]
                pub struct ClearModuleEnabledCache {
                    module: Option<String>,
                }

                let cmc = match serde_json::from_value::<ClearModuleEnabledCache>(data.clone()) {
                    Ok(cmc) => cmc,
                    Err(e) => {
                        log::error!("Failed to deserialize ClearModuleEnabledCache: {}", e);
                        return Ok(());
                    }
                };

                if let Some(module) = cmc.module {
                    ectx.data
                        .silverpelt_cache
                        .module_enabled_cache
                        .invalidate(&(ectx.guild_id, module))
                        .await;
                } else {
                    // Global enable/disable the module by iterating the entire cache
                    for (k, _) in ectx.data.silverpelt_cache.module_enabled_cache.iter() {
                        if k.0 == ectx.guild_id {
                            ectx.data
                                .silverpelt_cache
                                .module_enabled_cache
                                .invalidate(&(k.0, k.1.clone()))
                                .await;
                        }
                    }
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
