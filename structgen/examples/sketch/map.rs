//! A sketch seen from above as text, north up: the ground's marks, and each structure's
//! footprint as a letter, listed under the map. And the structures that overlap.

use std::fmt::Write;

use crate::Footprint;
use crate::raster::Ground;

const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn draw(ground: &Ground, footprints: &[Footprint]) -> String {
    let [w, h] = ground.size;
    let mut rows: Vec<Vec<char>> = (0..h)
        .map(|z| (0..w).map(|x| ground.get([x, z]).glyph()).collect())
        .collect();
    let mut legend = String::new();
    for (footprint, letter) in footprints.iter().zip(LETTERS.chars().cycle()) {
        for z in footprint.corner[1]..footprint.corner[1] + footprint.size[1] {
            for x in footprint.corner[0]..footprint.corner[0] + footprint.size[0] {
                let (gx, gz) = (x - ground.origin[0], z - ground.origin[1]);
                if (0..w as i32).contains(&gx) && (0..h as i32).contains(&gz) {
                    rows[gz as usize][gx as usize] = letter;
                }
            }
        }
        let [x, z] = footprint.corner;
        writeln!(legend, "{letter} {} at {x},{z}", footprint.name).ok();
    }
    let [x, z] = ground.origin;
    let mut map = format!(
        "north up, x from {x}, z from {z}; . path  t track  = street  : kerb  ~ gutter  + square  % crops  h hedge\n"
    );
    for row in rows {
        map.push_str(row.iter().collect::<String>().trim_end());
        map.push('\n');
    }
    map + &legend
}

/// A line for each two footprints that share a column.
pub fn overlaps(footprints: &[Footprint]) -> Vec<String> {
    let mut found = Vec::new();
    for (i, a) in footprints.iter().enumerate() {
        for b in &footprints[i + 1..] {
            let apart = (0..2).any(|k| {
                a.corner[k] + a.size[k] <= b.corner[k] || b.corner[k] + b.size[k] <= a.corner[k]
            });
            if !apart {
                found.push(format!(
                    "{} at {:?} overlaps {} at {:?}",
                    a.name, a.corner, b.name, b.corner
                ));
            }
        }
    }
    found
}
