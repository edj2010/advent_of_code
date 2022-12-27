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

/// K: Key type
/// D: Distance type
/// I: Adjacent iterator
/// Distance: Distance function
/// Adjacent: Adjacent function
/// Finished: Termination function
pub trait WeightedGraph<K: Clone + Eq + Hash, W: Clone + Add<W, Output = W> + Ord> {
    fn weight(&self, a: &K, b: &K) -> Option<W>;
    fn adjacent(&self, k: &K) -> Option<impl Iterator<Item = K>>;

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

/*
fn maximum_value_path<V, F: Fn(&K) -> V, G: Fn(&K) -> W>(
    &self,
    start: K,
    node_value: F,
    node_cost: G,
    remaining_weight: W,
) -> V {
       if time_left <= 1 {
    0
} else {
    let mut seen: HashSet<String> = HashSet::new();
    let mut to_search: VecDeque<(String, usize)> = VecDeque::from([(node, time_left)]);
    let mut best = 0;
    while let Some((front, remaining)) = to_search.pop_front() {
        seen.insert(front.clone());
        if valves.contains_key(&front) && !on.contains(&front) {
            let mut new_on = on.clone();
            new_on.insert(front.clone());
            best = max(
                best,
                explore(graph, valves, new_on, front.clone(), remaining - 1)
                    + (remaining - 1) * valves.get(&front).unwrap(),
            );
        }
        if remaining <= 2 {
            continue;
        }
        to_search.extend(graph.get(&front).unwrap().iter().filter_map(|s| {
            if !seen.contains(s) {
                Some((s.clone(), remaining - 1))
            } else {
                None
            }
        }));
    }
    best
} */
