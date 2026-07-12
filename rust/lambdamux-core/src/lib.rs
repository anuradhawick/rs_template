use std::collections::HashMap;

// Define a type for your route handlers. For simplicity, we use a function pointer that takes no arguments and returns nothing.
pub type Handler<Input, Output, Error> = fn(Input) -> Result<Output, Error>;
pub type RouteMatch<Input, Output, Error> =
    (Handler<Input, Output, Error>, HashMap<String, String>);
// for all async version follow below
// pub type Handler<T> = fn(LambdaEvent<T>) -> dyn Future<Output = Result<Value>>;
// this will be a breaking change and a TODO for now

#[derive(Debug)]
pub struct TrieNode<Input, Output, Error> {
    children: HashMap<String, TrieNode<Input, Output, Error>>,
    is_end_of_path: bool,
    parameter_name: Option<String>,
    handlers: HashMap<String, Handler<Input, Output, Error>>,
}

impl<Input, Output, Error> PartialEq for TrieNode<Input, Output, Error> {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
            && self.is_end_of_path == other.is_end_of_path
            && self.parameter_name == other.parameter_name
            && self.handlers.len() == other.handlers.len()
            && self.handlers.iter().all(|(method, handler)| {
                other
                    .handlers
                    .get(method)
                    .is_some_and(|other_handler| std::ptr::fn_addr_eq(*handler, *other_handler))
            })
    }
}

impl<Input, Output, Error> TrieNode<Input, Output, Error> {
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            is_end_of_path: false,
            parameter_name: None,
            handlers: HashMap::new(),
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
        current_node.handlers.insert(method.to_uppercase(), handler);
    }

    pub fn route(&self, method: &str, path: &str) -> Option<RouteMatch<Input, Output, Error>> {
        let mut current_node = &self.root;
        let mut params = HashMap::new();

        for part in path.split('/').filter(|part| !part.is_empty()) {
            if let Some(node) = current_node.children.get(part) {
                current_node = node;
            } else {
                let param_node = current_node.children.get(":")?;
                if let Some(param_name) = &param_node.parameter_name {
                    params.insert(param_name.clone(), part.to_string());
                }
                current_node = param_node;
            }
        }

        if !current_node.is_end_of_path {
            return None;
        }

        let handler = *current_node.handlers.get(&method.to_uppercase())?;
        Some((handler, params))
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
                        parameter_name: None,
                        handlers: HashMap::from([("GET".into(), blank as _)]),
                    },
                )]),
                is_end_of_path: false,
                parameter_name: None,
                handlers: HashMap::new(),
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
                                handlers: HashMap::from([("GET".into(), blank as _)]),
                                is_end_of_path: true,
                                parameter_name: Some("id".into()),
                            },
                        )]),
                        is_end_of_path: false,
                        parameter_name: None,
                        handlers: HashMap::new(),
                    },
                )]),
                is_end_of_path: false,
                parameter_name: None,
                handlers: HashMap::new(),
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

    #[test]
    fn add_multi_method_route() {
        fn get(_e: ()) -> Result<bool, ()> {
            Ok(true)
        }

        fn post(_e: ()) -> Result<bool, ()> {
            Ok(false)
        }

        let mut trie = Trie::new();
        trie.insert("get", "/", get);
        trie.insert("post", "/", post);

        let (get_handler, _) = trie.route("GET", "/").expect("expected GET route match");
        let (post_handler, _) = trie.route("POST", "/").expect("expected POST route match");

        assert!(get_handler(()).unwrap());
        assert!(!post_handler(()).unwrap());
        assert!(trie.route("DELETE", "/").is_none());
    }
}
