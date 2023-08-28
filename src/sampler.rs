use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fmt::Debug;
use std::hash::Hash;
// use std::sync::{Rc, Weak};
use std::rc::{Rc, Weak};
use rand::distributions::{Standard};
use rand::prelude::*;
use serde::{Serialize, Deserialize};

// enum Floatey {
//     Standard(f32),
//     Double(f64),
// }
//
// impl Into<f64> for Floatey {
//     fn into(self) -> f64 {
//         match self {
//             Floatey::Standard(x) => x.into(),
//             Floatey::Double(y) => y
//         }
//     }
// }

pub trait  DistributionKey: Copy + Eq + Hash + Debug + Default {}
impl<T: Copy + Eq + Hash + Debug + Default> DistributionKey for T {}

// pub trait Sample<K: DistributionKey> {
//     fn sample(&self) -> Option<K>;
// }

// impl<K: DistributionKey, T: Distribution<K>, R: Rng> Sample<K> for T {
//     fn sample(&self) -> K {
//         let mut rng: R = R::new();
//         Distribution::sample(&self, &mut rng)
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultinomialDistribution<K: DistributionKey> {
    weights: HashMap<Rc<K>, f32>,
    keys: Vec<Weak<K>>
}

impl<K: DistributionKey> From<HashMap<K, f32>> for MultinomialDistribution<K> {
    fn from(value: HashMap<K, f32>) -> Self {
        let mut weightmap = HashMap::with_capacity(value.len());
        let mut weightkeys = Vec::with_capacity(value.len());
        for (key, val) in value {
            let key_ref = Rc::new(key);
            weightmap.insert(key_ref.to_owned(), val);
            weightkeys.push(Rc::downgrade(&key_ref));
        }
        Self {
            weights: weightmap,
            keys: weightkeys
        }
    }
}

impl<K: DistributionKey> From<HashMap<Rc<K>, f32>> for MultinomialDistribution<K> {
    fn from(value: HashMap<Rc<K>, f32>) -> Self {
        let mut weightmap = HashMap::with_capacity(value.len());
        let mut weightkeys = Vec::with_capacity(value.len());
        for (key_ref, val) in value {
            weightmap.insert(key_ref.to_owned(), val);
            weightkeys.push(Rc::downgrade(&key_ref));
        }
        Self {
            weights: weightmap,
            keys: weightkeys
        }
    }
}

impl<K: DistributionKey + Copy> MultinomialDistribution<K> {
    pub fn total_weights(&self) -> f32 {
        let mut total = 0.0;
        for weight in self.weights.values() {
            total += weight;
        }
        total
    }

    pub fn uniform_over<I: IntoIterator<Item=K>>(keys: I) -> Self {
        let iterator = keys.into_iter();
        let size_estimate = iterator.size_hint().1.unwrap_or( iterator.size_hint().0);
        let mut weightmap: HashMap<K, f32> = HashMap::with_capacity(size_estimate);
        for key in iterator {
            weightmap.insert(key.to_owned(), 1.);
        }
        Self::from(weightmap)
    }

    pub fn normalized_weights(&self) -> HashMap<Rc<K>, f32> {
        let total = self.total_weights();
        let mut normalized_map = HashMap::with_capacity(self.weights.len());

        for (k, v) in self.weights.iter() {
            let normalized_v = v / total;
            normalized_map.insert(k.to_owned(), normalized_v);
        }

        normalized_map
    }

    pub fn entropy(&self) -> f32 {
        self.normalized_weights().values().map(
            |weight| weight * weight.log2()
        ).sum()
    }

    pub fn joint_probability_weights<BMD: Borrow<Self>>(&self, other: BMD) -> HashMap<Rc<K>, f32> {
        let normalized_other = other.borrow().normalized_weights();
        let my_weights = &self.weights;

        let mut union_keys = HashSet::with_capacity(my_weights.len() + normalized_other.len());
        for key in my_weights.keys() {
            union_keys.insert(key);
        };

        for key in normalized_other.keys() {
            union_keys.insert(key);
        };

        let union_keys = union_keys;

        let mut probability_map = HashMap::with_capacity(union_keys.len());

        for key in union_keys {
            let my_weight = my_weights.get(key).unwrap_or(&0.);
            let other_weight = normalized_other.get(key).unwrap_or(&0.);
            let new_weight = my_weight * other_weight;
            if new_weight > 0. {
                probability_map.insert(key.to_owned(), new_weight);
            }

        };

        probability_map
    }

    // pub fn joint_probability_weights_cached(
    //     &self,
    //     other: &Self,
    //     cache: HashMap<(&ArrayVec<(Rc<K>, f32), 100>, &ArrayVec<(Rc<K>, f32), 100>), ArrayVec<(Rc<K>, f32), 100>>
    // ) -> HashMap<Rc<K>, f32> {
    //     let my_weights_cachekey: ArrayVec<(Rc<K>, f32), 100> = self.weights.iter().collect();
    //     let other_weights = other.normalized_weights();
    //     let other_weights_cachekey: ArrayVec<(Rc<K>, f32), 100> = other_weights.iter().collect();
    //     let lookup = cache.get(&(my_weights_cachekey, other_weights_cachekey))
    //
    //     self.joint_probability_weights(other)
    // }

    pub fn joint_probability<BMD: Borrow<Self>>(&self, other: BMD) -> MultinomialDistribution<K> {
        MultinomialDistribution::from(self.joint_probability_weights(other))
    }
}

impl<K: DistributionKey> rand::distributions::Distribution<K> for MultinomialDistribution<K> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> K {
        let weights = &self.total_weights();
        let rope_len: f32 = rng.sample::<f32, _>(Standard) * weights;
        let mut curr_rope_len = rope_len;
        let curr_candidate_ref: Option<&Weak<K>> = self.keys.get(0);
        let mut curr_candidate = curr_candidate_ref.map(|x| x.upgrade().unwrap());

        while curr_rope_len > 0. {
            for (key, weight) in self.weights.iter() {
                if curr_rope_len <= *weight {
                    let owned_key = key.to_owned();
                    curr_candidate = Some(owned_key);
                    curr_rope_len -= *weight;
                    break
                }
                curr_rope_len -= *weight
            }
        }

        // match curr_candidate {
        //     Some(good_candidate) => *good_candidate,
        //     None => MaybeDistributionKey::None
        // }
        *curr_candidate.unwrap_or(K::default().into())
    }
}

impl<K: DistributionKey> MultinomialDistribution<K> {
    pub fn sample_with_default_rng(&self) -> K {
        let mut rng = thread_rng();
        self.sample(&mut rng)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_weights_work() {
        let keys = vec!["a", "b"];
        let dist = MultinomialDistribution::uniform_over(keys);
        assert_eq!(dist.total_weights(), 2.);
        let key_one = dist.keys.get(0).unwrap().upgrade().unwrap();
        assert!(key_one.contains("a"));
        //assert_eq!(dist.keys.get(0).unwrap().upgrade().unwrap(), keys.get(0).unwrap());
        assert_eq!(dist.weights.get(&"a").unwrap(), &1.);
        assert_eq!(dist.weights.get(&"b").unwrap(), &1.);
        assert_eq!(dist.weights.get(&"c").unwrap_or(&-666.), &-666.);
        assert_eq!(dist.weights.len(), 2);
    }

    #[test]
    fn sampling_works() {
        let dist = MultinomialDistribution::uniform_over(vec![1, 2]);
        let mut rng = thread_rng();
        let sample = dist.sample(&mut rng);
        assert!(sample > 0);
        assert!(sample < 3);
    }

    #[test]
    fn sampling_with_default_works() {
        let dist = MultinomialDistribution::uniform_over(vec![1, 2]);
        let sample = dist.sample_with_default_rng();
        assert!(sample > 0);
        assert!(sample < 3);
    }
}
