// Sources: vault/game_mechanics/flat_mode.md

use bevy::prelude::*;

use crate::sim::SimChecksumState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlatModeOccluderKind {
    Mountain,
    Forest,
}

#[derive(Component, Clone, Copy, Default)]
pub struct FlatModeOccluder {
    pub kind: Option<FlatModeOccluderKind>,
    pub rendered_as_flat_colored_cells: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct TerrainCellVisibility {
    pub behind_occluding_feature: bool,
    pub visible: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct BuildNearHiddenAreaAssist {
    pub near_hidden_part_of_map: bool,
    pub easier: bool,
}

#[derive(Resource, Clone, Copy)]
pub struct FlatModeMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub description_summary: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for FlatModeMechanicData {
    fn default() -> Self {
        Self {
            id: "flat_mode",
            name: "Flat mode",
            mechanic_type: "map_visibility_toggle",
            description_summary: "Map-view mechanic that renders mountains and forests as flat colored cells so terrain behind them is visible and building near hidden map areas is easier.",
            depends_on: &[],
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct FlatModeState {
    pub enabled: bool,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct FlatModeMetrics {
    pub occluders_rendered_flat_cells: u64,
    pub terrain_cells_revealed: u64,
    pub build_assist_cells: u64,
}

#[derive(Event, Clone, Copy)]
pub struct FlatModeSetIntentEvent {
    pub enabled: bool,
}

#[derive(Event, Clone, Copy)]
pub struct FlatModeAppliedEvent {
    pub enabled: bool,
}

fn apply_flat_mode_intents_system(
    mut state: ResMut<FlatModeState>,
    mut intents: EventReader<FlatModeSetIntentEvent>,
    mut applied: EventWriter<FlatModeAppliedEvent>,
) {
    for ev in intents.read() {
        if state.enabled == ev.enabled {
            continue;
        }
        state.enabled = ev.enabled;
        applied.send(FlatModeAppliedEvent {
            enabled: state.enabled,
        });
    }
}

fn apply_flat_mode_rendering_rule_system(
    state: Res<FlatModeState>,
    mut metrics: ResMut<FlatModeMetrics>,
    mut occluders: Query<&mut FlatModeOccluder>,
) {
    let mut flat_cells = 0_u64;

    for mut occluder in &mut occluders {
        occluder.rendered_as_flat_colored_cells = state.enabled;
        if occluder.rendered_as_flat_colored_cells {
            flat_cells = flat_cells.saturating_add(1);
        }
    }

    metrics.occluders_rendered_flat_cells = flat_cells;
}

fn apply_flat_mode_visibility_rule_system(
    mut metrics: ResMut<FlatModeMetrics>,
    occluders: Query<&FlatModeOccluder>,
    mut terrain_cells: Query<&mut TerrainCellVisibility>,
) {
    let mut any_flat_cells = false;
    for occluder in &occluders {
        if occluder.rendered_as_flat_colored_cells {
            any_flat_cells = true;
            break;
        }
    }

    let mut revealed = 0_u64;
    for mut cell in &mut terrain_cells {
        if any_flat_cells && cell.behind_occluding_feature {
            cell.visible = true;
        }
        if cell.visible {
            revealed = revealed.saturating_add(1);
        }
    }

    metrics.terrain_cells_revealed = revealed;
}

fn apply_flat_mode_building_assist_rule_system(
    mut metrics: ResMut<FlatModeMetrics>,
    terrain_cells: Query<&TerrainCellVisibility>,
    mut assists: Query<&mut BuildNearHiddenAreaAssist>,
) {
    let mut hidden_terrain_visible = false;
    for cell in &terrain_cells {
        if cell.behind_occluding_feature && cell.visible {
            hidden_terrain_visible = true;
            break;
        }
    }

    let mut easier_count = 0_u64;
    for mut assist in &mut assists {
        assist.easier = hidden_terrain_visible && assist.near_hidden_part_of_map;
        if assist.easier {
            easier_count = easier_count.saturating_add(1);
        }
    }

    metrics.build_assist_cells = easier_count;
}

fn flat_mode_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    state: Res<FlatModeState>,
    metrics: Res<FlatModeMetrics>,
    occluders: Query<&FlatModeOccluder>,
    terrain_cells: Query<&TerrainCellVisibility>,
    assists: Query<&BuildNearHiddenAreaAssist>,
) {
    checksum.accumulate(u64::from(state.enabled));
    checksum.accumulate(metrics.occluders_rendered_flat_cells);
    checksum.accumulate(metrics.terrain_cells_revealed);
    checksum.accumulate(metrics.build_assist_cells);

    for occluder in &occluders {
        let kind_bits = match occluder.kind {
            Some(FlatModeOccluderKind::Mountain) => 1_u64,
            Some(FlatModeOccluderKind::Forest) => 2_u64,
            None => 0_u64,
        };
        checksum.accumulate(kind_bits);
        checksum.accumulate(u64::from(occluder.rendered_as_flat_colored_cells));
    }

    for cell in &terrain_cells {
        checksum.accumulate(u64::from(cell.behind_occluding_feature));
        checksum.accumulate(u64::from(cell.visible));
    }

    for assist in &assists {
        checksum.accumulate(u64::from(assist.near_hidden_part_of_map));
        checksum.accumulate(u64::from(assist.easier));
    }
}

pub struct FlatModePlugin;

impl Plugin for FlatModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlatModeMechanicData>()
            .init_resource::<FlatModeState>()
            .init_resource::<FlatModeMetrics>()
            .add_event::<FlatModeSetIntentEvent>()
            .add_event::<FlatModeAppliedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_flat_mode_intents_system,
                    apply_flat_mode_rendering_rule_system,
                    apply_flat_mode_visibility_rule_system,
                    apply_flat_mode_building_assist_rule_system,
                    flat_mode_checksum_system,
                )
                    .chain(),
            );
    }
}