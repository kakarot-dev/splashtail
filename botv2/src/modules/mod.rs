// Auto-generated by build.rs
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
pub mod web;
pub mod root;

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<crate::silverpelt::Module> {
    vec![
        auditlogs::module(),
        core::module(),
        gitlogs::module(),
        info::module(),
        inspector::module(),
        limits::module(),
        moderation::module(),
        punishments::module(),
        server_backups::module(),
        server_member_backups::module(),
        settings::module(),
        temporary_punishments::module(),
        web::module(),
        root::module(),
    ]
}