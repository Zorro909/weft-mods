//! Which buildings go where: the heart's own, the landmarks, the homes along the fronts and
//! the mill outside.

use structgen_api::{Fit, Plan};
use weft_sdk::Template;

use super::frontage::{Faces, Front, Rules, Site, facing, next_site, stand_round};
use super::grid::{Cell, Grid};
use super::roads::Road;
use super::shape::{Point, add, heading, scale};
use super::{Kind, Settlement, one_of, planting};
use crate::worldgen::SEA;

/// The heart's own buildings, in it along the main road: the first in the middle.
pub fn centre(p: &mut Plan, grid: &mut Grid, s: &Settlement, m: Point, angle: f32) -> Vec<Site> {
    let rules = Rules {
        setback: [0, 0],
        gap: 2,
        fit: Fit::new().max_slope(3).min_height(SEA),
        path: Cell::Free,
    };
    let along = heading(angle);
    let mut sites = Vec::new();
    for (i, &template) in s.centre.iter().enumerate() {
        let turn = facing(scale(along, if i % 2 == 0 { 1.0 } else { -1.0 }));
        let offsets = [0.0, 10.0, -10.0, 16.0, -16.0];
        let mut try_at = |offset| {
            stand_round(
                p,
                grid,
                &rules,
                template,
                add(m, scale(along, offset)),
                turn,
            )
        };
        sites.extend(offsets.into_iter().find_map(&mut try_at));
    }
    sites
}

/// Each landmark at the first spot with room along a front of its kind; `next` moves past it.
pub fn landmarks(
    p: &mut Plan,
    grid: &mut Grid,
    s: &Settlement,
    rules: &Rules,
    fronts: &[Front],
    next: &mut [usize],
) -> Vec<Site> {
    let mut sites = Vec::new();
    for &(template, faces) in s.landmarks {
        let mut ours = fronts
            .iter()
            .zip(next.iter_mut())
            .filter(|(front, _)| front.faces == faces);
        let found = ours.find_map(|(front, from)| {
            let placed = next_site(p, grid, rules, front, *from, &mut |_, _| template)?;
            Some((placed, from))
        });
        if let Some(((placed, after), from)) = found {
            *from = after;
            sites.push(placed);
        }
    }
    sites
}

/// Homes, one to each front in turn from where `next` says, until there are `s.homes` or no
/// front has room left: houses, and farms where `far` says the farmland starts. A village house
/// gets a kitchen garden.
pub fn homes(
    p: &mut Plan,
    grid: &mut Grid,
    s: &Settlement,
    rules: &Rules,
    fronts: &[Front],
    next: &mut [usize],
    far: &dyn Fn(Point) -> bool,
) -> Vec<Site> {
    let mut pick = |p: &mut Plan, at: Point| {
        let farm = far(at) && !s.farms.is_empty();
        one_of(p, if farm { s.farms } else { s.houses }).expect("a settlement has houses")
    };
    let mut sites = Vec::new();
    let room = |next: &[usize]| {
        next.iter()
            .zip(fronts)
            .any(|(&i, front)| i < front.points.len())
    };
    while sites.len() < s.homes && room(next) {
        for (front, from) in fronts.iter().zip(next.iter_mut()) {
            if sites.len() == s.homes {
                break;
            }
            let Some((placed, after)) = next_site(p, grid, rules, front, *from, &mut pick) else {
                *from = front.points.len();
                continue;
            };
            if s.kind == Kind::Village {
                planting::garden(grid, &placed);
            }
            *from = after;
            sites.push(placed);
        }
    }
    sites
}

/// `template` past one end or the other of the main road, its front toward it, along a way
/// from the road's end as a building along a road has.
pub fn outskirts(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    template: &'static Template,
    main: &[Road],
) -> Option<Site> {
    main.iter().filter_map(Road::end).find_map(|end| {
        let front = Front {
            points: vec![end],
            reach: 2.0,
            faces: Faces::Road,
        };
        let (placed, _) = next_site(p, grid, rules, &front, 0, &mut |_, _| template)?;
        Some(placed)
    })
}
