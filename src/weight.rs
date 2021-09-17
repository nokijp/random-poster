use serde::{Deserialize};

#[derive(PartialEq, Clone, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WeightType {
    Uniform,
    MinOnly,
    Linear { bias: f64 },
    Boltzmann { beta: f64 },
}

impl WeightType {
    pub fn get_weights(&self, counts: &Vec<u32>) -> Vec<f64> {
        match self {
            &WeightType::Uniform => vec![1.0; counts.len()],
            &WeightType::MinOnly => {
                let min_count = counts.iter().min().unwrap();
                counts.iter().map(|count| if count == min_count { 1.0 } else { 0.0 }).collect()
            },
            &WeightType::Linear { bias } => {
                let max_count = counts.iter().max().unwrap();
                counts.iter().map(|count| (max_count - *count) as f64 + bias).collect()
            },
            &WeightType::Boltzmann { beta } => {
                let min_count = counts.iter().min().unwrap();
                counts.iter().map(|count| (- beta * (count - min_count) as f64).exp()).collect()
            },
        }
    }

    pub fn validate(&self) -> Result<(), &str> {
        match self {
            &WeightType::Uniform => Ok(()),
            &WeightType::MinOnly => Ok(()),
            &WeightType::Linear { bias } => if bias.is_nan() || bias < 0.0 { Err("bias must be positive") } else { Ok(()) }
            &WeightType::Boltzmann { beta } => if beta.is_nan() { Err("beta must not be NaN") } else { Ok(()) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_weights_should_return_uniform_weights() {
        let weights = WeightType::Uniform.get_weights(&vec![2, 1, 3, 4]);
        assert_eq!(weights, vec![1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn get_weights_should_return_min_only_weights() {
        let weights = WeightType::MinOnly.get_weights(&vec![2, 1, 3, 4]);
        assert_eq!(weights, vec![0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn get_weights_should_return_min_only_weights_if_all_the_values_are_the_same() {
        let weights = WeightType::MinOnly.get_weights(&vec![0, 0, 0, 0]);
        assert_eq!(weights, vec![1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn get_weights_should_return_linear_weights() {
        let weights = WeightType::Linear { bias: 0.25 }.get_weights(&vec![2, 1, 3, 4]);
        assert_eq!(weights, vec![2.25, 3.25, 1.25, 0.25]);
    }

    #[test]
    fn get_weights_should_return_boltzmann_weights() {
        let weights = WeightType::Boltzmann { beta: 0.25 }.get_weights(&vec![0, 2, 1, 3, 4]);
        assert_eq!(weights, vec![1.0, (-0.5_f64).exp(), (-0.25_f64).exp(), (-0.75_f64).exp(), (-1.0_f64).exp()]);
    }
}
