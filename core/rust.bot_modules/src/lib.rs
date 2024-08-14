// Auto-generated by build.rs. Edits will be overwritten
pub mod acl;
pub mod afk;
pub mod auditlogs;
pub mod core;
pub mod gitlogs;
pub mod info;
pub mod inspector;
pub mod limits;
pub mod moderation;
pub mod punishments;
pub mod server_backups;
pub mod server_member_backups;
pub mod settings;
pub mod temporary_punishments;
pub mod root;

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<silverpelt::Module> {
    vec![
        acl::module().parse(),
        afk::module().parse(),
        auditlogs::module().parse(),
        core::module().parse(),
        gitlogs::module().parse(),
        info::module().parse(),
        inspector::module().parse(),
        limits::module().parse(),
        moderation::module().parse(),
        punishments::module().parse(),
        server_backups::module().parse(),
        server_member_backups::module().parse(),
        settings::module().parse(),
        temporary_punishments::module().parse(),
        root::module().parse(),
    ]
}

/// Module id list
pub fn module_ids() -> Vec<&'static str> {
    vec![
        "acl",
        "afk",
        "auditlogs",
        "core",
        "gitlogs",
        "info",
        "inspector",
        "limits",
        "moderation",
        "punishments",
        "server_backups",
        "server_member_backups",
        "settings",
        "temporary_punishments",
        "root",
    ]
}