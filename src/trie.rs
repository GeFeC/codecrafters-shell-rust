use std::collections::{HashMap};

pub struct Trie{
    root: TrieNode
}

struct TrieNode{
    value: String,
    children: HashMap<u8, TrieNode>
}

impl TrieNode{
    fn new() -> Self{
        Self::from("")
    }

    fn from(str: &str) -> Self{
        Self{
            children: HashMap::new(),
            value: String::from(str)
        }
    }
}

impl Trie{
    pub fn new() -> Self{
        Self{
            root: TrieNode::new()
        }
    }

    pub fn from(values: &[&str]) -> Self{
        let mut trie = Self::new();

        for v in values{
            trie.insert(v);
        }

        trie
    }

    pub fn insert(&mut self, word: &str){
        let mut node = &mut self.root;

        for c in word.bytes(){
            node = node.children.entry(c).or_insert(TrieNode::new());
        }

        node.value = word.to_string();
    }

    fn for_each_recursively<F: FnMut(&TrieNode)>(node: &TrieNode, action: &mut F){
        action(node);

        for value in node.children.values(){
            Self::for_each_recursively(value, action);
        }
    }

    pub fn all_starting_with(&self, word: &str) -> Vec<String>{
        let mut node = &self.root;

        for c in word.bytes(){
            let Some(value) = node.children.get(&c) else{
                return Vec::new();
            };

            node = value;
        }

        let mut matches = Vec::new();
        Self::for_each_recursively(node, &mut |node|{
            if !node.value.is_empty(){
                matches.push(node.value.clone());
            }
        });

        matches
    }
}
