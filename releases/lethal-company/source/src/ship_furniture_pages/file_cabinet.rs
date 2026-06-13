// Sources: vault/ship_furniture_pages/file_cabinet.md, vault/outdoor_entity_pages/eyeless_dog.md

use bevy::prelude::*;
use crate::sim::SimChecksumState;

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileCabinetPlacement {
    #[default]
    Placed,
    Stored,
}

#[derive(Component, Default)]
pub struct FileCabinet;

#[derive(Bundle, Default)]
pub struct FileCabinetBundle {
    pub marker: FileCabinet,
    pub placement: FileCabinetPlacement,
}

fn file_cabinet_checksum(
    q: Query<&FileCabinetPlacement, With<FileCabinet>>,
    mut state: ResMut<SimChecksumState>,
) {
    let mut values: Vec<u64> = q
        .iter()
        .map(|p| match p {
            FileCabinetPlacement::Placed => 0,
            FileCabinetPlacement::Stored => 1,
        })
        .collect();
    values.sort_unstable();
    for v in values {
        state.accumulate(v);
    }
}

pub struct FileCabinetPlugin;

impl Plugin for FileCabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, file_cabinet_checksum);
    }
}