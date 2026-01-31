#![cfg(feature = "server")]

use ndarray::Array2;
use scirs2_optimize::unconstrained::{Method, minimize};
use statrs::distribution::{ContinuousCDF, Normal};
use std::error::Error;

#[derive(Clone)]
pub struct Rank {
    pub means: Vec<f64>,
    ranks: Array2<i32>,
}

impl Rank {
    pub fn new(ranks: Array2<i32>) -> Self {
        let n = ranks.dim().0;
        let mut rank = Rank {
            means: vec![0.0; n],
            ranks,
        };
        rank.calc_expected_means();
        rank
    }

    fn cost_function(&self, means: &[f64]) -> f64 {
        let (n, _) = self.ranks.dim();
        let mut cost = 0.0;
        let normal = Normal::new(0.0, 1.0).unwrap();

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                };
                if self.ranks[[i, j]] > 0 {
                    cost += (self.ranks[[i, j]] as f64) * normal.cdf(means[i] - means[j]).log2();
                }
            }
        }

        for i in 0..n {
            cost -= means[i] * means[i] / 2.0;
        }

        -cost
    }

    pub fn calc_expected_means(&mut self) {
        let result = minimize(
            |means| self.cost_function(&means.to_vec()),
            &self.means,
            Method::BFGS,
            None,
        )
        .unwrap();

        self.means = result.x.to_vec();
    }

    pub fn update_ranks(&mut self, ranks: Array2<i32>) {
        self.ranks = ranks;
        let n = self.ranks.dim().0;
        if self.means.len() != n {
            self.means = vec![0.0; n];
        }
        self.calc_expected_means();
    }

    pub fn rank(&self) -> Result<Vec<usize>, Box<dyn Error>> {
        let mut indices: Vec<usize> = (0..self.means.len()).collect();
        indices.sort_by(|&i, &j| self.means[j].partial_cmp(&self.means[i]).unwrap());
        Ok(indices)
    }

    pub fn get_means(&self) -> &[f64] {
        &self.means
    }
}
