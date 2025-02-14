extern crate vergen;
use anyhow::Result;
use vergen::*;

//type Error = Box<dyn std::error::Error + Send + Sync>;

/// Build src/modules/mod.rs based on the folder listing of src/modules
fn autogen_modules_mod_rs() -> Result<()> {
    const MODULE_TEMPLATE: &str = r#"
// Auto-generated by build.rs
{module_use_list}

/// List of modules available. Not all may be enabled
pub fn modules() -> Vec<crate::silverpelt::Module> {
    vec![
        {module_func_list}
    ]
}
    "#;

    let mut module_list = Vec::new();

    let folder_list = std::fs::read_dir("src/modules")?;

    for folder in folder_list {
        let folder = folder?;

        if !folder.file_type().unwrap().is_dir() {
            continue;
        }

        // Check that a mod.rs file exists in the folder
        let mod_rs_path = folder.path().join("mod.rs");

        // A TOCTOU here isn't important as this is just a one-of build script
        if !mod_rs_path.exists() {
            continue;
        }

        let folder_name = folder.file_name().into_string().unwrap();

        module_list.push(folder_name);
    }

    module_list.sort();

    // Move root to bottom
    if let Some(root_index) = module_list.iter().position(|x| x == "root") {
        let root = module_list.remove(root_index);
        module_list.push(root);
    }

    // Construct module_uses_list
    let mut module_use_list = Vec::new();

    for module in &module_list {
        module_use_list.push(format!("mod {};", module));
    }

    let module_use_list = module_use_list.join("\n");

    // Construct module_list
    let mut module_dat_list = Vec::new();

    for module in &module_list {
        module_dat_list.push(format!("{}::module(),", module));
    }

    let module_func_list = module_dat_list.join("\n        ");
    
    let mut module_ids_list = Vec::new();

    for module in &module_list {
        module_ids_list.push(format!("\"{}\",", module));
    }

    let module_list_final = MODULE_TEMPLATE
        .replace("{module_use_list}", &module_use_list)
        .replace("{module_func_list}", &module_func_list)
        .replace("{module_ids_list}", &module_ids_list.join("\n        ")); // Not used currently but may be used in the future

    let module_list_final = module_list_final.trim().to_string();

    std::fs::write("src/modules/mod.rs", module_list_final)?;

    Ok(())
}

fn main() -> Result<()> {
    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Normal;

    *config.git_mut().semver_kind_mut() = SemverKind::Normal;

    *config.git_mut().semver_mut() = true;

    *config.git_mut().semver_dirty_mut() = Some("-dirty");

    vergen(config)?;

    // Run the autogen stuff
    autogen_modules_mod_rs()?;

    Ok(())
}
