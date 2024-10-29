use serde::ser::SerializeMap;

pub struct MultiOption<T: for<'a> serde::Deserialize<'a> + serde::Serialize> {
    pub inner: Option<Option<T>>,
}

impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize> Default for MultiOption<T> {
    fn default() -> Self {
        Self { inner: None }
    }
}

// Deserialize
//
// If value is nil, we set it to None, if value is an empty object, we set it to Some(None), otherwise we set it to Some(Some(value))
impl<'de, T: for<'a> serde::Deserialize<'a> + serde::Serialize> serde::Deserialize<'de>
    for MultiOption<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
        let inner = match value {
            None => None,
            Some(v) if v.is_object() && v.as_object().unwrap().is_empty() => Some(None),
            Some(v) => Some(Some(
                serde_json::from_value(v).map_err(serde::de::Error::custom)?,
            )),
        };
        Ok(Self { inner })
    }
}

// Serialize impl
impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize> serde::Serialize for MultiOption<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.inner {
            None => serializer.serialize_none(),
            Some(None) => serializer.serialize_map(Some(0))?.end(),
            Some(Some(value)) => value.serialize(serializer),
        }
    }
}
