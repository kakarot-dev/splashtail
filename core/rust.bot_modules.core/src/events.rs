use silverpelt::ar_event::EventHandlerContext;

pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
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
