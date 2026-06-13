// Sources: vault/weapon_pages/homemade_flashbang.md, vault/weapon_pages/stun_grenade.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition,
};

pub const HOMEMADE_FLASHBANG_ID: &str = "homemade_flashbang";
pub const HOMEMADE_FLASHBANG_NAME: &str = "Homemade Flashbang";
pub const HOMEMADE_FLASHBANG_TYPE: &str = "weapon_pages";
pub const HOMEMADE_FLASHBANG_SUBTYPE: &str = "consumable_weapon";
pub const HOMEMADE_FLASHBANG_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/DIY-Flashbang";
pub const HOMEMADE_FLASHBANG_SOURCE_REVISION: u32 = 21224;
pub const HOMEMADE_FLASHBANG_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const HOMEMADE_FLASHBANG_CONFIDENCE_BASIS_POINTS: u16 = 91;

pub const HOMEMADE_FLASHBANG_EFFECTS: &str =
    "Damages the user, blinds employees and stuns entities";
pub const HOMEMADE_FLASHBANG_WEIGHT: I32F32 = I32F32::lit("5");
pub const HOMEMADE_FLASHBANG_CONDUCTIVE: bool = false;
pub const HOMEMADE_FLASHBANG_TWO_HANDED: bool = false;
pub const HOMEMADE_FLASHBANG_MIN_VALUE: I32F32 = I32F32::lit("10");
pub const HOMEMADE_FLASHBANG_MAX_VALUE: I32F32 = I32F32::lit("27");
pub const HOMEMADE_FLASHBANG_DAMAGE: I32F32 = I32F32::lit("20");
pub const HOMEMADE_FLASHBANG_WIKI_ID: u32 = 62;

pub const STUN_GRENADE_BASE_STUN_SECONDS: I32F32 = I32F32::lit("5");
pub const HOMEMADE_FLASHBANG_BLIND_TICKS_SECONDS: I32F32 = I32F32::lit("5");
pub const HOMEMADE_FLASHBANG_STUN_BASE_SECONDS: I32F32 = STUN_GRENADE_BASE_STUN_SECONDS;

pub const HOMEMADE_FLASHBANG_SPAWN_CHANCES: [HomemadeFlashbangSpawnChance; 8] = [
    HomemadeFlashbangSpawnChance {
        moon: "Experimentation",
        chance: I32F32::lit("3.87"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Adamance",
        chance: I32F32::lit("2.22"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Assurance",
        chance: I32F32::lit("1.65"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Offense",
        chance: I32F32::lit("1.56"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Vow",
        chance: I32F32::lit("1.44"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Embrion",
        chance: I32F32::lit("1.04"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Titan",
        chance: I32F32::lit("0.97"),
    },
    HomemadeFlashbangSpawnChance {
        moon: "Artifice",
        chance: I32F32::lit("0.96"),
    },
];

pub const HOMEMADE_FLASHBANG_BEHAVIORAL_MECHANICS: [HomemadeFlashbangBehaviorRule; 14] = [
    HomemadeFlashbangBehaviorRule {
        condition: "left click",
        outcome: "pull the pin and detonate the item immediately in the user's hand",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "the item detonates",
        outcome: "deal 20 damage to the user",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "the item detonates",
        outcome: "blind the user and nearby employees",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "the item detonates",
        outcome: "stun nearby entities for a short duration",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "the item detonates",
        outcome: "destroy the item after the explosion",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "compared with stun_grenade",
        outcome: "this item does not remain available for pickup after detonation",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "experimentation",
        outcome: "the spawn chance is 3.87%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "adamance",
        outcome: "the spawn chance is 2.22%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "assurance",
        outcome: "the spawn chance is 1.65%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "offense",
        outcome: "the spawn chance is 1.56%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "vow",
        outcome: "the spawn chance is 1.44%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "embrion",
        outcome: "the spawn chance is 1.04%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "titan",
        outcome: "the spawn chance is 0.97%",
    },
    HomemadeFlashbangBehaviorRule {
        condition: "artifice",
        outcome: "the spawn chance is 0.96%",
    },
];

pub const HOMEMADE_FLASHBANG_STUN_RULES: [HomemadeFlashbangStunRule; 23] = [
    HomemadeFlashbangStunRule {
        target_id: "hygrodere",
        multiplier: I32F32::lit("4.0"),
        stun_seconds: I32F32::lit("20"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "coil_head",
        multiplier: I32F32::lit("3.25"),
        stun_seconds: I32F32::lit("16.25"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "snare_flea",
        multiplier: I32F32::lit("3.0"),
        stun_seconds: I32F32::lit("15"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "barber",
        multiplier: I32F32::lit("1.35"),
        stun_seconds: I32F32::lit("6.75"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "bunker_spider",
        multiplier: I32F32::lit("1.0"),
        stun_seconds: I32F32::lit("5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "thumper",
        multiplier: I32F32::lit("1.0"),
        stun_seconds: I32F32::lit("5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "masked",
        multiplier: I32F32::lit("0.75"),
        stun_seconds: I32F32::lit("3.75"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "spore_lizard",
        multiplier: I32F32::lit("0.6"),
        stun_seconds: I32F32::lit("3"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "jester",
        multiplier: I32F32::lit("0.6"),
        stun_seconds: I32F32::lit("3"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "mask_hornets",
        multiplier: I32F32::lit("0.6"),
        stun_seconds: I32F32::lit("3"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "butler",
        multiplier: I32F32::lit("0.6"),
        stun_seconds: I32F32::lit("3"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "nutcracker",
        multiplier: I32F32::lit("0.5"),
        stun_seconds: I32F32::lit("2.5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "hoarding_bug",
        multiplier: I32F32::lit("0.5"),
        stun_seconds: I32F32::lit("2.5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "bracken",
        multiplier: I32F32::lit("0.25"),
        stun_seconds: I32F32::lit("1.25"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "maneater",
        multiplier: I32F32::lit("0.25"),
        stun_seconds: I32F32::lit("1.25"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "forest_keeper",
        multiplier: I32F32::lit("1.2"),
        stun_seconds: I32F32::lit("6"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "old_bird",
        multiplier: I32F32::lit("1.2"),
        stun_seconds: I32F32::lit("6"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "manticoil",
        multiplier: I32F32::lit("1.0"),
        stun_seconds: I32F32::lit("5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "eyeless_dog",
        multiplier: I32F32::lit("0.7"),
        stun_seconds: I32F32::lit("3.5"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "baboon_hawk",
        multiplier: I32F32::lit("0.4"),
        stun_seconds: I32F32::lit("2"),
        affects_target: true,
    },
    HomemadeFlashbangStunRule {
        target_id: "circuit_bee",
        multiplier: I32F32::ZERO,
        stun_seconds: I32F32::ZERO,
        affects_target: false,
    },
    HomemadeFlashbangStunRule {
        target_id: "earth_leviathan",
        multiplier: I32F32::ZERO,
        stun_seconds: I32F32::ZERO,
        affects_target: false,
    },
    HomemadeFlashbangStunRule {
        target_id: "ghost_girl",
        multiplier: I32F32::ZERO,
        stun_seconds: I32F32::ZERO,
        affects_target: false,
    },
];

pub struct HomemadeFlashbangPlugin;

impl Plugin for HomemadeFlashbangPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHomemadeFlashbangEvent>()
            .add_event::<HomemadeFlashbangPrimedEvent>()
            .add_event::<HomemadeFlashbangDetonatedEvent>()
            .add_event::<HomemadeFlashbangEmployeeBlindedEvent>()
            .add_event::<HomemadeFlashbangEntityStunnedEvent>()
            .add_event::<HomemadeFlashbangDestroyedEvent>()
            .add_event::<HomemadeFlashbangSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_homemade_flashbang,
                    homemade_flashbang_pickup_item_bar_bridge,
                    homemade_flashbang_use_from_item_bar,
                    homemade_flashbang_prime,
                    homemade_flashbang_detonate,
                    homemade_flashbang_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangStunRule {
    pub target_id: &'static str,
    pub multiplier: I32F32,
    pub stun_seconds: I32F32,
    pub affects_target: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomemadeFlashbang {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangWeapon {
    pub weight: I32F32,
    pub conductive: bool,
    pub min_value: I32F32,
    pub max_value: I32F32,
    pub two_handed: bool,
    pub damage: I32F32,
}

impl Default for HomemadeFlashbangWeapon {
    fn default() -> Self {
        Self {
            weight: HOMEMADE_FLASHBANG_WEIGHT,
            conductive: HOMEMADE_FLASHBANG_CONDUCTIVE,
            min_value: HOMEMADE_FLASHBANG_MIN_VALUE,
            max_value: HOMEMADE_FLASHBANG_MAX_VALUE,
            two_handed: HOMEMADE_FLASHBANG_TWO_HANDED,
            damage: HOMEMADE_FLASHBANG_DAMAGE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomemadeFlashbangHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomemadeFlashbangState {
    pub pin_pulled: bool,
    pub detonated: bool,
    pub destroyed: bool,
    pub detonation_tick: u64,
}

#[derive(Bundle)]
pub struct HomemadeFlashbangBundle {
    pub name: Name,
    pub homemade_flashbang: HomemadeFlashbang,
    pub weapon: HomemadeFlashbangWeapon,
    pub position: SimPosition,
    pub held_by: HomemadeFlashbangHeldBy,
    pub state: HomemadeFlashbangState,
}

impl HomemadeFlashbangBundle {
    pub fn new(event: SpawnHomemadeFlashbangEvent) -> Self {
        Self {
            name: Name::new(HOMEMADE_FLASHBANG_NAME),
            homemade_flashbang: HomemadeFlashbang {
                stable_id: event.stable_id,
            },
            weapon: HomemadeFlashbangWeapon::default(),
            position: event.position,
            held_by: HomemadeFlashbangHeldBy::default(),
            state: HomemadeFlashbangState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnHomemadeFlashbangEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangPrimedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangDetonatedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub position: SimPosition,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangEmployeeBlindedEvent {
    pub source: Entity,
    pub source_stable_id: u64,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub position: SimPosition,
    pub duration_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangEntityStunnedEvent {
    pub source: Entity,
    pub source_stable_id: u64,
    pub target: Entity,
    pub target_stable_id: u64,
    pub target_id: &'static str,
    pub position: SimPosition,
    pub duration_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangDestroyedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomemadeFlashbangSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomemadeFlashbangAffectedEmployee {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomemadeFlashbangStunnableEntity {
    pub stable_id: u64,
    pub target_id: &'static str,
}

pub fn homemade_flashbang_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    HOMEMADE_FLASHBANG_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

pub fn homemade_flashbang_stun_rule_for_target(
    target_id: &str,
) -> Option<HomemadeFlashbangStunRule> {
    HOMEMADE_FLASHBANG_STUN_RULES
        .iter()
        .copied()
        .find(|rule| rule.target_id == target_id)
}

fn spawn_homemade_flashbang(
    mut commands: Commands,
    mut events: EventReader<SpawnHomemadeFlashbangEvent>,
) {
    for event in events.read() {
        commands.spawn(HomemadeFlashbangBundle::new(*event));
    }
}

fn homemade_flashbang_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&HomemadeFlashbang, &HomemadeFlashbangHeldBy), Changed<HomemadeFlashbangHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: HOMEMADE_FLASHBANG_ID,
                two_handed: HOMEMADE_FLASHBANG_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: false,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn homemade_flashbang_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut primed_events: EventWriter<HomemadeFlashbangPrimedEvent>,
    weapons: Query<(Entity, &HomemadeFlashbang, &HomemadeFlashbangHeldBy, &SimPosition, &HomemadeFlashbangState)>,
) {
    for event in item_events.read() {
        if event.item_id != HOMEMADE_FLASHBANG_ID
            || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }

        for (weapon_entity, weapon, held_by, position, state) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id || state.destroyed {
                continue;
            }

            primed_events.send(HomemadeFlashbangPrimedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                user: weapon_entity,
                user_stable_id: event.employee_id,
                position: *position,
            });
        }
    }
}

fn homemade_flashbang_prime(
    mut primed_events: EventReader<HomemadeFlashbangPrimedEvent>,
    mut detonated_events: EventWriter<HomemadeFlashbangDetonatedEvent>,
    mut weapons: Query<(
        &HomemadeFlashbang,
        &mut HomemadeFlashbangState,
        &mut HomemadeFlashbangHeldBy,
    )>,
) {
    for event in primed_events.read() {
        let Ok((weapon, mut state, mut held_by)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if weapon.stable_id != event.weapon_stable_id || state.detonated || state.destroyed {
            continue;
        }

        state.pin_pulled = true;
        held_by.is_held = false;
        held_by.employee_id = 0;

        detonated_events.send(HomemadeFlashbangDetonatedEvent {
            weapon: event.weapon,
            weapon_stable_id: weapon.stable_id,
            user: event.user,
            user_stable_id: event.user_stable_id,
            position: event.position,
            damage: HOMEMADE_FLASHBANG_DAMAGE,
        });
    }
}

fn homemade_flashbang_detonate(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut detonated_events: EventReader<HomemadeFlashbangDetonatedEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut blinded_events: EventWriter<HomemadeFlashbangEmployeeBlindedEvent>,
    mut stunned_events: EventWriter<HomemadeFlashbangEntityStunnedEvent>,
    mut destroyed_events: EventWriter<HomemadeFlashbangDestroyedEvent>,
    mut weapons: Query<(&HomemadeFlashbang, &mut HomemadeFlashbangState)>,
    employees: Query<(Entity, &HomemadeFlashbangAffectedEmployee)>,
    stunnable_entities: Query<(Entity, &HomemadeFlashbangStunnableEntity)>,
) {
    let blind_ticks = seconds_to_ticks(HOMEMADE_FLASHBANG_BLIND_TICKS_SECONDS, sim_hz.0);

    for event in detonated_events.read() {
        let Ok((weapon, mut state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if weapon.stable_id != event.weapon_stable_id || state.detonated || state.destroyed {
            continue;
        }

        state.detonated = true;
        state.destroyed = true;

        damage_events.send(IncomingDamageEvent {
            target: event.user,
            raw_amount: HOMEMADE_FLASHBANG_DAMAGE,
            damage_type: DamageType::Standard,
            source: event.weapon,
        });

        for (employee, affected_employee) in &employees {
            blinded_events.send(HomemadeFlashbangEmployeeBlindedEvent {
                source: event.weapon,
                source_stable_id: weapon.stable_id,
                employee,
                employee_stable_id: affected_employee.stable_id,
                position: event.position,
                duration_ticks: blind_ticks,
            });
        }

        for (target, stunnable_entity) in &stunnable_entities {
            let Some(rule) = homemade_flashbang_stun_rule_for_target(stunnable_entity.target_id)
            else {
                continue;
            };

            if !rule.affects_target {
                continue;
            }

            stunned_events.send(HomemadeFlashbangEntityStunnedEvent {
                source: event.weapon,
                source_stable_id: weapon.stable_id,
                target,
                target_stable_id: stunnable_entity.stable_id,
                target_id: rule.target_id,
                position: event.position,
                duration_ticks: seconds_to_ticks(rule.stun_seconds, sim_hz.0),
            });
        }

        destroyed_events.send(HomemadeFlashbangDestroyedEvent {
            weapon: event.weapon,
            weapon_stable_id: weapon.stable_id,
            position: event.position,
        });

        commands.entity(event.weapon).despawn();
    }
}

fn homemade_flashbang_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &HomemadeFlashbang,
        &HomemadeFlashbangWeapon,
        &SimPosition,
        &HomemadeFlashbangHeldBy,
        &HomemadeFlashbangState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, HOMEMADE_FLASHBANG_ID);
    accumulate_str(&mut checksum, 0x1001, HOMEMADE_FLASHBANG_NAME);
    accumulate_str(&mut checksum, 0x1002, HOMEMADE_FLASHBANG_TYPE);
    accumulate_str(&mut checksum, 0x1003, HOMEMADE_FLASHBANG_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, HOMEMADE_FLASHBANG_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, HOMEMADE_FLASHBANG_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, HOMEMADE_FLASHBANG_EXTRACTED_AT);

    checksum.accumulate(HOMEMADE_FLASHBANG_SOURCE_REVISION as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_WEIGHT.to_bits() as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_CONDUCTIVE as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_TWO_HANDED as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_MIN_VALUE.to_bits() as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_MAX_VALUE.to_bits() as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_DAMAGE.to_bits() as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_WIKI_ID as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_BLIND_TICKS_SECONDS.to_bits() as u64);
    checksum.accumulate(HOMEMADE_FLASHBANG_STUN_BASE_SECONDS.to_bits() as u64);

    for spawn_chance in HOMEMADE_FLASHBANG_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x2000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    for rule in HOMEMADE_FLASHBANG_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for rule in HOMEMADE_FLASHBANG_STUN_RULES {
        accumulate_str(&mut checksum, 0x4000, rule.target_id);
        checksum.accumulate(rule.multiplier.to_bits() as u64);
        checksum.accumulate(rule.stun_seconds.to_bits() as u64);
        checksum.accumulate(rule.affects_target as u64);
    }

    for (weapon, weapon_data, position, held_by, state) in &weapons {
        checksum.accumulate(weapon.stable_id);
        checksum.accumulate(weapon_data.weight.to_bits() as u64);
        checksum.accumulate(weapon_data.conductive as u64);
        checksum.accumulate(weapon_data.min_value.to_bits() as u64);
        checksum.accumulate(weapon_data.max_value.to_bits() as u64);
        checksum.accumulate(weapon_data.two_handed as u64);
        checksum.accumulate(weapon_data.damage.to_bits() as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(state.pin_pulled as u64);
        checksum.accumulate(state.detonated as u64);
        checksum.accumulate(state.destroyed as u64);
        checksum.accumulate(state.detonation_tick);
    }
}

fn seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    let rounded_up = ticks.ceil();
    rounded_up.max(I32F32::from_num(1_u32)).to_num::<u32>()
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}