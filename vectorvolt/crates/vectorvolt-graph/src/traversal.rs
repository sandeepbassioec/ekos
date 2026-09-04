use crate::csr::CsrGraph;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn bfs(graph: &CsrGraph, start: u64, max_depth: usize) -> Vec<(u64, usize)> {
    if !graph.contains_node(start) {
        return vec![];
    }
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back((start, 0usize));
    while let Some((node, depth)) = queue.pop_front() {
        result.push((node, depth));
        if depth < max_depth {
            for &nb in graph.neighbors(node) {
                if visited.insert(nb) {
                    queue.push_back((nb, depth + 1));
                }
            }
        }
    }
    result
}

pub fn dfs(graph: &CsrGraph, start: u64, max_depth: usize) -> Vec<(u64, usize)> {
    if !graph.contains_node(start) {
        return vec![];
    }
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    dfs_inner(graph, start, 0, max_depth, &mut visited, &mut result);
    result
}

fn dfs_inner(
    graph: &CsrGraph,
    node: u64,
    depth: usize,
    max_depth: usize,
    visited: &mut HashSet<u64>,
    result: &mut Vec<(u64, usize)>,
) {
    if !visited.insert(node) {
        return;
    }
    result.push((node, depth));
    if depth < max_depth {
        for &nb in graph.neighbors(node) {
            dfs_inner(graph, nb, depth + 1, max_depth, visited, result);
        }
    }
}

pub fn reachable(graph: &CsrGraph, start: u64) -> HashSet<u64> {
    bfs(graph, start, usize::MAX)
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}

pub fn shortest_path(graph: &CsrGraph, src: u64, dst: u64) -> Option<Vec<u64>> {
    if !graph.contains_node(src) || !graph.contains_node(dst) {
        return None;
    }
    if src == dst {
        return Some(vec![src]);
    }
    let mut visited = HashSet::new();
    let mut parent: HashMap<u64, u64> = HashMap::new();
    let mut queue = VecDeque::new();
    visited.insert(src);
    queue.push_back(src);
    while let Some(node) = queue.pop_front() {
        for &nb in graph.neighbors(node) {
            if visited.insert(nb) {
                parent.insert(nb, node);
                if nb == dst {
                    let mut path = vec![dst];
                    let mut cur = dst;
                    while cur != src {
                        cur = *parent.get(&cur).unwrap();
                        path.push(cur);
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(nb);
            }
        }
    }
    None
}

pub fn connected_components(graph: &CsrGraph) -> Vec<Vec<u64>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    for &node in graph.nodes() {
        if !visited.contains(&node) {
            let comp = undirected_bfs(graph, node, &mut visited);
            components.push(comp);
        }
    }
    components
}

fn undirected_bfs(graph: &CsrGraph, start: u64, visited: &mut HashSet<u64>) -> Vec<u64> {
    let mut comp = Vec::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);
    while let Some(node) = queue.pop_front() {
        comp.push(node);
        for &nb in graph.neighbors(node) {
            if visited.insert(nb) {
                queue.push_back(nb);
            }
        }
    }
    comp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain_graph() -> CsrGraph {
        CsrGraph::from_edges(&[(1, 2), (2, 3), (3, 4)])
    }

    #[test]
    fn bfs_visits_all_reachable() {
        let g = chain_graph();
        let result = bfs(&g, 1, 10);
        let nodes: Vec<u64> = result.iter().map(|&(n, _)| n).collect();
        assert!(nodes.contains(&1));
        assert!(nodes.contains(&2));
        assert!(nodes.contains(&3));
        assert!(nodes.contains(&4));
    }

    #[test]
    fn bfs_max_depth() {
        let g = chain_graph();
        let result = bfs(&g, 1, 2);
        let nodes: Vec<u64> = result.iter().map(|&(n, _)| n).collect();
        assert!(nodes.contains(&1));
        assert!(nodes.contains(&2));
        assert!(nodes.contains(&3));
        assert!(!nodes.contains(&4));
    }

    #[test]
    fn bfs_unknown_start() {
        let g = chain_graph();
        let result = bfs(&g, 99, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn dfs_visits_all_reachable() {
        let g = chain_graph();
        let result = dfs(&g, 1, 10);
        let nodes: Vec<u64> = result.iter().map(|&(n, _)| n).collect();
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn dfs_max_depth() {
        let g = chain_graph();
        let result = dfs(&g, 1, 1);
        let nodes: Vec<u64> = result.iter().map(|&(n, _)| n).collect();
        assert!(nodes.contains(&1));
        assert!(nodes.contains(&2));
        assert!(!nodes.contains(&3));
    }

    #[test]
    fn reachable_set() {
        let g = chain_graph();
        let r = reachable(&g, 2);
        assert!(r.contains(&2));
        assert!(r.contains(&3));
        assert!(r.contains(&4));
        assert!(!r.contains(&1));
    }

    #[test]
    fn shortest_path_direct() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (2, 4), (3, 4)]);
        let path = shortest_path(&g, 1, 4).unwrap();
        assert_eq!(path[0], 1);
        assert_eq!(*path.last().unwrap(), 4);
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn shortest_path_same_node() {
        let g = chain_graph();
        let path = shortest_path(&g, 2, 2).unwrap();
        assert_eq!(path, vec![2]);
    }

    #[test]
    fn shortest_path_no_path() {
        let g = chain_graph();
        let result = shortest_path(&g, 4, 1);
        assert!(result.is_none());
    }

    #[test]
    fn shortest_path_unknown_node() {
        let g = chain_graph();
        assert!(shortest_path(&g, 99, 1).is_none());
        assert!(shortest_path(&g, 1, 99).is_none());
    }

    #[test]
    fn connected_components_single() {
        let g = chain_graph();
        let comps = connected_components(&g);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].len(), 4);
    }

    #[test]
    fn connected_components_two() {
        let g = CsrGraph::from_edges(&[(1, 2), (3, 4)]);
        let comps = connected_components(&g);
        assert_eq!(comps.len(), 2);
    }

    #[test]
    fn bfs_depths_correct() {
        let g = chain_graph();
        let result = bfs(&g, 1, 10);
        let depth_map: HashMap<u64, usize> = result.into_iter().collect();
        assert_eq!(depth_map[&1], 0);
        assert_eq!(depth_map[&2], 1);
        assert_eq!(depth_map[&3], 2);
        assert_eq!(depth_map[&4], 3);
    }
}
