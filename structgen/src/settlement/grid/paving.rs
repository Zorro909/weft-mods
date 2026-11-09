//! A town's paving on the grid: cobbles in front of its houses, and its streets fading to lanes
//! where the houses end.

use super::*;

impl Grid {
    /// Paves the bare ground between a town's streets and the buildings on them, so no grass is
    /// left in front of a house: every free column next to a building and near a road.
    pub fn pave_frontages(&mut self) {
        let paved: Vec<[i32; 2]> = (self.columns())
            .filter(|&(at, cell)| {
                cell == Cell::Free
                    && self.near(at, 2, |c| c == Cell::Building)
                    && self.near(at, 3, Cell::is_road)
            })
            .map(|(at, _)| at)
            .collect();
        for at in paved {
            self.put(at, Cell::Square);
        }
    }

    /// Turns a town's streets into lanes, their kerbs into grass, where no building stands
    /// within `reach`: the cobbles end where the town does.
    pub fn fade_streets(&mut self, reach: i32) {
        let faded: Vec<(usize, Cell)> = (self.columns().enumerate())
            .filter_map(|(i, (at, cell))| {
                let to = match cell {
                    Cell::Street => Cell::Lane,
                    Cell::Kerb => Cell::Free,
                    _ => return None,
                };
                (!self.near(at, reach, |c| c == Cell::Building)).then_some((i, to))
            })
            .collect();
        for (i, cell) in faded {
            self.cells[i] = cell;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_street_fades_to_a_lane_away_from_the_houses() {
        let mut street = row(&[Some(64); 20]);
        for x in 0..19 {
            street.put([x, 0], Cell::Street);
        }
        street.put([19, 0], Cell::Kerb);
        street.put([1, 0], Cell::Building);
        street.fade_streets(3);
        assert_eq!(street.get([4, 0]), Cell::Street, "three from the house");
        assert_eq!(street.get([5, 0]), Cell::Lane, "four from it");
        assert_eq!(
            street.get([19, 0]),
            Cell::Free,
            "a kerb out of town is grass"
        );
    }
}
