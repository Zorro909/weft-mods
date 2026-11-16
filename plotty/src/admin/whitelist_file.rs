//! The engine's whitelist file as text: one player key a line, which a dedicated server reads
//! again at every login (`Weft/crates/engine/src/server/player_file.rs`). Other lines an admin
//! wrote by hand are kept as they are.

/// The keys the file lets in.
pub fn keys(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

/// The file with `key` added, unless it is already in.
pub fn with(text: &str, key: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    if !lines.iter().any(|line| line.trim() == key) {
        lines.push(key);
    }
    joined(&lines)
}

/// The file without `key`.
pub fn without(text: &str, key: &str) -> String {
    let lines: Vec<&str> = text.lines().filter(|line| line.trim() != key).collect();
    joined(&lines)
}

/// The file to turn the whitelist on with: `text` and `keeping`, the admin turning it on;
/// `None` where it would let nobody in, the admins among them.
pub fn to_turn_on(text: &str, keeping: Option<&str>) -> Option<String> {
    let text = match keeping {
        Some(key) => with(text, key),
        None => text.to_string(),
    };
    (!keys(&text).is_empty()).then_some(text)
}

/// Lines ending with a newline each, as a hand-edited file would.
fn joined(lines: &[&str]) -> String {
    lines.iter().map(|line| format!("{line}\n")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_added_once() {
        let once = with("aa\n", "bb");
        assert_eq!(once, "aa\nbb\n", "added at the end");
        assert_eq!(with(&once, "bb"), once, "not twice");
    }

    #[test]
    fn a_key_is_removed_and_the_rest_kept() {
        assert_eq!(
            without("aa\nbb\n  cc  \n", "bb"),
            "aa\n  cc  \n",
            "only that line goes"
        );
    }

    #[test]
    fn blank_lines_let_nobody_in() {
        assert_eq!(keys("aa\n\n  \nbb"), vec!["aa", "bb"], "two keys");
    }

    #[test]
    fn the_whitelist_never_turns_on_empty() {
        assert_eq!(
            to_turn_on("", None),
            None,
            "from the console, before anyone joined"
        );
        assert_eq!(
            to_turn_on("", Some("aa")),
            Some("aa\n".to_string()),
            "the admin stays in"
        );
        assert_eq!(
            to_turn_on("bb\n", Some("bb")),
            Some("bb\n".to_string()),
            "once"
        );
    }

    #[test]
    fn an_empty_file_gets_its_first_key() {
        assert_eq!(with("", "aa"), "aa\n", "one line");
    }
}
