// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Thompson Sampling via Beta distribution.
//!
//! Provides `sample_beta(rng, alpha, beta)` used to score relays stochastically
//! based on their delivery history: `Beta(delivered + 1, expected - delivered + 1)`.

use rand::Rng;

/// Sample from `Beta(alpha, beta)` distribution.
///
/// Uses the gamma-variate method: `Beta(a,b) = X / (X + Y)` where
/// `X ~ Gamma(a)` and `Y ~ Gamma(b)`.
pub fn sample_beta<R: Rng>(rng: &mut R, alpha: f64, beta: f64) -> f64 {
    debug_assert!(alpha > 0.0, "alpha must be positive");
    debug_assert!(beta > 0.0, "beta must be positive");

    let x = sample_gamma(rng, alpha);
    let y = sample_gamma(rng, beta);

    if x + y == 0.0 {
        0.5
    } else {
        x / (x + y)
    }
}

/// Sample from `Gamma(shape, 1)` using Marsaglia-Tsang's method.
///
/// For `shape >= 1`: direct Marsaglia-Tsang.
/// For `shape < 1`: `Gamma(shape) = Gamma(shape + 1) * U^(1/shape)`.
fn sample_gamma<R: Rng>(rng: &mut R, shape: f64) -> f64 {
    if shape < 1.0 {
        // Boost: Gamma(a) = Gamma(a+1) * U^(1/a) for a < 1
        let u: f64 = rng.random::<f64>();
        return sample_gamma(rng, shape + 1.0) * u.powf(1.0 / shape);
    }

    // Marsaglia-Tsang method for shape >= 1
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();

    loop {
        let x: f64 = sample_standard_normal(rng);
        let v = 1.0 + c * x;
        if v <= 0.0 {
            continue;
        }
        let v = v * v * v;
        let u: f64 = rng.random::<f64>();

        // Squeeze test then full rejection check
        if u < 1.0 - 0.0331 * (x * x) * (x * x) {
            return d * v;
        }
        if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
            return d * v;
        }
    }
}

/// Sample from standard normal N(0,1) using Box-Muller transform.
fn sample_standard_normal<R: Rng>(rng: &mut R) -> f64 {
    // Map [0,1) to (0,1] to avoid ln(0)
    let u1: f64 = 1.0 - rng.random::<f64>();
    let u2: f64 = rng.random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_beta_bounds() {
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let val = sample_beta(&mut rng, 1.0, 1.0);
            assert!(
                (0.0..=1.0).contains(&val),
                "Beta sample out of [0,1]: {val}"
            );
        }
    }

    #[test]
    fn test_sample_beta_mean() {
        // Beta(2,5) has mean = 2/7 ≈ 0.2857
        let mut rng = rand::rng();
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| sample_beta(&mut rng, 2.0, 5.0)).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - 2.0 / 7.0).abs() < 0.03,
            "Beta(2,5) mean {mean} too far from 0.2857"
        );
    }

    #[test]
    fn test_sample_beta_small_params() {
        // Beta(0.5, 0.5) = arcsine distribution, mean = 0.5
        let mut rng = rand::rng();
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| sample_beta(&mut rng, 0.5, 0.5)).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - 0.5).abs() < 0.03,
            "Beta(0.5,0.5) mean {mean} too far from 0.5"
        );
    }

    #[test]
    fn test_uniform_prior() {
        // Beta(1,1) = uniform, mean = 0.5
        let mut rng = rand::rng();
        let n = 10_000;
        let sum: f64 = (0..n).map(|_| sample_beta(&mut rng, 1.0, 1.0)).sum();
        let mean = sum / n as f64;
        assert!(
            (mean - 0.5).abs() < 0.03,
            "Beta(1,1) mean {mean} too far from 0.5"
        );
    }
}
