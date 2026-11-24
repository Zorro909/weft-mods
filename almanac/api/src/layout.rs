//! How a chapter's quests are lined up on its page: in chains, each quest behind the one it
//! follows, a row per chain.

/// The rows of a chapter. `quests` are in page order; `follows(quest)` is the quest it comes
/// after in this chapter, if any. A quest joins the row that ends with the one it follows, and
/// starts a row of its own otherwise: the first of a chain, or a second quest after the same one.
pub fn rows(quests: &[usize], follows: impl Fn(usize) -> Option<usize>) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for &quest in quests {
        let behind = follows(quest);
        let row = rows
            .iter_mut()
            .find(|row| behind.is_some() && row.last().copied() == behind);
        match row {
            Some(row) => row.push(quest),
            None => rows.push(vec![quest]),
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `rows` where quest `n` follows `before[n]`.
    fn laid_out(before: &[Option<usize>]) -> Vec<Vec<usize>> {
        let quests: Vec<usize> = (0..before.len()).collect();
        rows(&quests, |quest| before[quest])
    }

    #[test]
    fn a_chain_is_one_row() {
        assert_eq!(laid_out(&[None, Some(0), Some(1)]), [[0, 1, 2]]);
    }

    #[test]
    fn every_start_gets_a_row() {
        let rows = laid_out(&[None, Some(0), None, Some(2)]);
        assert_eq!(rows, [[0, 1], [2, 3]], "two chains, two rows");
    }

    #[test]
    fn a_second_follower_starts_a_row() {
        let rows = laid_out(&[None, Some(0), Some(0)]);
        assert_eq!(
            rows,
            [vec![0, 1], vec![2]],
            "the row goes on with the first"
        );
    }

    #[test]
    fn a_follower_listed_first_stands_alone() {
        let rows = laid_out(&[Some(1), None]);
        assert_eq!(rows, [[0], [1]], "what it follows comes later on the page");
    }

    #[test]
    fn an_empty_chapter_has_no_rows() {
        assert!(laid_out(&[]).is_empty());
    }
}
