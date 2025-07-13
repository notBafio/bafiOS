#![no_std]

/*use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;*/

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use libk::hashmap::HashMap;
use libk::println;

#[derive(Debug, Clone)]
pub struct CustomFormat {
    pub metadata: HashMap<String, String>,
    pub data: Vec<DataEntry>,
}

#[derive(Debug, Clone)]
pub struct DataEntry {
    pub id: String,
    pub values: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl CustomFormat {
    pub fn new() -> Self {
        CustomFormat {
            metadata: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn add_entry(&mut self, entry: DataEntry) {
        self.data.push(entry);
    }

    pub fn save_to_file(&self, path: &str) {
        let file = libk::io::File::new(path);

        let bytes = self.to_bytes();
        file.write(&bytes);
    }

    pub fn load_from_file(path: &str) -> Self {
        let mut file = libk::io::File::new(path);

        Self::from_bytes(file.read_bytes()).unwrap()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut output = String::new();

        output.push_str("METADATA_START\n");
        for (key, value) in &self.metadata {
            output.push_str(&format!("{}={}\n", key, value));
        }
        output.push_str("METADATA_END\n");

        output.push_str("DATA_START\n");
        for entry in &self.data {
            output.push_str("ENTRY_START\n");
            output.push_str(&format!("ID={}\n", entry.id));
            for (key, value) in &entry.values {
                output.push_str(&format!("{}={}\n", key, self.serialize_value(value)));
            }
            output.push_str("ENTRY_END\n");
        }
        output.push_str("DATA_END\n");

        output.into_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let content = core::str::from_utf8(bytes).ok()?;
        let mut format = CustomFormat::new();

        let mut current_section = "";
        let mut current_entry: Option<DataEntry> = None;

        for line in content.split('\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match line {
                "METADATA_START" => current_section = "metadata",
                "METADATA_END" => current_section = "",
                "DATA_START" => current_section = "data",
                "DATA_END" => current_section = "",
                "ENTRY_START" => {
                    current_entry = Some(DataEntry {
                        id: String::new(),
                        values: HashMap::new(),
                    });
                }
                "ENTRY_END" => {
                    if let Some(entry) = current_entry.take() {
                        format.data.push(entry);
                    }
                }
                _ => {
                    if let Some(pos) = line.find('=') {
                        let (key, value) = line.split_at(pos);
                        let value = &value[1..];
                        match current_section {
                            "metadata" => {
                                format.metadata.insert(key.to_string(), value.to_string());
                            }
                            "data" => {
                                if let Some(entry) = current_entry.as_mut() {
                                    if key == "ID" {
                                        entry.id = value.to_string();
                                    } else {
                                        entry.values.insert(
                                            key.to_string(),
                                            Self::deserialize_value(value),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Some(format)
    }

    fn serialize_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("str:{}", s),
            Value::Number(n) => format!("num:{}", n),
            Value::Boolean(b) => format!("bool:{}", b),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.serialize_value(v)).collect();
                format!("arr:[{}]", items.join(","))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, self.serialize_value(v)))
                    .collect();
                format!("obj:{{{}}}", items.join(","))
            }
        }
    }

    fn deserialize_value(value: &str) -> Value {
        if let Some(pos) = value.find(':') {
            let (type_tag, content) = value.split_at(pos);
            let content = &content[1..];
            match type_tag {
                "str" => Value::String(content.to_string()),
                "num" => Value::Number(content.parse().unwrap_or(0.0)),
                "bool" => Value::Boolean(content.parse().unwrap_or(false)),
                "arr" => {
                    if content.starts_with('[') && content.ends_with(']') {
                        let items = content[1..content.len() - 1]
                            .split(',')
                            .filter(|s| !s.is_empty())
                            .map(|s| Self::deserialize_value(s))
                            .collect();
                        Value::Array(items)
                    } else {
                        Value::Array(vec![])
                    }
                }
                "obj" => {
                    if content.starts_with('{') && content.ends_with('}') {
                        let mut map = HashMap::new();
                        let items = content[1..content.len() - 1].split(',');
                        for item in items {
                            if let Some(pos) = item.find('=') {
                                let (key, val) = item.split_at(pos);
                                let val = &val[1..];
                                map.insert(key.to_string(), Self::deserialize_value(val));
                            }
                        }
                        Value::Object(map)
                    } else {
                        Value::Object(HashMap::new())
                    }
                }
                _ => Value::String(content.to_string()),
            }
        } else {
            Value::String(value.to_string())
        }
    }
}

pub fn test() {
    let mut custom_format = CustomFormat::new();
    custom_format.add_metadata("version", "1.9");
    custom_format.add_metadata("author", "Bafio");
    let mut values = HashMap::new();
    values.insert("name".to_string(), Value::String("Example".to_string()));
    values.insert("age".to_string(), Value::Number(25.0));

    let mut nested = HashMap::new();
    nested.insert(
        "street".to_string(),
        Value::String("123 Main St".to_string()),
    );
    values.insert("address".to_string(), Value::Object(nested));

    let entry = DataEntry {
        id: "001".to_string(),
        values,
    };
    custom_format.add_entry(entry);
    custom_format.save_to_file("/DATA.DB");
}

pub fn test2() {
    let loaded = CustomFormat::load_from_file("/DATA.DB");
    /*let bytes = custom_format.to_bytes();
    if let Some(parsed) = CustomFormat::from_bytes(&bytes) {
        println!("Parsed from bytes: {:#?}", parsed);
    }*/
}

pub fn load(path: &str) -> CustomFormat {
    let loaded = CustomFormat::load_from_file(path);

    loaded.clone()
}

impl CustomFormat {
    pub fn get(&self, arg: &str) -> Option<Value> {
        let val = None;

        for i in self.data.iter() {
            if i.values.get(arg).is_some() {
                return Some(i.values.get(arg).unwrap().clone());
            }
        }

        val
    }
}
