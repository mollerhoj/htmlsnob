use regex::Regex;

pub fn dynamic_format(template: &str, vars: &[(&str, String)]) -> String {
    let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();

    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        vars.iter()
            .find(|(k, _)| k == &key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| caps.get(0).unwrap().as_str().to_string())
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_name() {
        let template = "Hello {name}".to_string();
        let vars = &[("name", "Alice".to_string())];
        assert_eq!(dynamic_format(&template, vars), "Hello Alice");
    }

    #[test]
    fn test_using_unknown_variable() {
        let template = "Hello {name} {age}".to_string();
        let vars = &[("name", "Alice".to_string())];
        assert_eq!(dynamic_format(&template, vars), "Hello Alice {age}");
    }

    #[test]
    fn test_not_using_variable() {
        let template = "Hello World".to_string();
        let vars = &[("name", "Alice".to_string())];
        assert_eq!(dynamic_format(&template, vars), "Hello World");
    }
}
