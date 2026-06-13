// Sources: vault/store_items/extension_ladder.md, vault/item_index_pages/items.md

use bevy::prelude::*;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 60;
pub const SELL_VALUE: u32 = 0;
pub const CONDUCTIVE: bool = true;

const COLLAPSE_DELAY_TICKS: u32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LadderPhase {
    Extending = 0,
    Fallen = 1,
    Collapsed = 2,
}

#[derive(Component, Default)]
pub struct ExtensionLadder {
    pub phase: LadderPhase,
    pub ticks_fallen: u32,
    pub blocked_by_ship_roof: bool,
}

impl Default for LadderPhase {
    fn default() -> Self {
        LadderPhase::Extending
    }
}

#[derive(Event)]
pub struct ExtensionLadderPurchasedEvent;

#[derive(Event)]
pub struct ExtensionLadderFallTriggeredEvent {
    pub ladder_entity: Entity,
    pub inside_ship: bool,
}

#[derive(Event)]
pub struct ExtensionLadderKillsBelowEvent {
    pub ladder_entity: Entity,
}

#[derive(Event)]
pub struct ExtensionLadderCollapsedEvent {
    pub ladder_entity: Entity,
}

#[derive(Resource, Default)]
pub struct ExtensionLadderState {
    pub owned: u32,
}

pub struct ExtensionLadderPlugin;

impl Plugin for ExtensionLadderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExtensionLadderPurchasedEvent>()
            .add_event::<ExtensionLadderFallTriggeredEvent>()
            .add_event::<ExtensionLadderKillsBelowEvent>()
            .add_event::<ExtensionLadderCollapsedEvent>()
            .init_resource::<ExtensionLadderState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_fall_triggered,
                    tick_ladders,
                    extension_ladder_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<ExtensionLadderPurchasedEvent>,
    mut state: ResMut<ExtensionLadderState>,
) {
    for _ in events.read() {
        state.owned += 1;
    }
}

fn handle_fall_triggered(
    mut events: EventReader<ExtensionLadderFallTriggeredEvent>,
    mut ladders: Query<&mut ExtensionLadder>,
    mut kill_events: EventWriter<ExtensionLadderKillsBelowEvent>,
) {
    for ev in events.read() {
        let Ok(mut ladder) = ladders.get_mut(ev.ladder_entity) else {
            continue;
        };
        if ladder.phase != LadderPhase::Extending {
            continue;
        }
        ladder.phase = LadderPhase::Fallen;
        ladder.blocked_by_ship_roof = ev.inside_ship;
        if !ev.inside_ship {
            kill_events.send(ExtensionLadderKillsBelowEvent {
                ladder_entity: ev.ladder_entity,
            });
        }
    }
}

fn tick_ladders(
    mut ladders: Query<(Entity, &mut ExtensionLadder)>,
    mut collapsed_events: EventWriter<ExtensionLadderCollapsedEvent>,
) {
    for (entity, mut ladder) in ladders.iter_mut() {
        if ladder.phase != LadderPhase::Fallen {
            continue;
        }
        ladder.ticks_fallen += 1;
        if ladder.ticks_fallen >= COLLAPSE_DELAY_TICKS {
            ladder.phase = LadderPhase::Collapsed;
            collapsed_events.send(ExtensionLadderCollapsedEvent {
                ladder_entity: entity,
            });
        }
    }
}

fn extension_ladder_checksum(
    state: Res<ExtensionLadderState>,
    ladders: Query<&ExtensionLadder>,
    mut cs: ResMut<SimChecksumState>,
) {
    cs.accumulate(state.owned as u64);
    let mut entries: Vec<(u8, u32, u8)> = ladders
        .iter()
        .map(|l| (l.phase as u8, l.ticks_fallen, l.blocked_by_ship_roof as u8))
        .collect();
    entries.sort();
    for (phase, ticks, blocked) in entries {
        cs.accumulate(phase as u64);
        cs.accumulate(ticks as u64);
        cs.accumulate(blocked as u64);
    }
}