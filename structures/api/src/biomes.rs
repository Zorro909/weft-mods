//! Which biomes a structure may stand in, named as anywhere in Weft: `"#forest !base:birch_forest"`
//! is any biome tagged `forest` but the birch forest. Resolved at the first use, once the game
//! has loaded.

use std::sync::OnceLock;

use weft_sdk::*;

/// Biome names and `#tags` apart by spaces, `!` in front of one leaving out what it names.
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
    /// `u32::MAX` for a name no biome has, which matches nothing.
    Biome(u32),
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
        let terms = self
            .terms
            .get_or_init(|| split(self.text).map(resolve).collect());
        includes(terms, |named| match named {
            Named::Tag(tag) => tag.on_biome(id),
            Named::Biome(biome) => *biome == id,
        })
    }
}

fn split(text: &str) -> impl Iterator<Item = Term<&str>> {
    text.split_whitespace().map(|word| {
        let left_out = word.strip_prefix('!');
        Term {
            left_out: left_out.is_some(),
            what: left_out.unwrap_or(word),
        }
    })
}

fn resolve(term: Term<&str>) -> Term<Named> {
    let what = match term.what.strip_prefix('#') {
        Some(tag) => Named::Tag(Tag::named(tag.to_string().leak())),
        None => Named::Biome(biome_id(term.what)),
    };
    Term {
        left_out: term.left_out,
        what,
    }
}

fn biome_id(name: &str) -> u32 {
    let id = Biome::generated(name.to_string()).id();
    if id == u32::MAX {
        error!("a structure's biomes name {name}, which is no biome of this game");
    }
    id
}

/// Whether a biome is among `terms`, `is` telling whether it is what a term names: one of
/// those not left out (any, when there are none) and none of those left out.
fn includes<W>(terms: &[Term<W>], is: impl Fn(&W) -> bool) -> bool {
    let named = |left_out| terms.iter().filter(move |t| t.left_out == left_out);
    let wanted = named(false).next().is_none() || named(false).any(|t| is(&t.what));
    wanted && !named(true).any(|t| is(&t.what))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whether a biome that has `names` is in `text`.
    fn fits(text: &str, names: &[&str]) -> bool {
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
