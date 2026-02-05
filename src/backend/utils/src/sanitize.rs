use std::fmt::Display;

pub fn is_clean<T: Display>(string: T) -> bool {
    let bad_chars = [' ', '\\', '\n', '\r', '\t', '"', '\'', '`', ';'];
    !string.to_string().chars().any(|c| bad_chars.contains(&c))
}
