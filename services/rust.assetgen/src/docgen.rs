// Generates AntiRaid documentation from docgen data

use templating_docgen::{Method, Parameter, Plugin, Type};

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

    // Document the methods
    if !plugin.methods.is_empty() {
        markdown.push_str("## Methods\n\n");

        plugin.methods.iter().for_each(|method| {
            markdown.push_str(&format!(
                "**{}**\n\n{}\n\n",
                method.name,
                method_to_string(method)
            ));
        });
    }

    markdown // TODO: Implement the rest of the function
}

fn method_to_string(method: &Method) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!("```lua\n{}\n```", method.type_signature()));

    if !method.description.is_empty() {
        markdown.push_str(&format!("\n\n{}", method.description));
    }

    if !method.generics.is_empty() {
        markdown.push_str("\n\n### Generics\n\n");

        method.generics.iter().for_each(|gen| {
            markdown.push_str(&format!("- `{}`: {}", gen.param, gen.type_signature()));
        });
    }

    if !method.parameters.is_empty() {
        markdown.push_str("\n\n### Parameters\n\n");

        method.parameters.iter().for_each(|param| {
            markdown.push_str(&format!("{}\n", param_to_string(param)));
        });
    }

    if !method.returns.is_empty() {
        markdown.push_str("\n\n### Returns\n\n");

        method.returns.iter().for_each(|ret| {
            markdown.push_str(&param_to_string(ret));
        });
    }

    markdown
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

        let mut url = format!("https://docs.rs/{}/latest/", parts.remove(0));
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
