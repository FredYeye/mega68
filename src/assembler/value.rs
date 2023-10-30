use std::collections::HashMap;

use crate::logging::Log;

use super::parse_n;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, Sub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(u64),
    Label(String),
    Define(String),

    Expression(Vec<Value>),
    Operator(Operator),
}

impl Value {
    pub fn resolve_value(&self, labels: &HashMap<String, u32>, defines: &HashMap<String, u64>) -> Result<u64, Log> {
        match self {
            Value::Number(num) => Ok(*num),

            Value::Label(label) => match labels.get(label) {
                Some(val) => Ok(*val as u64),
                None => Err(Log::NoLabel),
            },

            Value::Define(name) => match defines.get(name) {
                Some(val) => Ok(*val),
                None => Err(Log::NoDefine),
            },

            Value::Expression(values) => {
                let mut values2 = values.to_owned();
                let last = values.len() - 1;

                while values2.len() != 1 {
                    for idx in 0 .. values2.len() {
                        match values2[idx].to_owned() {
                            Value::Operator(op) => match op {
                                Operator::Add | Operator::Sub => {
                                    if idx != 0 && idx < last {
                                        let second = values2.remove(idx + 1).resolve_value(labels, defines)?;
                                        let first = values2.remove(idx - 1).resolve_value(labels, defines)?;

                                        let result = match op {
                                            Operator::Add => first.wrapping_add(second),
                                            Operator::Sub => first.wrapping_sub(second),
                                        };

                                        values2[idx - 1] = Value::Number(result);
                                        break;
                                    }
                                }
                            }

                            _ => (),
                        }
                    }
                }

                Ok(values2[0].resolve_value(labels, defines)?)
            }

            _ => todo!(),
        }
    }

    pub fn new(token: &str, last_label: &str) -> Value {
        //todo: remove empty strings from proper_substrings
        //todo: - can be an operator or a negative number, handle this case

        //split strings on operators / other
        let substrings: Vec<&str> = token.split_inclusive(['+', '-']).collect();
        let mut proper_substrings = Vec::new();

        //split out operators into their own strings
        for &substring in &substrings[0 .. substrings.len() - 1] {
            proper_substrings.push(substring[0 .. substring.len() - 1].trim());
            proper_substrings.push(&substring[substring.len() - 1 ..]);
        }

        proper_substrings.push(substrings[substrings.len() - 1].trim());

        let mut values = Vec::new();
        for &sub in &proper_substrings {
            values.push( match sub {
                "+" => Value::Operator(Operator::Add),
                "-" => Value::Operator(Operator::Sub),
                
                _ => {
                    match parse_n(sub) {
                        Ok(number) => Value::Number(number),
                
                        Err(_) => if let Some(define) = sub.strip_prefix('!') {
                            Value::Define(define.to_string())
                        } else if sub.starts_with('.') {
                            let mut sub_label = last_label.to_string();
                            sub_label.push_str(sub);
                            Value::Label(sub_label)
                        } else {
                            Value::Label(sub.to_string())
                        }
                    }
                }
            })
        }

        if values.len() == 1 {
            values[0].to_owned()
        } else {
            Value::Expression(values)
        }
    }
}
