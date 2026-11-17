//! The thumper's screen: its energy, what it is doing, the button that starts a survey, and what
//! the last survey found: a table of ore by kind and band, and the oil fields it heard.

use weft_sdk::{Arg, ClientView, S, TextOut, Ui, item_name};

use super::oil::REACH;
use super::survey::{BANDS, KINDS};
use super::{FIELDS, START, SURVEY, Start, Status, Survey};
use crate::ghost::item_of;

/// Widths of the table's columns: the kind of ore, then each band.
const NAME_WIDTH: f32 = 90.0;
const BAND_WIDTH: f32 = 34.0;

/// The thumper's screen:
///
/// ```text
/// +-----------------------------------------------------------------------+
/// | Seismic Thumper                                                       |
/// | [#####.....]                                                          |  stored energy
/// | 6000 / 10000 energy                                                   |
/// | Using 500 energy a tick                                               |  drawn last tick
/// | Counting ore: layer 40 of 96                                          |  status
/// | [  Survey  ]  32 by 32 blocks, from y 63 down to y -32                |  its area
/// | Ore            y63   y47   y31   y15   y-1   y-17                     |  bands from the top
/// | Iron Ore        12    40    88     0     3     0                      |  a row a kind
/// | Coal Ore       130    41     0     0     0     0                      |
/// | Oil fields within 256 blocks                                          |
/// | 140 blocks north-east: oil from y 18 to y 6                           |  the nearest first
/// +-----------------------------------------------------------------------+
/// ```
pub fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        crate::machine::power_bar(ui);
        crate::machine::meter_line(ui);
        ui.text_bind(|view, out| status_line(&survey(view), out));
        ui.row(S::row().gap(8.0), |ui| {
            ui.button(
                "cogwork.seismic_thumper.survey",
                S::default().size(80.0, 20.0),
                start,
            );
            ui.text_bind(|view, out| area_line(&survey(view), out));
        });
        table(ui);
        oil_lines(ui);
    });
}

/// The survey of the thumper whose menu is open.
fn survey(view: &ClientView) -> Survey {
    view.menu_component(&SURVEY).unwrap_or_default()
}

/// Client side: asks the server to start a survey on the thumper whose screen is open.
fn start(view: &ClientView) {
    if let Some(owner) = view.menu_owner() {
        view.client().send(&START, &Start { owner });
    }
}

/// An empty cell of the table.
fn blank(out: &mut TextOut) {
    out.set("cogwork.seismic_thumper.cell", &[Arg::Text("")]);
}

/// What the thumper is doing, with how far it has got.
fn status_line(survey: &Survey, out: &mut TextOut) {
    let status = survey.status();
    let name: &str = status.into();
    let key = format!("cogwork.seismic_thumper.{name}");
    let layer_size = survey.total() / survey.layers.max(1);
    let args = match status {
        Status::Thumping => vec![Arg::Int(survey.spent), Arg::Int(super::work::SURVEY_COST)],
        Status::Counting => {
            let layer = survey.counted / layer_size.max(1) + 1;
            vec![Arg::Int(layer.into()), Arg::Int(survey.layers.into())]
        }
        _ => Vec::new(),
    };
    out.set(&key, &args);
}

/// The area of its last survey.
fn area_line(survey: &Survey, out: &mut TextOut) {
    if survey.layers == 0 {
        blank(out);
        return;
    }
    let [across, _, along] = survey.area().size();
    let [across, along] = [across, along].map(i64::from);
    let args = [across, along, survey.top(), survey.bottom()].map(Arg::Int);
    out.set("cogwork.seismic_thumper.area", &args);
}

/// The table of ore: a row of bands by their top layer, then a row a kind it found.
fn table(ui: &mut Ui) {
    let cell = |width: f32| S::default().w(width).mono();
    ui.row(S::row(), |ui| {
        ui.text_styled(cell(NAME_WIDTH), "cogwork.seismic_thumper.ore", &[]);
        for band in 0..BANDS {
            ui.text_bind_styled(cell(BAND_WIDTH), move |view, out| {
                let survey = survey(view);
                if band >= survey.bands() {
                    return blank(out);
                }
                let top = Arg::Int(survey.band_top(band));
                out.set("cogwork.seismic_thumper.band", &[top]);
            });
        }
    });
    for slot in 0..KINDS {
        ui.row(S::row(), |ui| {
            ui.text_bind_styled(cell(NAME_WIDTH), move |view, out| {
                kind_name(&survey(view), slot, out)
            });
            for band in 0..BANDS {
                ui.text_bind_styled(cell(BAND_WIDTH), move |view, out| {
                    let survey = survey(view);
                    if survey.kinds[slot] == 0 || band >= survey.bands() {
                        return blank(out);
                    }
                    out.set(
                        "cogwork.seismic_thumper.cell",
                        &[Arg::Int(survey.count(slot, band).into())],
                    );
                });
            }
        });
    }
}

/// The name of the `slot`th kind of ore it found, by the first block of it found.
fn kind_name(survey: &Survey, slot: usize, out: &mut TextOut) {
    if survey.kinds[slot] == 0 {
        return blank(out);
    }
    match item_of(survey.shown[slot]) {
        0 => out.set("cogwork.seismic_thumper.some_ore", &[]),
        item => {
            let name = format!("item:{}", item_name(item));
            out.set("cogwork.seismic_thumper.cell", &[Arg::Name(&name)]);
        }
    }
}

/// The oil fields it heard, once it has listened: a line each, or that it heard none.
fn oil_lines(ui: &mut Ui) {
    ui.text_bind(|view, out| {
        let heard = matches!(survey(view).status(), Status::Listening | Status::Done);
        if !heard {
            return blank(out);
        }
        out.set("cogwork.seismic_thumper.oil", &[Arg::Int(REACH.into())]);
    });
    for i in 0..FIELDS {
        ui.text_bind(move |view, out| field_line(&survey(view), i, out));
    }
}

/// The `i`th nearest oil field it heard: how far, which way and how deep.
fn field_line(survey: &Survey, i: usize, out: &mut TextOut) {
    let Some(field) = survey.field(i) else {
        if i == 0 && survey.status() == Status::Done {
            return out.set("cogwork.seismic_thumper.no_oil", &[]);
        }
        return blank(out);
    };
    let way = format!("cogwork.direction.{}", field.direction());
    let (distance, way) = (Arg::Int(field.distance()), Arg::Key(&way));
    match field.depth {
        Some((top, bottom)) => {
            let (top, bottom) = (Arg::Int(top.into()), Arg::Int(bottom.into()));
            out.set(
                "cogwork.seismic_thumper.field",
                &[distance, way, top, bottom],
            );
        }
        None => out.set("cogwork.seismic_thumper.field_somewhere", &[distance, way]),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_line_it_shows_has_a_name() {
        for key in [
            "block.cogwork.seismic_thumper",
            "cogwork.seismic_thumper.survey",
            "cogwork.seismic_thumper.cell",
            "cogwork.seismic_thumper.area",
            "cogwork.seismic_thumper.ore",
            "cogwork.seismic_thumper.band",
            "cogwork.seismic_thumper.some_ore",
            "cogwork.seismic_thumper.oil",
            "cogwork.seismic_thumper.no_oil",
            "cogwork.seismic_thumper.field",
            "cogwork.seismic_thumper.field_somewhere",
        ] {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }
}
