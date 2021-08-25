use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct RandomPicker<T> {
    items: Vec<RandomPickerItem<T>>,
    path: PathBuf,
    weight_bias: f64,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
struct RandomPickerItem<T> {
    value: T,
    count: u32,
}

impl<T: Hash + Eq + Serialize + DeserializeOwned> RandomPicker<T> {
    pub fn from_log_file<P: AsRef<Path>>(path: P, values: Vec<T>, weight_bias: f64) -> Result<RandomPicker<T>, String> {
        if values.is_empty() {
            return Err(String::from("values is empty"));
        }
        if weight_bias.is_nan() || weight_bias < 0.0 {
            return Err(String::from("weight_bias must be positive"));
        }

        let path_buf = path.as_ref().to_owned();
        if !path_buf.exists() {
            let items = values.into_iter().map(|value| RandomPickerItem { value, count: 0 }).collect();
            return Ok(RandomPicker {
                items,
                path: path_buf,
                weight_bias,
            });
        }

        let mut file = File::open(&path_buf).map_err(|_| format!("could not open file: {}", path_buf.display()))?;
        let mut file_reader = BufReader::new(&mut file);

        let log: Vec<RandomPickerItem<T>> = serde_json::from_reader(&mut file_reader).map_err(|e| format!("failed to read log: {}", e))?;
        let log_map: HashMap<T, u32> = log.into_iter().map(|item| (item.value, item.count)).collect();
        let value_into_item = |value| {
            let count = log_map.get(&value).map_or(0, |v| v.to_owned());
            RandomPickerItem {
                value,
                count,
            }
        };
        let items = values.into_iter().map(value_into_item).collect();

        Ok(RandomPicker {
            items,
            path: path_buf,
            weight_bias,
        })
    }

    pub fn write_log(&self) -> Result<(), String> {
        let mut file = File::create(&self.path).map_err(|_| format!("could not open file: {}", self.path.display()))?;
        let file_writer = BufWriter::new(&mut file);

        serde_json::to_writer(file_writer, &self.items).map_err(|e| format!("failed to write log: {}", e))
    }

    pub fn pick(&mut self) -> &T {
        let counts: Vec<u32> = self.items.iter().map(|item| item.count).collect();
        let max_count = counts.iter().max().unwrap();
        let raw_weights: Vec<f64> = counts.iter().map(|count| (max_count - *count) as f64 + self.weight_bias).collect();
        let weights = if raw_weights.iter().any(|w| w.is_infinite()) {
            raw_weights.iter().map(|w| if w.is_infinite() { 1.0 } else { 0.0 }).collect()
        } else if raw_weights.iter().all(|w| *w == 0.0) {
            vec![1.0; raw_weights.len()]
        } else {
            raw_weights
        };

        let weighted_index = WeightedIndex::new(weights).unwrap();
        let mut rng = ThreadRng::default();
        let picked_index = weighted_index.sample(&mut rng);

        let item = self.items.get_mut(picked_index).unwrap();
        item.count += 1;

        &item.value
    }
}

#[cfg(test)]
mod tests {
    extern crate indoc;
    extern crate tempfile;

    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn from_log_file_should_read_the_log_file() {
        let mut file = NamedTempFile::new().unwrap();
        let log = indoc! {r#"
            [
                { "value": "b", "count": 10 },
                { "value": "c", "count": 2 },
                { "value": "d", "count": 5 }
            ]
        "#};
        write!(file, "{}", log).unwrap();

        let expected = vec![
            RandomPickerItem {
                value: String::from("a"),
                count: 0,
            },
            RandomPickerItem {
                value: String::from("b"),
                count: 10,
            },
            RandomPickerItem {
                value: String::from("c"),
                count: 2,
            },
        ];

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let picker = RandomPicker::from_log_file(file.path(), values, 10.0).unwrap();
        assert_eq!(picker.items, expected);
    }

    #[test]
    fn from_log_file_should_return_a_random_picker_which_has_zero_initialized_items_if_the_log_file_does_not_exist() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        file.close().unwrap();

        let expected = vec![
            RandomPickerItem {
                value: String::from("a"),
                count: 0,
            },
            RandomPickerItem {
                value: String::from("b"),
                count: 0,
            },
            RandomPickerItem {
                value: String::from("c"),
                count: 0,
            },
        ];

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let picker = RandomPicker::from_log_file(path, values, 10.0).unwrap();
        assert_eq!(picker.items, expected);
    }

    #[test]
    fn from_log_file_should_fail_if_the_values_is_empty() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        file.close().unwrap();

        let result = RandomPicker::from_log_file(path, Vec::<String>::new(), 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn pick_should_pick_the_value_randomly() {
        let mut file = NamedTempFile::new().unwrap();
        let log = indoc! {r#"
            [
                { "value": "a", "count": 30 },
                { "value": "b", "count": 0 },
                { "value": "c", "count": 20 }
            ]
        "#};
        write!(file, "{}", log).unwrap();

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let picker_template = RandomPicker::from_log_file(file.path(), values.clone(), 20.0).unwrap();

        let mut count: HashMap<String, u64> = values.into_iter().map(|s| (s, 0)).collect();
        for _ in 1..=10000 {
            let mut picker = picker_template.clone();
            let value = picker.pick();
            *count.get_mut(value).unwrap() += 1;
        }
        // P(count["a"] > 1829) = P(X > 1829) for X ~ N(2000, 1600) > 0.99999
        assert!(count["a"] > 1829);
        // P(count["b"] > 4786) = P(X > 4786) for X ~ N(5000, 2500) > 0.99999
        assert!(count["b"] > 4786);
        // P(count["c"] > 2804) = P(X > 2804) for X ~ N(3000, 2100) > 0.99999
        assert!(count["c"] > 2804);
    }

    #[test]
    fn pick_should_pick_the_most_weighted_value() {
        let mut file = NamedTempFile::new().unwrap();
        let log = indoc! {r#"
            [
                { "value": "a", "count": 10 },
                { "value": "b", "count": 0 },
                { "value": "c", "count": 10 }
            ]
        "#};
        write!(file, "{}", log).unwrap();

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let mut picker = RandomPicker::from_log_file(file.path(), values, 0.0).unwrap();

        for _ in 1..=10 {
            let value = picker.pick();
            assert_eq!(value, "b");
        }
    }

    #[test]
    fn pick_should_not_fail_even_if_all_weights_are_zero() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        file.close().unwrap();

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let mut picker = RandomPicker::from_log_file(path, values.clone(), 0.0).unwrap();
        let value = picker.pick();
        assert!(values.iter().any(|s| s == value));
    }

    #[test]
    fn pick_should_pick_the_value_randomly_with_equal_probability_if_the_bias_is_infinity() {
        let mut file = NamedTempFile::new().unwrap();
        let log = indoc! {r#"
            [
                { "value": "a", "count": 1000000000 },
                { "value": "b", "count": 0 },
                { "value": "c", "count": 0 }
            ]
        "#};
        write!(file, "{}", log).unwrap();

        let values = vec![String::from("a"), String::from("b"), String::from("c")];
        let picker_template = RandomPicker::from_log_file(file.path(), values.clone(), f64::INFINITY).unwrap();

        let mut count: HashMap<String, u64> = values.into_iter().map(|s| (s, 0)).collect();
        for _ in 1..=10000 {
            let mut picker = picker_template.clone();
            let value = picker.pick();
            *count.get_mut(value).unwrap() += 1;
        }
        // P(count[*] > 3132) = P(X > 3132) for X ~ N(10000/3, 20000/9) > 0.99999
        assert!(count["a"] > 3132);
        assert!(count["b"] > 3132);
        assert!(count["c"] > 3132);
    }
}
