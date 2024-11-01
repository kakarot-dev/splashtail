use module_settings::types::OperationType;
use silverpelt::module::{CommandObj, Module};
use silverpelt::types::CommandExtendedData;

/// Base command for a virtual settings command
#[poise::command(slash_command)]
async fn config_opt_base_cmd(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}

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

    // Add in the settings related commands as virtual commands to allow configuring permissions while not listing in the bot
    for config_opt in module.config_options() {
        let mut created_cmd = config_opt_base_cmd();

        for (operation_type, _) in config_opt.operations.iter() {
            let mut subcmd = config_opt_base_cmd();
            subcmd.name = operation_type.corresponding_command_suffix().to_string();
            subcmd.qualified_name = operation_type.corresponding_command_suffix().to_string();
            subcmd.description = {
                match operation_type {
                    OperationType::View => Some(format!("View {}", config_opt.id)),
                    OperationType::Create => Some(format!("Create {}", config_opt.id)),
                    OperationType::Update => Some(format!("Update {}", config_opt.id)),
                    OperationType::Delete => Some(format!("Delete {}", config_opt.id)),
                }
            };
            created_cmd.subcommands.push(subcmd);
        }

        let mut extended_data = indexmap::IndexMap::new();

        // Add base command to extended data
        let mut command_extended_data =
            CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

        command_extended_data.virtual_command = true; // Ensure its virtual

        extended_data.insert("", command_extended_data);

        for sub in created_cmd.subcommands.iter() {
            let mut command_extended_data =
                CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

            command_extended_data.virtual_command = true; // Ensure its virtual

            extended_data.insert(
                string_to_static_str(sub.name.to_string()),
                command_extended_data,
            );
        }

        commands.push((created_cmd, extended_data));
    }

    commands
}
