use std::cmp::{Eq, PartialEq, PartialOrd};
use std::collections::{BinaryHeap, HashMap};
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

pub struct HeuristicWeight<W: Ord, C: Ord> {
    weight: W,
    cost: C,
}

impl<W: Ord, C: Ord> HeuristicWeight<W, C> {
    pub fn new(weight: W, cost: C) -> Self {
        HeuristicWeight { weight, cost }
    }
}

impl<W: Ord, C: Ord> PartialEq for HeuristicWeight<W, C> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight && self.cost == other.cost
    }
}

impl<W: Ord, C: Ord> Eq for HeuristicWeight<W, C> {}

impl<W: Ord, C: Ord> PartialOrd for HeuristicWeight<W, C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.weight.partial_cmp(&other.weight) {
            Some(std::cmp::Ordering::Equal) => self.cost.partial_cmp(&other.cost),
            cmp => cmp,
        }
    }
}

impl<W: Ord, C: Ord> Ord for HeuristicWeight<W, C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.weight.cmp(&other.weight) {
            std::cmp::Ordering::Equal => self.cost.cmp(&other.cost),
            cmp => cmp,
        }
    }
}

/// K: Key type
/// D: Distance type
/// I: Adjacent iterator
/// Distance: Distance function
/// Adjacent: Adjacent function
/// Finished: Termination function
pub trait WeightedGraphWithHeuristic {
    type Key: Clone + Eq + Hash;
    type Cost: Clone + Add<Self::Cost, Output = Self::Cost> + Ord;
    type Weight: Ord;

    fn cost(&self, a: &Self::Key, b: &Self::Key) -> Option<Self::Cost>;
    fn adjacent(&self, k: &Self::Key) -> Option<impl Iterator<Item = Self::Key>>;
    fn cost_to_weight(&self, k: &Self::Key, c: Self::Cost) -> Self::Weight;

    // paths and path count
    fn shortest_paths_to_many<
        F: Fn(
            &Self::Key,
            &Self::Cost,
            &HashMap<Self::Key, (Self::Cost, HashMap<Option<Self::Key>, u64>)>,
        ) -> bool,
    >(
        &self,
        start: Self::Key,
        early_finish: F,
        zero_distance: Self::Cost,
    ) -> (
        HashMap<Self::Key, (Self::Cost, HashMap<Option<Self::Key>, u64>)>,
        Option<(Self::Key, (Self::Cost, HashMap<Option<Self::Key>, u64>))>,
    ) {
        let mut to_search: BinaryHeap<
            ReverseWeightedKey<(Self::Key, Option<Self::Key>, Self::Cost), Self::Weight>,
        > = BinaryHeap::from([ReverseWeightedKey::new(
            (start.clone(), None, zero_distance.clone()),
            self.cost_to_weight(&start, zero_distance),
        )]);
        let mut results: HashMap<Self::Key, (Self::Cost, HashMap<Option<Self::Key>, u64>)> =
            HashMap::from([]);
        while let Some(ReverseWeightedKey { key, weight: _ }) = to_search.pop() {
            let (key, from, cost) = key;
            let route_count = from
                .clone()
                .and_then(|from| results.get(&from))
                .map(|result| result.1.values().sum())
                .unwrap_or(1);
            let (min_cost, precedent_routes) = results
                .entry(key.clone())
                .and_modify(|(c, current_paths)| {
                    if cost == *c {
                        current_paths
                            .entry(from.clone())
                            .and_modify(|n| *n += route_count)
                            .or_insert(route_count);
                    }
                })
                .or_insert_with(|| {
                    self.adjacent(&key).map(|i| {
                        i.filter_map(|adjacent_key| {
                            let total_cost = cost.clone() + self.cost(&key, &adjacent_key)?.clone();
                            Some(ReverseWeightedKey::new(
                                (adjacent_key.clone(), Some(key.clone()), total_cost.clone()),
                                self.cost_to_weight(&adjacent_key, total_cost),
                            ))
                        })
                        .for_each(|key| to_search.push(key));
                    });
                    (cost.clone(), HashMap::from([(from, route_count)]))
                })
                .clone();
            if early_finish(&key, &cost, &results) {
                return (results, Some((key, (min_cost, precedent_routes))));
            }
        }
        (results, None)
    }

    fn shortest_path<
        F: Fn(
            &Self::Key,
            &Self::Cost,
            &HashMap<Self::Key, (Self::Cost, HashMap<Option<Self::Key>, u64>)>,
        ) -> bool,
    >(
        &self,
        start: Self::Key,
        finished: F,
        zero_distance: Self::Cost,
    ) -> Option<(Self::Key, (Self::Cost, Vec<Self::Key>))> {
        if let (results, Some((key, (min_cost, _)))) =
            self.shortest_paths_to_many(start, finished, zero_distance)
        {
            let mut path: Vec<<Self as WeightedGraphWithHeuristic>::Key> = vec![key.clone()];
            let mut current = key.clone();
            while let Some(prev) = results.get(&current).and_then(|(_, precendent_map)| {
                precendent_map.keys().next().and_then(|v| v.clone())
            }) {
                path.push(prev.clone());
                current = prev;
            }
            path = path.into_iter().rev().collect();
            Some((key, (min_cost, path)))
        } else {
            None
        }
    }

    fn shortest_path_count(
        &self,
        start: Self::Key,
        end: Self::Key,
        zero_distance: Self::Cost,
    ) -> Option<u64> {
        self.shortest_paths_to_many(
            start,
            |key, cost, results| {
                results
                    .get(key)
                    .map(|(c, _)| key == &end && c > cost)
                    .unwrap_or(false)
            },
            zero_distance,
        )
        .0
        .get(&end)
        .map(|(_, precedent_map)| precedent_map.values().sum())
    }

    fn shortest_path_count_all(
        &self,
        start: Self::Key,
        zero_distance: Self::Cost,
    ) -> HashMap<Self::Key, (Self::Cost, u64)> {
        self.shortest_paths_to_many(start, |_, _, _| false, zero_distance)
            .0
            .into_iter()
            .map(|(key, (cost, value))| (key, (cost, value.values().sum())))
            .collect()
    }

    // distance only
    fn shortest_distance_to_many<F: Fn(&Self::Key) -> bool>(
        &self,
        start: Self::Key,
        early_finish: F,
        zero_distance: Self::Cost,
    ) -> (
        HashMap<Self::Key, Self::Cost>,
        Option<(Self::Key, Self::Cost)>,
    ) {
        let mut to_search: BinaryHeap<ReverseWeightedKey<(Self::Key, Self::Cost), Self::Weight>> =
            BinaryHeap::from([ReverseWeightedKey::new(
                (start.clone(), zero_distance.clone()),
                self.cost_to_weight(&start, zero_distance),
            )]);
        let mut results: HashMap<Self::Key, Self::Cost> = HashMap::new();
        while let Some(ReverseWeightedKey {
            key: (key, cost),
            weight: _,
        }) = to_search.pop()
        {
            results.entry(key.clone()).or_insert_with(|| {
                self.adjacent(&key).map(|i| {
                    i.filter_map(|adjacent_key| {
                        let total_cost = cost.clone() + self.cost(&key, &adjacent_key)?.clone();
                        Some(ReverseWeightedKey::new(
                            (adjacent_key.clone(), total_cost.clone()),
                            self.cost_to_weight(&adjacent_key, total_cost),
                        ))
                    })
                    .for_each(|key| to_search.push(key))
                });
                cost.clone()
            });
            if early_finish(&key) {
                return (results, Some((key, cost)));
            }
        }
        (results, None)
    }

    fn shortest_distance<F: Fn(&Self::Key) -> bool>(
        &self,
        start: Self::Key,
        finished: F,
        zero_distance: Self::Cost,
    ) -> Option<(Self::Key, Self::Cost)> {
        self.shortest_distance_to_many(start, finished, zero_distance)
            .1
    }

    fn shortest_distance_to_all(
        &self,
        start: Self::Key,
        zero_distance: Self::Cost,
    ) -> HashMap<Self::Key, Self::Cost> {
        self.shortest_distance_to_many(start, |_| false, zero_distance)
            .0
    }

    fn all_pairs_shortest_paths<I: Iterator<Item = Self::Key>>(
        &self,
        points: I,
        zero_distance: Self::Cost,
    ) -> HashMap<Self::Key, HashMap<Self::Key, Self::Cost>> {
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

pub trait WeightedGraph {
    type Key: Clone + Eq + Hash;
    type Cost: Clone + Add<Self::Cost, Output = Self::Cost> + Ord;

    fn cost(&self, a: &Self::Key, b: &Self::Key) -> Option<Self::Cost>;
    fn adjacent(&self, k: &Self::Key) -> Option<impl Iterator<Item = Self::Key>>;
}

impl<T> WeightedGraphWithHeuristic for T
where
    T: WeightedGraph,
{
    type Key = <Self as WeightedGraph>::Key;
    type Cost = <Self as WeightedGraph>::Cost;
    type Weight = <Self as WeightedGraph>::Cost;

    fn adjacent(&self, k: &Self::Key) -> Option<impl Iterator<Item = Self::Key>> {
        self.adjacent(k)
    }

    fn cost(&self, a: &Self::Key, b: &Self::Key) -> Option<Self::Cost> {
        self.cost(a, b)
    }

    fn cost_to_weight(&self, _k: &Self::Key, c: Self::Cost) -> Self::Weight {
        c
    }
}

impl<K: Clone + Eq + Hash, C: Clone + Add<C, Output = C> + Ord> WeightedGraph
    for HashMap<K, HashMap<K, C>>
{
    type Key = K;
    type Cost = C;

    fn cost(&self, a: &K, b: &K) -> Option<C> {
        Some(self.get(a)?.get(b)?.clone())
    }
    fn adjacent(&self, k: &K) -> Option<impl Iterator<Item = K>> {
        Some(self.get(k)?.keys().cloned().collect::<Vec<K>>().into_iter())
    }
}
