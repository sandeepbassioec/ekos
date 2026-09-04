//! Distance metrics for vector search.

/// Cosine similarity between two vectors. Returns 0.0 if either is zero-length.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// Euclidean (L2) distance between two vectors.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Dot product between two vectors.
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_same_vector() {
        let v = vec![1.0f32, 0.0, 0.0];
        let s = cosine_similarity(&v, &v);
        assert!((s - 1.0).abs() < 1e-6, "cosine of identical vectors = 1");
    }

    #[test]
    fn test_cosine_orthogonal() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        let s = cosine_similarity(&a, &b);
        assert!((s).abs() < 1e-6, "orthogonal vectors = 0");
    }

    #[test]
    fn test_euclidean_zero() {
        let v = vec![1.0f32, 2.0, 3.0];
        assert!((euclidean_distance(&v, &v)).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_known() {
        let a = vec![0.0f32, 0.0];
        let b = vec![3.0f32, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0f32, 2.0, 3.0];
        let b = vec![4.0f32, 5.0, 6.0];
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-6);
    }
}
