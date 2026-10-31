//! Current accepted voice intervention only. Source byte ranges, no deleted text,
//! no passage history. Never produce a target before the native commit succeeds.
use super::note::Patch;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeArea {
    pub from: usize,
    pub to: usize,
    pub kind: &'static str,
    pub removed_characters: usize,
}
#[derive(Clone, Serialize)]
pub struct ChangeFeedback {
    pub revision: u64,
    pub areas: Vec<ChangeArea>,
}

pub fn track_patch(areas: &mut Vec<ChangeArea>, patch: &Patch, kind: &'static str) {
    let shift = patch.text.len() as isize - (patch.to - patch.from) as isize;
    for area in areas.iter_mut() {
        if area.from >= patch.to {
            area.from = area.from.saturating_add_signed(shift);
            area.to = area.to.saturating_add_signed(shift);
        } else if area.to > patch.from {
            // A later operation edits an earlier result: keep its surviving span.
            area.from = area.from.min(patch.from);
            area.to = if area.to >= patch.to {
                area.to.saturating_add_signed(shift)
            } else {
                patch.from + patch.text.len()
            };
        }
    }
    areas.push(ChangeArea {
        from: patch.from,
        to: patch.from + patch.text.len(),
        kind,
        removed_characters: patch.expected.chars().count(),
    });
}
