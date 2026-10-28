use std::fmt;

use envsentry_core::{
    ConfigDocument, ConfigEntry, ConfigFormat, ConfigPath, ConfigValue, Sensitivity, ValueKind,
};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsonParseError;

impl fmt::Display for JsonParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid JSON configuration")
    }
}

impl std::error::Error for JsonParseError {}

#[derive(Debug)]
enum Node {
    Null,
    Bool(bool),
    Number(String, ValueKind),
    String(String),
    Array(Vec<Node>),
    Object(Vec<(String, Node)>),
}

struct NodeSeed;

impl<'de> DeserializeSeed<'de> for NodeSeed {
    type Value = Node;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NodeVisitor)
    }
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_unit<E>(self) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::Null)
    }

    fn visit_none<E>(self) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::Number(value.to_string(), ValueKind::Integer))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::Number(value.to_string(), ValueKind::Integer))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Node, E>
    where
        E: de::Error,
    {
        if !value.is_finite() {
            return Err(E::custom("non-finite JSON number"));
        }
        Ok(Node::Number(value.to_string(), ValueKind::Float))
    }

    fn visit_str<E>(self, value: &str) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Node, E>
    where
        E: de::Error,
    {
        Ok(Node::String(value))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Node, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(NodeSeed)? {
            values.push(value);
        }
        Ok(Node::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Node, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut fields = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(NodeSeed)?;
            fields.push((key, value));
        }
        Ok(Node::Object(fields))
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        NodeSeed.deserialize(deserializer)
    }
}

/// Parses JSON into flattened canonical paths. Duplicate object keys are
/// preserved as repeated entries so the comparison engine can diagnose them.
pub fn parse_json(input: &str) -> Result<ConfigDocument, JsonParseError> {
    let node = serde_json::from_str::<Node>(input).map_err(|_| JsonParseError)?;
    let mut entries = Vec::new();
    flatten(node, &ConfigPath::root(), &mut entries);

    Ok(ConfigDocument::new(ConfigFormat::Json, entries))
}

fn flatten(node: Node, path: &ConfigPath, entries: &mut Vec<ConfigEntry>) {
    match node {
        Node::Object(fields) => {
            if fields.is_empty() {
                if !path.is_root() {
                    push_value(path, "{}", ValueKind::Object, entries);
                }
                return;
            }
            for (key, value) in fields {
                flatten(value, &path.child_key(key), entries);
            }
        }
        Node::Array(values) => {
            if values.is_empty() {
                if !path.is_root() {
                    push_value(path, "[]", ValueKind::Array, entries);
                }
                return;
            }
            for (index, value) in values.into_iter().enumerate() {
                flatten(value, &path.child_index(index), entries);
            }
        }
        leaf => {
            let (raw, kind) = match leaf {
                Node::Null => ("null".to_owned(), ValueKind::Null),
                Node::Bool(value) => (value.to_string(), ValueKind::Boolean),
                Node::Number(value, kind) => (value, kind),
                Node::String(value) => (value, ValueKind::String),
                Node::Array(_) | Node::Object(_) => unreachable!(),
            };
            push_value(path, &raw, kind, entries);
        }
    }
}

fn push_value(path: &ConfigPath, raw: &str, kind: ValueKind, entries: &mut Vec<ConfigEntry>) {
    entries.push(
        ConfigEntry::new(path.clone(), ConfigValue::new(raw, kind))
            .with_sensitivity(Sensitivity::Unknown),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_nested_objects_and_arrays() {
        let document = parse_json(r#"{"database":{"port":5432},"nodes":[true,null]}"#).unwrap();
        let paths: Vec<_> =
            document.entries().iter().map(|entry| entry.path().to_string()).collect();

        assert_eq!(paths, ["database.port", "nodes[0]", "nodes[1]"]);
        assert_eq!(document.entries()[0].value().kind(), ValueKind::Integer);
        assert_eq!(document.entries()[1].value().kind(), ValueKind::Boolean);
        assert_eq!(document.entries()[2].value().kind(), ValueKind::Null);
    }

    #[test]
    fn preserves_duplicate_keys_for_comparison() {
        let document = parse_json(r#"{"port":3000,"port":4000}"#).unwrap();

        assert_eq!(document.entries().len(), 2);
        assert_eq!(document.entries()[0].path(), document.entries()[1].path());
        assert_eq!(document.entries()[0].value().as_str(), "3000");
        assert_eq!(document.entries()[1].value().as_str(), "4000");
    }

    #[test]
    fn parse_errors_do_not_echo_secret_values() {
        let error = parse_json(r#"{"password":"SENSITIVE_SENTINEL",}"#).unwrap_err().to_string();

        assert_eq!(error, "invalid JSON configuration");
        assert!(!error.contains("SENSITIVE_SENTINEL"));
    }

    #[test]
    fn preserves_nested_empty_objects_and_arrays() {
        let document = parse_json(r#"{"object":{},"array":[],"items":[{}]}"#).unwrap();
        let entries: Vec<_> = document
            .entries()
            .iter()
            .map(|entry| (entry.path().to_string(), entry.value().kind()))
            .collect();

        assert!(entries.contains(&("object".to_owned(), ValueKind::Object)));
        assert!(entries.contains(&("array".to_owned(), ValueKind::Array)));
        assert!(entries.contains(&("items[0]".to_owned(), ValueKind::Object)));
    }

    #[test]
    fn distinguishes_string_and_number() {
        let document = parse_json(r#"{"number":1,"string":"1"}"#).unwrap();

        assert_eq!(document.entries()[0].value().kind(), ValueKind::Integer);
        assert_eq!(document.entries()[1].value().kind(), ValueKind::String);
    }
}
