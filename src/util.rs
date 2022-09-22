pub fn slice_string(s: &str, amt: usize) -> &str {
    match s.char_indices().skip(amt).next() {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}
