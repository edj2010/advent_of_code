use std::cmp::{Eq, PartialEq, PartialOrd};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;
use std::ops::Add;

#[derive(Debug)]
struct ReverseWeightedKey<K, D: Ord> {
    key: K,
    weight: D,
}

impl<K, D: Ord> ReverseWeightedKey<K, D> {
    fn new(key: K, weight: D) -> Self {
        ReverseWeightedKey { key, weight }
    }
}

impl<K, D: Ord> PartialEq for ReverseWeightedKey<K, D> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl<K, D: Ord> Eq for ReverseWeightedKey<K, D> {}

// We reverse the ordering to make the below heaps min heaps
impl<K, D: Ord> PartialOrd for ReverseWeightedKey<K, D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.weight.partial_cmp(&self.weight)
    }
}

impl<K, D: Ord> Ord for ReverseWeightedKey<K, D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.weight.cmp(&self.weight)
    }
}

/// K: Key type
/// D: Distance type
/// I: Adjacent iterator
/// Distance: Distance function
/// Adjacent: Adjacent function
/// Finished: Termination function
pub trait WeightedGraph<K: Clone + Eq + Hash, W: Clone + Add<W, Output = W> + Ord> {
    fn weight(&self, a: &K, b: &K) -> Option<W>;
    fn adjacent(&self, k: &K) -> Option<impl Iterator<Item = K>>;

    // path
    fn shortest_paths_to_many<F: Fn(&K) -> bool>(
        &self,
        start: K,
        early_finish: F,
        zero_distance: W,
    ) -> (
        HashMap<K, (W, HashSet<Vec<K>>)>,
        Option<(K, (W, HashSet<Vec<K>>))>,
    ) {
        let mut to_search: BinaryHeap<ReverseWeightedKey<(K, Option<K>), W>> =
            BinaryHeap::from([ReverseWeightedKey::new((start, None), zero_distance)]);
        let mut results: HashMap<K, (W, HashSet<Vec<K>>)> = HashMap::new();
        while let Some(ReverseWeightedKey { key, weight }) = to_search.pop() {
            let (key, from) = key;
            let paths: HashSet<Vec<K>> = from
                .and_then(|from| results.get(&from))
                .map(|(_, paths)| paths)
                .unwrap_or(&HashSet::from([vec![]]))
                .iter()
                .map(|path| {
                    let mut new_path = path.clone();
                    new_path.push(key.clone());
                    new_path
                })
                .collect();
            let (min_weight, paths) = results
                .entry(key.clone())
                .and_modify(|(w, current_paths)| {
                    if weight == *w {
                        current_paths.extend(paths.clone());
                    }
                })
                .or_insert_with(|| {
                    self.adjacent(&key).map(|i| {
                        i.filter_map(|adjacent_key| {
                            let additional_weight = self.weight(&key, &adjacent_key)?.clone();
                            Some(ReverseWeightedKey::new(
                                (adjacent_key, Some(key.clone())),
                                weight.clone() + additional_weight,
                            ))
                        })
                        .for_each(|key| to_search.push(key));
                    });
                    (weight.clone(), paths)
                })
                .clone();
            if early_finish(&key) {
                return (results, Some((key, (min_weight, paths))));
            }
        }
        (results, None)
    }

    fn shortest_path<F: Fn(&K) -> bool>(
        &self,
        start: K,
        finished: F,
        zero_distance: W,
    ) -> Option<(K, (W, Vec<K>))> {
        if let (_, Some((key, (min_weight, paths)))) =
            self.shortest_paths_to_many(start, finished, zero_distance)
        {
            Some((key, (min_weight, paths.into_iter().next()?.clone())))
        } else {
            None
        }
    }

    fn shortest_paths_to_all(
        &self,
        start: K,
        zero_distance: W,
    ) -> HashMap<K, (W, HashSet<Vec<K>>)> {
        self.shortest_paths_to_many(start, |_| false, zero_distance)
            .0
    }

    fn shortest_paths(&self, start: K, end: K, zero_distance: W) -> Option<(W, HashSet<Vec<K>>)> {
        self.shortest_paths_to_many(start, |_| false, zero_distance)
            .0
            .get(&end)
            .cloned()
    }

    // distance only
    fn shortest_distance_to_many<F: Fn(&K) -> bool>(
        &self,
        start: K,
        early_finish: F,
        zero_distance: W,
    ) -> (HashMap<K, W>, Option<(K, W)>) {
        let mut to_search: BinaryHeap<ReverseWeightedKey<K, W>> =
            BinaryHeap::from([ReverseWeightedKey::new(start, zero_distance)]);
        let mut results: HashMap<K, W> = HashMap::new();
        while let Some(ReverseWeightedKey { key, weight }) = to_search.pop() {
            results.entry(key.clone()).or_insert_with(|| {
                self.adjacent(&key).map(|i| {
                    i.filter_map(|adjacent_key| {
                        let additional_weight = self.weight(&key, &adjacent_key)?.clone();
                        Some(ReverseWeightedKey::new(
                            adjacent_key,
                            weight.clone() + additional_weight,
                        ))
                    })
                    .for_each(|key| to_search.push(key))
                });
                weight.clone()
            });
            if early_finish(&key) {
                return (results, Some((key, weight)));
            }
        }
        (results, None)
    }

    fn shortest_distance<F: Fn(&K) -> bool>(
        &self,
        start: K,
        finished: F,
        zero_distance: W,
    ) -> Option<(K, W)> {
        self.shortest_distance_to_many(start, finished, zero_distance)
            .1
    }

    fn shortest_distance_to_all(&self, start: K, zero_distance: W) -> HashMap<K, W> {
        self.shortest_distance_to_many(start, |_| false, zero_distance)
            .0
    }

    fn all_pairs_shortest_paths<I: Iterator<Item = K>>(
        &self,
        points: I,
        zero_distance: W,
    ) -> HashMap<K, HashMap<K, W>> {
        points
            .map(|p| {
                (
                    p.clone(),
                    self.shortest_distance_to_all(p, zero_distance.clone()),
                )
            })
            .collect()
    }
}

impl<K: Clone + Eq + Hash, W: Clone + Add<W, Output = W> + Ord> WeightedGraph<K, W>
    for HashMap<K, HashMap<K, W>>
{
    fn weight(&self, a: &K, b: &K) -> Option<W> {
        Some(self.get(a)?.get(b)?.clone())
    }
    fn adjacent(&self, k: &K) -> Option<impl Iterator<Item = K>> {
        Some(self.get(k)?.keys().cloned().collect::<Vec<K>>().into_iter())
    }
}

pub trait NodeValuedWeightedGraph<
    K: Clone + Eq + Hash,
    W: Clone + Add<W, Output = W> + Ord,
    V: Clone + Add<W, Output = W>,
>: WeightedGraph<K, W>
{
    fn node_value(&self, k: &K) -> V;
    fn node_cost(&self, k: &K) -> W;
}
