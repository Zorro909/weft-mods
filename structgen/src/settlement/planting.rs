//! What grows round a settlement: kitchen gardens behind the houses, a few small strip fields
//! past the ends of the roads, and trees near the heart.

use structgen_api::{Plan, Turn};
use weft_sdk::Template;

use super::frontage::{Rules, Site, stand};
use super::grid::{Cell, Grid, column};
use super::roads::Road;
use super::shape::{Point, add, noise, scale};

/// Up to `count` small strip fields, each just past the end of a road, the branches' first.
pub fn fields(grid: &mut Grid, roads: &[Road], count: usize) {
    let mut placed = 0;
    for (end, way) in roads.iter().rev().filter_map(Road::end) {
        if placed == count {
            break;
        }
        let along_x = way[0].abs() > way[1].abs();
        let [w, d] = if along_x { [14, 9] } else { [9, 14] };
        let [x, z] = column(add(end, scale(way, 11.0)));
        let corner = [x - w / 2, z - d / 2];
        let last = [corner[0] + w - 1, corner[1] + d - 1];
        if !grid.all(corner, last, |c| c == Cell::Free) {
            continue;
        }
        for cz in corner[1]..=last[1] {
            for cx in corner[0]..=last[0] {
                let line = if along_x {
                    cz - corner[1]
                } else {
                    cx - corner[0]
                };
                grid.put([cx, cz], strip(line, 5, &[1, 2, 3, 4, 7]));
            }
        }
        placed += 1;
    }
}

/// A kitchen garden behind a house: a few rows of young wheat, if the ground there is free.
pub fn garden(grid: &mut Grid, site: &Site) {
    let [x, _, z] = site.corner;
    let [sx, _, sz] = site.size;
    let (corner, last) = match site.turn {
        Turn::None => ([x + 1, z + sz + 1], [x + sx - 2, z + sz + 4]),
        Turn::Back => ([x + 1, z - 5], [x + sx - 2, z - 2]),
        Turn::Left => ([x + sx + 1, z + 1], [x + sx + 4, z + sz - 2]),
        Turn::Right => ([x - 5, z + 1], [x - 2, z + sz - 2]),
    };
    if !grid.all(corner, last, |c| c == Cell::Free) {
        return;
    }
    let across_x = matches!(site.turn, Turn::None | Turn::Back);
    for gz in corner[1]..=last[1] {
        for gx in corner[0]..=last[0] {
            grid.put(
                [gx, gz],
                strip(if across_x { gx } else { gz }, 2, &[2, 3, 4]),
            );
        }
    }
}

/// Line `line` of a field: a furrow every `width + 1` lines, wheat of an age from `ages`
/// otherwise, the same all along a strip.
fn strip(line: i32, width: i32, ages: &[u8]) -> Cell {
    let index = line.div_euclid(width + 1);
    if line.rem_euclid(width + 1) == width {
        return Cell::Furrow;
    }
    Cell::Crop(ages[(noise(index, 7) * ages.len() as f32) as usize % ages.len()])
}

/// Trees on free ground within `reach` of the heart's middle: up to one of each of
/// `templates`, a few tries each.
pub fn trees(
    p: &mut Plan,
    grid: &mut Grid,
    templates: &[&'static Template],
    rules: &Rules,
    middle: Point,
    reach: i32,
) -> Vec<Site> {
    let mut sites = Vec::new();
    for &template in templates {
        let spot = |p: &mut Plan| p.random().between(-reach, reach);
        let tree = (0..8).find_map(|_| {
            let at = [middle[0] as i32 + spot(p), middle[1] as i32 + spot(p)];
            stand(p, grid, rules, template, at, Turn::None, None)
        });
        sites.extend(tree);
    }
    sites
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_has_a_furrow_between_strips() {
        let lines: Vec<bool> = (0..12)
            .map(|line| strip(line, 5, &[3]) == Cell::Furrow)
            .collect();
        let furrows: Vec<usize> = (0..12).filter(|&i| lines[i]).collect();
        assert_eq!(furrows, [5, 11], "five lines of wheat, then a furrow");
    }
}
