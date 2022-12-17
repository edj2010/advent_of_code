#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
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
    fn xor(self, other: Self) -> Self {
        match self {
            ParseState::Ok { .. } => match other {
                ParseState::Ok { rest, .. } => ParseState::error(ParseError::XorBothTrue, rest),
                ParseState::Err { .. } => self,
            },
            ParseState::Err { .. } => other,
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

    // Many
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ManyIter<T> {
        contents: Vec<T>,
    }

    impl<T> ManyIter<T> {
        fn empty() -> Self {
            ManyIter { contents: vec![] }
        }

        fn singleton(t: T) -> Self {
            ManyIter { contents: vec![t] }
        }

        fn extended(mut self, t: T) -> Self {
            self.contents.push(t);
            self
        }
    }

    impl<T> Iterator for ManyIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.contents.pop()
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

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            let ParseState::Ok{result, rest} = self.p.clone().parse(s) else {
                return ParseState::ok(ManyIter::empty(), s);
            };
            let ParseState::Ok {
                result: many, rest
            } = self.parse(rest)
            else {
                return ParseState::error_generic("Many parser should not fail", s)
            };
            ParseState::ok(many.extended(result), rest)
        }
    }

    // String
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
            parsers::many(parsers::chars(self.0))
                .map(|v: ManyIter<char>| v.collect::<String>())
                .parse(s)
        }
    }

    // Number
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Number<'a>(&'a str);

    impl<'a> Number<'a> {
        pub fn new(seps: &'a str) -> Self {
            Number(seps)
        }
    }

    impl<'b> Parser for Number<'b> {
        type Output = usize;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::many(parsers::chars(|c: char| {
                c.is_numeric() || self.0.contains(c)
            }))
            .map(|v: ManyIter<char>| {
                v.filter(|c: &char| c.is_numeric())
                    .collect::<String>()
                    .parse::<usize>()
                    .unwrap()
            })
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
            let ParseState::Ok{result, rest} = self.p.clone().parse(s) else {
                return ParseState::ok(ManyIter::empty(), s);
            };
            let ParseState::Ok{rest, ..} = parsers::tag(self.sep).parse(rest) else {
                return ParseState::ok(ManyIter::singleton(result), rest);
            };
            let ParseState::Ok {
                result: list, rest
            } = self.parse(rest)
            else {
                return ParseState::error_generic("List parser should not fail", s)
            };
            ParseState::ok(list.extended(result), rest)
        }
    }

    // Line
    pub struct Line(char);

    impl Line {
        pub fn new(terminator: char) -> Self {
            Line(terminator)
        }
    }

    impl Parser for Line {
        type Output = String;

        fn parse<'a>(self, s: &'a str) -> ParseState<'a, Self::Output> {
            parsers::many(parsers::chars(|c: char| c != self.0))
                .map(|v: ManyIter<char>| v.collect::<String>())
                .and_then(parsers::char(self.0))
                .map(|(s, _)| s)
                .parse(s)
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
    pub fn many<P>(p: P) -> parsers_internal::Many<P> {
        parsers_internal::Many::new(p)
    }

    #[inline]
    pub fn many_chars<F: Fn(char) -> bool + Clone>(f: F) -> parsers_internal::ManyChars<F> {
        parsers_internal::ManyChars::new(f)
    }

    #[inline]
    pub fn number<'a>() -> parsers_internal::Number<'a> {
        parsers_internal::Number::new("")
    }

    #[inline]
    pub fn number_with_seps<'a>(sep_chars: &'a str) -> parsers_internal::Number<'a> {
        parsers_internal::Number::new(sep_chars)
    }

    #[inline]
    pub fn list<'a, P>(sep: &'a str, p: P) -> parsers_internal::List<'a, P> {
        parsers_internal::List::new(sep, p)
    }

    #[inline]
    pub fn line(terminator: char) -> parsers_internal::Line {
        parsers_internal::Line::new(terminator)
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

#[cfg(test)]
mod tests {

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
            parsers::pair(
                parsers::many_chars(|c: char| c.is_alphabetic()),
                parsers::any(),
                " "
            )
            .parse("hello world!")
            .finish(),
            Ok(("hello".to_owned(), "world!".to_owned()))
        );
        assert_eq!(
            parsers::pair(
                parsers::many_chars(|c: char| c.is_alphabetic()),
                parsers::fail("oh no!"),
                " "
            )
            .parse("hello world!")
            .finish(),
            Err((ParseError::Generic("oh no!".to_owned()), "world!"))
        );
    }

    #[test]
    fn many() {
        assert_eq!(
            parsers::many(parsers::chars(|c: char| c.is_numeric()))
                .parse("12345")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec!['1', '2', '3', '4', '5'])
        );
        assert_eq!(
            parsers::many(parsers::chars(|c: char| c.is_numeric()))
                .parse("12345 ab")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Err((ParseError::RemainingUnparsed, " ab"))
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
            parsers::list(",", parsers::chars(|c: char| c.is_numeric()))
                .parse("1,2,3,4,5")
                .finish()
                .map(|v| v.collect::<Vec<char>>()),
            Ok(vec!['1', '2', '3', '4', '5'])
        );
        assert_eq!(
            parsers::list(
                ",",
                parsers::chars(|c: char| c.is_numeric())
                    .map(|c: char| c.to_string().parse::<usize>().unwrap())
            )
            .parse("1,2,3,4,5")
            .finish()
            .map(|v| v.collect::<Vec<usize>>()),
            Ok(vec![1, 2, 3, 4, 5])
        );
        let parser = parsers::list(
            ",",
            parsers::any().map(|s: String| s.parse::<usize>().unwrap()),
        );
        assert_eq!(
            parser
                .parse("12345")
                .finish()
                .map(|v| v.collect::<Vec<usize>>()),
            Ok(vec![12345])
        )
    }

    #[test]
    fn line() {
        assert_eq!(
            parsers::line('\n').parse("abc\ndef"),
            ParseState::ok("abc".to_owned(), "def")
        )
    }
}
