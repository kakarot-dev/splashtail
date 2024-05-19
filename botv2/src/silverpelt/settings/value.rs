use sqlx::{postgres::PgRow, postgres::PgTypeKind, Column, Row, TypeInfo};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
/// Represents a supported value type
pub(crate) enum Value {
    /// A uuid value
    Uuid(sqlx::types::Uuid),

    /// A string value
    String(String),

    /// An integer value
    Integer(i64),

    /// A float value
    Float(f64),

    /// A boolean value
    Boolean(bool),

    /// A list of values
    List(Vec<Value>),

    /// A (indexmap) of values
    Map(indexmap::IndexMap<String, Value>),

    /// None
    None,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Uuid(u) => u.hash(state),
            Value::String(s) => s.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::List(l) => l.hash(state),
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::None => None::<u8>.hash(state),
        }
    }
}

impl Value {
    /// Convert the Value to a serde_json::Value
    #[allow(dead_code)]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Uuid(u) => serde_json::Value::String(u.to_string()),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::Float(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
            ),
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::List(l) => serde_json::Value::Array(l.iter().map(|v| v.to_json()).collect()),
            Value::Map(m) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in m {
                    obj.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(obj)
            }
            Value::None => serde_json::Value::Null,
        }
    }

    /// Convert a serde_json::Value to a Value
    #[allow(dead_code)]
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Self::Integer(n.as_i64().unwrap())
                } else {
                    Self::Float(n.as_f64().unwrap())
                }
            }
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Array(a) => Self::List(a.iter().map(Value::from_json).collect()),
            serde_json::Value::Object(o) => {
                let mut m = indexmap::IndexMap::new();
                for (k, v) in o {
                    m.insert(k.clone(), Self::from_json(v));
                }
                Self::Map(m)
            }
            serde_json::Value::Null => Self::None,
        }
    }

    /// Converts a Value to a serde_cbor::Value
    #[allow(dead_code)]
    pub fn to_cbor(&self) -> serde_cbor::Value {
        match self {
            Value::Uuid(u) => serde_cbor::Value::Text(u.to_string()),
            Value::String(s) => serde_cbor::Value::Text(s.clone()),
            Value::Integer(i) => serde_cbor::Value::Integer(*i as i128),
            Value::Float(f) => serde_cbor::Value::Float(*f),
            Value::Boolean(b) => serde_cbor::Value::Bool(*b),
            Value::List(l) => serde_cbor::Value::Array(l.iter().map(Value::to_cbor).collect()),
            Value::Map(m) => {
                let mut obj = std::collections::BTreeMap::new();
                for (k, v) in m {
                    obj.insert(serde_cbor::Value::Text(k.clone()), v.to_cbor());
                }
                serde_cbor::Value::Map(obj)
            }
            Value::None => serde_cbor::Value::Null,
        }
    }

    /// Converts a PgColumn to a Value
    #[allow(dead_code)]
    pub fn from_sqlx(row: &PgRow, index: usize) -> Result<Self, crate::Error> {
        let typ_info = row.column(index).type_info();
        let typ = typ_info.name().to_lowercase();

        match typ_info.kind() {
            PgTypeKind::Simple => match typ.as_str() {
                "text" | "citext" => {
                    let Some(v) = row.try_get::<Option<String>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::String(v))
                }
                "int4" | "int8" => {
                    let Some(v) = row.try_get::<Option<i64>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Integer(v))
                }
                "float4" => {
                    let Some(v) = row.try_get::<Option<f32>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Float(v as f64))
                }
                "float8" => {
                    let Some(v) = row.try_get::<Option<f64>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Float(v))
                }
                "bool" => {
                    let Some(v) = row.try_get::<Option<bool>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Boolean(v))
                }
                "json" | "jsonb" => {
                    let Some(v) = row.try_get::<Option<serde_json::Value>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::from_json(&v))
                }
                "uuid" => {
                    let Some(v) = row.try_get::<Option<sqlx::types::Uuid>, _>(index)? else {
                        return Ok(Value::None);
                    };
                    Ok(Value::Uuid(v))
                }
                "timestamp" => {
                    let Some(v) =
                        row.try_get::<Option<sqlx::types::chrono::NaiveDateTime>, _>(index)?
                    else {
                        return Ok(Value::None);
                    };
                    Ok(Value::String(v.to_string()))
                }
                "timestamptz" => {
                    let Some(v) = row
                        .try_get::<Option<sqlx::types::chrono::DateTime<chrono::Utc>>, _>(index)?
                    else {
                        return Ok(Value::None);
                    };
                    Ok(Value::String(v.to_string()))
                }
                _ => Err("Unsupported type".into()),
            },
            PgTypeKind::Array(ar) => {
                let typ = ar.name().to_lowercase();
                match typ.as_str() {
                    "text" | "citext" => {
                        let Some(v) = row.try_get::<Option<Vec<String>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::String).collect()))
                    }
                    "int4" | "int8" => {
                        let Some(v) = row.try_get::<Option<Vec<i64>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Integer).collect()))
                    }
                    "float4" => {
                        let Some(v) = row.try_get::<Option<Vec<f32>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(
                            v.into_iter().map(|f| Value::Float(f as f64)).collect(),
                        ))
                    }
                    "float8" => {
                        let Some(v) = row.try_get::<Option<Vec<f64>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Float).collect()))
                    }
                    "bool" => {
                        let Some(v) = row.try_get::<Option<Vec<bool>>, _>(index)? else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Boolean).collect()))
                    }
                    "json" | "jsonb" => {
                        let Some(v) = row.try_get::<Option<Vec<serde_json::Value>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::from_json(&i));
                        }

                        Ok(Value::List(m))
                    }
                    "uuid" => {
                        let Some(v) = row.try_get::<Option<Vec<sqlx::types::Uuid>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        Ok(Value::List(v.into_iter().map(Value::Uuid).collect()))
                    }
                    "timestamp" => {
                        let Some(v) = row
                            .try_get::<Option<Vec<sqlx::types::chrono::NaiveDateTime>>, _>(index)?
                        else {
                            return Ok(Value::None);
                        };
                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::String(i.to_string()));
                        }

                        Ok(Value::List(m))
                    }
                    "timestamptz" => {
                        let Some(v) = row
                            .try_get::<Option<Vec<sqlx::types::chrono::DateTime<chrono::Utc>>>, _>(
                                index,
                            )?
                        else {
                            return Ok(Value::None);
                        };

                        let mut m: Vec<Value> = Vec::new();

                        for i in v {
                            m.push(Value::String(i.to_string()));
                        }

                        Ok(Value::List(m))
                    }
                    _ => Err("Unsupported type".into()),
                }
            }
            _ => Err("Unsupported type".into()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Uuid(u) => write!(f, "{}", u),
            Value::String(s) => write!(f, "{}", s),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::None => write!(f, "None"),
        }
    }
}
