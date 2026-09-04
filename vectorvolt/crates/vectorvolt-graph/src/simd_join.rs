pub fn intersect_sorted(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    result
}

pub fn intersect_sorted_simd(a: &[u64], b: &[u64]) -> Vec<u64> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { intersect_avx2(a, b) };
        }
    }
    intersect_sorted(a, b)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn intersect_avx2(a: &[u64], b: &[u64]) -> Vec<u64> {
    intersect_sorted(a, b)
}

pub fn union_sorted(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => {
                result.push(a[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(b[j]);
                j += 1;
            }
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

use crate::csr::CsrGraph;

pub fn common_neighbors(graph: &CsrGraph, a: u64, b: u64) -> Vec<u64> {
    intersect_sorted(graph.neighbors(a), graph.neighbors(b))
}

pub fn jaccard_similarity(graph: &CsrGraph, a: u64, b: u64) -> f64 {
    let nb_a = graph.neighbors(a);
    let nb_b = graph.neighbors(b);
    let inter = intersect_sorted(nb_a, nb_b).len();
    let union_len = union_sorted(nb_a, nb_b).len();
    if union_len == 0 {
        return 0.0;
    }
    inter as f64 / union_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_basic() {
        assert_eq!(intersect_sorted(&[1, 2, 3], &[2, 3, 4]), vec![2, 3]);
    }

    #[test]
    fn intersect_empty_left() {
        assert!(intersect_sorted(&[], &[1, 2]).is_empty());
    }

    #[test]
    fn intersect_empty_right() {
        assert!(intersect_sorted(&[1, 2], &[]).is_empty());
    }

    #[test]
    fn intersect_disjoint() {
        assert!(intersect_sorted(&[1, 3], &[2, 4]).is_empty());
    }

    #[test]
    fn intersect_identical() {
        assert_eq!(intersect_sorted(&[1, 2, 3], &[1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn union_basic() {
        assert_eq!(union_sorted(&[1, 3], &[2, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn union_overlap() {
        assert_eq!(union_sorted(&[1, 2, 3], &[2, 3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn union_empty() {
        assert_eq!(union_sorted(&[], &[1, 2]), vec![1, 2]);
    }

    #[test]
    fn common_neighbors_graph() {
        let g = CsrGraph::from_edges(&[(1, 3), (1, 4), (2, 3), (2, 5)]);
        let cn = common_neighbors(&g, 1, 2);
        assert_eq!(cn, vec![3]);
    }

    #[test]
    fn jaccard_full_overlap() {
        let g = CsrGraph::from_edges(&[(1, 2), (1, 3), (4, 2), (4, 3)]);
        let j = jaccard_similarity(&g, 1, 4);
        assert!((j - 1.0).abs() < 1e-9);
    }

    #[test]
    fn jaccard_no_overlap() {
        let g = CsrGraph::from_edges(&[(1, 2), (3, 4)]);
        let j = jaccard_similarity(&g, 1, 3);
        assert_eq!(j, 0.0);
    }

    #[test]
    fn intersect_simd_matches_scalar() {
        let a = vec![1u64, 3, 5, 7, 9];
        let b = vec![3u64, 5, 6, 9, 11];
        assert_eq!(intersect_sorted_simd(&a, &b), intersect_sorted(&a, &b));
    }

    #[test]
    fn jaccard_empty_nodes() {
        let g = CsrGraph::from_edges(&[(1, 2)]);
        assert_eq!(jaccard_similarity(&g, 2, 99), 0.0);
    }
}
