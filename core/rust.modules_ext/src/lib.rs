use silverpelt::module::{CommandObj, Module};
use silverpelt::types::CommandExtendedData;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn create_full_command_list<T: Module + ?Sized>(module: &T) -> Vec<CommandObj> {
    #[poise::command(slash_command, rename = "")]
    pub async fn base_cmd(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
        Ok(())
    }

    let mut commands = module.raw_commands();

    // acl__{module}_defaultperms_check is a special command that is added to all modules
    let mut acl_module_defaultperms_check = base_cmd();
    acl_module_defaultperms_check.name = format!("acl__{}_defaultperms_check", module.id());
    acl_module_defaultperms_check.qualified_name =
        format!("acl__{}_defaultperms_check", module.id());
    commands.push((
        acl_module_defaultperms_check,
        indexmap::indexmap! {
            "" => CommandExtendedData {
                virtual_command: true,
                ..Default::default()
            },
        },
    ));

    // Add in the settings related commands
    for config_opt in module.config_options() {
        let created_cmd =
            module_settings_poise::settings_autogen::create_poise_commands_from_setting(
                module.id(),
                &config_opt,
            );

        let mut extended_data = indexmap::IndexMap::new();

        // Add base command to extended data
        let mut command_extended_data =
            CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

        if module.root_module() {
            command_extended_data.virtual_command = true; // Root modules should not have any settings related commands accessible by default
        }

        extended_data.insert("", command_extended_data);

        for sub in created_cmd.subcommands.iter() {
            let mut command_extended_data =
                CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

            if module.root_module() {
                command_extended_data.virtual_command = true; // Root modules should not have any settings related commands accessible by default
            }

            extended_data.insert(
                string_to_static_str(sub.name.to_string()),
                command_extended_data,
            );
        }

        commands.push((created_cmd, extended_data));
    }

    commands
}
