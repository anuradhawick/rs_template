use std::{collections::HashMap, env::current_exe};

use aws_lambda_events::{apigw::ApiGatewayV2httpRequest, http::Result};
use lambda_runtime::LambdaEvent;
use serde_json::Value;

#[derive(Debug)]
pub struct TrieNode {
    children: HashMap<String, TrieNode>,
    is_end_of_path: bool,
    method: Option<String>,
    parameter_name: Option<String>,
    handler: Option<Handler>,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            is_end_of_path: false,
            method: None,
            parameter_name: None,
            handler: None,
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, method: &str, path: &str, handler: Handler) {
        let mut current_node = &mut self.root;
        let parts = path.split('/').filter(|part| !part.is_empty());

        for part in parts {
            let is_parameter = part.starts_with(':');
            let key = if is_parameter {
                ":".to_string()
            } else {
                part.to_string()
            };

            current_node = current_node
                .children
                .entry(key.clone())
                .or_insert_with(TrieNode::new);

            if is_parameter {
                current_node.parameter_name = Some(part[1..].to_string());
            }
        }
        current_node.is_end_of_path = true;
        current_node.handler = Some(handler);
        current_node.method = Some(method.to_string());
    }

    pub fn find(&self, method: &str, path: &str) -> Option<HashMap<String, String>> {
        let mut current_node = &self.root;
        let mut params = HashMap::new();

        for part in path.split('/').filter(|part| !part.is_empty()) {
            if let Some(node) = current_node.children.get(part) {
                current_node = node;
            } else if let Some(param_node) = current_node.children.get(":") {
                if let Some(param_name) = &param_node.parameter_name {
                    params.insert(param_name.clone(), part.to_string());
                }
                current_node = param_node;
            } else {
                return None;
            }
        }

        if current_node.is_end_of_path && current_node.method.as_deref() == Some(method) {
            Some(params)
        } else {
            None
        }
    }
}

// Define a type for your route handlers. For simplicity, we use a function pointer that takes no arguments and returns nothing.
pub type Handler = fn(LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value>;
