use crate::rule_trait::RuleTrait;
use serde::Deserialize;
use std::collections::HashMap;
use toml::Value;

type Factory = fn(Value) -> Result<Box<dyn RuleTrait>, String>;

pub struct Registry {
    factories: HashMap<String, Factory>,
}

// A registery holds factories for creating rules
impl Registry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register_rule<T>(mut self, kind: &str) -> Self
    where
        T: for<'de> Deserialize<'de> + RuleTrait + 'static,
    {
        self.factories.insert(kind.to_string(), |v| {
            v.try_into::<T>()
                .map(|t| Box::new(t) as Box<dyn RuleTrait>)
                .map_err(|e| e.to_string())
        });
        self
    }

    pub fn build_rule_instance(&self, value: Value) -> Result<Box<dyn RuleTrait>, String> {
        let kind = value
            .as_table()
            .and_then(|table| table.get("kind"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing field `kind`".to_string())?;

        let factory = self
            .factories
            .get(kind)
            .ok_or_else(|| format!("Unknown Rule of kind: {}", kind))?;

        factory(value)
    }
}
