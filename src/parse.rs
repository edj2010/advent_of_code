use std::convert::Infallible;
use std::ops::{ControlFlow, FromResidual, Try};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
    ParseIntError(String),
    UnexpectedChar(char),
    UnmatchedTag(String),
    Generic(String),
    RemainingUnparsed,
    XorBothTrue,
    EndOfString,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseState<'a, T> {
    Err { error: ParseError, rest: &'a str },
    Ok { result: T, rest: &'a str },
}

///////
///
/// Error shorthand
///
///////

#[allow(dead_code)]
impl<'a, T> ParseState<'a, T> {
    #[inline]
    fn error_unexpected_char(c: char, rest: &'a str) -> Self {
        Self::Err {
            error: ParseError::UnexpectedChar(c),
            rest,
        }
    }

    #[inline]
    fn error_generic(err: &str, rest: &'a str) -> Self {
        Self::Err {
            error: ParseError::Generic(err.to_owned()),
            rest,
        }
    }

    #[inline]
    fn error_end_of_string(rest: &'a str) -> Self {
        Self::Err {
            error: ParseError::EndOfString,
            rest,
        }
    }

    #[inline]
    fn error_unmatched_tag(s: &str, rest: &'a str) -> Self {
        Self::Err {
            error: ParseError::UnmatchedTag(s.to_owned()),
            rest,
        }
    }
}

///////
///
/// Parser State internal
///
///////
impl<'a, T> ParseState<'a, T> {
    #[inline]
    fn ok(result: T, rest: &'a str) -> Self {
        ParseState::Ok { result, rest }
    }

    #[inline]
    fn error(error: ParseError, rest: &'a str) -> Self {
        ParseState::Err { error, rest }
    }

    #[inline]
    fn and_then<U, F: FnOnce(T, &'a str) -> ParseState<'a, U>>(self, f: F) -> ParseState<'a, U> {
        match self {
            ParseState::Ok { result, rest } => f(result, rest),
            ParseState::Err { error, rest } => ParseState::Err { error, rest },
        }
    }

    #[inline]
    fn and<U>(self, other: ParseState<'a, U>) -> ParseState<'a, U> {
        self.and_then(|_, _| other)
    }

    #[inline]
    fn or(self, other: Self) -> Self {
        match self {
            ParseState::Ok { .. } => self,
            _ => other,
        }
    }

    #[inline]
    fn xor(self, other: Self, s: &'a str) -> Self {
        match self {
            ParseState::Ok { .. } => match other {
                ParseState::Ok { .. } => ParseState::error(ParseError::XorBothTrue, s),
                ParseState::Err { .. } => self,
            },
            ParseState::Err { .. } => other,
        }
    }
}

impl<'a, T> FromResidual<ParseState<'a, Infallible>> for ParseState<'a, T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            ParseState::Err { error, rest } => ParseState::Err { error, rest },
        }
    }
}

impl<'a, T> Try for ParseState<'a, T> {
    type Output = (T, &'a str);
    type Residual = ParseState<'a, Infallible>;
    fn from_output(output: Self::Output) -> Self {
        let (result, rest) = output;
        ParseState::Ok { result, rest }
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            ParseState::Ok { result, rest } => ControlFlow::Continue((result, rest)),
            ParseState::Err { error, rest } => ControlFlow::Break(ParseState::Err { error, rest }),
        }
    }
}

///////
///
/// Parser State interface
///
///////

/*
   pub fn fail(err: &str) -> ParseState<'a, T> {
       Self::error_generic(err)
   }
*/

impl<'a, T> ParseState<'a, T> {
    pub fn finish(self) -> Result<T, (ParseError, &'a str)> {
        match self {
            ParseState::Err { error, rest } => Err((error, rest)),
            ParseState::Ok { result, rest: "" } => Ok(result),
            ParseState::Ok { result: _, rest } => Err((ParseError::RemainingUnparsed, rest)),
        }
    }
}

///////
///
/// Parser combinators
///
///////

mod parsers_internal {
    use std::{collections::VecDeque, marker::PhantomData, ops::Neg, str::FromStr};

    use super::{super::grid, parsers, ParseError, ParseState, Parser};

    ////////
    ///
    /// Simple
    ///
    ////////

    // Pure
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Pure;

    impl Pure {
        pub fn new() -> Self {
            Pure
        }
    }

    impl Parser for Pure {
        type Output = ();

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            ParseState::ok((), s)
        }
    }

    // Fail
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Fail<'a>(&'a str);

    impl<'a> Fail<'a> {
        pub fn new(s: &'a str) -> Self {
            Fail(s)
        }
    }

    impl<'b> Parser for Fail<'b> {
        type Output = !;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            ParseState::error_generic(&self.0, s)
        }
    }

    // Chars
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Chars<F: Fn(char) -> bool>(F);

    impl<F: Fn(char) -> bool> Chars<F> {
        pub fn new(f: F) -> Self {
            Chars(f)
        }
    }

    impl<F: Fn(char) -> bool> Parser for Chars<F> {
        type Output = char;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            let mut cs = s.chars();
            match cs.next() {
                None => ParseState::error_end_of_string(s),
                Some(c) => {
                    if (self.0)(c) {
                        ParseState::ok(c, cs.as_str())
                    } else {
                        ParseState::error_unexpected_char(c, s)
                    }
                }
            }
        }
    }

    // Char
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Char(char);

    impl Char {
        pub fn new(c: char) -> Self {
            Char(c)
        }
    }

    impl Parser for Char {
        type Output = char;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::chars(|c| c == self.0).parse(s)
        }
    }

    // CharAny
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct CharAny;

    impl CharAny {
        pub fn new() -> Self {
            CharAny
        }
    }

    impl Parser for CharAny {
        type Output = char;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::chars(|_| true).parse(s)
        }
    }

    // CharMap
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct CharMap<T, F: Fn(char) -> Option<T>>(F);

    impl<T, F: Fn(char) -> Option<T>> CharMap<T, F> {
        pub fn new(f: F) -> Self {
            CharMap(f)
        }
    }

    impl<T, F: Fn(char) -> Option<T>> Parser for CharMap<T, F> {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            let mut cs = s.chars();
            match cs.next() {
                None => ParseState::error_end_of_string(s),
                Some(c) => match (self.0)(c) {
                    None => ParseState::error_unexpected_char(c, s),
                    Some(result) => ParseState::ok(result, cs.as_str()),
                },
            }
        }
    }

    // TagReplace
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct TagReplace<'a, T>(&'a str, T);

    impl<'a, T> TagReplace<'a, T> {
        pub fn new(s: &'a str, t: T) -> Self {
            TagReplace(s, t)
        }
    }

    impl<'b, T> Parser for TagReplace<'b, T> {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            match s.strip_prefix(self.0) {
                None => ParseState::error_unmatched_tag(self.0, s),
                Some(s) => ParseState::ok(self.1, s),
            }
        }
    }

    // Tag
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Tag<'a>(&'a str);

    impl<'a> Tag<'a> {
        pub fn new(s: &'a str) -> Self {
            Tag(s)
        }
    }

    impl<'b> Parser for Tag<'b> {
        type Output = &'b str;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            match s.strip_prefix(self.0) {
                None => ParseState::error_unmatched_tag(self.0, s),
                Some(s) => ParseState::ok(self.0, s),
            }
        }
    }

    // Any
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Any;

    impl Any {
        pub fn new() -> Self {
            Any
        }
    }

    impl Parser for Any {
        type Output = String;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            ParseState::ok(s.to_owned(), "")
        }
    }

    // Drop
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Drop;

    impl Drop {
        pub fn new() -> Self {
            Drop
        }
    }

    impl Parser for Drop {
        type Output = ();

        fn parse<'a>(self, _s: &'a str) -> ParseState<'a, Self::Output> {
            ParseState::ok((), "")
        }
    }

    ////////
    ///
    /// Combinators
    ///
    ////////

    // Or
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Or<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> Or<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            Or { p, q }
        }
    }

    impl<T, P, Q> Parser for Or<P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser<Output = T>,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.parse(s).or(self.q.parse(s))
        }
    }

    // Xor
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Xor<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> Xor<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            Xor { p, q }
        }
    }

    impl<T, P, Q> Parser for Xor<P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser<Output = T>,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.parse(s).xor(self.q.parse(s), s)
        }
    }

    // And
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct And<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> And<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            And { p, q }
        }
    }

    impl<T, U, P, Q> Parser for And<P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser<Output = U>,
    {
        type Output = U;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.parse(s).and(self.q.parse(s))
        }
    }

    // Map
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Map<P, F> {
        p: P,
        f: F,
    }

    impl<T, U, P, F> Map<P, F>
    where
        P: Parser<Output = T>,
        F: Fn(T) -> U,
    {
        pub fn new(p: P, f: F) -> Self {
            Map { p, f }
        }
    }

    impl<T, U, P, F> Parser for Map<P, F>
    where
        P: Parser<Output = T>,
        F: Fn(T) -> U,
    {
        type Output = U;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p
                .parse(s)
                .and_then(|result, rest| ParseState::ok((self.f)(result), rest))
        }
    }

    // Bind
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Bind<P, F> {
        p: P,
        f: F,
    }

    impl<P, F> Bind<P, F> {
        pub fn new(p: P, f: F) -> Self {
            Bind { p, f }
        }
    }

    impl<T, U, P, F> Parser for Bind<P, F>
    where
        P: Parser<Output = T>,
        F: Fn(T) -> Result<U, ParseError>,
    {
        type Output = U;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p
                .parse(s)
                .and_then(|result, rest| match (self.f)(result) {
                    Ok(result) => ParseState::ok(result, rest),
                    Err(error) => ParseState::error(error, s),
                })
        }
    }

    // And Then
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct AndThen<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> AndThen<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            AndThen { p, q }
        }
    }

    impl<T, U, P, Q> Parser for AndThen<P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser<Output = U>,
    {
        type Output = (T, U);

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.parse(s).and_then(|result_p, rest_p| {
                self.q
                    .parse(rest_p)
                    .and_then(|result, rest| ParseState::ok((result_p, result), rest))
            })
        }
    }

    // Pair
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Pair<'a, P, Q> {
        sep: &'a str,
        p: P,
        q: Q,
    }

    impl<'a, P, Q> Pair<'a, P, Q> {
        pub fn new(sep: &'a str, p: P, q: Q) -> Self {
            Pair { sep, p, q }
        }
    }

    impl<'a, T, U, P, Q> Parser for Pair<'a, P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser<Output = U>,
    {
        type Output = (T, U);

        fn parse<'b>(self, s: &'b str) -> ParseState<'b, Self::Output> {
            self.p
                .and_then(parsers::tag(self.sep))
                .and_then(self.q)
                .map(|((a, _), b)| (a, b))
                .parse(s)
        }
    }

    // Many
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ManyIter<T> {
        contents: VecDeque<T>,
    }

    impl<T> ManyIter<T> {
        fn empty() -> Self {
            ManyIter {
                contents: VecDeque::new(),
            }
        }

        fn cons(&mut self, t: T) {
            self.contents.push_front(t);
        }

        fn extend(&mut self, t: T) {
            self.contents.push_back(t);
        }
    }

    impl<T> Iterator for ManyIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.contents.pop_front()
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Many<P> {
        p: P,
    }

    impl<P> Many<P> {
        pub fn new(p: P) -> Self {
            Many { p }
        }
    }

    impl<T, P> Parser for Many<P>
    where
        P: Parser<Output = T> + Clone,
    {
        type Output = ManyIter<T>;

        fn parse<'a>(self, mut s: &'a str) -> ParseState<'a, Self::Output> {
            let mut many = ManyIter::empty();
            while let ParseState::Ok { result, rest } = self.p.clone().parse(s) {
                s = rest;
                many.extend(result);
            }
            ParseState::ok(many, s)
        }
    }

    // At least one
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct AtLeastOne<P> {
        p: P,
    }

    impl<P> AtLeastOne<P> {
        pub fn new(p: P) -> Self {
            AtLeastOne { p }
        }
    }

    impl<T, P> Parser for AtLeastOne<P>
    where
        P: Parser<Output = T> + Clone,
    {
        type Output = ManyIter<T>;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p
                .clone()
                .and_then(self.p.many())
                .map(|(t, mut ts)| {
                    ts.contents.push_front(t);
                    ts
                })
                .parse(s)
        }
    }

    // ManyChars
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct ManyChars<F: Fn(char) -> bool>(F);

    impl<F: Fn(char) -> bool + Clone> ManyChars<F> {
        pub fn new(f: F) -> Self {
            ManyChars(f)
        }
    }

    impl<F: Fn(char) -> bool + Clone> Parser for ManyChars<F> {
        type Output = String;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::chars(self.0)
                .many()
                .map(|v: ManyIter<char>| v.collect::<String>())
                .parse(s)
        }
    }

    // Repeat
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Repeat<P> {
        count: u32,
        p: P,
    }

    impl<P> Repeat<P> {
        pub fn new(count: u32, p: P) -> Self {
            Repeat { count, p }
        }
    }

    impl<T, P> Parser for Repeat<P>
    where
        P: Parser<Output = T> + Clone,
    {
        type Output = ManyIter<T>;

        fn parse<'a>(self, mut s: &'a str) -> ParseState<'a, Self::Output> {
            let mut many = ManyIter::empty();
            for _ in 0..self.count {
                match self.p.clone().parse(s) {
                    ParseState::Ok { result, rest } => {
                        s = rest;
                        many.extend(result);
                    }
                    ParseState::Err { error, rest } => {
                        return ParseState::error(error, rest);
                    }
                };
            }
            ParseState::ok(many, s)
        }
    }

    // Number
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Number<'a, T>(&'a str, PhantomData<T>);

    impl<'a, T> Number<'a, T> {
        pub fn new(seps: &'a str) -> Self {
            Number(seps, PhantomData)
        }
    }

    impl<'b, T> Parser for Number<'b, T>
    where
        T: FromStr,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::chars(|c: char| c.is_numeric() || self.0.contains(c))
                .many()
                .bind(|v: ManyIter<char>| {
                    let s = v.filter(|c: &char| c.is_numeric()).collect::<String>();
                    s.parse::<T>().map_err(|_| ParseError::ParseIntError(s))
                })
                .parse(s)
        }
    }

    // Signed Number
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct SignedNumber<'a, T>(&'a str, PhantomData<T>);

    impl<'a, T> SignedNumber<'a, T> {
        pub fn new(seps: &'a str) -> Self {
            SignedNumber(seps, PhantomData)
        }
    }

    impl<'b, T> Parser for SignedNumber<'b, T>
    where
        T: Neg<Output = T> + FromStr,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::char('-')
                .ignore_and_then(parsers::number_with_seps(self.0))
                .map(|n: T| -(n as T))
                .or(parsers::char('+')
                    .maybe()
                    .ignore_and_then(parsers::number_with_seps(self.0)))
                .parse(s)
        }
    }

    // List
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct List<'a, P> {
        sep: &'a str,
        p: P,
    }

    impl<'a, P> List<'a, P> {
        pub fn new(sep: &'a str, p: P) -> Self {
            List { sep, p }
        }
    }

    impl<'a, T, P> Parser for List<'a, P>
    where
        P: Parser<Output = T> + Clone,
    {
        type Output = ManyIter<T>;

        fn parse<'b>(self, s: &'b str) -> ParseState<'b, Self::Output> {
            self.p
                .clone()
                .and_then(
                    parsers::tag(self.sep)
                        .and_then(self.p.clone())
                        .map(|(_, v)| v)
                        .many(),
                )
                .map(|(head, mut tail): (T, ManyIter<T>)| {
                    tail.cons(head);
                    tail
                })
                .parse(s)
        }
    }

    // Line
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Line<'a, P> {
        terminator: &'a str,
        p: P,
    }

    impl<'a, P> Line<'a, P> {
        pub fn new(terminator: &'a str, p: P) -> Self {
            Line { terminator, p }
        }
    }

    impl<'b, T, P> Parser for Line<'b, P>
    where
        P: Parser<Output = T>,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.skip_tag(self.terminator).parse(s)
        }
    }

    // ManyLines
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct ManyLines<'a, P> {
        terminator: &'a str,
        p: P,
    }

    impl<'a, P> ManyLines<'a, P> {
        pub fn new(terminator: &'a str, p: P) -> Self {
            ManyLines { terminator, p }
        }
    }

    impl<'b, T, P> Parser for ManyLines<'b, P>
    where
        P: Parser<Output = T> + Clone,
    {
        type Output = ManyIter<T>;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.line(self.terminator).many().parse(s)
        }
    }

    // IngoreAndThen
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct IgnoreAndThen<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> IgnoreAndThen<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            IgnoreAndThen { p, q }
        }
    }

    impl<T, P, Q> Parser for IgnoreAndThen<P, Q>
    where
        P: Parser,
        Q: Parser<Output = T>,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.and_then(self.q).map(|(_, r)| r).parse(s)
        }
    }

    // Skip
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Skip<P, Q> {
        p: P,
        q: Q,
    }

    impl<P, Q> Skip<P, Q> {
        pub fn new(p: P, q: Q) -> Self {
            Skip { p, q }
        }
    }

    impl<T, P, Q> Parser for Skip<P, Q>
    where
        P: Parser<Output = T>,
        Q: Parser,
    {
        type Output = T;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            self.p.and_then(self.q).map(|(r, _)| r).parse(s)
        }
    }

    // Maybe
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Maybe<P> {
        p: P,
    }

    impl<P> Maybe<P> {
        pub fn new(p: P) -> Self {
            Maybe { p }
        }
    }

    impl<T, P> Parser for Maybe<P>
    where
        P: Parser<Output = T>,
    {
        type Output = Option<T>;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            match self.p.parse(s) {
                ParseState::Ok { result, rest } => ParseState::Ok {
                    result: Some(result),
                    rest,
                },
                ParseState::Err { .. } => ParseState::Ok {
                    result: None,
                    rest: s,
                },
            }
        }
    }

    ////////
    ///
    /// Grid related
    ///
    ////////

    // Grid
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Grid<'a, 'b, P> {
        seperator: &'a str,
        terminator: &'b str,
        p: P,
    }

    impl<'a, 'b, P> Grid<'a, 'b, P> {
        pub fn new(seperator: &'a str, terminator: &'b str, p: P) -> Self {
            Grid {
                seperator,
                terminator,
                p,
            }
        }
    }

    impl<'b, 'c, T, P> Parser for Grid<'b, 'c, P>
    where
        P: Parser<Output = T> + Clone,
        T: Clone,
    {
        type Output = grid::Grid<T>;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            let mut vec_of_vecs = Vec::new();
            let (result, rest) = self
                .p
                .clone()
                .list(self.seperator)
                .skip_tag(self.terminator)
                .parse(s)?;
            vec_of_vecs.push(result.collect::<Vec<T>>());
            let (result, rest) = self
                .p
                .clone()
                .and_then(
                    parsers::tag(self.seperator)
                        .and_then(self.p.clone())
                        .map(|(_, v)| v)
                        .repeat((vec_of_vecs[0].len() - 1) as u32),
                )
                .map(|(head, mut tail): (T, ManyIter<T>)| {
                    tail.cons(head);
                    tail.collect::<Vec<T>>()
                })
                .many_lines(self.terminator)
                .parse(rest)?;
            vec_of_vecs.extend(result);
            ParseState::Ok {
                result: grid::Grid::of_vec_of_vecs(vec_of_vecs).unwrap(),
                rest,
            }
        }
    }
}

pub mod parsers {
    use super::parsers_internal;
    #[inline]
    pub fn pure() -> parsers_internal::Pure {
        parsers_internal::Pure::new()
    }

    #[inline]
    pub fn fail<'a>(s: &'a str) -> parsers_internal::Fail<'a> {
        parsers_internal::Fail::new(s)
    }

    #[inline]
    pub fn chars<F: Fn(char) -> bool>(f: F) -> parsers_internal::Chars<F> {
        parsers_internal::Chars::new(f)
    }

    #[inline]
    pub fn char(c: char) -> parsers_internal::Char {
        parsers_internal::Char::new(c)
    }

    #[inline]
    pub fn char_any() -> parsers_internal::CharAny {
        parsers_internal::CharAny::new()
    }

    #[inline]
    pub fn char_map<T, F: Fn(char) -> Option<T>>(f: F) -> parsers_internal::CharMap<T, F> {
        parsers_internal::CharMap::new(f)
    }

    #[inline]
    pub fn tag_replace<'a, T>(s: &'a str, t: T) -> parsers_internal::TagReplace<'a, T> {
        parsers_internal::TagReplace::new(s, t)
    }

    #[inline]
    pub fn tag<'a>(s: &'a str) -> parsers_internal::Tag<'a> {
        parsers_internal::Tag::new(s)
    }

    #[inline]
    pub fn any() -> parsers_internal::Any {
        parsers_internal::Any
    }

    #[inline]
    pub fn drop() -> parsers_internal::Drop {
        parsers_internal::Drop
    }

    #[inline]
    pub fn many_chars<F: Fn(char) -> bool + Clone>(f: F) -> parsers_internal::ManyChars<F> {
        parsers_internal::ManyChars::new(f)
    }

    #[inline]
    pub fn number<'a, T>() -> parsers_internal::Number<'a, T> {
        parsers_internal::Number::new("")
    }

    #[inline]
    pub fn number_with_seps<'a, T>(sep_chars: &'a str) -> parsers_internal::Number<'a, T> {
        parsers_internal::Number::new(sep_chars)
    }

    #[inline]
    pub fn signed_number<T>() -> parsers_internal::SignedNumber<'static, T> {
        parsers_internal::SignedNumber::new("")
    }

    #[inline]
    pub fn signed_number_with_seps<'a, T>(
        sep_chars: &'a str,
    ) -> parsers_internal::SignedNumber<'a, T> {
        parsers_internal::SignedNumber::new(sep_chars)
    }
}

pub trait Parser: Sized {
    type Output;

    fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output>;

    #[inline]
    fn or<Q>(self, other: Q) -> parsers_internal::Or<Self, Q> {
        parsers_internal::Or::new(self, other)
    }

    #[inline]
    fn and<Q>(self, other: Q) -> parsers_internal::And<Self, Q> {
        parsers_internal::And::new(self, other)
    }

    #[inline]
    fn and_then<Q>(self, other: Q) -> parsers_internal::AndThen<Self, Q> {
        parsers_internal::AndThen::new(self, other)
    }

    #[inline]
    fn ignore_and_then<Q>(self, other: Q) -> parsers_internal::IgnoreAndThen<Self, Q> {
        parsers_internal::IgnoreAndThen::new(self, other)
    }

    #[inline]
    fn skip<Q>(self, other: Q) -> parsers_internal::Skip<Self, Q> {
        parsers_internal::Skip::new(self, other)
    }

    #[inline]
    fn skip_tag(self, tag: &str) -> parsers_internal::Skip<Self, parsers_internal::Tag> {
        parsers_internal::Skip::new(self, parsers::tag(tag))
    }

    #[inline]
    fn xor<Q>(self, other: Q) -> parsers_internal::Xor<Self, Q> {
        parsers_internal::Xor::new(self, other)
    }

    #[inline]
    fn map<T, U, F>(self, f: F) -> parsers_internal::Map<Self, F>
    where
        Self: Parser<Output = T>,
        F: Fn(T) -> U,
    {
        parsers_internal::Map::new(self, f)
    }

    #[inline]
    fn bind<F>(self, f: F) -> parsers_internal::Bind<Self, F> {
        parsers_internal::Bind::new(self, f)
    }

    #[inline]
    fn repeat(self, count: u32) -> parsers_internal::Repeat<Self> {
        parsers_internal::Repeat::new(count, self)
    }

    #[inline]
    fn line<'a>(self, terminator: &'a str) -> parsers_internal::Line<'a, Self> {
        parsers_internal::Line::new(terminator, self)
    }

    #[inline]
    fn list<'a>(self, sep: &'a str) -> parsers_internal::List<'a, Self> {
        parsers_internal::List::new(sep, self)
    }

    #[inline]
    fn pair<'a, Q>(self, sep: &'a str, q: Q) -> parsers_internal::Pair<'a, Self, Q> {
        parsers_internal::Pair::new(sep, self, q)
    }

    #[inline]
    fn many(self) -> parsers_internal::Many<Self> {
        parsers_internal::Many::new(self)
    }

    #[inline]
    fn many_at_least_one(self) -> parsers_internal::AtLeastOne<Self> {
        parsers_internal::AtLeastOne::new(self)
    }

    #[inline]
    fn many_lines<'a>(self, terminator: &'a str) -> parsers_internal::ManyLines<'a, Self> {
        parsers_internal::ManyLines::new(terminator, self)
    }

    #[inline]
    fn grid<'a, 'b>(
        self,
        seperator: &'a str,
        terminator: &'b str,
    ) -> parsers_internal::Grid<'a, 'b, Self> {
        parsers_internal::Grid::new(seperator, terminator, self)
    }

    #[inline]
    fn maybe(self) -> parsers_internal::Maybe<Self> {
        parsers_internal::Maybe::new(self)
    }
}

#[cfg(test)]
mod tests {

    use crate::grid::Grid;

    use super::*;

    #[test]
    fn pure() {
        assert_eq!(parsers::pure().parse("").finish(), Ok(()));
        assert_eq!(
            parsers::pure().parse("hello world!").finish(),
            Err((ParseError::RemainingUnparsed, "hello world!"))
        )
    }

    #[test]
    fn fail() {
        assert_eq!(
            parsers::fail("oh no!").parse("").finish(),
            Err((ParseError::Generic("oh no!".to_owned()), ""))
        );
    }

    #[test]
    fn chars() {
        assert_eq!(
            parsers::chars(|c: char| c.is_alphabetic())
                .parse("1")
                .finish(),
            Err((ParseError::UnexpectedChar('1'), "1"))
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_alphabetic())
                .parse("d")
                .finish(),
            Ok('d')
        );
    }

    #[test]
    fn char_map() {
        assert_eq!(
            parsers::char_map(|c: char| match c {
                '1' => Some(1),
                '2' => Some(2),
                _ => None,
            })
            .parse("3")
            .finish(),
            Err((ParseError::UnexpectedChar('3'), "3"))
        );
        assert_eq!(
            parsers::char_map(|c: char| match c {
                '1' => Some(1),
                '2' => Some(2),
                _ => None,
            })
            .parse("1")
            .finish(),
            Ok(1)
        );
    }

    #[test]
    fn char() {
        assert_eq!(
            parsers::char('d').parse("1").finish(),
            Err((ParseError::UnexpectedChar('1'), "1"))
        );
        assert_eq!(parsers::char('d').parse("d").finish(), Ok('d'));
    }

    #[test]
    fn tag() {
        assert_eq!(
            parsers::tag("hello").parse("hello world"),
            ParseState::Ok {
                result: "hello",
                rest: " world"
            }
        );
        assert_eq!(
            parsers::tag("bye").parse("hello world"),
            ParseState::Err {
                error: ParseError::UnmatchedTag("bye".to_owned()),
                rest: "hello world"
            }
        );
    }

    #[test]
    fn any() {
        assert_eq!(
            parsers::any().parse("hello world!").finish(),
            Ok("hello world!".to_owned())
        )
    }

    #[test]
    fn drop() {
        assert_eq!(parsers::drop().parse("hello world!").finish(), Ok(()))
    }

    #[test]
    fn pair() {
        assert_eq!(
            parsers::many_chars(|c: char| c.is_alphabetic())
                .pair(" ", parsers::any())
                .parse("hello world!")
                .finish(),
            Ok(("hello".to_owned(), "world!".to_owned()))
        );
        assert_eq!(
            parsers::many_chars(|c: char| c.is_alphabetic())
                .pair(" ", parsers::fail("oh no!"))
                .parse("hello world!")
                .finish(),
            Err((ParseError::Generic("oh no!".to_owned()), "world!"))
        );
    }

    #[test]
    fn many() {
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many()
                .parse("12345")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec!['1', '2', '3', '4', '5'])
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many()
                .parse("12345 ab")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Err((ParseError::RemainingUnparsed, " ab"))
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many()
                .parse("")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec![])
        );
    }

    #[test]
    fn at_least_one() {
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many_at_least_one()
                .parse("12345")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec!['1', '2', '3', '4', '5'])
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many_at_least_one()
                .parse("12345 ab")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Err((ParseError::RemainingUnparsed, " ab"))
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .many_at_least_one()
                .parse("")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Err((ParseError::EndOfString, ""))
        );
    }

    #[test]
    fn many_chars() {
        assert_eq!(
            parsers::many_chars(|c: char| c.is_numeric())
                .parse("12345")
                .finish(),
            Ok("12345".to_owned())
        );
        assert_eq!(
            parsers::many_chars(|c: char| c.is_numeric())
                .parse("12345 ab")
                .finish(),
            Err((ParseError::RemainingUnparsed, " ab"))
        );
    }

    #[test]
    fn signed_number() {
        assert_eq!(parsers::signed_number().parse("12345").finish(), Ok(12345));
        assert_eq!(
            parsers::signed_number().parse("-12345").finish(),
            Ok(-12345)
        );
        assert_eq!(parsers::signed_number().parse("+12345").finish(), Ok(12345));
        assert_eq!(parsers::signed_number().parse("12345").finish(), Ok(12345));
        assert_eq!(
            parsers::signed_number_with_seps("-")
                .parse("-12-345")
                .finish(),
            Ok(-12345)
        );
        assert_eq!(
            parsers::signed_number().parse("12,345"),
            ParseState::ok(12, ",345")
        );
    }

    #[test]
    fn number() {
        assert_eq!(parsers::number().parse("12345").finish(), Ok(12345));
        assert_eq!(
            parsers::number_with_seps(",").parse("12,345").finish(),
            Ok(12345)
        );
        assert_eq!(
            parsers::number().parse("12,345"),
            ParseState::ok(12, ",345")
        );
    }

    #[test]
    fn list() {
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .list(",")
                .parse("1,2,3,4,5")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec!['1', '2', '3', '4', '5'])
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .list(",")
                .skip(parsers::any())
                .parse(",1,2,3,4,5")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Err((ParseError::UnexpectedChar(','), ",1,2,3,4,5"))
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .map(|c: char| c.to_string().parse::<u32>().unwrap())
                .list(",")
                .parse("1,2,3,4,5")
                .finish()
                .map(|v| v.collect::<Vec<u32>>()),
            Ok(vec![1, 2, 3, 4, 5])
        );
        assert_eq!(
            parsers::chars(|c: char| c.is_numeric())
                .map(|c: char| c.to_string().parse::<u32>().unwrap())
                .list(",")
                .parse("1,2,3,4,5,")
                .finish()
                .map(|v| v.collect::<Vec<u32>>()),
            Err((ParseError::RemainingUnparsed, ","))
        );
        assert_eq!(
            parsers::any()
                .map(|s: String| s.parse::<u32>().unwrap())
                .list(",")
                .parse("12345")
                .finish()
                .map(|v| v.collect::<Vec<u32>>()),
            Ok(vec![12345])
        );
        assert_eq!(
            parsers::number()
                .list("\n\n")
                .parse(
                    "123

456"
                )
                .finish()
                .unwrap()
                .collect::<Vec<u32>>(),
            vec![123, 456]
        )
    }

    #[test]
    fn line() {
        assert_eq!(
            parsers::many_chars(|c| c.is_alphabetic())
                .line("\n")
                .parse("abc\ndef"),
            ParseState::ok("abc".to_owned(), "def")
        )
    }

    #[test]
    fn many_lines() {
        assert_eq!(
            parsers::many_chars(|c| c.is_alphabetic())
                .many_lines("\n")
                .parse("abc\ndef\nghi\n")
                .finish()
                .map(|v| v.collect::<Vec<String>>()),
            Ok(vec!["abc".to_owned(), "def".to_string(), "ghi".to_string()])
        );
        assert_eq!(
            parsers::many_chars(|c| c.is_alphabetic())
                .many_lines("\n")
                .parse("abc\ndef\nghi")
                .finish()
                .map(|v| v.collect::<Vec<String>>()),
            Err((ParseError::RemainingUnparsed, "ghi"))
        );
        assert_eq!(
            parsers::many_chars(|c| c != '\n')
                .many_lines("\n")
                .parse(
                    "QvJvQbjbCgCQRBhzzRsNWNBC
bjgGqQGbQnjGQgnQgbGgjJnDLHLdfPVtdDmLZdBFVVZttdTf
"
                )
                .finish()
                .unwrap()
                .collect::<Vec<String>>(),
            vec![
                "QvJvQbjbCgCQRBhzzRsNWNBC".to_owned(),
                "bjgGqQGbQnjGQgnQgbGgjJnDLHLdfPVtdDmLZdBFVVZttdTf".to_owned()
            ]
        );
    }

    #[test]
    fn grid() {
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .grid("", "\n")
                .parse("abc\ndef\nghi\n")
                .finish(),
            Ok(Grid::from(vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'], 3, 3).unwrap())
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .grid(" ", "\n")
                .parse("a b c\nd e f\ng h i\nj k l\nm n o\n")
                .finish(),
            Ok(Grid::from(
                vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o'],
                5,
                3
            )
            .unwrap())
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .grid("", "\n")
                .parse("abc\ndefg\nhi\n")
                .finish(),
            Err((ParseError::RemainingUnparsed, "defg\nhi\n"))
        );

        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .grid(" ", "\n")
                .parse("a bc\ndef\nghi")
                .finish(),
            Err((ParseError::UnmatchedTag("\n".to_owned()), "c\ndef\nghi"))
        );

        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .grid("", "\n")
                .parse("abc\ndef\nghi")
                .finish(),
            Err((ParseError::RemainingUnparsed, "ghi"))
        );
        assert_eq!(
            parsers::chars(|c| c != '\n')
                .grid("", "\n")
                .parse(
                    "QvJvQbjbCgCQRBhzzRsNWNBC
bjgGqQGbQnjGQgnQgbGgjJnDLHLdfPVtdDmLZdBFVVZttdTf
"
                )
                .finish(),
            Err((
                ParseError::RemainingUnparsed,
                "bjgGqQGbQnjGQgnQgbGgjJnDLHLdfPVtdDmLZdBFVVZttdTf
"
            ))
        );
    }

    #[test]
    fn list_of_lists() {
        let input = "123
456

789
123";
        let result = parsers::number()
            .list("\n")
            .list("\n\n")
            .parse(input)
            .finish()
            .unwrap()
            .map(|v| v.collect::<Vec<u32>>())
            .collect::<Vec<Vec<u32>>>();

        assert_eq!(result, vec![vec![123, 456], vec![789, 123]]);
    }

    #[test]
    fn or() {
        assert_eq!(
            parsers::char('a')
                .or(parsers::char('b'))
                .parse("a")
                .finish(),
            Ok('a')
        );
        assert_eq!(
            parsers::char('a')
                .or(parsers::char('b'))
                .parse("b")
                .finish(),
            Ok('b')
        );
        assert_eq!(
            parsers::char('a')
                .or(parsers::char('b'))
                .parse("c")
                .finish(),
            Err((ParseError::UnexpectedChar('c'), "c"))
        );
    }

    #[test]
    fn and() {
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .and(parsers::char('b'))
                .parse("a")
                .finish(),
            Err((ParseError::UnexpectedChar('a'), "a"))
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .and(parsers::char('b'))
                .parse("b")
                .finish(),
            Ok('b')
        );
        assert_eq!(
            parsers::chars(|c| c.is_numeric())
                .and(parsers::char('c'))
                .parse("c")
                .finish(),
            Err((ParseError::UnexpectedChar('c'), "c"))
        );
    }

    #[test]
    fn ignore() {
        assert_eq!(
            parsers::number::<u32>()
                .ignore_and_then(parsers::any())
                .parse("123abc")
                .finish(),
            Ok("abc".to_owned())
        );
        assert_eq!(
            parsers::char('b')
                .ignore_and_then(parsers::number::<u32>())
                .parse("a123abc")
                .finish(),
            Err((ParseError::UnexpectedChar('a'), "a123abc"))
        );
        assert_eq!(
            parsers::number::<u32>()
                .ignore_and_then(parsers::number::<u32>())
                .parse("123abc")
                .finish(),
            Err((ParseError::ParseIntError("".to_string()), "abc"))
        );
    }

    #[test]
    fn skip() {
        assert_eq!(
            parsers::number()
                .skip(parsers::any())
                .parse("123abc")
                .finish(),
            Ok(123)
        );
        assert_eq!(
            parsers::number()
                .skip(parsers::char('\n'))
                .parse("123\n")
                .finish(),
            Ok(123)
        );
        assert_eq!(
            parsers::char('b')
                .skip(parsers::number::<u32>())
                .parse("a123abc")
                .finish(),
            Err((ParseError::UnexpectedChar('a'), "a123abc"))
        );
        assert_eq!(
            parsers::number::<u32>()
                .skip(parsers::number::<u32>())
                .parse("123abc")
                .finish(),
            Err((ParseError::ParseIntError("".to_string()), "abc"))
        );
    }

    #[test]
    fn and_then() {
        assert_eq!(
            parsers::number()
                .and_then(parsers::any())
                .parse("123abc")
                .finish(),
            Ok((123, "abc".to_owned()))
        );
        assert_eq!(
            parsers::char('b')
                .and_then(parsers::number::<u32>())
                .parse("a123abc")
                .finish(),
            Err((ParseError::UnexpectedChar('a'), "a123abc"))
        );
        assert_eq!(
            parsers::number::<u32>()
                .and_then(parsers::number::<u32>())
                .parse("123abc")
                .finish(),
            Err((ParseError::ParseIntError("".to_string()), "abc"))
        );
    }

    #[test]
    fn xor() {
        assert_eq!(
            parsers::char('a')
                .xor(parsers::char('b'))
                .parse("a")
                .finish(),
            Ok('a')
        );
        assert_eq!(
            parsers::char('a')
                .xor(parsers::char('b'))
                .parse("b")
                .finish(),
            Ok('b')
        );
        assert_eq!(
            parsers::char('a')
                .xor(parsers::char('b'))
                .parse("c")
                .finish(),
            Err((ParseError::UnexpectedChar('c'), "c"))
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .xor(parsers::char('b'))
                .parse("b")
                .finish(),
            Err((ParseError::XorBothTrue, "b"))
        );
    }

    #[test]
    fn map() {
        assert_eq!(
            parsers::any()
                .map(|s: String| s.trim().to_string())
                .parse(" abc ")
                .finish(),
            Ok("abc".to_string())
        )
    }

    #[test]
    fn bind() {
        assert_eq!(
            parsers::any()
                .bind(|s: String| Ok(s.trim().to_string()))
                .parse(" abc ")
                .finish(),
            Ok("abc".to_string())
        );
        assert_eq!(
            parsers::any()
                .bind(|s: String| s.parse::<u32>().map_err(|_| ParseError::ParseIntError(s)))
                .parse("123")
                .finish(),
            Ok(123)
        );
        assert_eq!(
            parsers::any()
                .bind(|s: String| s.parse::<u32>().map_err(|_| ParseError::ParseIntError(s)))
                .parse("a123")
                .finish(),
            Err((ParseError::ParseIntError("a123".to_string()), "a123"))
        )
    }

    #[test]
    fn repeat() {
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .repeat(5)
                .parse("abcde")
                .finish()
                .unwrap()
                .collect::<String>(),
            "abcde".to_string()
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .repeat(5)
                .parse("abcd")
                .finish(),
            Err((ParseError::EndOfString, ""))
        );
        assert_eq!(
            parsers::chars(|c| c.is_alphabetic())
                .repeat(5)
                .parse("abcdef")
                .finish(),
            Err((ParseError::RemainingUnparsed, "f"))
        );
    }
}
