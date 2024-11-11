// Generates AntiRaid documentation from docgen data
use templating_docgen::{Field, Method, Parameter, Plugin, Type};

pub fn document_all_plugins() -> String {
    let mut markdown = String::new();

    for (plugin_name, data) in templating::PLUGINS.iter() {
        if data.1.is_none() {
            eprintln!(
                "Skipping plugin {} as it has no documentation available",
                plugin_name
            );
            continue;
        }

        let plugin = data.1.unwrap()();

        if plugin.name != *plugin_name {
            panic!("Plugin name mismatch: {} != {}", plugin.name, plugin_name);
        }

        markdown.push_str(&generate_markdown_for_plugin(plugin));
        markdown.push_str("\n\n---\n\n");
    }

    markdown
}

fn generate_markdown_for_plugin(plugin: Plugin) -> String {
    let mut markdown = String::new();

    // Write Base Info
    markdown.push_str(&format!("# {}\n\n", plugin.name));

    if !plugin.description.is_empty() {
        markdown.push_str(&format!("{}\n\n", plugin.description));
    }

    // Document the types
    if !plugin.types.is_empty() {
        markdown.push_str("## Types\n\n");

        plugin.types.iter().for_each(|typ| {
            markdown.push_str(&format!("{}\n\n", type_to_string(typ)));
        });
    }

    // Document the methods
    if !plugin.methods.is_empty() {
        markdown.push_str("## Methods\n\n");

        plugin.methods.iter().for_each(|method| {
            markdown.push_str(&format!("{}\n\n", method_to_string(method, None)));
        });
    }

    markdown // TODO: Implement the rest of the function
}

fn type_to_string(typ: &Type) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!(
        "<div id=\"type.{}\" />\n\n### {}\n\n{}\n\n",
        typ.name, typ.name, typ.description
    ));

    if let Some(ref example) = typ.example {
        let example_json = serde_json::to_string_pretty(&example).unwrap();

        markdown.push_str(&format!("```json\n{}\n```", example_json));
    }

    if !typ.fields.is_empty() {
        markdown.push_str("\n\n#### Fields\n\n");

        typ.fields.iter().for_each(|field| {
            markdown.push_str(&format!("{}\n", field_to_string(field)));
        });
    }

    if !typ.methods.is_empty() {
        markdown.push_str("\n\n#### Methods\n\n");

        typ.methods.iter().for_each(|method| {
            markdown.push_str(&format!(
                "{}\n",
                method_to_string(method, Some(typ.name.clone()))
            ));
        });
    }

    markdown
}

fn method_to_string(method: &Method, cls: Option<String>) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!(
        "### {}\n\n```lua\n{}\n```",
        method.func_name(&cls),
        method.type_signature(&cls)
    ));

    if !method.description.is_empty() {
        markdown.push_str(&format!("\n\n{}", method.description));
    }

    if !method.generics.is_empty() {
        markdown.push_str("\n\n#### Generics\n\n");

        method.generics.iter().for_each(|gen| {
            markdown.push_str(&format!("- `{}`: {}", gen.param, gen.type_signature()));
        });
    }

    if !method.parameters.is_empty() {
        markdown.push_str("\n\n#### Parameters\n\n");

        method.parameters.iter().for_each(|param| {
            markdown.push_str(&format!("{}\n", param_to_string(param)));
        });
    }

    if !method.returns.is_empty() {
        markdown.push_str("\n\n#### Returns\n\n");

        method.returns.iter().for_each(|ret| {
            markdown.push_str(&param_to_string(ret));
        });
    }

    markdown
}

fn field_to_string(field: &Field) -> String {
    format!(
        "- `{}` ({}): {}",
        field.name,
        typeref_to_link(&field.r#type),
        field.description
    )
}

fn param_to_string(param: &Parameter) -> String {
    format!(
        "- `{}` ({}): {}",
        param.name,
        typeref_to_link(&param.r#type),
        param.description
    )
}

fn typeref_to_link(tref: &str) -> String {
    if tref.contains("::") {
        // Module on docs.rs, generate link
        // E.g. std::sync::Arc -> [std::sync::Arc](https://docs.rs/std/latest/std/sync/struct.Arc.html)
        // serenity::model::user::User -> [serenity::model::user::User](https://docs.rs/serenity/latest/serenity/model/user/struct.User.html)

        let mut parts = tref.split("::").collect::<Vec<_>>();
        let last = parts.pop().unwrap();
        let first = parts.remove(0);

        let mut url = format!("https://docs.rs/{}/latest/{}/", first, first);
        url.push_str(&parts.join("/"));
        url.push_str(&format!("/struct.{}.html", last));

        format!("[{}]({})", tref, url)
    } else {
        format!("[{}](#type.{})", tref, {
            let mut tref = tref.to_string();

            // Handle tables
            if tref.starts_with('{') {
                tref.remove(0);
            }

            if tref.ends_with('}') {
                tref.pop();
            }

            // Handle optional
            if tref.ends_with('?') {
                tref.pop();
            }

            tref
        })
    }
}
