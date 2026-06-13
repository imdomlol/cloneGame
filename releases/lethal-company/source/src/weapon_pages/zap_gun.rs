// Sources: vault/weapon_pages/zap_gun.md, vault/item_index_pages/weapon.md, vault/gameplay_mechanics/cause_of_death.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{SimChecksumState, SimHz, SimPosition, SimTick};

pub const ZAP_GUN_ID: &str = "zap_gun";
pub const ZAP_GUN_NAME: &str = "Zap gun";
pub const ZAP_GUN_TYPE: &str = "weapon_pages";
pub const ZAP_GUN_SUBTYPE: &str = "weapon";
pub const ZAP_GUN_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Zap_gun";
pub const ZAP_GUN_SOURCE_REVISION: u32 = 21226;
pub const ZAP_GUN_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const ZAP_GUN_CONFIDENCE_BASIS_POINTS: u16 = 97;

pub const ZAP_GUN_EFFECTS: &str = "Stuns targets when used properly.";
pub const ZAP_GUN_BUY_VALUE: I32F32 = I32F32::lit("400");
pub const ZAP_GUN_SELL_VALUE: I32F32 = I32F32::lit("0");
pub const ZAP_GUN_WEIGHT: I32F32 = I32F32::lit("11");
pub const ZAP_GUN_BATTERY_LIFE: &str = "Varies";
pub const ZAP_GUN_CONDUCTIVE: bool = true;
pub const ZAP_GUN_TWO_HANDED: bool = false;
pub const ZAP_GUN_COOLDOWN_SECONDS: I32F32 = I32F32::lit("5");
pub const ZAP_GUN_APPROX_OPERATING_SECONDS: I32F32 = I32F32::lit("60");
pub const ZAP_GUN_SCAN_RANGE_UNITS: I32F32 = I32F32::lit("12");
pub const ZAP_GUN_BASE_STUN_SECONDS: I32F32 = I32F32::lit("3");
pub const ZAP_GUN_PULL_EXTENSION_SECONDS: I32F32 = I32F32::lit("1");
pub const ZAP_GUN_EARLY_END_SECONDS: I32F32 = I32F32::lit("1");

pub const ZAP_GUN_EFFECTIVE_TARGETS: [&str; 12] = [
    "snare_flea",
    "bunker_spider",
    "hoarding_bug",
    "bracken",
    "thumper",
    "hygrodere",
    "spore_lizard",
    "jester",
    "eyeless_dog",
    "forest_keeper",
    "masked",
    "barber",
];

pub const ZAP_GUN_SOMEWHAT_EFFECTIVE_TARGETS: [&str; 2] = ["nutcracker", "employee"];

pub const ZAP_GUN_USELESS_TARGETS: [&str; 9] = [
    "manticoil",
    "roaming_locust",
    "circuit_bee",
    "earth_leviathan",
    "ghost_girl",
    "coil_head",
    "landmine",
    "turret",
    "nutcracker_attack_targeting",
];

pub const ZAP_GUN_BEHAVIORAL_MECHANICS: [ZapGunBehaviorRule; 11] = [
    ZapGunBehaviorRule {
        condition: "the user holds the primary fire button while the Zap gun is equipped",
        outcome: "it scans the area in front of the user for a valid target and starts a continuous electric beam when one is detected",
    },
    ZapGunBehaviorRule {
        condition: "the user pulls against the current by looking left or right",
        outcome: "the stun duration is prolonged",
    },
    ZapGunBehaviorRule {
        condition: "the user does not pull against the current correctly",
        outcome: "the stun ends early",
    },
    ZapGunBehaviorRule {
        condition: "beam disconnect",
        outcome: "the Zap gun enters a 5 second cooldown before it can be used again",
    },
    ZapGunBehaviorRule {
        condition: "the Zap gun is used to completion",
        outcome: "its total operating time is approximately 60 seconds",
    },
    ZapGunBehaviorRule {
        condition: "the target is a snare_flea, bunker_spider, hoarding_bug, bracken, thumper, hygrodere, spore_lizard, jester, eyeless_dog, forest_keeper, masked, or barber",
        outcome: "the stun is effective and the target is immobilized for most of the beam duration",
    },
    ZapGunBehaviorRule {
        condition: "the target is a jester",
        outcome: "the stun applies in both boxed and popped states, and the popped state speed resets when the stun starts",
    },
    ZapGunBehaviorRule {
        condition: "the target is a forest_keeper",
        outcome: "the stun can stop the chase temporarily, but the target will resume chasing immediately if it still has line of sight after the stun ends",
    },
    ZapGunBehaviorRule {
        condition: "the target is a nutcracker or employee",
        outcome: "the stun is only somewhat effective and the target cannot attack during the stun",
    },
    ZapGunBehaviorRule {
        condition: "the target is a nutcracker",
        outcome: "it can still target employees and will attack immediately after the stun ends",
    },
    ZapGunBehaviorRule {
        condition: "the target is a manticoil, roaming_locust, circuit_bee, earth_leviathan, ghost_girl, coil_head, landmine, or turret",
        outcome: "the Zap gun is useless against that target",
    },
];

pub struct ZapGunPlugin;

impl Plugin for ZapGunPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnZapGunEvent>()
            .add_event::<ZapGunPrimaryFireHeldEvent>()
            .add_event::<ZapGunBeamStartedEvent>()
            .add_event::<ZapGunBeamExtendedEvent>()
            .add_event::<ZapGunBeamDisconnectedEvent>()
            .add_event::<ZapGunStunEndedEvent>()
            .add_event::<ZapGunCooldownStartedEvent>()
            .add_event::<ZapGunSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_zap_gun,
                    zap_gun_pickup_item_bar_bridge,
                    zap_gun_tick_state,
                    zap_gun_use_from_item_bar,
                    zap_gun_primary_fire_held,
                    zap_gun_beam_disconnected,
                    zap_gun_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZapGun {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunWeapon {
    pub buy_value: I32F32,
    pub sell_value: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
}

impl Default for ZapGunWeapon {
    fn default() -> Self {
        Self {
            buy_value: ZAP_GUN_BUY_VALUE,
            sell_value: ZAP_GUN_SELL_VALUE,
            weight: ZAP_GUN_WEIGHT,
            conductive: ZAP_GUN_CONDUCTIVE,
            two_handed: ZAP_GUN_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZapGunHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunCharge {
    pub operating_ticks_remaining: u32,
    pub cooldown_ticks_remaining: u32,
}

impl Default for ZapGunCharge {
    fn default() -> Self {
        Self {
            operating_ticks_remaining: 0,
            cooldown_ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZapGunBeamState {
    pub active: bool,
    pub target: Option<Entity>,
    pub target_kind: ZapGunTargetKind,
    pub stun_ticks_remaining: u32,
    pub beam_ticks_active: u32,
    pub successful_pulls: u32,
    pub failed_pulls: u32,
    pub last_used_tick: u64,
}

#[derive(Bundle)]
pub struct ZapGunBundle {
    pub name: Name,
    pub zap_gun: ZapGun,
    pub weapon: ZapGunWeapon,
    pub position: SimPosition,
    pub held_by: ZapGunHeldBy,
    pub charge: ZapGunCharge,
    pub beam_state: ZapGunBeamState,
}

impl ZapGunBundle {
    pub fn new(event: SpawnZapGunEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(ZAP_GUN_NAME),
            zap_gun: ZapGun {
                stable_id: event.stable_id,
            },
            weapon: ZapGunWeapon::default(),
            position: event.position,
            held_by: ZapGunHeldBy::default(),
            charge: ZapGunCharge {
                operating_ticks_remaining: seconds_to_ticks(
                    ZAP_GUN_APPROX_OPERATING_SECONDS,
                    sim_hz,
                ),
                cooldown_ticks_remaining: 0,
            },
            beam_state: ZapGunBeamState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnZapGunEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunPrimaryFireHeldEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub user_position: SimPosition,
    pub target: Option<Entity>,
    pub target_position: SimPosition,
    pub target_kind: ZapGunTargetKind,
    pub pulling_against_current: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunBeamStartedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Entity,
    pub target_kind: ZapGunTargetKind,
    pub effectiveness: ZapGunEffectiveness,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunBeamExtendedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub target: Entity,
    pub stun_ticks_remaining: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunBeamDisconnectedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub target: Option<Entity>,
    pub reason: ZapGunDisconnectReason,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunStunEndedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub target: Option<Entity>,
    pub target_kind: ZapGunTargetKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunCooldownStartedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub cooldown_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZapGunSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ZapGunTargetKind {
    SnareFlea = 0,
    BunkerSpider = 1,
    HoardingBug = 2,
    Bracken = 3,
    Thumper = 4,
    Hygrodere = 5,
    SporeLizard = 6,
    Jester = 7,
    EyelessDog = 8,
    ForestKeeper = 9,
    Masked = 10,
    Barber = 11,
    Nutcracker = 12,
    #[default]
    Employee = 13,
    Manticoil = 14,
    RoamingLocust = 15,
    CircuitBee = 16,
    EarthLeviathan = 17,
    GhostGirl = 18,
    CoilHead = 19,
    Landmine = 20,
    Turret = 21,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ZapGunEffectiveness {
    #[default]
    Effective = 0,
    SomewhatEffective = 1,
    Useless = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ZapGunDisconnectReason {
    #[default]
    NoTarget = 0,
    FailedCurrentPull = 1,
    BatteryDepleted = 2,
    CompletedUse = 3,
    ManualRelease = 4,
}

pub fn zap_gun_effectiveness_for_target(target_kind: ZapGunTargetKind) -> ZapGunEffectiveness {
    match target_kind {
        ZapGunTargetKind::SnareFlea
        | ZapGunTargetKind::BunkerSpider
        | ZapGunTargetKind::HoardingBug
        | ZapGunTargetKind::Bracken
        | ZapGunTargetKind::Thumper
        | ZapGunTargetKind::Hygrodere
        | ZapGunTargetKind::SporeLizard
        | ZapGunTargetKind::Jester
        | ZapGunTargetKind::EyelessDog
        | ZapGunTargetKind::ForestKeeper
        | ZapGunTargetKind::Masked
        | ZapGunTargetKind::Barber => ZapGunEffectiveness::Effective,
        ZapGunTargetKind::Nutcracker | ZapGunTargetKind::Employee => {
            ZapGunEffectiveness::SomewhatEffective
        }
        ZapGunTargetKind::Manticoil
        | ZapGunTargetKind::RoamingLocust
        | ZapGunTargetKind::CircuitBee
        | ZapGunTargetKind::EarthLeviathan
        | ZapGunTargetKind::GhostGirl
        | ZapGunTargetKind::CoilHead
        | ZapGunTargetKind::Landmine
        | ZapGunTargetKind::Turret => ZapGunEffectiveness::Useless,
    }
}

pub fn zap_gun_target_id(target_kind: ZapGunTargetKind) -> &'static str {
    match target_kind {
        ZapGunTargetKind::SnareFlea => "snare_flea",
        ZapGunTargetKind::BunkerSpider => "bunker_spider",
        ZapGunTargetKind::HoardingBug => "hoarding_bug",
        ZapGunTargetKind::Bracken => "bracken",
        ZapGunTargetKind::Thumper => "thumper",
        ZapGunTargetKind::Hygrodere => "hygrodere",
        ZapGunTargetKind::SporeLizard => "spore_lizard",
        ZapGunTargetKind::Jester => "jester",
        ZapGunTargetKind::EyelessDog => "eyeless_dog",
        ZapGunTargetKind::ForestKeeper => "forest_keeper",
        ZapGunTargetKind::Masked => "masked",
        ZapGunTargetKind::Barber => "barber",
        ZapGunTargetKind::Nutcracker => "nutcracker",
        ZapGunTargetKind::Employee => "employee",
        ZapGunTargetKind::Manticoil => "manticoil",
        ZapGunTargetKind::RoamingLocust => "roaming_locust",
        ZapGunTargetKind::CircuitBee => "circuit_bee",
        ZapGunTargetKind::EarthLeviathan => "earth_leviathan",
        ZapGunTargetKind::GhostGirl => "ghost_girl",
        ZapGunTargetKind::CoilHead => "coil_head",
        ZapGunTargetKind::Landmine => "landmine",
        ZapGunTargetKind::Turret => "turret",
    }
}

fn spawn_zap_gun(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnZapGunEvent>,
) {
    for event in events.read() {
        commands.spawn(ZapGunBundle::new(*event, sim_hz.0));
    }
}

fn zap_gun_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&ZapGun, &ZapGunHeldBy), Changed<ZapGunHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: ZAP_GUN_ID,
                two_handed: ZAP_GUN_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: true,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn zap_gun_tick_state(
    mut ended_events: EventWriter<ZapGunStunEndedEvent>,
    mut disconnected_events: EventWriter<ZapGunBeamDisconnectedEvent>,
    mut weapons: Query<(Entity, &ZapGun, &mut ZapGunCharge, &mut ZapGunBeamState)>,
) {
    for (entity, weapon, mut charge, mut beam) in &mut weapons {
        if charge.cooldown_ticks_remaining > 0 {
            charge.cooldown_ticks_remaining -= 1;
        }

        if !beam.active {
            continue;
        }

        if beam.stun_ticks_remaining > 0 {
            beam.stun_ticks_remaining -= 1;
        }

        if charge.operating_ticks_remaining > 0 {
            charge.operating_ticks_remaining -= 1;
        }

        beam.beam_ticks_active = beam.beam_ticks_active.saturating_add(1);

        if charge.operating_ticks_remaining == 0 {
            disconnected_events.send(ZapGunBeamDisconnectedEvent {
                weapon: entity,
                weapon_stable_id: weapon.stable_id,
                target: beam.target,
                reason: ZapGunDisconnectReason::BatteryDepleted,
            });
        } else if beam.stun_ticks_remaining == 0 {
            ended_events.send(ZapGunStunEndedEvent {
                weapon: entity,
                weapon_stable_id: weapon.stable_id,
                target: beam.target,
                target_kind: beam.target_kind,
            });
            disconnected_events.send(ZapGunBeamDisconnectedEvent {
                weapon: entity,
                weapon_stable_id: weapon.stable_id,
                target: beam.target,
                reason: ZapGunDisconnectReason::CompletedUse,
            });
        }
    }
}

fn zap_gun_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut use_events: EventWriter<ZapGunPrimaryFireHeldEvent>,
    weapons: Query<(Entity, &ZapGunHeldBy, &SimPosition), With<ZapGun>>,
) {
    for event in item_events.read() {
        if event.item_id != ZAP_GUN_ID || event.effect != ItemBarItemEffect::FunctionalActivated {
            continue;
        }

        for (weapon, held_by, position) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            use_events.send(ZapGunPrimaryFireHeldEvent {
                weapon,
                user: weapon,
                user_stable_id: event.employee_id,
                user_position: *position,
                target: None,
                target_position: *position,
                target_kind: ZapGunTargetKind::Employee,
                pulling_against_current: false,
            });
        }
    }
}

fn zap_gun_primary_fire_held(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut use_events: EventReader<ZapGunPrimaryFireHeldEvent>,
    mut started_events: EventWriter<ZapGunBeamStartedEvent>,
    mut extended_events: EventWriter<ZapGunBeamExtendedEvent>,
    mut disconnected_events: EventWriter<ZapGunBeamDisconnectedEvent>,
    mut weapons: Query<(Entity, &ZapGun, &mut ZapGunCharge, &mut ZapGunBeamState)>,
) {
    let base_stun_ticks = seconds_to_ticks(ZAP_GUN_BASE_STUN_SECONDS, sim_hz.0);
    let pull_extension_ticks = seconds_to_ticks(ZAP_GUN_PULL_EXTENSION_SECONDS, sim_hz.0);
    let early_end_ticks = seconds_to_ticks(ZAP_GUN_EARLY_END_SECONDS, sim_hz.0);

    for event in use_events.read() {
        let Ok((weapon_entity, weapon, charge, mut beam)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if charge.cooldown_ticks_remaining > 0 || charge.operating_ticks_remaining == 0 {
            continue;
        }

        let Some(target) = event.target else {
            disconnected_events.send(ZapGunBeamDisconnectedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                target: None,
                reason: ZapGunDisconnectReason::NoTarget,
            });
            continue;
        };

        let dx = event.target_position.x - event.user_position.x;
        let dy = event.target_position.y - event.user_position.y;
        let distance_squared = dx * dx + dy * dy;
        let scan_range_squared = ZAP_GUN_SCAN_RANGE_UNITS * ZAP_GUN_SCAN_RANGE_UNITS;

        if distance_squared > scan_range_squared {
            disconnected_events.send(ZapGunBeamDisconnectedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                target: Some(target),
                reason: ZapGunDisconnectReason::NoTarget,
            });
            continue;
        }

        let effectiveness = zap_gun_effectiveness_for_target(event.target_kind);

        if effectiveness == ZapGunEffectiveness::Useless {
            disconnected_events.send(ZapGunBeamDisconnectedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                target: Some(target),
                reason: ZapGunDisconnectReason::NoTarget,
            });
            continue;
        }

        if !beam.active {
            beam.active = true;
            beam.target = Some(target);
            beam.target_kind = event.target_kind;
            beam.stun_ticks_remaining = base_stun_ticks;
            beam.beam_ticks_active = 0;
            beam.last_used_tick = tick.0;

            started_events.send(ZapGunBeamStartedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                user: event.user,
                user_stable_id: event.user_stable_id,
                target,
                target_kind: event.target_kind,
                effectiveness,
            });
        }

        if event.pulling_against_current {
            beam.successful_pulls = beam.successful_pulls.saturating_add(1);
            beam.stun_ticks_remaining = beam
                .stun_ticks_remaining
                .saturating_add(pull_extension_ticks);

            extended_events.send(ZapGunBeamExtendedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                target,
                stun_ticks_remaining: beam.stun_ticks_remaining,
            });
        } else {
            beam.failed_pulls = beam.failed_pulls.saturating_add(1);
            beam.stun_ticks_remaining = beam.stun_ticks_remaining.min(early_end_ticks);
        }
    }
}

fn zap_gun_beam_disconnected(
    sim_hz: Res<SimHz>,
    mut cooldown_events: EventWriter<ZapGunCooldownStartedEvent>,
    mut disconnect_events: EventReader<ZapGunBeamDisconnectedEvent>,
    mut weapons: Query<(&ZapGun, &mut ZapGunCharge, &mut ZapGunBeamState)>,
) {
    let cooldown_ticks = seconds_to_ticks(ZAP_GUN_COOLDOWN_SECONDS, sim_hz.0);

    for event in disconnect_events.read() {
        let Ok((weapon, mut charge, mut beam)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if weapon.stable_id != event.weapon_stable_id {
            continue;
        }

        beam.active = false;
        beam.target = None;
        beam.stun_ticks_remaining = 0;
        charge.cooldown_ticks_remaining = cooldown_ticks;

        cooldown_events.send(ZapGunCooldownStartedEvent {
            weapon: event.weapon,
            weapon_stable_id: weapon.stable_id,
            cooldown_ticks,
        });
    }
}

fn zap_gun_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &ZapGun,
        &ZapGunWeapon,
        &SimPosition,
        &ZapGunHeldBy,
        &ZapGunCharge,
        &ZapGunBeamState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, ZAP_GUN_ID);
    accumulate_str(&mut checksum, 0x1001, ZAP_GUN_NAME);
    accumulate_str(&mut checksum, 0x1002, ZAP_GUN_TYPE);
    accumulate_str(&mut checksum, 0x1003, ZAP_GUN_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, ZAP_GUN_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, ZAP_GUN_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, ZAP_GUN_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1007, ZAP_GUN_BATTERY_LIFE);

    checksum.accumulate(ZAP_GUN_SOURCE_REVISION as u64);
    checksum.accumulate(ZAP_GUN_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(ZAP_GUN_BUY_VALUE.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_WEIGHT.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_CONDUCTIVE as u64);
    checksum.accumulate(ZAP_GUN_TWO_HANDED as u64);
    checksum.accumulate(ZAP_GUN_COOLDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_APPROX_OPERATING_SECONDS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_SCAN_RANGE_UNITS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_BASE_STUN_SECONDS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_PULL_EXTENSION_SECONDS.to_bits() as u64);
    checksum.accumulate(ZAP_GUN_EARLY_END_SECONDS.to_bits() as u64);

    for target in ZAP_GUN_EFFECTIVE_TARGETS {
        accumulate_str(&mut checksum, 0x2000, target);
    }

    for target in ZAP_GUN_SOMEWHAT_EFFECTIVE_TARGETS {
        accumulate_str(&mut checksum, 0x2001, target);
    }

    for target in ZAP_GUN_USELESS_TARGETS {
        accumulate_str(&mut checksum, 0x2002, target);
    }

    for rule in ZAP_GUN_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    for (weapon, weapon_data, position, held_by, charge, beam) in &weapons {
        checksum.accumulate(weapon.stable_id);
        checksum.accumulate(weapon_data.buy_value.to_bits() as u64);
        checksum.accumulate(weapon_data.sell_value.to_bits() as u64);
        checksum.accumulate(weapon_data.weight.to_bits() as u64);
        checksum.accumulate(weapon_data.conductive as u64);
        checksum.accumulate(weapon_data.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(charge.operating_ticks_remaining as u64);
        checksum.accumulate(charge.cooldown_ticks_remaining as u64);
        checksum.accumulate(beam.active as u64);
        checksum.accumulate(beam.target.map(|target| target.index() as u64).unwrap_or(0));
        checksum.accumulate(beam.target_kind as u64);
        checksum.accumulate(beam.stun_ticks_remaining as u64);
        checksum.accumulate(beam.beam_ticks_active as u64);
        checksum.accumulate(beam.successful_pulls as u64);
        checksum.accumulate(beam.failed_pulls as u64);
        checksum.accumulate(beam.last_used_tick);
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