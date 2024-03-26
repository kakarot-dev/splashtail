// Auto-generated by build.rs
pub mod auditlogs;
pub mod core;
pub mod gitlogs;
pub mod limits;
pub mod moderation;
pub mod server_backups;
pub mod server_member_backups;
pub mod settings;
pub mod root;

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<crate::silverpelt::Module> {
    vec![
        auditlogs::module(),
        core::module(),
        gitlogs::module(),
        limits::module(),
        moderation::module(),
        server_backups::module(),
        server_member_backups::module(),
        settings::module(),
        root::module(),
    ]
}