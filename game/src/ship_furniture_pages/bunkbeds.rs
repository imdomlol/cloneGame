// Sources: vault/ship_furniture_pages/bunkbeds.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use crate::sim::SimChecksumState;

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BunkbedsPlacement {
    #[default]
    Placed,
    Stored,
}

#[derive(Component, Default)]
pub struct Bunkbeds;

#[derive(Bundle, Default)]
pub struct BunkbedsBundle {
    pub marker: Bunkbeds,
    pub placement: BunkbedsPlacement,
}

fn bunkbeds_checksum(
    q: Query<&BunkbedsPlacement, With<Bunkbeds>>,
    mut state: ResMut<SimChecksumState>,
) {
    let mut values: Vec<u64> = q
        .iter()
        .map(|p| match p {
            BunkbedsPlacement::Placed => 0,
            BunkbedsPlacement::Stored => 1,
        })
        .collect();
    values.sort_unstable();
    for v in values {
        state.accumulate(v);
    }
}

pub struct BunkbedsPlugin;

impl Plugin for BunkbedsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, bunkbeds_checksum);
    }
}