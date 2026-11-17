//! The picture of the washer's paddle: oak boards, three crossed into six blades, their edges
//! darker where the water wets them, on an iron axle.

use super::processing::{OAK, oak};
use super::{CLEAR, IRON, Picture, Unwrap, noise};

/// The paddle's texture and where its model's unwrap puts each box: a board, which all three
/// share, then the axle.
const SIZE: (usize, usize) = (64, 32);
const BOARD: Unwrap = Unwrap::new((0, 0), (10, 10, 1));
const AXLE: Unwrap = Unwrap::new((24, 0), (14, 3, 3));

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![("entity/washer_paddle", paddle())]
}

/// The paddle: oak boards rimmed in wet, dark oak, on an iron axle.
fn paddle() -> Picture {
    let mut picture = Picture::sized(SIZE.0, SIZE.1, CLEAR);
    BOARD.paint(&mut picture, oak);
    let rim = |x, y, edge| if edge { OAK[0] } else { oak(x, y) };
    BOARD.paint_ends(&mut picture, rim);
    BOARD.paint_caps(&mut picture, rim);
    AXLE.paint(&mut picture, |x, y| {
        if noise(x, y, 331).is_multiple_of(4) {
            IRON[1]
        } else {
            IRON[2]
        }
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_box_of_the_paddle_is_painted() {
        let paddle = paddle();
        assert_ne!(paddle.get(2, 0)[3], 0, "the board's top");
        assert_ne!(paddle.get(5, 6)[3], 0, "its face");
        assert_ne!(paddle.get(30, 4)[3], 0, "the axle");
    }
}
