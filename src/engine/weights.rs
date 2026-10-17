//! Weight Drift: dimensions the user attends to come to count for more.

use std::collections::HashMap;

use super::constants::weights::RECALIBRATION_THRESHOLD;

/// Rescales `weights` so that their mean is exactly 1.0. Leaves them
/// unchanged if empty or if their sum is not positive.
pub fn normalize(weights: &mut [(String, f64)]) {
    let sum: f64 = weights.iter().map(|(_, w)| w).sum();
    if weights.is_empty() || sum <= 0.0 {
        return;
    }
    let scale = weights.len() as f64 / sum;
    for (_, w) in weights.iter_mut() {
        *w *= scale;
    }
}

/// A weight revised by more than the recalibration threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct Revision {
    pub dimension_id: String,
    pub from: f64,
    pub to: f64,
}

/// Weights that differ from `baseline` by more than the threshold, largest
/// relative change first (ties by identifier). Dimensions absent from the baseline are compared
/// with 1.0.
pub fn revisions(baseline: &HashMap<String, f64>, current: &[(String, f64)]) -> Vec<Revision> {
    let mut out: Vec<Revision> = current
        .iter()
        .filter_map(|(id, to)| {
            let from = baseline.get(id).copied().unwrap_or(1.0);
            ((to - from).abs() / from > RECALIBRATION_THRESHOLD).then(|| Revision {
                dimension_id: id.clone(),
                from,
                to: *to,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        let rel = |r: &Revision| (r.to - r.from).abs() / r.from;
        rel(b)
            .total_cmp(&rel(a))
            .then_with(|| a.dimension_id.cmp(&b.dimension_id))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(spec: &[(&str, f64)]) -> Vec<(String, f64)> {
        spec.iter().map(|&(id, v)| (id.to_owned(), v)).collect()
    }

    #[test]
    fn normalized_mean_is_one() {
        let mut ws = w(&[("A", 1.06), ("B", 1.0), ("C", 1.0), ("D", 1.02)]);
        normalize(&mut ws);
        let mean = ws.iter().map(|(_, v)| v).sum::<f64>() / ws.len() as f64;
        assert!((mean - 1.0).abs() < 1e-12);
        // Relative order and proportions are kept.
        assert!((ws[0].1 / ws[1].1 - 1.06).abs() < 1e-12);
        let mut empty: Vec<(String, f64)> = vec![];
        normalize(&mut empty);
    }

    #[test]
    fn small_drifts_do_not_prompt_recalibration() {
        let baseline = HashMap::new();
        let mut ws = w(&[("A", 1.04), ("B", 1.0), ("C", 1.0)]);
        normalize(&mut ws);
        assert!(revisions(&baseline, &ws).is_empty());
    }

    #[test]
    fn dwelling_on_a_dimension_does() {
        let baseline = HashMap::new();
        let mut ws = w(&[("A", 1.08), ("B", 1.0), ("C", 1.0), ("D", 1.0)]);
        normalize(&mut ws);
        let r = revisions(&baseline, &ws);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].dimension_id, "A");
        assert!(r[0].to > 1.05);
    }

    #[test]
    fn compared_with_the_last_recalibration() {
        let baseline: HashMap<String, f64> = [("A".to_owned(), 1.2), ("B".to_owned(), 0.8)].into();
        let current = w(&[("A", 1.2), ("B", 0.9)]);
        let r = revisions(&baseline, &current);
        assert_eq!(r.len(), 1);
        assert_eq!((r[0].from, r[0].to), (0.8, 0.9));
    }
}
