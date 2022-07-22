use std::{hash::Hash, collections::HashMap};
use super::graph::Graph;

pub struct GraphMap<K: Eq + Hash + Clone, V, E> {
    graph: Graph<K, E>,
    map: HashMap<K, Node<V>>,
}

pub struct Node<V>(V, usize);

impl<V> Node<V> {
    pub fn index(&self) -> usize {
        self.1
    }

    pub fn as_ref(&self) -> &V {
        &self.0
    }

    pub fn as_ref_mut(&mut self) -> &mut V {
        &mut self.0
    }

    pub fn take(self) -> V {
        self.0
    }
}

impl<K: Eq + Hash + Clone, V, E> GraphMap<K, V, E> {
    pub fn new() -> Self {
        Self { graph: Graph::new(), map: HashMap::new() }
    }

    pub fn add_node(&mut self, key: K, value: V) -> usize {
        let iden = self.graph.add_node(key.clone());
        self.map.insert(key, Node(value, iden));
        iden
    }

    pub fn add_edge_by_key(&mut self, source: &K, target: &K, cost: E) -> bool {
        let source = match self.map.get(source) {
            Some(i) => i.index(),
            None => return false,
        };
        let target = match self.map.get(target) {
            Some(i) => i.index(),
            None => return false,
        };
        self.graph.add_edge(source, target, cost)
    }

    pub fn add_edge_by_index(&mut self, source: usize, target: usize, cost: E) -> bool {
        self.graph.add_edge(source, target, cost)
    }

    pub fn get_node_by_key(&self, key: &K) -> Option<&Node<V>> {
        self.map.get(key)
    }

    pub fn get_node_by_index(&self, index: usize) -> Option<&Node<V>> {
        self.map.get(self.graph.get_node(index)?)
    }

    pub fn get_node_by_key_mut(&mut self, key: &K) -> Option<&mut Node<V>> {
        self.map.get_mut(key)
    }

    pub fn get_key_by_index(&self, index: usize) -> Option<&K> {
        Some(self.graph.get_node(index)?)
    }

    pub fn get_map(&self) -> &HashMap<K, Node<V>> {
        &self.map
    }

    pub fn get_map_mut(&mut self) -> &mut HashMap<K, Node<V>> {
        &mut self.map
    }

    pub fn find_root(&self) -> Result<&Node<V>, Vec<&Node<V>>> {
        match self.graph.find_root() {
            Ok(n) => Ok(self.map.get(self.graph.get_node(n).unwrap()).unwrap()),
            Err(e) => {
                Err(e.into_iter().map(|f| {
                    self.map.get(self.graph.get_node(f).unwrap()).unwrap()
                }).collect())
            }
        }
    }

    pub fn get_graph(&self) -> &Graph<K, E> {
        &self.graph
    }
}
