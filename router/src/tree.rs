use std::collections::HashMap;
use std::mem;

pub type Params = HashMap<String, String>;

#[derive(Debug)]
pub struct Tree<'a, T>(Option<Node<'a, T>>);

#[derive(Debug, PartialEq)]
pub struct Node<'a, T> {
    path: &'a str,
    value: Option<T>,
    childs: HashMap<char, Node<'a, T>>,
    wildcard: Option<Param<'a, T>>,
}

#[derive(Debug, PartialEq)]
pub struct Param<'a, T> {
    name: &'a str,
    node: Box<Node<'a, T>>,
}

impl<'a, T> Tree<'a, T> {
    pub fn new() -> Self {
        return Tree(None);
    }

    pub fn add_path(&mut self, path: &'a str, value: T) {
        match self.0 {
            Some(ref mut node) => node.add_path(path, Some(value)),
            None => self.0 = Some(Node::new(path, Some(value))),
        }
    }

    pub fn find(&self, path: &str) -> Option<(&T, Params)> {
        let params = HashMap::new();
        self.0.as_ref().and_then(|node| node.find(path, params))
    }

    #[cfg(test)]
    fn find_test(&self, path: &str) -> Option<&T> {
        self.find(path).map(|v| v.0)
    }
}

impl<'a, T> Node<'a, T> {
    fn new(path: &'a str, value: Option<T>) -> Self {
        // extract params from path
        let mut value = value;
        let mut actual_path = None;
        let mut wildcard = None;
        let mut is_new_path_segment = false;
        for (i, ch) in path.chars().enumerate() {
            match ch {
                '/' => is_new_path_segment = true,
                ':' if is_new_path_segment => {
                    let (left, right) = path.split_at(i);
                    actual_path = Some(left);
                    wildcard = Some(Param::new(&right, value.take()));
                    break;
                }
                _ if is_new_path_segment => is_new_path_segment = false,
                _ => {}
            }
        }

        Node {
            path: actual_path.unwrap_or(path),
            value: value,
            childs: HashMap::new(),
            wildcard: wildcard,
        }
    }

    fn add_path(&mut self, path: &'a str, value: Option<T>) {
        let mut path = path;

        // split self
        let split_at = {
            // iterate path and compare it agains current tree
            let mut chars = self.path.chars();
            let mut is_new_path_segment = false;
            let mut i = 0;
            for lhs in path.chars() {
                match lhs {
                    '/' => is_new_path_segment = true,
                    ':' if is_new_path_segment => {
                        // a param has been found -> continue in param node
                        let (_, right) = path.split_at(i);
                        if let Some(ref mut param) = self.wildcard {
                            param.add_path(right, value);
                        } else {
                            self.wildcard = Some(Param::new(right, value));
                        }
                        return;
                    }
                    _ if is_new_path_segment => is_new_path_segment = false,
                    _ => {}
                }

                // compare position with current node's path
                if let Some(rhs) = chars.next() {
                    if lhs != rhs {
                        break;
                    }
                } else {
                    break;
                }
                i += 1;
            }
            i
        };

        if split_at < self.path.len() {
            // branch self
            let (left, right) = self.path.split_at(split_at);
            let right_first_char = right.chars().next().unwrap();
            let node = Node {
                path: right,
                value: self.value.take(),
                childs: mem::replace(&mut self.childs, HashMap::new()),
                wildcard: self.wildcard.take(),
            };

            self.path = left;
            self.childs.insert(right_first_char, node);

            // add new node
            if path != "" {
                let (_, right) = path.split_at(split_at);
                self.childs.insert(
                    right.chars().next().unwrap(),
                    Node::new(right, value),
                );
            } else {
                self.value = value;
            }
            return;
        } else {
            let (_, right) = path.split_at(split_at);
            path = right;
        }

        let first_char = match path.chars().next() {
            Some(ch) => ch,
            None => return,
        };

        if self.childs.contains_key(&first_char) {
            let node = self.childs.get_mut(&first_char).unwrap();
            node.add_path(path, value);
        } else {
            self.childs.insert(first_char, Node::new(path, value));
        }
    }

    fn find(&self, path: &str, params: Params) -> Option<(&T, Params)> {
        if !path.starts_with(self.path) {
            return None;
        }

        let (_, path) = path.split_at(self.path.len());

        let first_char = match path.chars().next() {
            Some(ch) => ch,
            None => return self.value.as_ref().map(|v| (v, params)),
        };

        if let Some(child) = self.childs.get(&first_char) {
            child.find(path, params)
        } else if self.wildcard.is_some() {
            self.wildcard.as_ref().unwrap().find(path, params)
        } else {
            None
        }
    }
}

impl<'a, T> Param<'a, T> {
    fn new(path: &'a str, value: Option<T>) -> Self {
        let (name, path) = extract_param_name(path);
        Param {
            name: name,
            node: Box::new(Node::new(path, value)),
        }
    }

    fn add_path(&mut self, path: &'a str, value: Option<T>) {
        let (name, path) = extract_param_name(path);
        if name != self.name {
            // TODO: really panic?
            panic!("cannot have different parameter names at the same position");
        }
        self.node.add_path(path, value);
    }

    fn find(&self, path: &str, mut params: Params) -> Option<(&T, Params)> {
        let (value, path) = split_at_next_param_delimiter(path);
        params.insert(self.name.to_string(), value.to_string());
        self.node.find(path, params)
    }

    #[cfg(test)]
    fn find_test(&self, path: &str) -> Option<&T> {
        self.find(path, HashMap::new()).map(|v| v.0)
    }
}

fn extract_param_name(path: &str) -> (&str, &str) {
    let (colon, path) = path.split_at(1);
    assert_eq!(colon, ":");

    split_at_next_param_delimiter(path)
}

fn split_at_next_param_delimiter(path: &str) -> (&str, &str) {
    let split_at = {
        let mut i = 0;
        for ch in path.chars() {
            if ch == '/' || ch == '.' {
                break;
            }
            i += 1;
        }
        i
    };
    path.split_at(split_at)
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use tree::{Tree, Node, Param};

    #[test]
    fn root_branch() {
        let mut tree = Tree::new();
        tree.add_path("a", 1);
        tree.add_path("b", 2);

        let mut childs = HashMap::new();
        childs.insert(
            'a',
            Node {
                path: "a",
                value: Some(1),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        childs.insert(
            'b',
            Node {
                path: "b",
                value: Some(2),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        assert_eq!(
            tree.0,
            Some(Node {
                path: "",
                value: None,
                childs: childs,
                wildcard: None,
            })
        );
    }

    #[test]
    fn branch() {
        let mut tree = Tree::new();

        tree.add_path("/foobar", 1);
        assert_eq!(
            tree.0,
            Some(Node {
                path: "/foobar",
                value: Some(1),
                childs: HashMap::new(),
                wildcard: None,
            })
        );

        tree.add_path("/foocar", 2);
        let mut childs = HashMap::new();
        childs.insert(
            'b',
            Node {
                path: "bar",
                value: Some(1),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        childs.insert(
            'c',
            Node {
                path: "car",
                value: Some(2),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        assert_eq!(
            tree.0,
            Some(Node {
                path: "/foo",
                value: None,
                childs: childs,
                wildcard: None,
            })
        );

        tree.add_path("/otherwise", 3);
        let mut subchilds = HashMap::new();
        subchilds.insert(
            'b',
            Node {
                path: "bar",
                value: Some(1),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        subchilds.insert(
            'c',
            Node {
                path: "car",
                value: Some(2),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        let mut childs = HashMap::new();
        childs.insert(
            'f',
            Node {
                path: "foo",
                value: None,
                childs: subchilds,
                wildcard: None,
            },
        );
        childs.insert(
            'o',
            Node {
                path: "otherwise",
                value: Some(3),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        assert_eq!(
            tree.0,
            Some(Node {
                path: "/",
                value: None,
                childs: childs,
                wildcard: None,
            })
        );
    }

    #[test]
    fn append() {
        let mut tree = Tree::new();

        tree.add_path("/foo", 1);
        assert_eq!(
            tree.0,
            Some(Node {
                path: "/foo",
                value: Some(1),
                childs: HashMap::new(),
                wildcard: None,
            })
        );

        tree.add_path("/foobar", 2);
        let mut childs = HashMap::new();
        childs.insert(
            'b',
            Node {
                path: "bar",
                value: Some(2),
                childs: HashMap::new(),
                wildcard: None,
            },
        );
        assert_eq!(
            tree.0,
            Some(Node {
                path: "/foo",
                value: Some(1),
                childs: childs,
                wildcard: None,
            })
        );
    }

    #[test]
    fn wildcard() {
        let node = Node::new("/foo/:bar/more", Some(1));

        assert_eq!(
            node,
            Node {
                path: "/foo/",
                value: None,
                childs: HashMap::new(),
                wildcard: Some(Param {
                    name: "bar",
                    node: Box::new(Node {
                        path: "/more",
                        value: Some(1),
                        childs: HashMap::new(),
                        wildcard: None,
                    }),
                }),
            }
        );
    }

    #[test]
    fn param_find() {
        let param = Param::new(":id/prop", Some(1));
        assert_eq!(param.find_test("whatever/prop"), Some(&1));

        let param = Param::new(":id", Some(1));
        assert_eq!(param.find_test("whatever"), Some(&1));
    }

    #[test]
    fn param_special_character() {
        let mut tree = Tree::new();
        tree.add_path("/foo/:id.json", 1);
        let mut params = HashMap::new();
        params.insert("id".to_string(), "42".to_string());
        assert_eq!(tree.find("/foo/42.json"), Some((&1, params)));
    }

    #[test]
    fn find() {
        let mut tree = Tree::new();

        tree.add_path("/foobar", 1);
        assert_eq!(tree.find_test("/foobar"), Some(&1));
        assert_eq!(tree.find_test("/foo"), None);

        tree.add_path("/foocar", 2);
        assert_eq!(tree.find_test("/foobar"), Some(&1));
        assert_eq!(tree.find_test("/foocar"), Some(&2));
        assert_eq!(tree.find_test("/foo"), None);

        tree.add_path("/one/:id", 3);
        tree.add_path("/one/:id/more", 4);
        assert_eq!(tree.find_test("/one/42"), Some(&3));
        assert_eq!(tree.find_test("/one/13/more"), Some(&4));

        tree.add_path("/two/:id/more", 5);
        tree.add_path("/two/:id", 6);
        assert_eq!(tree.find_test("/two/13/more"), Some(&5));
        assert_eq!(tree.find_test("/two/42"), Some(&6));

        tree.add_path("/articles/:article/comments/:comment/author", 7);
        assert_eq!(tree.find_test("/articles/42/comments/13/author"), Some(&7));
        assert_eq!(tree.find_test("/articles/42/comments/13"), None);
    }

    #[test]
    fn params() {
        let mut tree = Tree::new();
        tree.add_path("/a/:a/b/:b", 1);
        let mut params = HashMap::new();
        params.insert("a".to_string(), "12".to_string());
        params.insert("b".to_string(), "345".to_string());
        assert_eq!(tree.find("/a/12/b/345"), Some((&1, params)));
    }
}
