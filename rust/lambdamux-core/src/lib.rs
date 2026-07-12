use std::collections::HashMap;

// Define a type for your route handlers. For simplicity, we use a function pointer that takes no arguments and returns nothing.
pub type Handler<Input, Output, Error> = fn(Input) -> Result<Output, Error>;
// for all async version follow below
// pub type Handler<T> = fn(LambdaEvent<T>) -> dyn Future<Output = Result<Value>>;
// this will be a breaking change and a TODO for now

#[derive(Debug)]
pub struct TrieNode<Input, Output, Error> {
    children: HashMap<String, TrieNode<Input, Output, Error>>,
    is_end_of_path: bool,
    method: Option<String>,
    parameter_name: Option<String>,
    handler: Option<Handler<Input, Output, Error>>,
}

impl<Input, Output, Error> PartialEq for TrieNode<Input, Output, Error> {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
            && self.is_end_of_path == other.is_end_of_path
            && self.method == other.method
            && self.parameter_name == other.parameter_name
            && match (self.handler, other.handler) {
                (Some(left), Some(right)) => std::ptr::fn_addr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
    }
}

impl<Input, Output, Error> TrieNode<Input, Output, Error> {
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
pub struct Trie<Input, Output, Error> {
    root: TrieNode<Input, Output, Error>,
}

impl<Input, Output, Error> PartialEq for Trie<Input, Output, Error> {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl<Input, Output, Error> Default for Trie<Input, Output, Error> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Input, Output, Error> Trie<Input, Output, Error> {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, method: &str, path: &str, handler: Handler<Input, Output, Error>) {
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
        current_node.method = Some(method.to_uppercase().to_string());
    }

    pub fn route(
        &self,
        method: &str,
        path: &str,
    ) -> Option<(Handler<Input, Output, Error>, HashMap<String, String>)> {
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
            let handler = current_node.handler?;
            Some((handler, params))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod trie_tests {
    use crate::{Trie, TrieNode};
    use std::collections::HashMap;

    fn blank(_e: ()) -> Result<bool, ()> {
        Ok(true)
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
                        method: Some("GET".into()),
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
                                method: Some("GET".into()),
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

    #[test]
    fn route_matches_dynamic_path() {
        let mut trie = Trie::new();
        trie.insert("get", "/test/:id", blank);

        let Some((handler, params)) = trie.route("GET", "/test/123") else {
            panic!("expected route match")
        };

        assert!(handler(()).unwrap());
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }
}
