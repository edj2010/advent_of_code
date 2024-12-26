use std::cmp::{Eq, PartialEq, PartialOrd};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::fmt::Debug;
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

// Helper Return Structs
pub struct ShortestPathPrecedentMapWithCount<
    Key: Clone + Eq + Hash + Debug,
    Cost: Clone + Add<Cost, Output = Cost> + Ord + Debug,
>(HashMap<Key, (Cost, HashMap<Option<Key>, u64>)>);

impl<Key: Clone + Eq + Hash + Debug, Cost: Clone + Debug + Add<Cost, Output = Cost> + Ord>
    ShortestPathPrecedentMapWithCount<Key, Cost>
{
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn maybe_add_paths(&mut self, from: &Option<Key>, to: &Key, cost: &Cost, count: u64) {
        self.0
            .entry(to.clone())
            .and_modify(|(min_cost, from_map)| {
                if min_cost == cost {
                    from_map
                        .entry(from.clone())
                        .and_modify(|n| *n += count)
                        .or_insert(count);
                }
            })
            .or_insert((cost.clone(), HashMap::from([(from.clone(), count)])));
    }

    fn contains(&self, key: &Key) -> bool {
        self.0.contains_key(key)
    }

    pub fn shortest_cost(&self, key: &Key) -> Option<&Cost> {
        self.0.get(key).map(|(cost, _)| cost)
    }

    pub fn shortest_route_count(&self, to: &Key) -> Option<u64> {
        Some(self.0.get(to)?.1.values().sum())
    }

    pub fn into_route_counts(self) -> HashMap<Key, u64> {
        self.0
            .into_iter()
            .map(|(key, (_, counts))| (key, counts.values().sum()))
            .collect()
    }

    pub fn into_shortest_costs(self) -> HashMap<Key, Cost> {
        self.0
            .into_iter()
            .map(|(key, (cost, _))| (key, cost))
            .collect()
    }

    pub fn shortest_path(&self, to: &Key) -> Vec<Key> {
        let mut path = vec![to];
        let mut current = to;
        while let Some(Some(prev)) = self
            .0
            .get(current)
            .and_then(|(_, precedents)| precedents.keys().next())
        {
            path.push(prev);
            current = prev;
        }
        path.into_iter().rev().cloned().collect()
    }

    pub fn all_precedents(&self, to: &Key) -> HashSet<Key> {
        let mut to_search: VecDeque<Key> = VecDeque::from([to.clone()]);
        let mut seen: HashSet<Key> = HashSet::new();
        while let Some(next) = to_search.pop_front() {
            if seen.contains(&next) {
                continue;
            }
            seen.insert(next.clone());
            if let Some((_, precedents)) = self.0.get(&next) {
                precedents
                    .keys()
                    .filter_map(|k| k.clone())
                    .for_each(|key| to_search.push_back(key));
            }
        }
        seen
    }
}

pub struct ShortestPathPrecedentMap<
    Key: Clone + Eq + Hash + Debug,
    Cost: Clone + Add<Cost, Output = Cost> + Ord + Debug,
>(HashMap<Key, (Cost, HashSet<Key>)>);

impl<Key: Clone + Eq + Hash + Debug, Cost: Clone + Debug + Add<Cost, Output = Cost> + Ord>
    ShortestPathPrecedentMap<Key, Cost>
{
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn maybe_add_paths(&mut self, from: &Option<Key>, to: &Key, cost: &Cost) {
        if let Some(from) = from {
            self.0
                .entry(to.clone())
                .and_modify(|(min_cost, from_set)| {
                    if min_cost == cost {
                        from_set.insert(from.clone());
                    }
                })
                .or_insert((cost.clone(), HashSet::from([from.clone()])));
        }
    }

    fn contains(&self, key: &Key) -> bool {
        self.0.contains_key(key)
    }

    pub fn shortest_cost(&self, key: &Key) -> Option<&Cost> {
        self.0.get(key).map(|(cost, _)| cost)
    }

    pub fn into_shortest_costs(self) -> HashMap<Key, Cost> {
        self.0
            .into_iter()
            .map(|(key, (cost, _))| (key, cost))
            .collect()
    }

    pub fn shortest_path(&self, to: &Key) -> Vec<Key> {
        let mut path = vec![to];
        let mut current = to;
        while let Some(prev) = self
            .0
            .get(current)
            .and_then(|(_, precedents)| precedents.iter().next())
        {
            path.push(prev);
            current = prev;
        }
        path.into_iter().rev().cloned().collect()
    }

    pub fn all_precedents(&self, to: &Key) -> HashSet<Key> {
        let mut to_search: VecDeque<Key> = VecDeque::from([to.clone()]);
        let mut seen: HashSet<Key> = HashSet::new();
        while let Some(next) = to_search.pop_front() {
            if seen.contains(&next) {
                continue;
            }
            seen.insert(next.clone());
            if let Some((_, precedents)) = self.0.get(&next) {
                precedents
                    .iter()
                    .for_each(|key| to_search.push_back(key.clone()));
            }
        }
        seen
    }
}

pub trait WeightedGraphWithHeuristic {
    type Key: Clone + Eq + Hash + Debug;
    type Cost: Clone + Add<Self::Cost, Output = Self::Cost> + Ord + Debug;
    type Weight: Ord;

    fn cost(&self, a: &Self::Key, b: &Self::Key) -> Option<Self::Cost>;
    fn adjacent(&self, k: &Self::Key) -> Option<impl Iterator<Item = Self::Key>>;
    fn cost_to_weight(&self, k: &Self::Key, c: Self::Cost) -> Self::Weight;

    fn shortest_paths_to_many_with_count<
        F: Fn(
            &Self::Key,
            &Self::Cost,
            &ShortestPathPrecedentMapWithCount<Self::Key, Self::Cost>,
        ) -> bool,
    >(
        &self,
        start: Self::Key,
        early_finish: F,
        zero_distance: Self::Cost,
    ) -> (
        ShortestPathPrecedentMapWithCount<Self::Key, Self::Cost>,
        Option<Self::Key>,
    ) {
        let mut to_search: BinaryHeap<
            ReverseWeightedKey<(Self::Key, Option<Self::Key>, Self::Cost), Self::Weight>,
        > = BinaryHeap::from([ReverseWeightedKey::new(
            (start.clone(), None, zero_distance.clone()),
            self.cost_to_weight(&start, zero_distance),
        )]);
        let mut results: ShortestPathPrecedentMapWithCount<Self::Key, Self::Cost> =
            ShortestPathPrecedentMapWithCount::new();
        while let Some(ReverseWeightedKey {
            key: (key, from, cost),
            weight: _,
        }) = to_search.pop()
        {
            let route_count = from
                .clone()
                .and_then(|from| results.shortest_route_count(&from))
                .unwrap_or(1);
            if !results.contains(&key) {
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
            }
            results.maybe_add_paths(&from, &key, &cost, route_count);
            if early_finish(&key, &cost, &results) {
                return (results, Some(key));
            }
        }
        (results, None)
    }

    fn shortest_paths_to_many<
        F: Fn(&Self::Key, &Self::Cost, &ShortestPathPrecedentMap<Self::Key, Self::Cost>) -> bool,
    >(
        &self,
        start: Self::Key,
        early_finish: F,
        zero_distance: Self::Cost,
    ) -> (
        ShortestPathPrecedentMap<Self::Key, Self::Cost>,
        Option<Self::Key>,
    ) {
        let mut to_search: BinaryHeap<
            ReverseWeightedKey<(Self::Key, Option<Self::Key>, Self::Cost), Self::Weight>,
        > = BinaryHeap::from([ReverseWeightedKey::new(
            (start.clone(), None, zero_distance.clone()),
            self.cost_to_weight(&start, zero_distance),
        )]);
        let mut results: ShortestPathPrecedentMap<Self::Key, Self::Cost> =
            ShortestPathPrecedentMap::new();
        while let Some(ReverseWeightedKey {
            key: (key, from, cost),
            weight: _,
        }) = to_search.pop()
        {
            if !results.contains(&key) {
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
            }
            results.maybe_add_paths(&from, &key, &cost);
            if early_finish(&key, &cost, &results) {
                return (results, Some(key));
            }
        }
        (results, None)
    }

    fn shortest_path_with_condition<
        F: Fn(&Self::Key, &Self::Cost, &ShortestPathPrecedentMap<Self::Key, Self::Cost>) -> bool,
    >(
        &self,
        start: Self::Key,
        finished: F,
        zero_distance: Self::Cost,
    ) -> Option<(Self::Key, Vec<Self::Key>)> {
        if let (results, Some(key)) = self.shortest_paths_to_many(start, finished, zero_distance) {
            let path = results.shortest_path(&key);
            Some((key, path))
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
        self.shortest_paths_to_many_with_count(
            start,
            |key, cost, results| {
                key == &end
                    && results
                        .shortest_cost(&key)
                        .map(|c| c < cost)
                        .unwrap_or(false)
            },
            zero_distance,
        )
        .0
        .shortest_route_count(&end)
    }

    fn shortest_path_count_all(
        &self,
        start: Self::Key,
        zero_distance: Self::Cost,
    ) -> HashMap<Self::Key, u64> {
        self.shortest_paths_to_many_with_count(start, |_, _, _| false, zero_distance)
            .0
            .into_route_counts()
    }

    fn shortest_distance_with_condition<
        F: Fn(&Self::Key, &Self::Cost, &ShortestPathPrecedentMap<Self::Key, Self::Cost>) -> bool,
    >(
        &self,
        start: Self::Key,
        early_finish: F,
        zero_distance: Self::Cost,
    ) -> Option<(Self::Key, Self::Cost)> {
        let (precedent_map, key) = self.shortest_paths_to_many(start, early_finish, zero_distance);
        Some((key.clone()?, precedent_map.shortest_cost(&(key?))?.clone()))
    }

    fn shortest_distance_to_all(
        &self,
        start: Self::Key,
        zero_distance: Self::Cost,
    ) -> HashMap<Self::Key, Self::Cost> {
        self.shortest_paths_to_many(start, |_, _, _| false, zero_distance)
            .0
            .into_shortest_costs()
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
    type Key: Clone + Eq + Hash + Debug;
    type Cost: Clone + Add<Self::Cost, Output = Self::Cost> + Ord + Debug;

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

impl<K: Clone + Eq + Hash + Debug, C: Clone + Debug + Add<C, Output = C> + Ord> WeightedGraph
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
