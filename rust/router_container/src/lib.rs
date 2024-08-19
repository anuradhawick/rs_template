use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use serde_json::Value;
use std::collections::HashMap;

// Define a type for your route handlers. For simplicity, we use a function pointer that takes no arguments and returns nothing.
pub type Handler<T> = fn(LambdaEvent<T>) -> Result<Value>;

#[derive(Debug, PartialEq)]
pub struct TrieNode<T> {
    children: HashMap<String, TrieNode<T>>,
    is_end_of_path: bool,
    method: Option<String>,
    parameter_name: Option<String>,
    handler: Option<Handler<T>>,
}

impl<T> TrieNode<T> {
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

#[derive(Debug, PartialEq)]
pub struct Trie<T> {
    root: TrieNode<T>,
}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Trie<T> {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, method: &str, path: &str, handler: Handler<T>) {
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

    pub fn route(&self, method: &str, path: &str) -> Option<(Handler<T>, HashMap<String, String>)> {
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

        if current_node.is_end_of_path
            && current_node.method.as_deref() == Some(method.to_uppercase().as_str())
        {
            let handler = current_node.handler.unwrap();
            Some((handler, params))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod trie_tests {
    use crate::{Trie, TrieNode};
    use aws_lambda_events::{apigw::ApiGatewayV2httpRequest, http::Result};
    use lambda_runtime::LambdaEvent;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    fn blank(_e: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
        Ok(json!({"success": true}))
    }

    #[test]
    fn add_routes() {
        let mut trie = Trie::new();
        trie.insert("get", "/test", blank);

        let trie_ref = Trie {
            root: TrieNode {
                children: HashMap::from([(
                    "test".into(),
                    TrieNode {
                        children: HashMap::new(),
                        is_end_of_path: true,
                        method: Some("get".into()),
                        parameter_name: None,
                        handler: Some(blank),
                    },
                )]),
                is_end_of_path: false,
                method: None,
                parameter_name: None,
                handler: None,
            },
        };
        assert_eq!(trie, trie_ref);
    }

    #[test]
    fn add_param_routes() {
        let mut trie = Trie::new();
        trie.insert("get", "/test/:id", blank);

        let trie_ref = Trie {
            root: TrieNode {
                children: HashMap::from([(
                    "test".into(),
                    TrieNode {
                        children: HashMap::from([(
                            ":".into(),
                            TrieNode {
                                children: HashMap::new(),
                                handler: Some(blank),
                                is_end_of_path: true,
                                parameter_name: Some("id".into()),
                                method: Some("get".into()),
                            },
                        )]),
                        is_end_of_path: false,
                        method: None,
                        parameter_name: None,
                        handler: None,
                    },
                )]),
                is_end_of_path: false,
                method: None,
                parameter_name: None,
                handler: None,
            },
        };
        assert_eq!(trie, trie_ref);
    }
}
