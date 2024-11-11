/// Tooling to document the Anti-Raid Luau API.
//
use std::sync::Arc;

#[derive(Default, Debug, serde::Serialize, Clone)]
/// The root of the documentation.
pub struct Docs {
    pub plugins: Vec<Plugin>,
}

// Docs builder code
impl Docs {
    pub fn add_plugin(self, plugin: Plugin) -> Self {
        let mut d = self;
        d.plugins.push(plugin);
        d
    }

    pub fn build(self) -> Docs {
        self
    }
}

/// A plugin in the Anti-Raid Luau API.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
    pub types: Vec<Type>,
}

#[allow(dead_code)]
// Plugin builder code
impl Plugin {
    pub fn name(self, name: &str) -> Self {
        let mut p = self;
        p.name = name.to_string();
        p
    }

    pub fn description(self, description: &str) -> Self {
        let mut p = self;
        p.description = description.to_string();
        p
    }

    pub fn add_method(self, methods: Method) -> Self {
        let mut p = self;
        p.methods.push(methods);
        p
    }

    pub fn method_mut(mut self, name: &str, f: impl FnOnce(Method) -> Method) -> Self {
        let method = self.methods.iter_mut().find(|m| m.name == name);

        if let Some(method) = method {
            let new_method = f(method.clone());

            *method = new_method;
        } else {
            let mut method = Method::default();
            method.name = name.to_string();
            self.methods.push(f(method));
        }

        self
    }

    pub fn add_field(self, fields: Field) -> Self {
        let mut p = self;
        p.fields.push(fields);
        p
    }

    pub fn add_type(self, types: Type) -> Self {
        let mut p = self;
        p.types.push(types);
        p
    }

    pub fn type_mut(self, name: &str, description: &str, f: impl FnOnce(Type) -> Type) -> Self {
        let mut p = self;
        let new_typ = Type::new(name, description);
        p.types.push(f(new_typ));

        p
    }

    pub fn build(self) -> Plugin {
        self
    }
}

/*impl Plugin {
    pub fn to_markdown(&self) -> String {

    }
}*/

/// A method in a plugin.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Method {
    pub name: String,
    pub generics: Vec<MethodGeneric>,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub returns: Vec<Parameter>,
}

// Method builder code
impl Method {
    pub fn name(self, name: &str) -> Self {
        let mut m = self;
        m.name = name.to_string();
        m
    }

    pub fn add_generic(self, generics: MethodGeneric) -> Self {
        let mut m = self;
        m.generics.push(generics);
        m
    }

    pub fn description(self, description: &str) -> Self {
        let mut m = self;
        m.description = description.to_string();
        m
    }

    pub fn add_parameter(self, parameters: Parameter) -> Self {
        let mut m = self;
        m.parameters.push(parameters);
        m
    }

    pub fn parameter(&mut self, name: &str, f: impl FnOnce(Parameter) -> Parameter) -> Self {
        let parameter = self.parameters.iter_mut().find(|p| p.name == name);

        if let Some(parameter) = parameter {
            let new_parameter = f(parameter.clone());

            *parameter = new_parameter;
        } else {
            let mut parameter = Parameter::default();
            parameter.name = name.to_string();
            self.parameters.push(f(parameter));
        }

        self.clone()
    }

    pub fn add_return(self, returns: Parameter) -> Self {
        let mut m = self;
        m.returns.push(returns);
        m
    }

    pub fn return_(&mut self, name: &str, f: impl FnOnce(Parameter) -> Parameter) -> Self {
        let return_ = self.returns.iter_mut().find(|r| r.name == name);

        if let Some(return_) = return_ {
            let new_return_ = f(return_.clone());

            *return_ = new_return_;
        } else {
            let mut return_ = Parameter::default();
            return_.name = name.to_string();
            self.returns.push(f(return_));
        }

        self.clone()
    }

    pub fn build(self) -> Method {
        self
    }
}

impl Method {
    pub fn func_name(&self, cls: &Option<String>) -> String {
        if let Some(cls) = cls {
            format!("{}:{}", cls, self.name)
        } else {
            self.name.clone()
        }
    }

    /// Format: function name<GENERICS>(parameters) -> returns
    pub fn type_signature(&self, cls: &Option<String>) -> String {
        let mut out = String::new();
        out.push_str(&format!("function {}", self.func_name(cls)));

        // Add in the generics if they exist
        if !self.generics.is_empty() {
            out.push_str(&format!(
                "<{}>",
                self.generics
                    .iter()
                    .map(|g| g.type_signature())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        out.push_str(&format!(
            "({})",
            self.parameters
                .iter()
                .map(|p| p.type_signature())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if self.returns.len() == 1 {
            out.push_str(&format!(" -> {}", self.returns[0].r#type.clone()));
        } else if self.returns.len() > 2 {
            out.push_str(&format!(
                " -> ({})",
                self.returns
                    .iter()
                    .map(|r| r.r#type.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        out
    }
}

/// A generic in a method.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct MethodGeneric {
    pub param: String,
    pub constraints: String,
}

// MethodGeneric builder code
impl MethodGeneric {
    pub fn param(self, param: &str) -> Self {
        let mut m = self;
        m.param = param.to_string();
        m
    }

    pub fn constraints(self, constraints: String) -> Self {
        let mut m = self;
        m.constraints = constraints;
        m
    }

    pub fn build(self) -> MethodGeneric {
        self
    }
}

impl MethodGeneric {
    /// Format: <param>: <constraints>
    pub fn type_signature(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}: {}", self.param, self.constraints));
        out
    }
}

/// A parameter in a method.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub r#type: String,
}

// Parameter builder code
impl Parameter {
    pub fn name(self, name: &str) -> Self {
        let mut p = self;
        p.name = name.to_string();
        p
    }

    pub fn description(self, description: &str) -> Self {
        let mut p = self;
        p.description = description.to_string();
        p
    }

    pub fn typ(self, r#type: &str) -> Self {
        let mut p = self;
        p.r#type = r#type.to_string();
        p
    }

    pub fn build(self) -> Parameter {
        self
    }
}

impl Parameter {
    /// Format: <name>: <description>
    pub fn type_signature(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}: {}", self.name, self.r#type));
        out
    }
}

#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Field {
    pub name: String,
    pub description: String,
    pub r#type: String,
}

// Field builder code
impl Field {
    pub fn name(self, name: &str) -> Self {
        let mut f = self;
        f.name = name.to_string();
        f
    }

    pub fn description(self, description: &str) -> Self {
        let mut f = self;
        f.description = description.to_string();
        f
    }

    pub fn typ(self, r#type: &str) -> Self {
        let mut f = self;
        f.r#type = r#type.to_string();
        f
    }

    pub fn build(self) -> Field {
        self
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Type {
    pub name: String,
    pub description: String,
    pub example: Option<Arc<dyn erased_serde::Serialize + Send + Sync>>,
    pub fields: Vec<Field>, // Description of the fields in type
    pub methods: Vec<Method>,
}

impl Default for Type {
    fn default() -> Self {
        Type {
            name: String::new(),
            description: String::new(),
            example: None,
            methods: Vec::new(),
            fields: Vec::new(),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type")
            .field("description", &self.description)
            .field("methods", &self.methods)
            .finish()
    }
}

// Type builder code
impl Type {
    pub fn new(name: &str, description: &str) -> Self {
        let mut t = Type::default();
        t.name = name.to_string();
        t.description = description.to_string();
        t
    }

    pub fn example(self, example: Arc<dyn erased_serde::Serialize + Send + Sync>) -> Self {
        let mut t = self;
        t.example = Some(example);
        t
    }

    pub fn method_mut(&mut self, name: &str, f: impl FnOnce(Method) -> Method) -> Self {
        let method = self.methods.iter_mut().find(|m| m.name == name);

        if let Some(method) = method {
            let new_method = f(method.clone());

            *method = new_method;
        } else {
            let mut method = Method::default();
            method.name = name.to_string();
            self.methods.push(f(method));
        }

        self.clone()
    }

    pub fn field(&mut self, name: &str, f: impl FnOnce(Field) -> Field) -> Self {
        let fields = self.fields.iter_mut().find(|p| p.name == name);

        if let Some(field) = fields {
            let new_field = f(field.clone());

            *field = new_field;
        } else {
            let mut field = Field::default();
            field.name = name.to_string();
            self.fields.push(f(field));
        }

        self.clone()
    }

    pub fn build(self) -> Type {
        self
    }
}
