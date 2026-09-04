use crate::error::GraphResult;
use crate::reverse_csr::ReverseCsr;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};

pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_size_bytes: u64,
}

pub struct GraphStore {
    base_dir: PathBuf,
    edges: RwLock<Vec<(u64, u64)>>,
}

impl GraphStore {
    pub fn open(dir: &Path) -> GraphResult<Self> {
        std::fs::create_dir_all(dir)?;
        let graph_file = dir.join("graph.bin");
        let edges = if graph_file.exists() {
            Self::load_edges(&graph_file)?
        } else {
            Vec::new()
        };
        Ok(Self {
            base_dir: dir.to_path_buf(),
            edges: RwLock::new(edges),
        })
    }

    fn load_edges(path: &Path) -> GraphResult<Vec<(u64, u64)>> {
        let data = std::fs::read(path)?;
        let mut edges = Vec::new();
        let mut i = 0;
        while i + 16 <= data.len() {
            let src = u64::from_le_bytes(data[i..i + 8].try_into().unwrap());
            let dst = u64::from_le_bytes(data[i + 8..i + 16].try_into().unwrap());
            edges.push((src, dst));
            i += 16;
        }
        Ok(edges)
    }

    fn serialize_edges(edges: &[(u64, u64)]) -> Vec<u8> {
        let mut data = Vec::with_capacity(edges.len() * 16);
        for &(src, dst) in edges {
            data.extend_from_slice(&src.to_le_bytes());
            data.extend_from_slice(&dst.to_le_bytes());
        }
        data
    }

    pub fn add_edge(&self, src: u64, dst: u64) -> GraphResult<()> {
        let mut edges = self.edges.write();
        if !edges.contains(&(src, dst)) {
            edges.push((src, dst));
        }
        Ok(())
    }

    pub fn remove_edge(&self, src: u64, dst: u64) -> GraphResult<()> {
        let mut edges = self.edges.write();
        edges.retain(|&(s, d)| !(s == src && d == dst));
        Ok(())
    }

    pub fn remove_node(&self, node: u64) -> GraphResult<Vec<u64>> {
        let edges_snap = self.edges.read().clone();
        let reverse = ReverseCsr::from_edges(&edges_snap);
        let cascade = crate::cascade::CascadeDelete::new(reverse);
        let mut to_delete = cascade.affected_nodes(node);
        to_delete.push(node);
        let delete_set: std::collections::HashSet<u64> = to_delete.iter().copied().collect();
        let mut edges = self.edges.write();
        edges.retain(|&(s, d)| !delete_set.contains(&s) && !delete_set.contains(&d));
        Ok(to_delete)
    }

    pub fn save(&self) -> GraphResult<()> {
        let edges = self.edges.read();
        let data = Self::serialize_edges(&edges);
        let path = self.base_dir.join("graph.bin");
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn reload(&self) -> GraphResult<()> {
        let path = self.base_dir.join("graph.bin");
        if path.exists() {
            let new_edges = Self::load_edges(&path)?;
            let mut edges = self.edges.write();
            *edges = new_edges;
        }
        Ok(())
    }

    pub fn stats(&self) -> GraphStats {
        let edges = self.edges.read();
        let path = self.base_dir.join("graph.bin");
        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let reverse = ReverseCsr::from_edges(&edges);
        GraphStats {
            node_count: reverse.node_count(),
            edge_count: reverse.edge_count(),
            file_size_bytes,
        }
    }

    pub fn snapshot(&self) -> ReverseCsr {
        let edges = self.edges.read();
        ReverseCsr::from_edges(&edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn open_empty() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        let stats = store.stats();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
    }

    #[test]
    fn add_and_save_reload() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.add_edge(2, 3).unwrap();
        store.save().unwrap();

        let store2 = GraphStore::open(dir.path()).unwrap();
        let stats = store2.stats();
        assert_eq!(stats.edge_count, 2);
    }

    #[test]
    fn remove_edge() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.add_edge(2, 3).unwrap();
        store.remove_edge(1, 2).unwrap();
        let stats = store.stats();
        assert_eq!(stats.edge_count, 1);
    }

    #[test]
    fn remove_nonexistent_edge_ok() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.remove_edge(99, 100).unwrap();
    }

    #[test]
    fn snapshot_correct() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        let snap = store.snapshot();
        assert_eq!(snap.edge_count(), 1);
    }

    #[test]
    fn remove_node_cascade() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.add_edge(2, 3).unwrap();
        let deleted = store.remove_node(2).unwrap();
        assert!(deleted.contains(&2));
        assert!(deleted.contains(&1));
    }

    #[test]
    fn stats_after_add() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(10, 20).unwrap();
        store.add_edge(20, 30).unwrap();
        let stats = store.stats();
        assert_eq!(stats.edge_count, 2);
        assert_eq!(stats.node_count, 3);
    }

    #[test]
    fn duplicate_edge_not_added() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.add_edge(1, 2).unwrap();
        assert_eq!(store.stats().edge_count, 1);
    }

    #[test]
    fn reload_after_save() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(5, 6).unwrap();
        store.save().unwrap();
        store.reload().unwrap();
        assert_eq!(store.stats().edge_count, 1);
    }

    #[test]
    fn persist_and_load_correct_edges() {
        let dir = tempdir().unwrap();
        {
            let store = GraphStore::open(dir.path()).unwrap();
            store.add_edge(100, 200).unwrap();
            store.add_edge(200, 300).unwrap();
            store.save().unwrap();
        }
        let store2 = GraphStore::open(dir.path()).unwrap();
        let snap = store2.snapshot();
        assert!(snap.forward().contains_edge(100, 200));
        assert!(snap.forward().contains_edge(200, 300));
    }

    #[test]
    fn file_size_nonzero_after_save() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.save().unwrap();
        let stats = store.stats();
        assert!(stats.file_size_bytes > 0);
    }

    #[test]
    fn concurrent_add_edge() {
        use std::thread;
        let dir = tempdir().unwrap();
        let store = Arc::new(GraphStore::open(dir.path()).unwrap());
        let handles: Vec<_> = (0u64..10)
            .map(|i| {
                let s = Arc::clone(&store);
                thread::spawn(move || {
                    s.add_edge(i, i + 100).unwrap();
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(store.stats().edge_count, 10);
    }

    #[test]
    fn remove_node_cleans_edges() {
        let dir = tempdir().unwrap();
        let store = GraphStore::open(dir.path()).unwrap();
        store.add_edge(1, 2).unwrap();
        store.add_edge(2, 3).unwrap();
        store.add_edge(3, 4).unwrap();
        store.remove_node(3).unwrap();
        let snap = store.snapshot();
        assert!(!snap.forward().contains_edge(2, 3));
        assert!(!snap.forward().contains_edge(3, 4));
    }
}
