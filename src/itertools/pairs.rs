use std::iter::Peekable;

pub struct Pairs<I: Iterator + Clone> {
    first_iter: Peekable<I>,
    second_iter: Peekable<I>,
}

impl<I: Iterator + Clone> Pairs<I>
where
    <I as Iterator>::Item: Clone,
{
    pub fn new(mut iter: I) -> Self {
        let first_iter = iter.clone().peekable();
        iter.next();
        let second_iter = iter.peekable();

        Pairs {
            first_iter,
            second_iter,
        }
    }
}

impl<I: Iterator + Clone> Iterator for Pairs<I>
where
    <I as Iterator>::Item: Clone,
{
    type Item = (<I as Iterator>::Item, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let mut first = self.first_iter.peek().cloned()?;
        if let Some(second) = self.second_iter.next() {
            return Some((first, second));
        }
        self.first_iter.next()?;
        first = self.first_iter.peek().cloned()?;
        self.second_iter = self.first_iter.clone();
        self.second_iter.next()?;
        let second = self.second_iter.next()?;
        Some((first, second))
    }
}

pub struct PairsWithRepeats<I: Iterator + Clone> {
    first_iter: Peekable<I>,
    second_iter: Peekable<I>,
}

impl<I: Iterator + Clone> PairsWithRepeats<I>
where
    <I as Iterator>::Item: Clone,
{
    pub fn new(iter: I) -> Self {
        let first_iter = iter.clone().peekable();
        let second_iter = first_iter.clone();
        PairsWithRepeats {
            first_iter,
            second_iter,
        }
    }
}

impl<I: Iterator + Clone> Iterator for PairsWithRepeats<I>
where
    <I as Iterator>::Item: Clone,
{
    type Item = (<I as Iterator>::Item, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let mut first = self.first_iter.peek().cloned()?;
        if let Some(second) = self.second_iter.next() {
            return Some((first, second));
        }
        self.first_iter.next()?;
        first = self.first_iter.peek().cloned()?;
        self.second_iter = self.first_iter.clone();
        let second = self.second_iter.next()?;
        Some((first, second))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn pairs() {
        assert_eq!(
            Pairs::new([1, 2, 3, 4].into_iter()).collect::<Vec<(u32, u32)>>(),
            vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]
        );
    }

    #[test]
    fn pairs_with_repeats() {
        assert_eq!(
            PairsWithRepeats::new([1, 2, 3, 4].into_iter()).collect::<Vec<(u32, u32)>>(),
            vec![
                (1, 1),
                (1, 2),
                (1, 3),
                (1, 4),
                (2, 2),
                (2, 3),
                (2, 4),
                (3, 3),
                (3, 4),
                (4, 4)
            ]
        );
    }
}
