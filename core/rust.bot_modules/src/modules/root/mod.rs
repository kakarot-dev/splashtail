mod cmds;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        description: "Commands that are only available to staff members.",
        toggleable: false,
        commands_toggleable: false,
        virtual_module: false,
        web_hidden: true,
        is_default_enabled: true,
        // These commands do not follow the typical permission system anyways
        commands: vec![(
            cmds::sudo(),
            indexmap::indexmap! {
                "register" => crate::silverpelt::CommandExtendedData::none(),
                "cub" => crate::silverpelt::CommandExtendedData::none(),
            },
        )],
        ..Default::default()
    }
}