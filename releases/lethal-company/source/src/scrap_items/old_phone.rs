// Sources: vault/scrap_items/old_phone.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::credits::SellScrapForCreditsEvent;
use crate::gameplay_mechanics::item_bar::ItemBarPickupEvent;
use crate::sim::{SimChecksumState, SimPosition};

pub const OLD_PHONE_ID: &str = "old_phone";
pub const OLD_PHONE_NAME: &str = "Old phone";
pub const OLD_PHONE_TYPE: &str = "scrap_items";
pub const OLD_PHONE_SUBTYPE: &str = "item";
pub const OLD_PHONE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Old_Phone";
pub const OLD_PHONE_SOURCE_REVISION: u32 = 20358;
pub const OLD_PHONE_EXTRACTED_AT: &str = "2026-06-07";
pub const OLD_PHONE_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const OLD_PHONE_EFFECTS: &str = "Plays a woman's scream when picked up";
pub const OLD_PHONE_WEIGHT: I32F32 = I32F32::lit("5");
pub const OLD_PHONE_CONDUCTIVE: bool = false;
pub const OLD_PHONE_MIN_VALUE: I32F32 = I32F32::lit("48");
pub const OLD_PHONE_MAX_VALUE: I32F32 = I32F32::lit("63");
pub const OLD_PHONE_TWO_HANDED: bool = false;
pub const OLD_PHONE_WIKI_ID: u32 = 43;
pub const OLD_PHONE_USAGE_NOTE: &str = "Can be sold for credits.";

pub const OLD_PHONE_DEPENDS_ON: [&str; 0] = [];

pub const OLD_PHONE_SPAWN_CHANCES: [OldPhoneSpawnChance; 8] = [
    OldPhoneSpawnChance {
        moon: "titan",
        chance: I32F32::lit("2.37"),
    },
    OldPhoneSpawnChance {
        moon: "artifice",
        chance: I32F32::lit("1.85"),
    },
    OldPhoneSpawnChance {
        moon: "rend",
        chance: I32F32::lit("1.58"),
    },
    OldPhoneSpawnChance {
        moon: "offense",
        chance: I32F32::lit("0.96"),
    },
    OldPhoneSpawnChance {
        moon: "embrion",
        chance: I32F32::lit("0.93"),
    },
    OldPhoneSpawnChance {
        moon: "assurance",
        chance: I32F32::lit("0.82"),
    },
    OldPhoneSpawnChance {
        moon: "vow",
        chance: I32F32::lit("0.72"),
    },
    OldPhoneSpawnChance {
        moon: "adamance",
        chance: I32F32::lit("0.42"),
    },
];

pub const OLD_PHONE_BEHAVIORAL_MECHANICS: [OldPhoneBehaviorRule; 2] = [
    OldPhoneBehaviorRule {
        condition: "picked up or equipped",
        outcome: "it plays a woman's scream followed by a long audible disconnect tone",
    },
    OldPhoneBehaviorRule {
        condition: "picked up or equipped",
        outcome: "it has no known functional effect",
    },
];

pub struct OldPhonePlugin;

impl Plugin for OldPhonePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnOldPhoneEvent>()
            .add_event::<OldPhoneScreamEvent>()
            .add_event::<OldPhoneSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_old_phone,
                    old_phone_pickup_item_bar_bridge,
                    old_phone_sell_bridge,
                    old_phone_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldPhoneBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldPhoneSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldPhone {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldPhoneScrap {
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for OldPhoneScrap {
    fn default() -> Self {
        Self {
            min_value: OLD_PHONE_MIN_VALUE,
            max_value: OLD_PHONE_MAX_VALUE,
            weight: OLD_PHONE_WEIGHT,
            conductive: OLD_PHONE_CONDUCTIVE,
            two_handed: OLD_PHONE_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldPhoneHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OldPhoneSoundState {
    pub screams_played: u64,
    pub disconnect_tones_played: u64,
}

#[derive(Bundle)]
pub struct OldPhoneBundle {
    pub name: Name,
    pub old_phone: OldPhone,
    pub scrap: OldPhoneScrap,
    pub position: SimPosition,
    pub held_by: OldPhoneHeldBy,
    pub sound_state: OldPhoneSoundState,
}

impl OldPhoneBundle {
    pub fn new(event: SpawnOldPhoneEvent) -> Self {
        Self {
            name: Name::new(OLD_PHONE_NAME),
            old_phone: OldPhone {
                stable_id: event.stable_id,
            },
            scrap: OldPhoneScrap::default(),
            position: event.position,
            held_by: OldPhoneHeldBy::default(),
            sound_state: OldPhoneSoundState::default(),
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnOldPhoneEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OldPhoneScreamEvent {
    pub old_phone_entity: Entity,
    pub old_phone_stable_id: u64,
    pub employee_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OldPhoneSoldEvent {
    pub old_phone_stable_id: u64,
    pub credit_value: I32F32,
}

pub fn old_phone_value_range() -> (I32F32, I32F32) {
    (OLD_PHONE_MIN_VALUE, OLD_PHONE_MAX_VALUE)
}

pub fn old_phone_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    OLD_PHONE_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

fn spawn_old_phone(mut commands: Commands, mut events: EventReader<SpawnOldPhoneEvent>) {
    for event in events.read() {
        commands.spawn(OldPhoneBundle::new(*event));
    }
}

fn old_phone_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    mut scream_events: EventWriter<OldPhoneScreamEvent>,
    mut old_phones: Query<
        (
            Entity,
            &OldPhone,
            &OldPhoneHeldBy,
            &SimPosition,
            &mut OldPhoneSoundState,
        ),
        Changed<OldPhoneHeldBy>,
    >,
) {
    for (entity, old_phone, held_by, position, mut sound_state) in &mut old_phones {
        if !held_by.is_held {
            continue;
        }

        pickup_events.send(ItemBarPickupEvent {
            employee_id: held_by.employee_id,
            item_id: OLD_PHONE_ID,
            two_handed: OLD_PHONE_TWO_HANDED,
            functional: false,
            passive: true,
            from_store_or_valueless: false,
        });

        sound_state.screams_played = sound_state.screams_played.wrapping_add(1);
        sound_state.disconnect_tones_played = sound_state.disconnect_tones_played.wrapping_add(1);

        scream_events.send(OldPhoneScreamEvent {
            old_phone_entity: entity,
            old_phone_stable_id: old_phone.stable_id,
            employee_id: held_by.employee_id,
            position: *position,
        });
    }
}

fn old_phone_sell_bridge(
    mut sell_events: EventReader<SellScrapForCreditsEvent>,
    mut sold_events: EventWriter<OldPhoneSoldEvent>,
    old_phones: Query<(&OldPhone, &OldPhoneScrap)>,
) {
    for event in sell_events.read() {
        for (old_phone, scrap) in &old_phones {
            if old_phone.stable_id != event.scrap_entity_id {
                continue;
            }

            sold_events.send(OldPhoneSoldEvent {
                old_phone_stable_id: old_phone.stable_id,
                credit_value: scrap.max_value,
            });
        }
    }
}

fn old_phone_checksum(
    mut checksum: ResMut<SimChecksumState>,
    old_phones: Query<(
        &OldPhone,
        &OldPhoneScrap,
        &SimPosition,
        &OldPhoneHeldBy,
        &OldPhoneSoundState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, OLD_PHONE_ID);
    accumulate_str(&mut checksum, 0x1001, OLD_PHONE_NAME);
    accumulate_str(&mut checksum, 0x1002, OLD_PHONE_TYPE);
    accumulate_str(&mut checksum, 0x1003, OLD_PHONE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, OLD_PHONE_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, OLD_PHONE_USAGE_NOTE);
    accumulate_str(&mut checksum, 0x1006, OLD_PHONE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1007, OLD_PHONE_EXTRACTED_AT);

    checksum.accumulate(OLD_PHONE_SOURCE_REVISION as u64);
    checksum.accumulate(OLD_PHONE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(OLD_PHONE_WEIGHT.to_bits() as u64);
    checksum.accumulate(OLD_PHONE_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(OLD_PHONE_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(OLD_PHONE_CONDUCTIVE as u64);
    checksum.accumulate(OLD_PHONE_TWO_HANDED as u64);
    checksum.accumulate(OLD_PHONE_WIKI_ID as u64);

    for dependency in OLD_PHONE_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for spawn_chance in OLD_PHONE_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x3000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in OLD_PHONE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (old_phone, scrap, position, held_by, sound_state) in &old_phones {
        checksum.accumulate(old_phone.stable_id);
        checksum.accumulate(scrap.min_value.to_bits() as u64);
        checksum.accumulate(scrap.max_value.to_bits() as u64);
        checksum.accumulate(scrap.weight.to_bits() as u64);
        checksum.accumulate(scrap.conductive as u64);
        checksum.accumulate(scrap.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(sound_state.screams_played);
        checksum.accumulate(sound_state.disconnect_tones_played);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt);
    for byte in value.as_bytes() {
        checksum.accumulate(*byte as u64);
    }
}