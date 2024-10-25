mod cmds;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "templating"
    }

    fn name(&self) -> &'static str {
        "Templating"
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Commands related to templating!"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::exec_template(),
            indexmap::indexmap! {
                "" => silverpelt::types::CommandExtendedData::kittycat_simple("templating", "exec_template")
            },
        )]
    }

    fn full_command_list(&self) -> Vec<silverpelt::module::CommandObj> {
        modules_ext::create_full_command_list(self)
    }
}
