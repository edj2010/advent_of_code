use std::{
    cmp::Ordering,
    collections::BTreeSet,
    mem::take,
    ops::{Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntervalBound<T> {
    Inclusive(T),
    Exclusive(T),
}

impl<T> IntervalBound<T> {
    fn inclusive(&self) -> bool {
        match self {
            Self::Inclusive(_) => true,
            _ => false,
        }
    }

    fn inner(&self) -> &T {
        match self {
            Self::Inclusive(t) => t,
            Self::Exclusive(t) => t,
        }
    }

    fn invert(self) -> Self {
        match self {
            Self::Inclusive(t) => Self::Exclusive(t),
            Self::Exclusive(t) => Self::Inclusive(t),
        }
    }
}

impl<T> IntervalBound<T>
where
    T: PartialOrd,
{
    pub fn bounds_below(&self, t: &T) -> bool {
        match self {
            Self::Inclusive(s) => s <= t,
            Self::Exclusive(s) => s < t,
        }
    }

    pub fn bounds_above(&self, t: &T) -> bool {
        match self {
            Self::Inclusive(s) => s >= t,
            Self::Exclusive(s) => s > t,
        }
    }

    pub fn compare_internal(&self, other: &Self) -> Option<Ordering> {
        self.inner().partial_cmp(other.inner())
    }

    pub fn compare_as_lower_bound(&self, other: &Self) -> Option<Ordering> {
        match self.compare_internal(&other)? {
            Ordering::Equal => match (&self, &other) {
                (IntervalBound::Inclusive(_), IntervalBound::Exclusive(_)) => Some(Ordering::Less),
                (IntervalBound::Exclusive(_), IntervalBound::Inclusive(_)) => {
                    Some(Ordering::Greater)
                }
                _ => Some(Ordering::Equal),
            },
            o => Some(o),
        }
    }

    pub fn compare_as_upper_bound(&self, other: &Self) -> Option<Ordering> {
        match self.compare_internal(&other)? {
            Ordering::Equal => match (&self, &other) {
                (IntervalBound::Inclusive(_), IntervalBound::Exclusive(_)) => {
                    Some(Ordering::Greater)
                }
                (IntervalBound::Exclusive(_), IntervalBound::Inclusive(_)) => Some(Ordering::Less),
                _ => Some(Ordering::Equal),
            },
            o => Some(o),
        }
    }
}

impl<T> Add<T> for IntervalBound<T>
where
    T: Add<T, Output = T>,
{
    type Output = IntervalBound<T>;

    fn add(self, rhs: T) -> Self::Output {
        match self {
            Self::Inclusive(t) => Self::Inclusive(t + rhs),
            Self::Exclusive(t) => Self::Exclusive(t + rhs),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Interval<T> {
    begin: IntervalBound<T>,
    end: IntervalBound<T>,
}

impl<T> Interval<T> {
    pub fn new(begin: IntervalBound<T>, end: IntervalBound<T>) -> Self {
        Interval { begin, end }
    }

    pub fn lower_bound(&self) -> &T {
        self.begin.inner()
    }

    pub fn upper_bound(&self) -> &T {
        self.end.inner()
    }
}

impl<T> Interval<T>
where
    T: PartialOrd,
{
    pub fn contains(&self, t: &T) -> bool {
        self.begin.bounds_below(t) && self.end.bounds_above(t)
    }

    pub fn non_empty(&self) -> bool {
        match self.begin.compare_internal(&self.end) {
            Some(Ordering::Less) => true,
            Some(Ordering::Equal) => self.begin.inclusive() && self.end.inclusive(),
            _ => false,
        }
    }

    pub fn intersection(&self, other: &Self) -> Option<Self>
    where
        T: Clone,
    {
        let begin = match self.begin.compare_as_lower_bound(&other.begin)? {
            Ordering::Less => other.begin.clone(),
            _ => self.begin.clone(),
        };
        let end = match self.end.compare_internal(&other.end)? {
            Ordering::Greater => other.end.clone(),
            _ => self.end.clone(),
        };
        match begin.compare_internal(&end)? {
            Ordering::Less => Some(Interval { begin, end }),
            Ordering::Greater => None,
            Ordering::Equal => {
                if begin.inclusive() && end.inclusive() {
                    Some(Interval { begin, end })
                } else {
                    None
                }
            }
        }
    }

    pub fn intersects(&self, other: &Self) -> bool
    where
        T: Clone,
    {
        self.intersection(other).is_some()
    }

    pub fn difference(&self, other: &Self) -> DisjointIntervalUnion<T>
    where
        T: Clone + Ord,
    {
        if !self.intersects(other) {
            return DisjointIntervalUnion::singleton(self.clone());
        }

        DisjointIntervalUnion::from(
            [
                Interval {
                    begin: self.begin.clone(),
                    end: other.begin.clone().invert(),
                },
                Interval {
                    begin: other.end.clone().invert(),
                    end: self.end.clone(),
                },
            ]
            .into_iter()
            .filter(|i| i.non_empty()),
        )
    }
}

impl<T> Add<T> for Interval<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Interval<T>;

    fn add(self, rhs: T) -> Self::Output {
        Interval {
            begin: self.begin + rhs.clone(),
            end: self.end + rhs,
        }
    }
}

impl<T> PartialOrd for Interval<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.begin.compare_as_lower_bound(&other.begin) {
            Some(Ordering::Equal) => self.end.compare_as_upper_bound(&other.end),
            o => o,
        }
    }
}

impl<T> Ord for Interval<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointIntervalUnion<T>
where
    T: Ord,
{
    intervals: BTreeSet<Interval<T>>,
}

impl<T> DisjointIntervalUnion<T>
where
    T: Ord,
{
    pub fn empty() -> Self {
        Self {
            intervals: BTreeSet::new(),
        }
    }

    pub fn singleton(interval: Interval<T>) -> Self {
        DisjointIntervalUnion {
            intervals: BTreeSet::from([interval]),
        }
    }

    pub fn from<I: IntoIterator<Item = Interval<T>>>(i: I) -> Self
    where
        T: Clone,
    {
        let mut to_return = Self::empty();
        i.into_iter().for_each(|interval| to_return |= interval);
        to_return
    }

    pub fn lower_bound(&self) -> Option<&T> {
        self.intervals
            .first()
            .map(|interval| interval.lower_bound())
    }

    pub fn upper_bound(&self) -> Option<&T> {
        self.intervals.last().map(|interval| interval.upper_bound())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Interval<T>> {
        self.intervals.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = Interval<T>> {
        self.intervals.into_iter()
    }
}

impl<T: Ord> Default for DisjointIntervalUnion<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Ord + Clone> FromIterator<Interval<T>> for DisjointIntervalUnion<T> {
    fn from_iter<I: IntoIterator<Item = Interval<T>>>(iter: I) -> Self {
        Self::from(iter)
    }
}

impl<T: Ord + Clone> FromIterator<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    fn from_iter<I: IntoIterator<Item = DisjointIntervalUnion<T>>>(iter: I) -> Self {
        Self::from(iter.into_iter().flat_map(|i| i.into_iter()))
    }
}

impl<T> DisjointIntervalUnion<T>
where
    T: Ord + Clone,
{
    pub fn union_interval_assign(&mut self, interval: Interval<T>) {
        let begin = self
            .intervals
            .iter()
            .filter(|i| i.intersects(&interval))
            .map(|i| &i.begin)
            .fold(interval.begin.clone(), |bound, new_bound| {
                if new_bound.compare_as_lower_bound(&bound) == Some(Ordering::Less) {
                    new_bound.clone()
                } else {
                    bound
                }
            });
        let end = self
            .intervals
            .iter()
            .filter(|i| i.intersects(&interval))
            .map(|i| &i.end)
            .fold(interval.end.clone(), |bound, new_bound| {
                if new_bound.compare_as_upper_bound(&bound) == Some(Ordering::Greater) {
                    new_bound.clone()
                } else {
                    bound
                }
            });
        self.intervals.retain(|i| !i.intersects(&interval));
        self.intervals.insert(Interval::new(begin, end));
    }

    pub fn union_interval(&self, interval: Interval<T>) -> Self {
        let mut to_return = self.clone();
        to_return.union_interval_assign(interval);
        to_return
    }

    pub fn intersect_interval_assign(&mut self, interval: Interval<T>) {
        self.intervals = take(&mut self.intervals)
            .into_iter()
            .filter_map(|i| i.intersection(&interval))
            .collect()
    }

    pub fn intersect_interval(&self, interval: Interval<T>) -> Self {
        let mut to_return = self.clone();
        to_return.intersect_interval_assign(interval);
        to_return
    }

    pub fn subtract_interval_assign(&mut self, interval: Interval<T>) {
        self.intervals = take(&mut self.intervals)
            .into_iter()
            .flat_map(|i| i.difference(&interval).intervals.into_iter())
            .collect()
    }

    pub fn subtract_interval(&self, interval: Interval<T>) -> Self {
        let mut to_return = self.clone();
        to_return.subtract_interval_assign(interval);
        to_return
    }
}

impl<T: Ord + Clone> BitOr<Interval<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn bitor(self, rhs: Interval<T>) -> Self::Output {
        self.union_interval(rhs)
    }
}

impl<T: Ord + Clone> BitOrAssign<Interval<T>> for DisjointIntervalUnion<T> {
    fn bitor_assign(&mut self, rhs: Interval<T>) {
        self.union_interval_assign(rhs);
    }
}

impl<T: Ord + Clone> BitAnd<Interval<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn bitand(self, rhs: Interval<T>) -> Self::Output {
        self.intersect_interval(rhs)
    }
}

impl<T: Ord + Clone> BitAndAssign<Interval<T>> for DisjointIntervalUnion<T> {
    fn bitand_assign(&mut self, rhs: Interval<T>) {
        self.intersect_interval_assign(rhs)
    }
}

impl<T: Ord + Clone> Sub<Interval<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn sub(self, rhs: Interval<T>) -> Self::Output {
        self.subtract_interval(rhs)
    }
}

impl<T: Ord + Clone> SubAssign<Interval<T>> for DisjointIntervalUnion<T> {
    fn sub_assign(&mut self, rhs: Interval<T>) {
        self.subtract_interval_assign(rhs)
    }
}

impl<T: Ord + Clone> BitOr<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn bitor(self, rhs: DisjointIntervalUnion<T>) -> Self::Output {
        rhs.intervals
            .into_iter()
            .fold(self, |interval_union, interval| {
                interval_union.union_interval(interval)
            })
    }
}

impl<T: Ord + Clone> BitOrAssign<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    fn bitor_assign(&mut self, rhs: DisjointIntervalUnion<T>) {
        rhs.intervals
            .into_iter()
            .for_each(|interval| self.union_interval_assign(interval))
    }
}

impl<T: Ord + Clone> BitAnd<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn bitand(self, rhs: DisjointIntervalUnion<T>) -> Self::Output {
        rhs.intervals.into_iter().fold(
            DisjointIntervalUnion::empty(),
            |interval_union, interval| interval_union | (self.intersect_interval(interval)),
        )
    }
}

impl<T: Ord + Clone> BitAndAssign<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    fn bitand_assign(&mut self, rhs: DisjointIntervalUnion<T>) {
        *self = take(self) & rhs
    }
}

impl<T: Ord + Clone> Sub<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    type Output = DisjointIntervalUnion<T>;

    fn sub(self, rhs: DisjointIntervalUnion<T>) -> Self::Output {
        rhs.intervals
            .into_iter()
            .fold(self, |interval_union, interval| {
                interval_union.subtract_interval(interval)
            })
    }
}

impl<T: Ord + Clone> SubAssign<DisjointIntervalUnion<T>> for DisjointIntervalUnion<T> {
    fn sub_assign(&mut self, rhs: DisjointIntervalUnion<T>) {
        rhs.intervals
            .into_iter()
            .for_each(|interval| self.subtract_interval_assign(interval))
    }
}
