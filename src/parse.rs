pub enum ParseError {
    UnexpectedChar(char),
    UnmatchedTag(String),
    Generic(String),
    RemainingUnparsed(String),
    XorBothTrue,
    EndOfString,
}

pub enum ParseState<'a, T> {
    Err(ParseError),
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
    fn error_unexpected_char(c: char) -> Self {
        Self::Err(ParseError::UnexpectedChar(c))
    }

    #[inline]
    fn error_generic(err: &str) -> Self {
        Self::Err(ParseError::Generic(err.to_owned()))
    }

    #[inline]
    fn error_end_of_string() -> Self {
        Self::Err(ParseError::EndOfString)
    }

    #[inline]
    fn error_unmatched_tag(s: &str) -> Self {
        Self::Err(ParseError::UnmatchedTag(s.to_owned()))
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
    fn error(err: ParseError) -> Self {
        ParseState::Err(err)
    }

    #[inline]
    fn and_then<U, F: FnOnce(T, &'a str) -> ParseState<'a, U>>(self, f: F) -> ParseState<'a, U> {
        match self {
            ParseState::Ok { result, rest } => f(result, rest),
            ParseState::Err(err) => ParseState::Err(err),
        }
    }

    #[inline]
    fn and<'b, U>(self, other: ParseState<'b, U>) -> ParseState<'b, U> {
        match self {
            ParseState::Ok { .. } => other,
            ParseState::Err(err) => ParseState::error(err),
        }
    }

    #[inline]
    fn or(self, other: Self) -> Self {
        match self {
            ParseState::Ok { .. } => self,
            _ => other,
        }
    }

    #[inline]
    fn xor(self, other: Self) -> Self {
        match self {
            ParseState::Ok { .. } => match other {
                ParseState::Ok { .. } => ParseState::error(ParseError::XorBothTrue),
                ParseState::Err(..) => self,
            },
            ParseState::Err(..) => other,
        }
    }
}

///////
///
/// Parser State interface
///
///////

impl<'a, T> ParseState<'a, T> {
    pub fn fail(err: &str) -> ParseState<'a, T> {
        Self::error_generic(err)
    }

    pub fn finish(self) -> Result<T, ParseError> {
        match self {
            ParseState::Err(err) => Err(err),
            ParseState::Ok { result, rest: "" } => Ok(result),
            ParseState::Ok { result: _, rest } => {
                Err(ParseError::RemainingUnparsed(rest.to_owned()))
            }
        }
    }
}

///////
///
/// Parser combinators
///
///////

mod parsers_internal {
    use std::collections::VecDeque;
    use std::convert::Infallible;

    use super::{parsers, ParseState, Parser};

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
        type Output = Infallible;

        fn parse<'a>(self, _: &'a str) -> ParseState<'a, Self::Output> {
            ParseState::error_generic(&self.0)
        }
    }

    // Chars
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Chars<F>(F);

    impl<F> Chars<F> {
        pub fn new(f: F) -> Self {
            Chars(f)
        }
    }

    impl<F> Parser for Chars<F>
    where
        F: Fn(char) -> bool,
    {
        type Output = char;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            let mut s = s.chars();
            match s.next() {
                None => ParseState::error_end_of_string(),
                Some(c) => {
                    if (self.0)(c) {
                        ParseState::ok(c, s.as_str())
                    } else {
                        ParseState::error_unexpected_char(c)
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
            let mut s = s.chars();
            match s.next() {
                None => ParseState::error_end_of_string(),
                Some(c) => {
                    if self.0 == c {
                        ParseState::ok(c, s.as_str())
                    } else {
                        ParseState::error_unexpected_char(c)
                    }
                }
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
                None => ParseState::error_unmatched_tag(self.0),
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
            self.p.parse(s).xor(self.q.parse(s))
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

    impl<P, F> Map<P, F> {
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
        type Output = VecDeque<T>;

        fn parse<'b>(self, s: &'b str) -> ParseState<'b, Self::Output> {
            let ParseState::Ok{result, rest} = self.p.clone().parse(s) else {
                return ParseState::ok(VecDeque::new(), s);
            };
            let ParseState::Ok{rest, ..} = parsers::tag(self.sep).parse(rest) else {
                return ParseState::ok(VecDeque::from([result]), rest);
            };
            let ParseState::Ok {
                result: mut list, rest
            } = self.parse(rest)
            else {
                return ParseState::error_generic("List parser should not fail")
            };

            list.push_front(result);
            ParseState::ok(list, rest)
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
    pub fn chars<F>(f: F) -> parsers_internal::Chars<F> {
        parsers_internal::Chars::new(f)
    }

    #[inline]
    pub fn char(c: char) -> parsers_internal::Char {
        parsers_internal::Char::new(c)
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
    pub fn pair<'a, P, Q>(p: P, q: Q, sep: &'a str) -> parsers_internal::Pair<'a, P, Q> {
        parsers_internal::Pair::new(sep, p, q)
    }

    #[inline]
    pub fn list<'a, P>(p: P, sep: &'a str) -> parsers_internal::List<'a, P> {
        parsers_internal::List::new(sep, p)
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
    fn xor<Q>(self, other: Q) -> parsers_internal::Xor<Self, Q> {
        parsers_internal::Xor::new(self, other)
    }

    #[inline]
    fn map<F>(self, f: F) -> parsers_internal::Map<Self, F> {
        parsers_internal::Map::new(self, f)
    }
}
