pub fn parse_chunks<U, F: Fn(&str) -> Option<U> + Copy + 'static>(
    s: &str,
    f: F,
) -> impl Iterator<Item = impl Iterator<Item = U> + '_> {
    s.split("\n\n").map(move |s| s.lines().filter_map(f))
}

pub fn parse_lines<'a, U, F: Fn(&str) -> Option<U> + Copy + 'a>(
    s: &'a str,
    f: F,
) -> impl Iterator<Item = U> + 'a {
    s.lines().filter_map(f)
}
