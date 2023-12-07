use std::{cmp::Ordering, ops::Add};

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

    pub fn begin(&self) -> &T {
        self.begin.inner()
    }

    pub fn end(&self) -> &T {
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

    pub fn intersection(&self, other: &Self) -> Option<Self>
    where
        T: Clone,
    {
        let begin = match self.begin.compare_internal(&other.begin)? {
            Ordering::Less => other.begin.clone(),
            Ordering::Greater => self.begin.clone(),
            Ordering::Equal => match (&self.begin, &other.begin) {
                (IntervalBound::Exclusive(_), _) => self.begin.clone(),
                _ => other.begin.clone(),
            },
        };
        let end = match self.end.compare_internal(&other.end)? {
            Ordering::Less => self.end.clone(),
            Ordering::Greater => other.end.clone(),
            Ordering::Equal => match (&self.end, &other.end) {
                (IntervalBound::Exclusive(_), _) => self.end.clone(),
                _ => other.end.clone(),
            },
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
