use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
    str::Chars,
};

pub enum TokenTree {
    Integer(isize),
    Float(f64),
    String(String),
    Char(char),
    List(Vec<TokenTree>),
}

pub enum ParseError {
    UnexpectedChar(char),
    EndOfString,
    Generic(String),
}

pub struct ParseSuccess<'a, T> {
    result: T,
    rest: Chars<'a>,
}

pub enum ParseState<'a, T> {
    Err(ParseError),
    Ok(ParseSuccess<'a, T>),
}

impl<'a, T> FromResidual<ParseState<'a, Infallible>> for ParseState<'a, T> {
    #[track_caller]
    fn from_residual(residual: ParseState<'a, Infallible>) -> Self {
        match residual {
            ParseState::Err(e) => ParseState::Err(e),
            ParseState::Ok(_) => {
                panic!("Residual should never by type okay, how did you get here?")
            }
        }
    }
}

impl<'a, T> Try for ParseState<'a, T> {
    type Output = ParseSuccess<'a, T>;
    type Residual = ParseState<'a, Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(v) => ControlFlow::Continue(v),
            Self::Err(e) => ControlFlow::Break(ParseState::Err(e)),
        }
    }
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
    fn error_end_of_string() -> Self {
        Self::Err(ParseError::EndOfString)
    }

    #[inline]
    fn error_generic(err: &str) -> Self {
        Self::Err(ParseError::Generic(err.to_owned()))
    }
}

///////
///
/// Some internal functions
///
///////
impl<'a, T> ParseState<'a, T> {
    #[inline]
    fn ok(result: T, rest: Chars<'a>) -> Self {
        ParseState::Ok(ParseSuccess { result, rest })
    }

    #[inline]
    fn error(err: ParseError) -> Self {
        ParseState::Err(err)
    }

    #[inline]
    fn bind_success<U, F: Fn(ParseSuccess<'a, T>) -> ParseState<'a, U>>(
        self,
        f: F,
    ) -> ParseState<'a, U> {
        (f)(self?)
    }
}

///////
///
/// Parser implementation
///
///////
impl<'a, T> ParseState<'a, T> {
    pub fn pure(string: &'a str) -> ParseState<'a, ()> {
        ParseState::ok((), string.chars())
    }

    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> ParseState<'a, U> {
        self.bind_success(|ParseSuccess { result, rest }| ParseState::ok((f)(result), rest))
    }

    pub fn bind<U, F: Fn(T) -> Result<U, ParseError>>(self, f: F) -> ParseState<'a, U> {
        self.bind_success(|ParseSuccess { result, rest }| match f(result) {
            Result::Err(e) => ParseState::error(e),
            Result::Ok(result) => ParseState::ok(result, rest),
        })
    }
}
