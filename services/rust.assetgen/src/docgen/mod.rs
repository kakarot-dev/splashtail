// Generates AntiRaid documentation from docgen data
use templating_docgen::{Field, Method, Parameter, Plugin, Primitive, PrimitiveConstraint, Type};

pub fn create_documentation() -> String {
    let mut markdown = String::new();

    // First, document all the plugins
    markdown.push_str(&document_all_plugins(1));

    // Next, document all the primitive types
    markdown.push_str(&document_all_primitives(1));

    markdown
}

pub fn document_all_plugins(heading_level: usize) -> String {
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

        markdown.push_str(&generate_markdown_for_plugin(plugin, heading_level));
        markdown.push_str("\n\n---\n\n");
    }

    markdown
}

pub fn document_all_primitives(heading_level: usize) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!("{} Primitives\n\n", _headings(heading_level)));

    for primitive in templating::primitives_docs::document_primitives() {
        markdown.push_str(&generate_markdown_for_primitive(
            primitive,
            heading_level + 1,
        ));
        markdown.push_str("\n\n---\n\n");
    }

    markdown
}

fn generate_markdown_for_plugin(plugin: Plugin, heading_level: usize) -> String {
    let mut markdown = String::new();

    // Write Base Info
    markdown.push_str(&format!("{} {}\n\n", _headings(heading_level), plugin.name));

    if !plugin.description.is_empty() {
        markdown.push_str(&format!("{}\n\n", plugin.description));
    }

    // Document the types
    if !plugin.types.is_empty() {
        markdown.push_str(&format!("{} Types\n\n", _headings(heading_level + 1)));

        plugin.types.iter().for_each(|typ| {
            markdown.push_str(&format!("{}\n\n", type_to_string(typ, heading_level + 2)));
        });
    }

    // Document the methods
    if !plugin.methods.is_empty() {
        markdown.push_str(&format!("{} Methods\n\n", _headings(heading_level + 1)));

        plugin.methods.iter().for_each(|method| {
            markdown.push_str(&format!(
                "{}\n\n",
                method_to_string(method, None, heading_level + 2)
            ));
        });
    }

    markdown // TODO: Implement the rest of the function
}

fn generate_markdown_for_primitive(primitive: Primitive, heading_level: usize) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!(
        "<div id=\"type.{}\" />\n\n{} {}\n\n``{}``\n\n{}",
        primitive.name,
        _headings(heading_level),
        primitive.name,
        primitive.type_definition(),
        primitive.description
    ));

    // Add Constraints if any
    if !primitive.constraints.is_empty() {
        markdown.push_str(&format!(
            "\n\n{} Constraints\n\n",
            _headings(heading_level + 1)
        ));

        markdown.push_str(
            &primitive
                .constraints
                .iter()
                .map(|constraint| primitive_constraint_to_string(constraint))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    markdown
}

fn primitive_constraint_to_string(p_constraint: &PrimitiveConstraint) -> String {
    format!(
        "- **{}**: {} (accepted values: {})",
        p_constraint.name, p_constraint.description, p_constraint.accepted_values
    )
}

fn type_to_string(typ: &Type, heading_level: usize) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!(
        "<div id=\"type.{}\" />\n\n{} {}\n\n{}\n\n",
        typ.name,
        _headings(heading_level),
        typ.genericized_name(),
        typ.description
    ));

    if let Some(ref example) = typ.example {
        let example_json = serde_json::to_string_pretty(&example).unwrap();

        markdown.push_str(&format!("```json\n{}\n```", example_json));
    }

    if !typ.fields.is_empty() {
        markdown.push_str(&format!("\n\n{} Fields\n\n", _headings(heading_level + 1)));

        typ.fields.iter().for_each(|field| {
            markdown.push_str(&format!("{}\n", field_to_string(field)));
        });
    }

    if !typ.methods.is_empty() {
        markdown.push_str(&format!("\n\n{} Methods\n\n", _headings(heading_level + 1)));

        typ.methods.iter().for_each(|method| {
            markdown.push_str(&format!(
                "{}\n",
                method_to_string(method, Some(typ.name.clone()), heading_level + 2),
            ));
        });
    }

    markdown
}

fn method_to_string(method: &Method, cls: Option<String>, heading_level: usize) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!(
        "{} {}\n\n```lua\n{}\n```",
        _headings(heading_level),
        method.func_name(&cls),
        method.type_signature(&cls)
    ));

    if !method.description.is_empty() {
        markdown.push_str(&format!("\n\n{}", method.description));
    }

    if !method.parameters.is_empty() {
        markdown.push_str(&format!(
            "\n\n{} Parameters\n\n",
            _headings(heading_level + 1)
        ));

        method.parameters.iter().for_each(|param| {
            markdown.push_str(&format!("{}\n", param_to_string(param)));
        });
    }

    if !method.returns.is_empty() {
        markdown.push_str(&format!("\n\n{} Returns\n\n", _headings(heading_level + 1)));

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
    } else if tref.starts_with("<") {
        format!("`{}`", tref)
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

/// Helper function to generate a string of `#` characters
fn _headings(level: usize) -> String {
    let mut s = String::new();

    for _ in 0..level {
        s.push('#');
    }

    s
}
