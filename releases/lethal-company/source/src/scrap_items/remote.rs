// Sources: vault/scrap_items/remote.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{SimChecksumState, SimPosition, SimTick};

pub const REMOTE_ID: &str = "remote";
pub const REMOTE_NAME: &str = "Remote";
pub const REMOTE_TYPE: &str = "scrap_items";
pub const REMOTE_SUBTYPE: &str = "remote";
pub const REMOTE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Remote";
pub const REMOTE_SOURCE_REVISION: u32 = 20356;
pub const REMOTE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const REMOTE_CONFIDENCE_BASIS_POINTS: u16 = 98;

pub const REMOTE_EFFECTS: &str = "Can toggle the ship's lights on/off";
pub const REMOTE_WEIGHT: I32F32 = I32F32::lit("0");
pub const REMOTE_CONDUCTIVE: bool = false;
pub const REMOTE_MIN_VALUE: I32F32 = I32F32::lit("20");
pub const REMOTE_MAX_VALUE: I32F32 = I32F32::lit("47");
pub const REMOTE_TWO_HANDED: bool = false;

pub const REMOTE_DEPENDS_ON: [&str; 4] = [
    "the_ship",
    "eyeless_dog",
    "audible_sounds",
    "the_company",
];

pub const REMOTE_SPAWN_CHANCES: [RemoteSpawnChance; 6] = [
    RemoteSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("3.24"),
    },
    RemoteSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("2.53"),
    },
    RemoteSpawnChance {
        moon: "vow",
        chance: I32F32::lit("1.95"),
    },
    RemoteSpawnChance {
        moon: "march",
        chance: I32F32::lit("1.83"),
    },
    RemoteSpawnChance {
        moon: "offense",
        chance: I32F32::lit("1.8"),
    },
    RemoteSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("1.53"),
    },
];

pub const REMOTE_BEHAVIORAL_MECHANICS: [RemoteBehaviorRule; 6] = [
    RemoteBehaviorRule {
        condition: "the player presses LMB",
        outcome: "the Remote toggles the ship's lights between on and off",
    },
    RemoteBehaviorRule {
        condition: "the Remote is used",
        outcome: "it can be activated from any location, including inside the Facility",
    },
    RemoteBehaviorRule {
        condition: "the Remote is used",
        outcome: "its click sound does not alert Eyeless Dogs",
    },
    RemoteBehaviorRule {
        condition: "the Remote is used",
        outcome: "its click sound does not alert other entities that can hear audible_sounds",
    },
    RemoteBehaviorRule {
        condition: "the Remote is sold to the_company",
        outcome: "it can be exchanged for credits",
    },
    RemoteBehaviorRule {
        condition: "the Remote spawns on a moon",
        outcome: "its spawn chance is determined per moon",
    },
];

pub struct RemotePlugin;

impl Plugin for RemotePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnRemoteEvent>()
            .add_event::<RemoteUsedEvent>()
            .add_event::<RemoteToggledShipLightsEvent>()
            .add_event::<RemoteSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_remote,
                    remote_pickup_item_bar_bridge,
                    remote_use_from_item_bar,
                    remote_toggle_ship_lights,
                    remote_sell_bridge,
                    remote_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Remote {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for RemoteScrap {
    fn default() -> Self {
        Self {
            min_value: REMOTE_MIN_VALUE,
            max_value: REMOTE_MAX_VALUE,
            weight: REMOTE_WEIGHT,
            conductive: REMOTE_CONDUCTIVE,
            two_handed: REMOTE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RemoteHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RemoteUseState {
    pub toggles: u64,
    pub last_toggled_tick: u64,
}

#[derive(Bundle)]
pub struct RemoteBundle {
    pub name: Name,
    pub remote: Remote,
    pub scrap: RemoteScrap,
    pub position: SimPosition,
    pub held_by: RemoteHeldBy,
    pub use_state: RemoteUseState,
}

impl RemoteBundle {
    pub fn new(event: SpawnRemoteEvent) -> Self {
        Self {
            name: Name::new(REMOTE_NAME),
            remote: Remote {
                stable_id: event.stable_id,
            },
            scrap: RemoteScrap::default(),
            position: event.position,
            held_by: RemoteHeldBy::default(),
            use_state: RemoteUseState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRemoteEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteUsedEvent {
    pub remote_entity: Entity,
    pub remote_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

/// Emitted when the Remote toggles the ship's lights. ship_system reacts to this.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteToggledShipLightsEvent {
    pub remote_stable_id: u64,
    pub employee_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteSoldEvent {
    pub remote_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn remote_value_range() -> (I32F32, I32F32) {
    (REMOTE_MIN_VALUE, REMOTE_MAX_VALUE)
}

pub fn remote_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    REMOTE_SPAWN_CHANCES
        .iter()
        .find(|sc| sc.moon == moon)
        .map(|sc| sc.chance)
}

fn spawn_remote(mut commands: Commands, mut events: EventReader<SpawnRemoteEvent>) {
    for event in events.read() {
        commands.spawn(RemoteBundle::new(*event));
    }
}

fn remote_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    remotes: Query<(&Remote, &RemoteHeldBy), Changed<RemoteHeldBy>>,
) {
    for (remote, held_by) in &remotes {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: REMOTE_ID,
                two_handed: REMOTE_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = remote.stable_id;
        }
    }
}

fn remote_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut used_events: EventWriter<RemoteUsedEvent>,
    mut remotes: Query<(Entity, &Remote, &RemoteHeldBy, &SimPosition, &mut RemoteUseState)>,
    tick: Res<SimTick>,
) {
    for event in item_events.read() {
        if event.item_id != REMOTE_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (entity, remote, held_by, position, mut use_state) in &mut remotes {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            use_state.toggles = use_state.toggles.wrapping_add(1);
            use_state.last_toggled_tick = tick.0;

            used_events.send(RemoteUsedEvent {
                remote_entity: entity,
                remote_stable_id: remote.stable_id,
                employee_id: event.employee_id,
                position: *position,
            });
        }
    }
}

fn remote_toggle_ship_lights(
    mut used_events: EventReader<RemoteUsedEvent>,
    mut toggle_events: EventWriter<RemoteToggledShipLightsEvent>,
) {
    for event in used_events.read() {
        // Click is silent: no NoiseEmittedEvent per vault spec.
        toggle_events.send(RemoteToggledShipLightsEvent {
            remote_stable_id: event.remote_stable_id,
            employee_id: event.employee_id,
        });
    }
}

fn remote_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<RemoteSoldEvent>,
    remotes: Query<(&Remote, &RemoteScrap)>,
) {
    for event in sell_events.read() {
        for (remote, scrap) in &remotes {
            if remote.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(RemoteSoldEvent {
                remote_stable_id: remote.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn remote_checksum(
    mut checksum: ResMut<SimChecksumState>,
    remotes: Query<(&Remote, &RemoteScrap, &SimPosition, &RemoteHeldBy, &RemoteUseState)>,
) {
    accumulate_str(&mut checksum, 0x1000, REMOTE_ID);
    accumulate_str(&mut checksum, 0x1001, REMOTE_NAME);
    accumulate_str(&mut checksum, 0x1002, REMOTE_TYPE);
    accumulate_str(&mut checksum, 0x1003, REMOTE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, REMOTE_EFFECTS);

    checksum.accumulate(REMOTE_SOURCE_REVISION as u64);
    checksum.accumulate(REMOTE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(REMOTE_WEIGHT.to_bits() as u64);
    checksum.accumulate(REMOTE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(REMOTE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(REMOTE_CONDUCTIVE as u64);
    checksum.accumulate(REMOTE_TWO_HANDED as u64);

    for dependency in REMOTE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in REMOTE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in REMOTE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (remote, scrap, position, held_by, use_state) in &remotes {
        checksum.accumulate(remote.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(use_state.toggles);
        checksum.accumulate(use_state.last_toggled_tick);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}