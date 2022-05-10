/// Basic graph data structure
/// - source: http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
use std::collections::HashSet;

type NodeIndex = usize;

#[derive(Debug, PartialEq)]
struct NodeData { 
    first_outgoing_edge: Option<EdgeIndex>,
    first_incoming_edge: Option<EdgeIndex>,
}

type EdgeIndex = usize;

#[derive(Debug, PartialEq)]
struct EdgeData { 
    source: NodeIndex,
    target: NodeIndex, 
    next_outgoing_edge: Option<EdgeIndex>,
    next_incoming_edge: Option<EdgeIndex>,
}

#[derive(Debug, PartialEq)]
pub struct Graph {
    vertices: Vec<NodeData>,
    edges: Vec<EdgeData>,
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, data) in self.vertices.iter().enumerate() {
            write!(f, "n{} {:?} {:?}\n", i, data.first_outgoing_edge, data.first_incoming_edge)?
        }
        write!(f, "\n")?;
        for (i, data) in self.edges.iter().enumerate() {
            write!(f, "e{} n{} s{} {:?} {:?}\n", i, data.target, data.source, data.next_outgoing_edge, data.next_incoming_edge)?
        }
        Ok(())
    }
}

impl Graph {
    /// Creates an empty `Graph` struct.
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            vertices: Vec::new(),
        }
    }

    /// Creates an empty `Graph` struct with reserved capacities for `nodes` and `edges`.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
        }
    }

    /// Adds a new node to the graph.
    /// 
    /// Returns the `NodeIndex` to remember the node.
    pub fn add_node(&mut self) -> NodeIndex {
        let index = self.vertices.len();
        self.vertices.push(NodeData { 
            first_outgoing_edge: None,
            first_incoming_edge: None,
        });
        index
    }

    /// Checks if a given `source` node is connected to the given `target` node.
    pub fn has_edge(&self, source: NodeIndex, target: NodeIndex) -> bool {
        let mut successors = self.successors(source);
        successors.find(|f| f == &target).is_some()
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Adds a new edge to the graph from `source` to `target`.
    /// 
    /// Returns true if the edge insertion was successful. Returns false if the edge
    /// relationship already exists or the edge is a self-loop.
    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) -> bool {
        // do not allow duplicate edges or self-loops
        if self.has_edge(source, target) == true || source == target { return false }

        let edge_index = self.edges.len();
        // enter source -> target data
        {
            let node_data = &mut self.vertices[source];
            self.edges.push(EdgeData { 
                source: source,
                target: target, 
                next_outgoing_edge: node_data.first_outgoing_edge,
                next_incoming_edge: None,
            });
            node_data.first_outgoing_edge = Some(edge_index);
        }
        // enter target <- source data
        let rev_node_data = &mut self.vertices[target];
        let incoming_edge = rev_node_data.first_incoming_edge;
        rev_node_data.first_incoming_edge = Some(edge_index);
        // update the edge data
        self.edges.last_mut().unwrap().next_incoming_edge = incoming_edge;
        true
    }

    /// Returns the number of successors to the `source` node.
    pub fn out_degree(&self, source: NodeIndex) -> usize {
        self.successors(source).count()
    }

    /// Returns the number of predecessors to the `source` node.
    pub fn in_degree(&self, target: NodeIndex) -> usize {
        self.predecessors(target).count()
    }

    /// Creates an iterator over the incoming nodes to the `target` source.
    pub fn predecessors(&self, target: NodeIndex) -> Predecessors {
        let first_incoming_edge = self.vertices[target].first_incoming_edge;
        Predecessors { graph: self, current_edge_index: first_incoming_edge }
    }

    /// Creates an iterator over the outgoing nodes from the `source` node.
    pub fn successors(&self, source: NodeIndex) -> Successors {
        let first_outgoing_edge = self.vertices[source].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }

    /// Checks if the graph has zero nodes.
    pub fn is_empty(&self) -> bool {
        self.node_count() == 0
    }

    /// Checks if the graph contains a cycle.
    pub fn is_cyclic(&self) -> bool {
        if self.is_empty() { return false }

        for i in 0..self.node_count() {
            if self.in_degree(i) == 0 {
                return false
            }
        }
        true
    }

    /// Determines which node has zero outgoing edges as the 'root' node.
    /// 
    /// Returns a list of possible roots (potentially zero) as an err if there is
    /// not just a definite single root.
    pub fn find_root(&self) -> Result<NodeIndex, Vec<NodeIndex>> {
        // grab a list of all nodes that have zero outgoing edges
        let roots: Vec<NodeIndex> = self.vertices.iter().enumerate().filter_map(|(i, n)| {
            if n.first_outgoing_edge.is_none() { Some(i) } else { None }
        }).collect();
        // check how many roots were detected
        match roots.len() {
            1 => Ok(*roots.first().unwrap()),
            _ => Err(roots)
        }
    }

    /// Performs depth-first search algorithm starting from the `target` node.
    pub fn depth_first_search(&self, target: NodeIndex) -> Vec<NodeIndex> {
        let mut traversal = Vec::new();
        // add target to the list
        traversal.push(target);
        // select predecessors
        let mut tunnels: Box<dyn Iterator<Item=usize>> = Box::new(self.predecessors(target));
        while let Some(n) = tunnels.next() {
            // select node
            traversal.push(n);
            // add its predecessors to list to process (in front of current nodes to process)
            tunnels = Box::new(self.predecessors(n).chain(tunnels));
        }
        traversal
    }

    /// Performs topological sort to give in-order nodes to perform given tasks
    /// based upon dependencies.
    pub fn topological_sort(&self) -> Vec<NodeIndex> {
        let mut order = Vec::<NodeIndex>::with_capacity(self.node_count());
        // store the set of remaining nodes with incoming edges
        let mut store: Vec<Option<HashSet<NodeIndex>>> = Vec::with_capacity(self.node_count());
        let mut able_to_complete: Option<NodeIndex> = None;
        for i in 0..self.node_count() {
            let deps = self.predecessors(i).collect::<HashSet<NodeIndex>>();
            // mark first node which has 0 dependencies
            if deps.len() == 0 && able_to_complete.is_none() {
                able_to_complete = Some(i);
            }
            store.push(Some(deps));
        }
        // continue processing tasks able to be completed
        while let Some(current) = able_to_complete {
            // take from store
            store[current] = None;
            able_to_complete = None;
            for (i, task) in store.iter_mut().enumerate() {
                if let Some(deps) = task {
                    // remove current task from this task's dependencies
                    deps.remove(&current);
                    // find node with zero predecessors
                    if deps.len() == 0 && able_to_complete.is_none() {
                        able_to_complete = Some(i);
                    }
                }
            }
            order.push(current);
        }
        order
    }

    /// Performs topological sort on the entire graph and then only selects the
    /// minimal number of affected nodes needed process up to `target`.
    pub fn minimal_topological_sort(&self, target: NodeIndex) -> Vec<NodeIndex> {
        // order the nodes
        let total_order = self.topological_sort();
        // collect number of nodes a part of the partial tree starting from `target`
        let effected_nodes: HashSet<usize> = self.depth_first_search(target).into_iter().collect();
        // filter out all nodes not in the hashset
        total_order.into_iter()
            .filter(|f| { effected_nodes.contains(f) == true })
            .collect()
    }
}

pub struct Predecessors<'graph> {
    graph: &'graph Graph,
    current_edge_index: Option<EdgeIndex>
}

impl<'graph> Iterator for Predecessors<'graph> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_incoming_edge;
                Some(edge.source)
            }
        }
    }
}

pub struct Successors<'graph> {
    graph: &'graph Graph,
    current_edge_index: Option<EdgeIndex>,
}

impl<'graph> Iterator for Successors<'graph> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    /// Creates basic graph illustrated in this blog post:
    /// - source: http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
    fn basic_graph() -> Graph {
        let mut g = Graph::new();
        let n0 = g.add_node();
        let n1 = g.add_node();
        let n2 = g.add_node();
        let n3 = g.add_node();
        g.add_edge(n0, n1);
        g.add_edge(n1, n2);
        g.add_edge(n0, n3);
        g.add_edge(n3, n2);
        g
    }

    /// Creates an example binary tree with height = 2. Edges are directed up indicating
    /// the child is a dependency to the parent.
    ///
    ///         n0
    ///        /  \
    ///     n1     n4
    ///    /  \   /  \
    /// n2   n3 n5   n6
    fn binary_tree() -> Graph {
        // create binary tree
        let mut g = Graph::new();
        let n0 = g.add_node();
        let n1 = g.add_node();
        let n2 = g.add_node();
        let n3 = g.add_node();
        let n4 = g.add_node();
        let n5 = g.add_node();
        let n6 = g.add_node();
        // level 1
        g.add_edge(n1, n0); // n1 -> n0
        g.add_edge(n4, n0); // n4 -> n0
        // level 2 - L
        g.add_edge(n2, n1); // n2 -> n1
        g.add_edge(n3, n1); // n3 -> n1
        // level 2 - R
        g.add_edge(n5, n4); // n5 -> n4
        g.add_edge(n6, n4); // n6 -> n4
        g
    }

    #[test]
    fn is_cyclic() {
        let mut g = basic_graph();
        assert_eq!(g.is_cyclic(), false);
        // add in edge to make cycle
        g.add_edge(1, 0);
        assert_eq!(g.is_cyclic(), true);

        let mut g = basic_graph();
        g.add_edge(2, 0);
        assert_eq!(g.is_cyclic(), true);

        let g = binary_tree();
        assert_eq!(g.is_cyclic(), false);
    }

    #[test]
    fn topological_sort() {
        let mut g = basic_graph();
        assert_eq!(g.topological_sort(), vec![0, 1, 3, 2]);

        let n0 = g.add_node();
        g.add_edge(n0, 0);
        assert_eq!(g.topological_sort(), vec![4, 0, 1, 3, 2]);
        
        let g = binary_tree();
        assert_eq!(g.topological_sort(), vec![2, 3, 1, 5, 6, 4, 0]);
    }

    #[test]
    fn add_node() {
        let mut g = Graph::new();
        assert_eq!(g.node_count(), 0);
        g.add_node();
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn add_edge() {
        let mut g = Graph::new();
        assert_eq!(g.edge_count(), 0);
        let n0 = g.add_node();
        let n1 = g.add_node();
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g.add_edge(n0, n1), true);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.add_edge(n1, n0), true);
        assert_eq!(g.edge_count(), 2);
        // do not allow duplicate edges
        assert_eq!(g.add_edge(n1, n0), false);
        assert_eq!(g.edge_count(), 2);
        // do not allow self-loops
        assert_eq!(g.add_edge(n0, n0), false);
    }

    #[test]
    fn find_root() {
        let mut g = binary_tree();
        assert_eq!(g.find_root(), Ok(0));

        // add a second possible root to the tree
        let n7 = g.add_node();
        g.add_edge(4, n7);
        assert_eq!(g.find_root(), Err(vec![0, 7]));

        // add edges between the two known roots: n0 and n7
        g.add_edge(0, n7);
        g.add_edge(n7,0);
        assert_eq!(g.find_root(), Err(vec![]));
    }

    #[test]
    fn out_degree() {
        let g = basic_graph();
        assert_eq!(g.out_degree(0), 2);
        assert_eq!(g.out_degree(1), 1);
        assert_eq!(g.out_degree(2), 0);
        assert_eq!(g.out_degree(3), 1);
    }

    #[test]
    fn in_degree() {
        let g = basic_graph();
        assert_eq!(g.in_degree(0), 0);
        assert_eq!(g.in_degree(1), 1);
        assert_eq!(g.in_degree(2), 2);
        assert_eq!(g.in_degree(3), 1);
    }

    #[test]
    fn successors() {
        let g = basic_graph();
        assert_eq!(g.successors(0).collect::<Vec<NodeIndex>>(), vec![3, 1]);
        assert_eq!(g.successors(1).collect::<Vec<NodeIndex>>(), vec![2]);
        assert_eq!(g.successors(2).collect::<Vec<NodeIndex>>(), vec![]);
        assert_eq!(g.successors(3).collect::<Vec<NodeIndex>>(), vec![2]);
    }

    #[test]
    fn predecessors() {
        let g = basic_graph();
        assert_eq!(g.predecessors(0).collect::<Vec<usize>>(), vec![]);
        assert_eq!(g.predecessors(1).collect::<Vec<usize>>(), vec![0]);
        assert_eq!(g.predecessors(2).collect::<Vec<usize>>(), vec![3, 1]);
        assert_eq!(g.predecessors(3).collect::<Vec<usize>>(), vec![0]);
    }

    #[test]
    fn min_top_sort() {
        let g = binary_tree();
        assert_eq!(g.minimal_topological_sort(0), vec![2, 3, 1, 5, 6, 4, 0]);

        assert_eq!(g.minimal_topological_sort(1), vec![2, 3, 1]);

        assert_eq!(g.minimal_topological_sort(4), vec![5, 6, 4]);
    }

    #[test]
    fn has_edge() {
        let mut g = Graph::new();
        let n0 = g.add_node();
        let n1 = g.add_node();
        let n2 = g.add_node();
        assert_eq!(g.has_edge(n0, n1), false);
        g.add_edge(n0, n1);
        assert_eq!(g.has_edge(n0, n1), true);
        assert_eq!(g.has_edge(n1, n0), false);

        assert_eq!(g.has_edge(n1, n2), false);
        g.add_edge(n1, n2);
        assert_eq!(g.has_edge(n1, n2), true);
    }

    #[test]
    fn dfs() {
        let g = binary_tree();
        // from top node
        assert_eq!(g.depth_first_search(0), vec![0, 4, 6, 5, 1, 3, 2]);

        // from intermediate node
        assert_eq!(g.depth_first_search(1), vec![1, 3, 2]);

        // from intermediate node
        assert_eq!(g.depth_first_search(4), vec![4, 6, 5]);

        // from leaf node
        assert_eq!(g.depth_first_search(6), vec![6]);
    }
}

use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Eq;

#[derive(Debug, PartialEq)]
struct HashGraph<V: Hash + Eq> {
    table: HashMap<usize, V>,
    // data: Vec<V>,
    inner: Graph,
}

// @IDEA look up a node by its key, access its value

impl<V> HashGraph<V> where V: Hash + Eq {
    pub fn new() -> Self {
        Self {
            // map: HashMap::new(),
            table: HashMap::new(),
            inner: Graph::new(),
        }
    }

    pub fn add_node(&mut self, node: V) -> NodeIndex {
        let marker = self.inner.add_node();
        self.table.insert(marker, node);
        // self.map.insert(self.get_vertex(marker), marker);
        marker
    }

    // pub fn get_vertex_index(&self, vertex: &V) -> NodeIndex {
    //     *self.map.get(vertex).unwrap()
    // }

    pub fn get_vertex(&self, node: NodeIndex) -> &V {
        self.table.get(&node).unwrap()
    }

    // pub fn add_edge(&mut self, source: &V, target: &V) -> bool {
    //     let s0 = self.map.get(source).unwrap();
    //     let s1 = self.map.get(target).unwrap();
    //     self.inner.add_edge(*s0, *s1)
    // }

    pub fn add_edge_by_index(&mut self, source: NodeIndex, target: NodeIndex) -> bool {
        self.inner.add_edge(source, target)
    }

    pub fn topological_sort(&self) -> Vec<&V> {
        self.inner.topological_sort()
            .into_iter()
            .map(|f| {
                self.table.get(&f).unwrap()
            }).collect()
    }
}

#[cfg(test)]
mod hg_test {
    use super::*;

    #[test]
    fn it_works() {
        let mut g = HashGraph::<String>::new();
        g.add_node(String::from("hello world!"));
        g.add_node(String::from("hello world!"));
    }
}