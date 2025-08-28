use regex::Regex;
use std::collections::HashMap;
use std::env;

#[derive(Clone)]
pub struct VariableContext {
    pub vars: HashMap<String, String>,
}

impl VariableContext {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn with_env_vars(mut self) -> Self {
        // Load environment variables
        for (key, value) in env::vars() {
            self.vars.insert(key, value);
        }
        self
    }

    pub fn with_config_vars(mut self, config_vars: Option<&HashMap<String, String>>) -> Self {
        if let Some(vars) = config_vars {
            for (key, value) in vars {
                let expanded_value = self.substitute_variables(value);
                self.vars.insert(key.clone(), expanded_value);
            }
        }
        self
    }

    pub fn with_data_row(mut self, data: &HashMap<String, String>) -> Self {
        for (key, value) in data {
            self.vars.insert(key.clone(), value.clone());
        }
        self
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    pub fn substitute_variables(&self, text: &str) -> String {
        let var_regex = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let env_regex = Regex::new(r"\$\{([^:}]+)(?::([^}]*))?\}").unwrap();

        let mut result = text.to_string();

        // Substitute {{variable}} patterns
        result = var_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                self.vars.get(var_name).cloned().unwrap_or_else(|| {
                    format!("{{{{{}}}}}", var_name) // Return original if not found
                })
            })
            .to_string();

        // Substitute ${VARIABLE:default} patterns
        result = env_regex
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                env::var(var_name).unwrap_or_else(|_| {
                    self.vars
                        .get(var_name)
                        .cloned()
                        .unwrap_or_else(|| default_value.to_string())
                })
            })
            .to_string();

        result
    }

    pub fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let mut ctx = VariableContext::new();
        ctx.set("baseUrl".to_string(), "https://api.example.com".to_string());
        ctx.set("userId".to_string(), "123".to_string());

        let result = ctx.substitute_variables("{{baseUrl}}/users/{{userId}}");
        assert_eq!(result, "https://api.example.com/users/123");
    }

    #[test]
    fn test_env_variable_substitution() {
        env::set_var("TEST_VAR", "test_value");
        let ctx = VariableContext::new();

        let result = ctx.substitute_variables("${TEST_VAR:default}");
        assert_eq!(result, "test_value");

        let result = ctx.substitute_variables("${NONEXISTENT:default_val}");
        assert_eq!(result, "default_val");
    }
}
