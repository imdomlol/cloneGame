// Sources: vault/weapon_pages/stun_grenade.md, vault/weapon_pages/homemade_flashbang.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    DamageType, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition,
};

pub const STUN_GRENADE_ID: &str = "stun_grenade";
pub const STUN_GRENADE_NAME: &str = "Stun grenade";
pub const STUN_GRENADE_TYPE: &str = "weapon_pages";
pub const STUN_GRENADE_SUBTYPE: &str = "weapon";
pub const STUN_GRENADE_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Stun_Grenade";
pub const STUN_GRENADE_SOURCE_REVISION: u32 = 21225;
pub const STUN_GRENADE_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const STUN_GRENADE_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const STUN_GRENADE_EFFECTS: &str = "Detonates 3 seconds after the striker lever is pulled, stunning most entities for a short duration; deals 20 damage to the holder if it detonates while still held.";
pub const STUN_GRENADE_BUY_PRICE: I32F32 = I32F32::lit("30");
pub const STUN_GRENADE_WEIGHT: I32F32 = I32F32::lit("5");
pub const STUN_GRENADE_CONDUCTIVE: bool = false;
pub const STUN_GRENADE_TWO_HANDED: bool = false;
pub const STUN_GRENADE_DAMAGE: I32F32 = I32F32::lit("20");
pub const STUN_GRENADE_PRIME_SECONDS: I32F32 = I32F32::lit("3");
pub const STUN_GRENADE_BASE_STUN_SECONDS: I32F32 = I32F32::lit("5");

pub const STUN_GRENADE_STUN_RULES: [StunGrenadeStunRule; 24] = [
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Hygrodere, "4.0", "20"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::CoilHead, "3.25", "16.25"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::SnareFlea, "3.0", "15"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Barber, "1.35", "6.75"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::BunkerSpider, "1.0", "5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Thumper, "1.0", "5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Masked, "0.75", "3.75"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::SporeLizard, "0.6", "3"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Jester, "0.6", "3"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::MaskHornets, "0.6", "3"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Butler, "0.6", "3"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Nutcracker, "0.5", "2.5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::HoardingBug, "0.5", "2.5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Bracken, "0.25", "1.25"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Maneater, "0.25", "1.25"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::ForestKeeper, "1.2", "6"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::OldBird, "1.2", "6"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::Manticoil, "1.0", "5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::EyelessDog, "0.7", "3.5"),
    StunGrenadeStunRule::new(StunGrenadeTargetKind::BaboonHawk, "0.4", "2"),
    StunGrenadeStunRule::no_effect(StunGrenadeTargetKind::CircuitBee),
    StunGrenadeStunRule::no_effect(StunGrenadeTargetKind::EarthLeviathan),
    StunGrenadeStunRule::no_effect(StunGrenadeTargetKind::GhostGirl),
    StunGrenadeStunRule::no_effect(StunGrenadeTargetKind::RoamingLocust),
];

pub const STUN_GRENADE_EXTRA_NO_EFFECT_TARGET: StunGrenadeTargetKind = StunGrenadeTargetKind::TulipSnake;

pub const STUN_GRENADE_BEHAVIORAL_MECHANICS: [StunGrenadeBehaviorRule; 34] = [
    StunGrenadeBehaviorRule {
        condition: "the player left-clicks",
        outcome: "the striker lever is removed and the detonation timer starts",
    },
    StunGrenadeBehaviorRule {
        condition: "the striker lever has already been removed",
        outcome: "a left-click throws the grenade",
    },
    StunGrenadeBehaviorRule {
        condition: "the grenade detonates while still held",
        outcome: "it deals 20 damage to the holder",
    },
    StunGrenadeBehaviorRule {
        condition: "an entity is affected by the stun",
        outcome: "total stun duration equals 5 seconds multiplied by that entity's stun multiplier",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a hygrodere",
        outcome: "the stun multiplier is 4.0 and the stun duration is 20 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a coil_head",
        outcome: "the stun multiplier is 3.25 and the stun duration is 16.25 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a snare_flea",
        outcome: "the stun multiplier is 3.0 and the stun duration is 15 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a barber",
        outcome: "the stun multiplier is 1.35 and the stun duration is 6.75 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a bunker_spider",
        outcome: "the stun multiplier is 1.0 and the stun duration is 5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a thumper",
        outcome: "the stun multiplier is 1.0 and the stun duration is 5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a masked",
        outcome: "the stun multiplier is 0.75 and the stun duration is 3.75 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a spore_lizard",
        outcome: "the stun multiplier is 0.6 and the stun duration is 3 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a jester",
        outcome: "the stun multiplier is 0.6 and the stun duration is 3 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a mask_hornets",
        outcome: "the stun multiplier is 0.6 and the stun duration is 3 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a butler",
        outcome: "the stun multiplier is 0.6 and the stun duration is 3 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a nutcracker",
        outcome: "the stun multiplier is 0.5 and the stun duration is 2.5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a hoarding_bug",
        outcome: "the stun multiplier is 0.5 and the stun duration is 2.5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a bracken",
        outcome: "the stun multiplier is 0.25 and the stun duration is 1.25 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a maneater",
        outcome: "the stun multiplier is 0.25 and the stun duration is 1.25 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a forest_keeper",
        outcome: "the stun multiplier is 1.2 and the stun duration is 6 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is an old_bird",
        outcome: "the stun multiplier is 1.2 and the stun duration is 6 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a manticoil",
        outcome: "the stun multiplier is 1.0 and the stun duration is 5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is an eyeless_dog",
        outcome: "the stun multiplier is 0.7 and the stun duration is 3.5 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a baboon_hawk",
        outcome: "the stun multiplier is 0.4 and the stun duration is 2 seconds",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a circuit_bee",
        outcome: "the stun has no effect",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is an earth_leviathan",
        outcome: "the stun has no effect",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a ghost_girl",
        outcome: "the stun has no effect",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a roaming_locust",
        outcome: "the stun has no effect",
    },
    StunGrenadeBehaviorRule {
        condition: "the target is a tulip_snake",
        outcome: "the stun has no effect",
    },
    StunGrenadeBehaviorRule {
        condition: "a jester is stunned during windup",
        outcome: "its windup is delayed for the full stun duration",
    },
    StunGrenadeBehaviorRule {
        condition: "an employee primes the grenade shortly before a forest_keeper grab",
        outcome: "the explosion frees the employee but does not stun the forest_keeper and its eating animation still completes",
    },
    StunGrenadeBehaviorRule {
        condition: "the grenade has already detonated",
        outcome: "it can be picked up and thrown again without exploding",
    },
    StunGrenadeBehaviorRule {
        condition: "a used grenade is thrown onto landmines",
        outcome: "it can trigger them safely from a distance",
    },
    StunGrenadeBehaviorRule {
        condition: "the grenade is inside the factory_layout",
        outcome: "it falls through grates",
    },
];

pub struct StunGrenadePlugin;

impl Plugin for StunGrenadePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnStunGrenadeEvent>()
            .add_event::<StunGrenadePrimedEvent>()
            .add_event::<StunGrenadeThrownEvent>()
            .add_event::<StunGrenadeDetonatedEvent>()
            .add_event::<StunGrenadeHolderDamagedEvent>()
            .add_event::<StunGrenadeEntityStunnedEvent>()
            .add_event::<StunGrenadeNoEffectEvent>()
            .add_event::<StunGrenadeJesterWindupDelayedEvent>()
            .add_event::<StunGrenadeForestKeeperGrabFreedEvent>()
            .add_event::<StunGrenadeLandmineTriggeredEvent>()
            .add_event::<StunGrenadeFellThroughGrateEvent>()
            .add_event::<StunGrenadeApplyStunRequest>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_stun_grenade,
                    stun_grenade_pickup_item_bar_bridge,
                    stun_grenade_use_from_item_bar,
                    stun_grenade_prime,
                    stun_grenade_throw,
                    stun_grenade_tick_timer,
                    stun_grenade_detonate,
                    stun_grenade_apply_stun,
                    stun_grenade_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeStunRule {
    pub target: StunGrenadeTargetKind,
    pub multiplier: I32F32,
    pub duration_seconds: I32F32,
    pub has_effect: bool,
}

impl StunGrenadeStunRule {
    pub const fn new(
        target: StunGrenadeTargetKind,
        multiplier: &'static str,
        duration_seconds: &'static str,
    ) -> Self {
        Self {
            target,
            multiplier: I32F32::lit(multiplier),
            duration_seconds: I32F32::lit(duration_seconds),
            has_effect: true,
        }
    }

    pub const fn no_effect(target: StunGrenadeTargetKind) -> Self {
        Self {
            target,
            multiplier: I32F32::ZERO,
            duration_seconds: I32F32::ZERO,
            has_effect: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StunGrenade {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeWeapon {
    pub buy_price: I32F32,
    pub weight: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub damage: I32F32,
}

impl Default for StunGrenadeWeapon {
    fn default() -> Self {
        Self {
            buy_price: STUN_GRENADE_BUY_PRICE,
            weight: STUN_GRENADE_WEIGHT,
            conductive: STUN_GRENADE_CONDUCTIVE,
            two_handed: STUN_GRENADE_TWO_HANDED,
            damage: STUN_GRENADE_DAMAGE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeHeldBy {
    pub employee_id: u64,
    pub employee_entity: Option<Entity>,
    pub is_held: bool,
}

impl Default for StunGrenadeHeldBy {
    fn default() -> Self {
        Self {
            employee_id: 0,
            employee_entity: None,
            is_held: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeFuse {
    pub lever_removed: bool,
    pub detonated: bool,
    pub thrown: bool,
    pub ticks_remaining: u32,
    pub primes: u64,
    pub throws: u64,
    pub detonations: u64,
}

impl Default for StunGrenadeFuse {
    fn default() -> Self {
        Self {
            lever_removed: false,
            detonated: false,
            thrown: false,
            ticks_remaining: 0,
            primes: 0,
            throws: 0,
            detonations: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StunGrenadeEnvironment {
    pub inside_factory_layout: bool,
    pub on_landmine: bool,
    pub fell_through_grate: bool,
}

#[derive(Bundle)]
pub struct StunGrenadeBundle {
    pub name: Name,
    pub stun_grenade: StunGrenade,
    pub weapon: StunGrenadeWeapon,
    pub position: SimPosition,
    pub held_by: StunGrenadeHeldBy,
    pub fuse: StunGrenadeFuse,
    pub environment: StunGrenadeEnvironment,
}

impl StunGrenadeBundle {
    pub fn new(event: SpawnStunGrenadeEvent) -> Self {
        Self {
            name: Name::new(STUN_GRENADE_NAME),
            stun_grenade: StunGrenade {
                stable_id: event.stable_id,
            },
            weapon: StunGrenadeWeapon::default(),
            position: event.position,
            held_by: StunGrenadeHeldBy::default(),
            fuse: StunGrenadeFuse::default(),
            environment: StunGrenadeEnvironment {
                inside_factory_layout: event.inside_factory_layout,
                on_landmine: false,
                fell_through_grate: false,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnStunGrenadeEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub inside_factory_layout: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadePrimedEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeThrownEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub user_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeDetonatedEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub position: SimPosition,
    pub held_by: Option<Entity>,
    pub holder_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeHolderDamagedEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub holder: Entity,
    pub holder_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeEntityStunnedEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub target: Entity,
    pub target_stable_id: u64,
    pub target_kind: StunGrenadeTargetKind,
    pub duration_ticks: u32,
    pub multiplier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeNoEffectEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub target: Entity,
    pub target_stable_id: u64,
    pub target_kind: StunGrenadeTargetKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeJesterWindupDelayedEvent {
    pub grenade: Entity,
    pub jester: Entity,
    pub jester_stable_id: u64,
    pub delayed_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeForestKeeperGrabFreedEvent {
    pub grenade: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub forest_keeper: Entity,
    pub forest_keeper_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeLandmineTriggeredEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeFellThroughGrateEvent {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunGrenadeApplyStunRequest {
    pub grenade: Entity,
    pub grenade_stable_id: u64,
    pub target: Entity,
    pub target_stable_id: u64,
    pub target_kind: StunGrenadeTargetKind,
    pub jester_in_windup: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StunGrenadeTargetKind {
    #[default]
    Entity = 0,
    Hygrodere = 1,
    CoilHead = 2,
    SnareFlea = 3,
    Barber = 4,
    BunkerSpider = 5,
    Thumper = 6,
    Masked = 7,
    SporeLizard = 8,
    Jester = 9,
    MaskHornets = 10,
    Butler = 11,
    Nutcracker = 12,
    HoardingBug = 13,
    Bracken = 14,
    Maneater = 15,
    ForestKeeper = 16,
    OldBird = 17,
    Manticoil = 18,
    EyelessDog = 19,
    BaboonHawk = 20,
    CircuitBee = 21,
    EarthLeviathan = 22,
    GhostGirl = 23,
    RoamingLocust = 24,
    TulipSnake = 25,
}

pub fn stun_grenade_rule_for_target(target: StunGrenadeTargetKind) -> StunGrenadeStunRule {
    if target == STUN_GRENADE_EXTRA_NO_EFFECT_TARGET {
        return StunGrenadeStunRule::no_effect(target);
    }

    STUN_GRENADE_STUN_RULES
        .iter()
        .copied()
        .find(|rule| rule.target == target)
        .unwrap_or(StunGrenadeStunRule {
            target,
            multiplier: I32F32::lit("1.0"),
            duration_seconds: STUN_GRENADE_BASE_STUN_SECONDS,
            has_effect: true,
        })
}

pub fn stun_grenade_stun_ticks_for_target(
    target: StunGrenadeTargetKind,
    sim_hz: I32F32,
) -> Option<u32> {
    let rule = stun_grenade_rule_for_target(target);

    if !rule.has_effect {
        return None;
    }

    Some(seconds_to_ticks(rule.duration_seconds, sim_hz))
}

fn spawn_stun_grenade(mut commands: Commands, mut events: EventReader<SpawnStunGrenadeEvent>) {
    for event in events.read() {
        commands.spawn(StunGrenadeBundle::new(*event));
    }
}

fn stun_grenade_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    grenades: Query<(&StunGrenade, &StunGrenadeHeldBy), Changed<StunGrenadeHeldBy>>,
) {
    for (grenade, held_by) in &grenades {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: STUN_GRENADE_ID,
                two_handed: STUN_GRENADE_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: true,
            });
        } else {
            let _ = grenade.stable_id;
        }
    }
}

fn stun_grenade_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut prime_events: EventWriter<StunGrenadePrimedEvent>,
    mut throw_events: EventWriter<StunGrenadeThrownEvent>,
    grenades: Query<(Entity, &StunGrenade, &StunGrenadeHeldBy, &StunGrenadeFuse, &SimPosition)>,
) {
    for event in item_events.read() {
        if event.item_id != STUN_GRENADE_ID || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }

        for (grenade_entity, grenade, held_by, fuse, position) in &grenades {
            if !held_by.is_held || held_by.employee_id != event.employee_id || fuse.detonated {
                continue;
            }

            if fuse.lever_removed {
                throw_events.send(StunGrenadeThrownEvent {
                    grenade: grenade_entity,
                    grenade_stable_id: grenade.stable_id,
                    user_stable_id: event.employee_id,
                    position: *position,
                });
            } else {
                prime_events.send(StunGrenadePrimedEvent {
                    grenade: grenade_entity,
                    grenade_stable_id: grenade.stable_id,
                    user: held_by.employee_entity.unwrap_or(grenade_entity),
                    user_stable_id: event.employee_id,
                });
            }
        }
    }
}

fn stun_grenade_prime(
    sim_hz: Res<SimHz>,
    mut events: EventReader<StunGrenadePrimedEvent>,
    mut grenades: Query<(&StunGrenade, &mut StunGrenadeFuse)>,
) {
    let fuse_ticks = seconds_to_ticks(STUN_GRENADE_PRIME_SECONDS, sim_hz.0);

    for event in events.read() {
        let Ok((grenade, mut fuse)) = grenades.get_mut(event.grenade) else {
            continue;
        };

        if grenade.stable_id != event.grenade_stable_id || fuse.lever_removed || fuse.detonated {
            continue;
        }

        fuse.lever_removed = true;
        fuse.ticks_remaining = fuse_ticks;
        fuse.primes = fuse.primes.wrapping_add(1);
    }
}

fn stun_grenade_throw(
    mut events: EventReader<StunGrenadeThrownEvent>,
    mut landmine_events: EventWriter<StunGrenadeLandmineTriggeredEvent>,
    mut grate_events: EventWriter<StunGrenadeFellThroughGrateEvent>,
    mut grenades: Query<(
        &StunGrenade,
        &mut StunGrenadeHeldBy,
        &mut StunGrenadeFuse,
        &mut StunGrenadeEnvironment,
        &mut SimPosition,
    )>,
) {
    for event in events.read() {
        let Ok((grenade, mut held_by, mut fuse, mut environment, mut position)) =
            grenades.get_mut(event.grenade)
        else {
            continue;
        };

        if grenade.stable_id != event.grenade_stable_id || fuse.detonated {
            continue;
        }

        held_by.employee_id = 0;
        held_by.employee_entity = None;
        held_by.is_held = false;
        fuse.thrown = true;
        fuse.throws = fuse.throws.wrapping_add(1);
        *position = event.position;

        if environment.on_landmine {
            landmine_events.send(StunGrenadeLandmineTriggeredEvent {
                grenade: event.grenade,
                grenade_stable_id: grenade.stable_id,
                position: event.position,
            });
        }

        if environment.inside_factory_layout {
            environment.fell_through_grate = true;
            grate_events.send(StunGrenadeFellThroughGrateEvent {
                grenade: event.grenade,
                grenade_stable_id: grenade.stable_id,
                position: event.position,
            });
        }
    }
}

fn stun_grenade_tick_timer(mut grenades: Query<&mut StunGrenadeFuse, With<StunGrenade>>) {
    for mut fuse in &mut grenades {
        if fuse.lever_removed && !fuse.detonated && fuse.ticks_remaining > 0 {
            fuse.ticks_remaining -= 1;
        }
    }
}

fn stun_grenade_detonate(
    mut detonated_events: EventWriter<StunGrenadeDetonatedEvent>,
    mut holder_damaged_events: EventWriter<StunGrenadeHolderDamagedEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut grenades: Query<(
        Entity,
        &StunGrenade,
        &mut StunGrenadeFuse,
        &StunGrenadeHeldBy,
        &SimPosition,
    )>,
) {
    for (grenade_entity, grenade, mut fuse, held_by, position) in &mut grenades {
        if !fuse.lever_removed || fuse.detonated || fuse.ticks_remaining > 0 {
            continue;
        }

        fuse.detonated = true;
        fuse.detonations = fuse.detonations.wrapping_add(1);

        let held_entity = held_by.is_held.then_some(held_by.employee_entity.unwrap_or(grenade_entity));

        detonated_events.send(StunGrenadeDetonatedEvent {
            grenade: grenade_entity,
            grenade_stable_id: grenade.stable_id,
            position: *position,
            held_by: held_entity,
            holder_stable_id: held_by.employee_id,
        });

        if let Some(holder) = held_entity {
            holder_damaged_events.send(StunGrenadeHolderDamagedEvent {
                grenade: grenade_entity,
                grenade_stable_id: grenade.stable_id,
                holder,
                holder_stable_id: held_by.employee_id,
                damage: STUN_GRENADE_DAMAGE,
            });

            damage_events.send(IncomingDamageEvent {
                target: holder,
                raw_amount: STUN_GRENADE_DAMAGE,
                damage_type: DamageType::Standard,
                source: grenade_entity,
            });
        }
    }
}

fn stun_grenade_apply_stun(
    sim_hz: Res<SimHz>,
    mut requests: EventReader<StunGrenadeApplyStunRequest>,
    mut stunned_events: EventWriter<StunGrenadeEntityStunnedEvent>,
    mut no_effect_events: EventWriter<StunGrenadeNoEffectEvent>,
    mut jester_events: EventWriter<StunGrenadeJesterWindupDelayedEvent>,
) {
    for request in requests.read() {
        let rule = stun_grenade_rule_for_target(request.target_kind);

        if !rule.has_effect {
            no_effect_events.send(StunGrenadeNoEffectEvent {
                grenade: request.grenade,
                grenade_stable_id: request.grenade_stable_id,
                target: request.target,
                target_stable_id: request.target_stable_id,
                target_kind: request.target_kind,
            });
            continue;
        }

        let duration_ticks = seconds_to_ticks(rule.duration_seconds, sim_hz.0);

        stunned_events.send(StunGrenadeEntityStunnedEvent {
            grenade: request.grenade,
            grenade_stable_id: request.grenade_stable_id,
            target: request.target,
            target_stable_id: request.target_stable_id,
            target_kind: request.target_kind,
            duration_ticks,
            multiplier: rule.multiplier,
        });

        if request.target_kind == StunGrenadeTargetKind::Jester && request.jester_in_windup {
            jester_events.send(StunGrenadeJesterWindupDelayedEvent {
                grenade: request.grenade,
                jester: request.target,
                jester_stable_id: request.target_stable_id,
                delayed_ticks: duration_ticks,
            });
        }
    }
}

fn stun_grenade_checksum(
    mut checksum: ResMut<SimChecksumState>,
    grenades: Query<(
        &StunGrenade,
        &StunGrenadeWeapon,
        &SimPosition,
        &StunGrenadeHeldBy,
        &StunGrenadeFuse,
        &StunGrenadeEnvironment,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, STUN_GRENADE_ID);
    accumulate_str(&mut checksum, 0x1001, STUN_GRENADE_NAME);
    accumulate_str(&mut checksum, 0x1002, STUN_GRENADE_TYPE);
    accumulate_str(&mut checksum, 0x1003, STUN_GRENADE_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, STUN_GRENADE_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, STUN_GRENADE_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, STUN_GRENADE_EXTRACTED_AT);

    checksum.accumulate(STUN_GRENADE_SOURCE_REVISION as u64);
    checksum.accumulate(STUN_GRENADE_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(STUN_GRENADE_BUY_PRICE.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_WEIGHT.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_CONDUCTIVE as u64);
    checksum.accumulate(STUN_GRENADE_TWO_HANDED as u64);
    checksum.accumulate(STUN_GRENADE_DAMAGE.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_PRIME_SECONDS.to_bits() as u64);
    checksum.accumulate(STUN_GRENADE_BASE_STUN_SECONDS.to_bits() as u64);

    for rule in STUN_GRENADE_STUN_RULES {
        checksum.accumulate(rule.target as u64);
        checksum.accumulate(rule.multiplier.to_bits() as u64);
        checksum.accumulate(rule.duration_seconds.to_bits() as u64);
        checksum.accumulate(rule.has_effect as u64);
    }

    checksum.accumulate(STUN_GRENADE_EXTRA_NO_EFFECT_TARGET as u64);

    for rule in STUN_GRENADE_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for (grenade, weapon, position, held_by, fuse, environment) in &grenades {
        checksum.accumulate(grenade.stable_id);
        checksum.accumulate(weapon.buy_price.to_bits() as u64);
        checksum.accumulate(weapon.weight.to_bits() as u64);
        checksum.accumulate(weapon.conductive as u64);
        checksum.accumulate(weapon.two_handed as u64);
        checksum.accumulate(weapon.damage.to_bits() as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(fuse.lever_removed as u64);
        checksum.accumulate(fuse.detonated as u64);
        checksum.accumulate(fuse.thrown as u64);
        checksum.accumulate(fuse.ticks_remaining as u64);
        checksum.accumulate(fuse.primes);
        checksum.accumulate(fuse.throws);
        checksum.accumulate(fuse.detonations);
        checksum.accumulate(environment.inside_factory_layout as u64);
        checksum.accumulate(environment.on_landmine as u64);
        checksum.accumulate(environment.fell_through_grate as u64);
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