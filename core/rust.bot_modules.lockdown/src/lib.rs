pub mod cache;
pub mod cmds;
pub mod core;
pub mod settings;

use silverpelt::types::CommandExtendedData;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "lockdown"
    }

    fn name(&self) -> &'static str {
        "Lockdown"
    }

    fn description(&self) -> &'static str {
        "Lockdown module for quickly locking/unlocking your whole server or individual channels"
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::LOCKDOWN_SETTINGS).clone(),
            (*settings::LOCKDOWNS).clone(),
        ]
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::lockdowns(),
            indexmap::indexmap! {
                "list" => CommandExtendedData::kittycat_or_admin("lockdowns", "list"),
                "tsl" => CommandExtendedData::kittycat_or_admin("lockdowns", "create"),
                "qsl" => CommandExtendedData::kittycat_or_admin("lockdowns", "create"),
                "scl" => CommandExtendedData::kittycat_or_admin("lockdowns", "create"),
                "remove" => CommandExtendedData::kittycat_or_admin("lockdowns", "remove"),
            },
        )]
    }

    fn full_command_list(&self) -> Vec<silverpelt::module::CommandObj> {
        modules_ext::create_full_command_list(self)
    }
}
