use std::collections::HashMap;
use toml::Value;

pub fn expand_serde(config: &mut Value) {
    // unwrap or return from function without modifying the config
    let expansions_table = config.get("expansions");

    if expansions_table.is_none() {
        return;
    }
    let expansions_table = expansions_table
        .unwrap()
        .as_table()
        .expect("Expected 'expansions' to be a table");

    let mut expansions_map: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in expansions_table {
        if let Some(arr) = value.as_array() {
            let string_vec: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            expansions_map.insert(key.clone(), string_vec);
        }
    }

    if let Value::Table(root) = config {
        // Temporarily take ownership of the table to avoid cloning
        let original = std::mem::take(root);

        for (key, mut value) in original {
            if key == "expansions" {
                // Preserve the "expansions" section as-is
                root.insert(key, value);
                continue;
            }

            expand_serde_value(&mut value, &expansions_map);
            root.insert(key, value);
        }
    }
}

fn expand_serde_value(value: &mut Value, expansions: &HashMap<String, Vec<String>>) {
    match value {
        Value::String(s) => {
            if let Some(expanded) = expansions.get(s) {
                *value = Value::Array(expanded.iter().map(|v| Value::String(v.clone())).collect());
            }
        }

        Value::Array(arr) => {
            let mut expanded_items = Vec::new();
            for mut item in arr.drain(..) {
                expand_serde_value(&mut item, expansions);

                match item {
                    Value::Array(inner) => expanded_items.extend(inner),
                    other => expanded_items.push(other),
                }
            }
            *arr = expanded_items;
        }

        Value::Table(table) => {
            let original = std::mem::take(table);

            for (key, mut val) in original {
                expand_serde_value(&mut val, expansions);

                if let Some(expanded_keys) = expansions.get(&key) {
                    for new_key in expanded_keys {
                        table.insert(new_key.clone(), val.clone());
                    }
                } else {
                    table.insert(key, val);
                }
            }
        }

        _ => {} // No expansion needed
    }
}
