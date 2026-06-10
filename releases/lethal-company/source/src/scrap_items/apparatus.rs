// Sources: vault/scrap_items/apparatus.md, vault/item_index_pages/scrap.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::interior::InteriorApparatusRemovedEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::outdoor_entity_pages::old_bird::{
    OldBird, OldBirdActivatedEvent, OldBirdState, OLD_BIRD_ROAM_SPEED,
};
use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick, UnitStats};

pub const APPARATUS_ID: &str = "apparatus";
pub const APPARATUS_NAME: &str = "Apparatus";
pub const APPARATUS_TYPE: &str = "scrap_items";
pub const APPARATUS_SUBTYPE: &str = "special";
pub const APPARATUS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Apparatus";
pub const APPARATUS_SOURCE_REVISION: u32 = 21211;
pub const APPARATUS_EXTRACTED_AT: &str = "2026-06-07";
pub const APPARATUS_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const APPARATUS_EFFECTS: &str =
    "Shuts off facility power; 70% chance to increase entity spawning; activates all Old Birds on the moon.";
pub const APPARATUS_WEIGHT: I32F32 = I32F32::lit("31");
pub const APPARATUS_CONDUCTIVE: bool = true;
pub const APPARATUS_SELL_VALUE: I32F32 = I32F32::lit("80");
pub const APPARATUS_TWO_HANDED: bool = true;
pub const APPARATUS_MIN_VALUE: I32F32 = I32F32::lit("16");
pub const APPARATUS_MAX_VALUE: I32F32 = I32F32::lit("47");
pub const APPARATUS_ENTITY_SPAWN_INCREASE_CHANCE_PERCENT: u32 = 70;
pub const APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL: u8 = 2;
pub const APPARATUS_SPAWN_ROLL_SALT: u64 = 0x6170_7061_7261_7473;

pub const APPARATUS_ALIASES: [&str; 2] = ["Apparatice", "Lung"];

pub const APPARATUS_DEPENDS_ON: [&str; 10] = [
    "scrap",
    "lethal_company",
    "factory_interior",
    "mansion",
    "mineshaft",
    "old_bird",
    "turret",
    "landmine",
    "spike_trap",
    "terminal",
];

pub const APPARATUS_SPAWN_CHANCES: [ApparatusSpawnChance; 11] = [
    ApparatusSpawnChance {
        moon: "experimentation",
        chance: I32F32::lit("98.68"),
    },
    ApparatusSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("87.46"),
    },
    ApparatusSpawnChance {
        moon: "vow",
        chance: I32F32::lit("60.61"),
    },
    ApparatusSpawnChance {
        moon: "offense",
        chance: I32F32::lit("17.39"),
    },
    ApparatusSpawnChance {
        moon: "march",
        chance: I32F32::lit("100"),
    },
    ApparatusSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("84.27"),
    },
    ApparatusSpawnChance {
        moon: "rend",
        chance: I32F32::lit("1.5"),
    },
    ApparatusSpawnChance {
        moon: "dine",
        chance: I32F32::lit("2.16"),
    },
    ApparatusSpawnChance {
        moon: "titan",
        chance: I32F32::lit("63.56"),
    },
    ApparatusSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("14.95"),
    },
    ApparatusSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("84.75"),
    },
];

pub const APPARATUS_BEHAVIORAL_MECHANICS: [ApparatusBehaviorRule; 6] = [
    ApparatusBehaviorRule {
        condition: "the Apparatus is removed from its socket",
        outcome: "facility lights flicker, terminal-operated doors open, and internal power shuts off permanently",
    },
    ApparatusBehaviorRule {
        condition: "the Apparatus is disconnected",
        outcome: "all old_birds on the moon activate immediately",
    },
    ApparatusBehaviorRule {
        condition: "the Apparatus is disconnected",
        outcome: "there is a 70% chance that the minimum spawned entity count becomes 2 for the next spawn cycles",
    },
    ApparatusBehaviorRule {
        condition: "the Apparatus is disconnected",
        outcome: "a radiation warning is sent to all players, and the radiation has no effect on the player",
    },
    ApparatusBehaviorRule {
        condition: "a turret, landmine, or spike_trap is present",
        outcome: "it continues functioning normally and can still be temporarily disabled with the ship's terminal",
    },
    ApparatusBehaviorRule {
        condition: "the Apparatus is removed from its slot",
        outcome: "it cannot be reinserted",
    },
];

pub struct ApparatusPlugin;

impl Plugin for ApparatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnApparatusEvent>()
            .add_event::<ApparatusDisconnectedEvent>()
            .add_event::<ApparatusSpawnPressureRollEvent>()
            .add_event::<ApparatusRadiationWarningEvent>()
            .add_event::<ApparatusSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_apparatus,
                    apparatus_pickup_item_bar_bridge,
                    apparatus_disconnect_on_first_pickup,
                    apparatus_activate_old_birds,
                    apparatus_sell_bridge,
                    apparatus_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApparatusBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApparatusSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Apparatus {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApparatusScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub sell_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ApparatusScrap {
    fn default() -> Self {
        Self {
            min_value: APPARATUS_MIN_VALUE,
            max_value: APPARATUS_MAX_VALUE,
            sell_value: APPARATUS_SELL_VALUE,
            weight: APPARATUS_WEIGHT,
            conductive: APPARATUS_CONDUCTIVE,
            two_handed: APPARATUS_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApparatusHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApparatusSocketState {
    pub removed_from_socket: bool,
    pub disconnected_tick: u64,
    pub spawn_pressure_roll_succeeded: bool,
    pub old_birds_activated: u32,
}

#[derive(Bundle)]
pub struct ApparatusBundle {
    pub name: Name,
    pub apparatus: Apparatus,
    pub scrap: ApparatusScrap,
    pub position: SimPosition,
    pub held_by: ApparatusHeldBy,
    pub socket_state: ApparatusSocketState,
}

impl ApparatusBundle {
    pub fn new(event: SpawnApparatusEvent) -> Self {
        Self {
            name: Name::new(APPARATUS_NAME),
            apparatus: Apparatus {
                stable_id: event.stable_id,
            },
            scrap: ApparatusScrap::default(),
            position: event.position,
            held_by: ApparatusHeldBy::default(),
            socket_state: ApparatusSocketState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnApparatusEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApparatusDisconnectedEvent {
    pub apparatus_entity: Entity,
    pub apparatus_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
    pub spawn_pressure_roll_succeeded: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApparatusSpawnPressureRollEvent {
    pub apparatus_stable_id: u64,
    pub succeeded: bool,
    pub minimum_spawned_entity_count: u8,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApparatusRadiationWarningEvent {
    pub apparatus_stable_id: u64,
    pub employee_id: u64,
    pub harmless_to_player: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApparatusSoldEvent {
    pub apparatus_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn apparatus_value_range() -> (I32F32, I32F32) {
    (APPARATUS_MIN_VALUE, APPARATUS_MAX_VALUE)
}

pub fn apparatus_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    APPARATUS_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_apparatus(mut commands: Commands, mut events: EventReader<SpawnApparatusEvent>) {
    for event in events.read() {
        commands.spawn(ApparatusBundle::new(*event));
    }
}

fn apparatus_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    apparatuses: Query<(&Apparatus, &ApparatusHeldBy), Changed<ApparatusHeldBy>>,
) {
    for (apparatus, held_by) in &apparatuses {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: APPARATUS_ID,
                two_handed: APPARATUS_TWO_HANDED,
                functional: false,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = apparatus.stable_id;
        }
    }
}

fn apparatus_disconnect_on_first_pickup(
    mut disconnected_events: EventWriter<ApparatusDisconnectedEvent>,
    mut spawn_roll_events: EventWriter<ApparatusSpawnPressureRollEvent>,
    mut radiation_events: EventWriter<ApparatusRadiationWarningEvent>,
    mut interior_events: EventWriter<InteriorApparatusRemovedEvent>,
    mut apparatuses: Query<(
        Entity,
        &Apparatus,
        &ApparatusHeldBy,
        &SimPosition,
        &mut ApparatusSocketState,
    )>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
) {
    for (entity, apparatus, held_by, position, mut socket_state) in &mut apparatuses {
        if !held_by.is_held || socket_state.removed_from_socket {
            continue;
        }

        let salt = APPARATUS_SPAWN_ROLL_SALT ^ apparatus.stable_id;
        let mut rng = tick_rng(seed.0, tick.0, salt);
        let spawn_pressure_roll_succeeded =
            rng.next_u32() % 100 < APPARATUS_ENTITY_SPAWN_INCREASE_CHANCE_PERCENT;

        socket_state.removed_from_socket = true;
        socket_state.disconnected_tick = tick.0;
        socket_state.spawn_pressure_roll_succeeded = spawn_pressure_roll_succeeded;

        interior_events.send(InteriorApparatusRemovedEvent {
            from_factory_socket: true,
        });

        spawn_roll_events.send(ApparatusSpawnPressureRollEvent {
            apparatus_stable_id: apparatus.stable_id,
            succeeded: spawn_pressure_roll_succeeded,
            minimum_spawned_entity_count: if spawn_pressure_roll_succeeded {
                APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL
            } else {
                0
            },
        });

        radiation_events.send(ApparatusRadiationWarningEvent {
            apparatus_stable_id: apparatus.stable_id,
            employee_id: held_by.employee_id,
            harmless_to_player: true,
        });

        disconnected_events.send(ApparatusDisconnectedEvent {
            apparatus_entity: entity,
            apparatus_stable_id: apparatus.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
            spawn_pressure_roll_succeeded,
        });
    }
}

fn apparatus_activate_old_birds(
    mut disconnected_events: EventReader<ApparatusDisconnectedEvent>,
    mut activated_events: EventWriter<OldBirdActivatedEvent>,
    mut apparatuses: Query<(&Apparatus, &mut ApparatusSocketState)>,
    mut old_birds: Query<(Entity, &mut OldBird, &mut OldBirdState, &mut UnitStats)>,
) {
    for event in disconnected_events.read() {
        let mut activated_count = 0_u32;

        for (old_bird_entity, mut old_bird, mut state, mut stats) in &mut old_birds {
            if old_bird.activated {
                continue;
            }

            old_bird.activated = true;
            *state = OldBirdState::Roaming;
            stats.move_speed = OLD_BIRD_ROAM_SPEED;
            activated_count = activated_count.wrapping_add(1);

            activated_events.send(OldBirdActivatedEvent {
                old_bird: old_bird_entity,
                stable_id: old_bird.stable_id,
            });
        }

        for (apparatus, mut socket_state) in &mut apparatuses {
            if apparatus.stable_id == event.apparatus_stable_id {
                socket_state.old_birds_activated =
                    socket_state.old_birds_activated.wrapping_add(activated_count);
            }
        }
    }
}

fn apparatus_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<ApparatusSoldEvent>,
    apparatuses: Query<(&Apparatus, &ApparatusScrap)>,
) {
    for event in sell_events.read() {
        for (apparatus, scrap) in &apparatuses {
            if apparatus.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(ApparatusSoldEvent {
                apparatus_stable_id: apparatus.stable_id,
                credit_value: scrap.sell_value,
            });
        }
    }
}

fn apparatus_checksum(
    mut checksum: ResMut<SimChecksumState>,
    apparatuses: Query<(
        &Apparatus,
        &ApparatusScrap,
        &SimPosition,
        &ApparatusHeldBy,
        &ApparatusSocketState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, APPARATUS_ID);
    accumulate_str(&mut checksum, 0x1001, APPARATUS_NAME);
    accumulate_str(&mut checksum, 0x1002, APPARATUS_TYPE);
    accumulate_str(&mut checksum, 0x1003, APPARATUS_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, APPARATUS_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, APPARATUS_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, APPARATUS_EFFECTS);

    checksum.accumulate(APPARATUS_SOURCE_REVISION as u64);
    checksum.accumulate(APPARATUS_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(APPARATUS_WEIGHT.to_bits() as u64);
    checksum.accumulate(APPARATUS_CONDUCTIVE as u64);
    checksum.accumulate(APPARATUS_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(APPARATUS_TWO_HANDED as u64);
    checksum.accumulate(APPARATUS_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(APPARATUS_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(APPARATUS_ENTITY_SPAWN_INCREASE_CHANCE_PERCENT as u64);
    checksum.accumulate(APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL as u64);

    for alias in APPARATUS_ALIASES {
        accumulate_str(&mut checksum, 0x2000, alias);
    }

    for dependency in APPARATUS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x3000, dependency);
    }

    for spawn_chance in APPARATUS_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x4000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in APPARATUS_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, rule.condition);
        accumulate_str(&mut checksum, 0x5001, rule.outcome);
    }

    for (apparatus, scrap, position, held_by, socket_state) in &apparatuses {
        checksum.accumulate(apparatus.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.sell_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(socket_state.removed_from_socket as u64);
        checksum.accumulate(socket_state.disconnected_tick);
        checksum.accumulate(socket_state.spawn_pressure_roll_succeeded as u64);
        checksum.accumulate(socket_state.old_birds_activated as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}