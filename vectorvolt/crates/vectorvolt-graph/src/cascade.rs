use crate::reverse_csr::ReverseCsr;
use std::collections::HashSet;

pub struct CascadeDelete {
    reverse: ReverseCsr,
}

impl CascadeDelete {
    pub fn new(reverse: ReverseCsr) -> Self {
        Self { reverse }
    }

    pub fn affected_nodes(&self, root: u64) -> Vec<u64> {
        use std::collections::VecDeque;
        let mut visited = HashSet::new();
        visited.insert(root);
        let mut queue = VecDeque::new();
        queue.push_back(root);
        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            for &dep in self.reverse.predecessors(node) {
                if visited.insert(dep) {
                    result.push(dep);
                    queue.push_back(dep);
                }
            }
        }
        result
    }

    pub fn deletion_order(&self, root: u64) -> Vec<u64> {
        let mut affected = self.affected_nodes(root);
        affected.push(root);
        let affected_set: HashSet<u64> = affected.iter().copied().collect();
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        for &node in &affected {
            self.dfs_postorder(node, &affected_set, &mut visited, &mut order);
        }
        order
    }

    fn dfs_postorder(
        &self,
        node: u64,
        set: &HashSet<u64>,
        visited: &mut HashSet<u64>,
        order: &mut Vec<u64>,
    ) {
        if !visited.insert(node) {
            return;
        }
        for &pred in self.reverse.predecessors(node) {
            if set.contains(&pred) {
                self.dfs_postorder(pred, set, visited, order);
            }
        }
        order.push(node);
    }

    pub fn has_cycle(&self) -> bool {
        let graph = self.reverse.forward();
        let mut color = std::collections::HashMap::<u64, u8>::new();
        for &node in graph.nodes() {
            if !color.contains_key(&node) && Self::dfs_cycle(graph, node, &mut color) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(
        graph: &crate::csr::CsrGraph,
        node: u64,
        color: &mut std::collections::HashMap<u64, u8>,
    ) -> bool {
        color.insert(node, 1);
        for &nb in graph.neighbors(node) {
            match color.get(&nb).copied().unwrap_or(0) {
                1 => return true,
                0 if Self::dfs_cycle(graph, nb, color) => return true,
                _ => {}
            }
        }
        color.insert(node, 2);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cascade(edges: &[(u64, u64)]) -> CascadeDelete {
        CascadeDelete::new(ReverseCsr::from_edges(edges))
    }

    #[test]
    fn affected_nodes_chain() {
        // 1->2->3: deleting the source (1) has no predecessors, so nothing else is affected
        let c = make_cascade(&[(1, 2), (2, 3)]);
        assert!(c.affected_nodes(1).is_empty());
    }

    #[test]
    fn affected_nodes_reverse_chain() {
        let c = make_cascade(&[(1, 2), (2, 3)]);
        let affected = c.affected_nodes(1);
        assert!(affected.is_empty());
    }

    #[test]
    fn affected_nodes_fan_in() {
        let c = make_cascade(&[(1, 3), (2, 3)]);
        let affected = c.affected_nodes(3);
        assert!(affected.contains(&1));
        assert!(affected.contains(&2));
    }

    #[test]
    fn affected_nodes_excludes_root() {
        let c = make_cascade(&[(1, 2)]);
        let affected = c.affected_nodes(2);
        assert!(!affected.contains(&2));
    }

    #[test]
    fn deletion_order_includes_root() {
        let c = make_cascade(&[(1, 2)]);
        let order = c.deletion_order(2);
        assert!(order.contains(&2));
    }

    #[test]
    fn deletion_order_leaves_first() {
        let c = make_cascade(&[(1, 3), (2, 3)]);
        let order = c.deletion_order(3);
        let pos_1 = order.iter().position(|&x| x == 1).unwrap();
        let pos_3 = order.iter().position(|&x| x == 3).unwrap();
        assert!(pos_1 < pos_3);
    }

    #[test]
    fn no_cycle_dag() {
        let c = make_cascade(&[(1, 2), (2, 3), (1, 3)]);
        assert!(!c.has_cycle());
    }

    #[test]
    fn has_cycle_simple() {
        let c = make_cascade(&[(1, 2), (2, 3), (3, 1)]);
        assert!(c.has_cycle());
    }

    #[test]
    fn has_cycle_self_loop() {
        let c = make_cascade(&[(1, 1)]);
        assert!(c.has_cycle());
    }

    #[test]
    fn empty_no_cycle() {
        let c = make_cascade(&[]);
        assert!(!c.has_cycle());
    }

    #[test]
    fn affected_empty_graph() {
        let c = make_cascade(&[]);
        let affected = c.affected_nodes(99);
        assert!(affected.is_empty());
    }

    #[test]
    fn deletion_order_single_node() {
        let c = make_cascade(&[(1, 2)]);
        let order = c.deletion_order(1);
        assert_eq!(order, vec![1]);
    }

    #[test]
    fn has_cycle_two_nodes() {
        let c = make_cascade(&[(1, 2), (2, 1)]);
        assert!(c.has_cycle());
    }
}
