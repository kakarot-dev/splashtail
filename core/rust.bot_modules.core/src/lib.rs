mod commands;
mod help;
mod modules;
mod ping;
mod sandwich_status_task;
mod settings;
mod stats;
mod whois;

use futures_util::future::FutureExt;
use indexmap::indexmap;

pub struct Module;

#[async_trait::async_trait]
impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "core"
    }

    fn name(&self) -> &'static str {
        "Core Commands"
    }

    fn description(&self) -> &'static str {
        "Core commands for the bot"
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn commands_toggleable(&self) -> bool {
        true
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
            (
                help::help(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                help::simplehelp(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                stats::stats(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                ping::ping(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                whois::whois(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                modules::modules(),
                indexmap! {
                    "" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "*"),
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                    "modperms" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "modperms"),
                },
            ),
            (
                commands::commands(),
                indexmap! {
                    "check" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "check"),
                    "enable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "enable"),
                    "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "disable"),
                    "modperms" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "modperms"),
                },
            ),
        ]
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![(
            botox::taskman::Task {
                name: "Sandwich Status Task",
                description: "Checks the status of the sandwich http server",
                duration: std::time::Duration::from_secs(30),
                enabled: true,
                run: Box::new(move |ctx| sandwich_status_task::sandwich_status_task(ctx).boxed()),
            },
            |_ctx| (true, "Sandwich HTTP API is enabled".to_string()),
        )]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::GUILD_ROLES).clone(),
            (*settings::GUILD_MEMBERS).clone(),
            (*settings::GUILD_TEMPLATES).clone(),
            (*settings::GUILD_TEMPLATES_KV).clone(),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn full_command_list(&self) -> Vec<silverpelt::module::CommandObj> {
        modules_ext::create_full_command_list(self)
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn event_handler(
        &self,
        ectx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
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

                    let cmc = match serde_json::from_value::<ClearModuleEnabledCache>(data.clone())
                    {
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

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        match event {
            silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((event_name, _)) => {
                event_name == "settings.clearModuleEnabledCache"
            }
            _ => false,
        }
    }
}
