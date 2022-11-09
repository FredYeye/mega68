use std::collections::HashMap;

use crate::logging::Log;

use super::parse_n;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(u32),
    Label(String),
    Define(String),
}

impl Value {
    pub fn resolve_value(&self, labels: &HashMap<String, u32>, defines: &HashMap<String, u32>) -> Result<u32, Log> {
        match self {
            Value::Number(num) => Ok(*num),

            Value::Label(label) => match labels.get(label) {
                Some(val) => Ok(*val),
                None => Err(Log::NoLabel),
            },

            Value::Define(name) => match defines.get(name) {
                Some(val) => Ok(*val),
                None => Err(Log::NoDefine),
            },
        }
    }

    pub fn new(token: &str, last_label: &str) -> Value {
        match parse_n(token) {
            Ok(number) => Value::Number(number as u32),
    
            Err(_) => if let Some(define) = token.strip_prefix('!') {
                Value::Define(define.to_string())
            } else if token.starts_with('.') {
                let mut sub_label = last_label.to_string();
                sub_label.push_str(token);
                Value::Label(sub_label)
            } else {
                Value::Label(token.to_string())
            }
        }
    }
}
