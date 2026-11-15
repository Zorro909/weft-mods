//! The screen effects showcase brings for `Film::screen`: post shaders in its
//! `assets/shaders/post/`, by their full names. Any mod's own post shader works as well.

/// The picture going black: `Film::fade_out` and `fade_in` use it.
pub const BLACK: &str = "showcase:black";
/// The picture going white, for a fade to white.
pub const WHITE: &str = "showcase:white";
/// A golden-hour grade: warmer and a little richer.
pub const WARM: &str = "showcase:warm";
/// A moonlit grade: bluer and a little paler.
pub const COOL: &str = "showcase:cool";
