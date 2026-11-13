//! Names as scripts write them: a script's own, and its components'.
//!
//! A component is named as the game names its own, `<script>:<name>`, so two scripts never
//! write into each other's by accident. A script says the short `name` for its own and the
//! full one for another script's.

/// What parts a script's name from a component's.
const SEPARATOR: char = ':';

/// Whether `name` is only lowercase letters, digits and `_`, and not empty: what a script, a
/// command and a component may be called.
pub fn is_plain(name: &str) -> bool {
    let plain =
        |letter: char| letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '_';
    !name.is_empty() && name.chars().all(plain)
}

/// The full name of the component `name` as `script` wrote it: its own when short, another
/// script's when it says whose. `None` for a name that is neither.
pub fn qualified(script: &str, name: &str) -> Option<String> {
    let (owner, short) = name.split_once(SEPARATOR).unwrap_or((script, name));
    (is_plain(owner) && is_plain(short)).then(|| format!("{owner}{SEPARATOR}{short}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_name_is_the_scripts_own() {
        assert_eq!(qualified("camps", "camp").as_deref(), Some("camps:camp"));
    }

    #[test]
    fn a_full_name_reaches_another_scripts() {
        let full = qualified("camps", "welcome:visits");
        assert_eq!(full.as_deref(), Some("welcome:visits"));
    }

    #[test]
    fn a_name_with_anything_else_is_none() {
        for name in ["", "Camp", "my camp", "a:b:c", ":camp", "camps:", "käse"] {
            assert_eq!(qualified("camps", name), None, "{name:?} is no component");
        }
    }

    #[test]
    fn plain_is_lowercase_letters_digits_and_underscores() {
        assert!(is_plain("first_kit2"));
        for name in ["", "Kit", "first kit", "kit:one", "kit-one"] {
            assert!(!is_plain(name), "{name:?} is not plain");
        }
    }
}
