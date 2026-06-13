// Sources: vault/ship_furniture_pages/cupboard.md

use bevy::prelude::*;
use crate::sim::SimChecksumState;

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CupboardPlacement {
    #[default]
    Placed,
    Stored,
}

/// Number of one-handed items or scrap currently sitting on the cupboard shelves.
/// Items do not follow the cupboard if it is moved; this count resets on move.
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CupboardStoredCount(pub u32);

#[derive(Component, Default)]
pub struct Cupboard;

#[derive(Bundle, Default)]
pub struct CupboardBundle {
    pub marker: Cupboard,
    pub placement: CupboardPlacement,
    pub stored_count: CupboardStoredCount,
}

fn cupboard_checksum(
    q: Query<(&CupboardPlacement, &CupboardStoredCount), With<Cupboard>>,
    mut state: ResMut<SimChecksumState>,
) {
    let mut values: Vec<u64> = q
        .iter()
        .map(|(p, s)| {
            let placement_bit: u64 = match p {
                CupboardPlacement::Placed => 0,
                CupboardPlacement::Stored => 1,
            };
            (placement_bit << 32) | (s.0 as u64)
        })
        .collect();
    values.sort_unstable();
    for v in values {
        state.accumulate(v);
    }
}

pub struct CupboardPlugin;

impl Plugin for CupboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, cupboard_checksum);
    }
}