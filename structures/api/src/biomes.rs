//! Which biomes a structure may stand in: `"#forest !base:birch_forest"`, resolved at the first
//! use, once the game has loaded.

use std::sync::OnceLock;

use weft_sdk::*;

/// Biome names and `#tags` apart by spaces, `!` in front of one leaving it out. No `*` patterns.
pub struct Biomes {
    text: &'static str,
    terms: OnceLock<Vec<Term<Named>>>,
}

/// One name or tag of the text.
struct Term<W> {
    left_out: bool,
    what: W,
}

enum Named {
    Tag(Tag<kind::Biome>),
    Biome(Biome),
}

impl Biomes {
    pub const fn named(text: &'static str) -> Biomes {
        Biomes {
            text,
            terms: OnceLock::new(),
        }
    }

    /// Whether biome `id` is one of them.
    pub fn contains(&self, id: u32) -> bool {
        let terms = self.terms.get_or_init(|| resolve(self.text));
        includes(terms, |named| match named {
            Named::Tag(tag) => tag.on_biome(id),
            Named::Biome(biome) => biome.id() == id,
        })
    }
}

/// Each term with its tag or biome; the SDK logs a biome name nothing has.
fn resolve(text: &'static str) -> Vec<Term<Named>> {
    let named = |what: &'static str| match what.strip_prefix('#') {
        Some(tag) => Named::Tag(Tag::named(tag)),
        None => Named::Biome(Biome::named(what)),
    };
    split(text)
        .map(|term| Term {
            left_out: term.left_out,
            what: named(term.what),
        })
        .collect()
}

fn split(text: &'static str) -> impl Iterator<Item = Term<&'static str>> {
    text.split_whitespace().map(|word| {
        let left_out = word.strip_prefix('!');
        Term {
            left_out: left_out.is_some(),
            what: left_out.unwrap_or(word),
        }
    })
}

/// Whether a biome is among `terms`, `matches` telling whether a term names it. With only `!`
/// terms, every biome but those.
fn includes<W>(terms: &[Term<W>], matches: impl Fn(&W) -> bool) -> bool {
    let any = |left_out| {
        terms
            .iter()
            .any(|term| term.left_out == left_out && matches(&term.what))
    };
    let only_left_out = terms.iter().all(|term| term.left_out);
    (only_left_out || any(false)) && !any(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether a biome that has `names` is in `text`.
    fn fits(text: &'static str, names: &[&str]) -> bool {
        let terms: Vec<_> = split(text).collect();
        includes(&terms, |what| names.contains(what))
    }

    #[test]
    fn one_name_or_tag_is_enough() {
        assert!(fits("#forest #taiga", &["#taiga"]), "the second tag");
        assert!(!fits("#forest", &["#plains"]), "another tag");
    }

    #[test]
    fn a_bang_leaves_out() {
        let text = "#forest !base:birch_forest";
        assert!(fits(text, &["#forest", "base:oak_forest"]), "a forest");
        assert!(!fits(text, &["#forest", "base:birch_forest"]), "not birch");
    }

    #[test]
    fn only_left_out_means_everything_else() {
        assert!(
            fits("!#ocean", &["#plains"]),
            "anything that isn't an ocean"
        );
        assert!(!fits("!#ocean", &["#ocean"]), "but no ocean");
    }
}
