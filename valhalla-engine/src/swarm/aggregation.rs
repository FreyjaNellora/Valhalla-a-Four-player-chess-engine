//! Swarm aggregation — combines six layers into composite score + stability.
//!
//! composite = Σ(contribution_i × confidence_i × weight_i) / Σ(confidence_i × weight_i)
//! stability = min(confidence_i for layers where weight_i > threshold)

/// Aggregate six layer outputs into a composite score and stability.
///
/// # Arguments
/// - `contributions`: per-layer contribution values
/// - `confidences`: per-layer confidence values (0.0..=1.0)
/// - `weights`: per-layer weights
/// - `threshold`: minimum weight for a layer to affect stability
///
/// # Returns
/// `(composite_score, stability)` where composite is the weighted average
/// and stability is the minimum confidence across significant layers.
pub fn aggregate(
    contributions: &[f32; 6],
    confidences: &[f32; 6],
    weights: &[f32; 6],
    threshold: f32,
) -> (f32, f32) {
    let mut weighted_sum = 0.0f32;
    let mut weight_total = 0.0f32;
    let mut min_confidence = 1.0f32;

    for i in 0..6 {
        let w = weights[i] * confidences[i];
        weighted_sum += contributions[i] * w;
        weight_total += w;

        if weights[i] > threshold {
            min_confidence = min_confidence.min(confidences[i]);
        }
    }

    let composite = if weight_total > 0.001 {
        weighted_sum / weight_total
    } else {
        0.0
    };

    (composite, min_confidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_zero() {
        let c = [0.0; 6];
        let conf = [1.0; 6];
        let w = [1.0; 6];
        let (composite, stability) = aggregate(&c, &conf, &w, 0.1);
        assert_eq!(composite, 0.0);
        assert_eq!(stability, 1.0);
    }

    #[test]
    fn test_single_layer_positive() {
        let c = [0.5, 0.0, 0.0, 0.0, 0.0, 0.0];
        let conf = [0.8, 1.0, 1.0, 1.0, 1.0, 1.0];
        let w = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // only L1 active
        let (composite, stability) = aggregate(&c, &conf, &w, 0.1);
        assert!((composite - 0.5).abs() < 0.01);
        assert_eq!(stability, 0.8);
    }

    #[test]
    fn test_low_confidence_reduces_stability() {
        let c = [0.5, 0.3, 0.0, 0.0, 0.0, 0.0];
        let conf = [0.9, 0.2, 1.0, 1.0, 1.0, 1.0]; // L2 low confidence
        let w = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let (_, stability) = aggregate(&c, &conf, &w, 0.1);
        assert_eq!(stability, 0.2); // min of significant layer confidences
    }

    #[test]
    fn test_below_threshold_ignored_for_stability() {
        let c = [0.5, 0.0, 0.0, 0.0, 0.0, 0.0];
        let conf = [0.9, 0.1, 1.0, 1.0, 1.0, 1.0];
        let w = [1.0, 0.05, 0.0, 0.0, 0.0, 0.0]; // L2 below threshold
        let (_, stability) = aggregate(&c, &conf, &w, 0.1);
        assert_eq!(stability, 0.9); // L2 ignored because weight < threshold
    }
}
