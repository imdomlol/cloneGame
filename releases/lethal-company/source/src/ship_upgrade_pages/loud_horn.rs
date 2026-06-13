// Sources: vault/ship_upgrade_pages/loud_horn.md

use bevy::prelude::*;
use crate::sim::SimChecksumState;

pub const LOUD_HORN_BUY_COST: u32 = 100;

#[derive(Component, Default)]
pub struct LoudHorn {
    pub cord_held: bool,
}

#[derive(Bundle, Default)]
pub struct LoudHornBundle {
    pub loud_horn: LoudHorn,
}

pub struct LoudHornPlugin;

impl Plugin for LoudHornPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, loud_horn_checksum_system);
    }
}

fn loud_horn_checksum_system(
    horn_query: Query<&LoudHorn>,
    mut checksum: ResMut<SimChecksumState>,
) {
    for horn in horn_query.iter() {
        checksum.accumulate(horn.cord_held as u64);
    }
}