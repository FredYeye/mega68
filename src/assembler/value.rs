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
                                Operator::Add => {
                                    if idx != 0 && idx < last {
                                        let second = values2.remove(idx + 1).resolve_value(labels, defines)?;
                                        let first = values2.remove(idx - 1).resolve_value(labels, defines)?;
                                        let result = first.wrapping_add(second);
                                        values2[idx - 1] = Value::Number(result);
                                        break;
                                    } else {
                                        todo!()
                                    }
                                }

                                Operator::Sub => {
                                    let is_operator = |x: &Value| -> bool {
                                        match x {
                                            Value::Operator(_) => true,
                                            _ => false,
                                        }
                                    };

                                    if idx < last {
                                        if idx == 0 || is_operator(&values2[idx - 1]) {
                                            //unary operator -
                                            let val = values2.remove(idx + 1).resolve_value(labels, defines)?;
                                            let val2 = -(val as i64);
                                            values2[idx] = Value::Number(val2 as u64);
                                            break;
                                        } else { //might need to put conditions here
                                            let second = values2.remove(idx + 1).resolve_value(labels, defines)?;
                                            let first = values2.remove(idx - 1).resolve_value(labels, defines)?;
                                            let result = first.wrapping_sub(second);
                                            values2[idx - 1] = Value::Number(result);
                                            break;
                                        }
                                    } else {
                                        todo!()
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
        //split strings on operators / other
        let substrings: Vec<&str> = token.split_inclusive(['+', '-']).collect();
        let mut proper_substrings = Vec::new();

        //split out operators into their own strings
        for &substring in &substrings[0 .. substrings.len() - 1] {
            proper_substrings.push(substring[0 .. substring.len() - 1].trim());
            proper_substrings.push(&substring[substring.len() - 1 ..]);
        }

        proper_substrings.push(substrings[substrings.len() - 1].trim());
        proper_substrings.retain(|&s| !s.is_empty());

        println!("{:?}", proper_substrings);

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
