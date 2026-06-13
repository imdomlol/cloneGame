// Sources: vault/weapon_pages/double_barrel.md, vault/item_index_pages/items.md, vault/gameplay_mechanics/cause_of_death.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::gameplay_mechanics::item_bar::{
    ItemBarItemEffect, ItemBarItemEffectEvent, ItemBarPickupEvent,
};
use crate::sim::{
    tick_rng, DamageType, GameSeed, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState,
    SimHz, SimPosition, SimTick,
};

pub const DOUBLE_BARREL_ID: &str = "double_barrel";
pub const DOUBLE_BARREL_NAME: &str = "Double-barrel";
pub const DOUBLE_BARREL_TYPE: &str = "weapon_pages";
pub const DOUBLE_BARREL_SUBTYPE: &str = "weapon";
pub const DOUBLE_BARREL_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Double-Barrel";
pub const DOUBLE_BARREL_SOURCE_REVISION: u32 = 21221;
pub const DOUBLE_BARREL_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const DOUBLE_BARREL_CONFIDENCE_BASIS_POINTS: u16 = 87;

pub const DOUBLE_BARREL_EFFECTS: &str = "Fires Shotgun shells, damaging entities or employees";
pub const DOUBLE_BARREL_WEIGHT: I32F32 = I32F32::lit("16");
pub const DOUBLE_BARREL_CONDUCTIVE: bool = false;
pub const DOUBLE_BARREL_SELL_VALUE: I32F32 = I32F32::lit("60");
pub const DOUBLE_BARREL_TWO_HANDED: bool = false;
pub const DOUBLE_BARREL_COOLDOWN_SECONDS: I32F32 = I32F32::lit("0.5");
pub const DOUBLE_BARREL_CHAMBERS: u8 = 2;
pub const DOUBLE_BARREL_SHOT_NOISE_AMOUNT: I32F32 = I32F32::lit("100");
pub const DOUBLE_BARREL_MISFIRE_PERCENT: u32 = 5;
pub const DOUBLE_BARREL_MISFIRE_ROLL_MODULUS: u32 = 100;
pub const DOUBLE_BARREL_MISFIRE_SALT: u64 = 0x646f_7562_6c65_6472;

pub const DOUBLE_BARREL_DAMAGE_NON_PLAYER: [I32F32; 3] = [
    I32F32::lit("2"),
    I32F32::lit("3"),
    I32F32::lit("5"),
];
pub const DOUBLE_BARREL_DAMAGE_PLAYER: [I32F32; 3] = [
    I32F32::lit("20"),
    I32F32::lit("40"),
    I32F32::lit("100"),
];
pub const DOUBLE_BARREL_DAMAGE_ZONE_NON_PLAYER_UNITS: [I32F32; 3] = [
    I32F32::lit("3.7"),
    I32F32::lit("6.0"),
    I32F32::lit("15.0"),
];
pub const DOUBLE_BARREL_DAMAGE_ZONE_PLAYER_UNITS: [I32F32; 3] = [
    I32F32::lit("15.0"),
    I32F32::lit("23.0"),
    I32F32::lit("30.0"),
];

pub const DOUBLE_BARREL_SPAWN_CHANCES: [DoubleBarrelSpawnChance; 10] = [
    DoubleBarrelSpawnChance {
        moon: "Rend",
        chance: I32F32::lit("23.42"),
    },
    DoubleBarrelSpawnChance {
        moon: "Titan",
        chance: I32F32::lit("11.64"),
    },
    DoubleBarrelSpawnChance {
        moon: "Artifice",
        chance: I32F32::lit("7.94"),
    },
    DoubleBarrelSpawnChance {
        moon: "Embrion",
        chance: I32F32::lit("6.94"),
    },
    DoubleBarrelSpawnChance {
        moon: "Dine",
        chance: I32F32::lit("5.77"),
    },
    DoubleBarrelSpawnChance {
        moon: "March",
        chance: I32F32::lit("0.83"),
    },
    DoubleBarrelSpawnChance {
        moon: "Offense",
        chance: I32F32::lit("0.82"),
    },
    DoubleBarrelSpawnChance {
        moon: "Experimentation",
        chance: I32F32::lit("0.44"),
    },
    DoubleBarrelSpawnChance {
        moon: "Adamance",
        chance: I32F32::lit("0.38"),
    },
    DoubleBarrelSpawnChance {
        moon: "Assurance",
        chance: I32F32::lit("0.29"),
    },
];

pub const DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS: DoubleBarrelSpecialItem = DoubleBarrelSpecialItem {
    name: "shotgun_shells",
    value: I32F32::lit("0"),
    weight_lbs: I32F32::lit("0"),
    conductive: false,
    two_handed: false,
    info: "Two are dropped whenever a nutcracker dies; they can be used as ammunition for the Double-Barrel.",
};

pub const DOUBLE_BARREL_BEHAVIORAL_MECHANICS: [DoubleBarrelBehaviorRule; 7] = [
    DoubleBarrelBehaviorRule {
        condition: "the trigger is pulled while at least 1 shell is loaded and the safety is off",
        outcome: "the weapon fires 1 shot and enters a 0.5-second cooldown",
    },
    DoubleBarrelBehaviorRule {
        condition: "the trigger is pulled while no ammunition is loaded",
        outcome: "the weapon produces a click and fires nothing",
    },
    DoubleBarrelBehaviorRule {
        condition: "the reload key is pressed while the weapon has at least 1 empty chamber and the user has compatible ammunition in inventory",
        outcome: "1 shell is moved into the weapon",
    },
    DoubleBarrelBehaviorRule {
        condition: "the target is a non-player entity",
        outcome: "damage falls across 3 zones: 5 HP within 3.7 units, 3 HP from 3.7 to 6.0 units, and 2 HP from 6.0 to 15.0 units",
    },
    DoubleBarrelBehaviorRule {
        condition: "the target is a player employee",
        outcome: "damage falls across 3 zones: 100 HP within 15.0 units, 40 HP from 15.0 to 23.0 units, and 20 HP from 23.0 to 30.0 units",
    },
    DoubleBarrelBehaviorRule {
        condition: "the safety switch is toggled",
        outcome: "the weapon changes between armed and safe states",
    },
    DoubleBarrelBehaviorRule {
        condition: "the weapon is dropped",
        outcome: "there is about a 5% chance it misfires and consumes 1 loaded shell",
    },
];

pub struct DoubleBarrelPlugin;

impl Plugin for DoubleBarrelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDoubleBarrelEvent>()
            .add_event::<DoubleBarrelTriggerPulledEvent>()
            .add_event::<DoubleBarrelFiredEvent>()
            .add_event::<DoubleBarrelClickedEvent>()
            .add_event::<DoubleBarrelReloadRequestedEvent>()
            .add_event::<DoubleBarrelReloadedEvent>()
            .add_event::<DoubleBarrelSafetyToggledEvent>()
            .add_event::<DoubleBarrelDroppedEvent>()
            .add_event::<DoubleBarrelMisfiredEvent>()
            .add_event::<DoubleBarrelSoldEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_double_barrel,
                    double_barrel_pickup_item_bar_bridge,
                    double_barrel_tick_cooldowns,
                    double_barrel_use_from_item_bar,
                    double_barrel_trigger_pulled,
                    double_barrel_reload_requested,
                    double_barrel_toggle_safety,
                    double_barrel_drop_misfire,
                    double_barrel_emit_noise,
                    double_barrel_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelSpecialItem {
    pub name: &'static str,
    pub value: I32F32,
    pub weight_lbs: I32F32,
    pub conductive: bool,
    pub two_handed: bool,
    pub info: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DoubleBarrel {
    pub stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelWeapon {
    pub weight: I32F32,
    pub conductive: bool,
    pub sell_value: I32F32,
    pub two_handed: bool,
}

impl Default for DoubleBarrelWeapon {
    fn default() -> Self {
        Self {
            weight: DOUBLE_BARREL_WEIGHT,
            conductive: DOUBLE_BARREL_CONDUCTIVE,
            sell_value: DOUBLE_BARREL_SELL_VALUE,
            two_handed: DOUBLE_BARREL_TWO_HANDED,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DoubleBarrelHeldBy {
    pub employee_id: u64,
    pub is_held: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelAmmo {
    pub loaded_shells: u8,
    pub reserve_shells: u8,
}

impl Default for DoubleBarrelAmmo {
    fn default() -> Self {
        Self {
            loaded_shells: 0,
            reserve_shells: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelFireState {
    pub safety_on: bool,
    pub cooldown_ticks_remaining: u32,
    pub shots_fired: u64,
    pub clicks: u64,
    pub reloads: u64,
    pub misfires: u64,
    pub last_used_tick: u64,
}

impl Default for DoubleBarrelFireState {
    fn default() -> Self {
        Self {
            safety_on: true,
            cooldown_ticks_remaining: 0,
            shots_fired: 0,
            clicks: 0,
            reloads: 0,
            misfires: 0,
            last_used_tick: 0,
        }
    }
}

#[derive(Bundle)]
pub struct DoubleBarrelBundle {
    pub name: Name,
    pub double_barrel: DoubleBarrel,
    pub weapon: DoubleBarrelWeapon,
    pub position: SimPosition,
    pub held_by: DoubleBarrelHeldBy,
    pub ammo: DoubleBarrelAmmo,
    pub fire_state: DoubleBarrelFireState,
}

impl DoubleBarrelBundle {
    pub fn new(event: SpawnDoubleBarrelEvent) -> Self {
        Self {
            name: Name::new(DOUBLE_BARREL_NAME),
            double_barrel: DoubleBarrel {
                stable_id: event.stable_id,
            },
            weapon: DoubleBarrelWeapon::default(),
            position: event.position,
            held_by: DoubleBarrelHeldBy::default(),
            ammo: DoubleBarrelAmmo {
                loaded_shells: event.loaded_shells.min(DOUBLE_BARREL_CHAMBERS),
                reserve_shells: event.reserve_shells,
            },
            fire_state: DoubleBarrelFireState::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnDoubleBarrelEvent {
    pub stable_id: u64,
    pub position: SimPosition,
    pub loaded_shells: u8,
    pub reserve_shells: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelTriggerPulledEvent {
    pub weapon: Entity,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub user_position: SimPosition,
    pub target_position: SimPosition,
    pub target_kind: DoubleBarrelTargetKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelFiredEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub target: Option<Entity>,
    pub damage: I32F32,
    pub target_kind: DoubleBarrelTargetKind,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelClickedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user: Entity,
    pub user_stable_id: u64,
    pub safety_on: bool,
    pub loaded_shells: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelReloadRequestedEvent {
    pub weapon: Entity,
    pub user_stable_id: u64,
    pub compatible_shells_available: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelReloadedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user_stable_id: u64,
    pub loaded_shells: u8,
    pub reserve_shells: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelSafetyToggledEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub safety_on: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelDroppedEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub user_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelMisfiredEvent {
    pub weapon: Entity,
    pub weapon_stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleBarrelSoldEvent {
    pub weapon_stable_id: u64,
    pub credit_value: I32F32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DoubleBarrelTargetKind {
    #[default]
    NonPlayerEntity = 0,
    PlayerEmployee = 1,
}

pub fn double_barrel_spawn_chance_for_moon(moon: &str) -> Option<I32F32> {
    DOUBLE_BARREL_SPAWN_CHANCES
        .iter()
        .find(|spawn_chance| spawn_chance.moon == moon)
        .map(|spawn_chance| spawn_chance.chance)
}

pub fn double_barrel_damage_for_distance(
    target_kind: DoubleBarrelTargetKind,
    distance_squared: I32F32,
) -> I32F32 {
    match target_kind {
        DoubleBarrelTargetKind::NonPlayerEntity => damage_from_zones(
            distance_squared,
            DOUBLE_BARREL_DAMAGE_ZONE_NON_PLAYER_UNITS,
            [
                DOUBLE_BARREL_DAMAGE_NON_PLAYER[2],
                DOUBLE_BARREL_DAMAGE_NON_PLAYER[1],
                DOUBLE_BARREL_DAMAGE_NON_PLAYER[0],
            ],
        ),
        DoubleBarrelTargetKind::PlayerEmployee => damage_from_zones(
            distance_squared,
            DOUBLE_BARREL_DAMAGE_ZONE_PLAYER_UNITS,
            [
                DOUBLE_BARREL_DAMAGE_PLAYER[2],
                DOUBLE_BARREL_DAMAGE_PLAYER[1],
                DOUBLE_BARREL_DAMAGE_PLAYER[0],
            ],
        ),
    }
}

fn damage_from_zones(
    distance_squared: I32F32,
    zone_radii: [I32F32; 3],
    zone_damage_near_to_far: [I32F32; 3],
) -> I32F32 {
    let near_squared = zone_radii[0] * zone_radii[0];
    let mid_squared = zone_radii[1] * zone_radii[1];
    let far_squared = zone_radii[2] * zone_radii[2];

    if distance_squared <= near_squared {
        zone_damage_near_to_far[0]
    } else if distance_squared <= mid_squared {
        zone_damage_near_to_far[1]
    } else if distance_squared <= far_squared {
        zone_damage_near_to_far[2]
    } else {
        I32F32::ZERO
    }
}

fn spawn_double_barrel(mut commands: Commands, mut events: EventReader<SpawnDoubleBarrelEvent>) {
    for event in events.read() {
        commands.spawn(DoubleBarrelBundle::new(*event));
    }
}

fn double_barrel_pickup_item_bar_bridge(
    mut pickup_events: EventWriter<ItemBarPickupEvent>,
    weapons: Query<(&DoubleBarrel, &DoubleBarrelHeldBy), Changed<DoubleBarrelHeldBy>>,
) {
    for (weapon, held_by) in &weapons {
        if held_by.is_held {
            pickup_events.send(ItemBarPickupEvent {
                employee_id: held_by.employee_id,
                item_id: DOUBLE_BARREL_ID,
                two_handed: DOUBLE_BARREL_TWO_HANDED,
                functional: true,
                passive: false,
                from_store_or_valueless: true,
            });
        } else {
            let _ = weapon.stable_id;
        }
    }
}

fn double_barrel_tick_cooldowns(mut weapons: Query<&mut DoubleBarrelFireState, With<DoubleBarrel>>) {
    for mut fire_state in &mut weapons {
        if fire_state.cooldown_ticks_remaining > 0 {
            fire_state.cooldown_ticks_remaining -= 1;
        }
    }
}

fn double_barrel_use_from_item_bar(
    mut item_events: EventReader<ItemBarItemEffectEvent>,
    mut trigger_events: EventWriter<DoubleBarrelTriggerPulledEvent>,
    weapons: Query<(Entity, &DoubleBarrelHeldBy, &SimPosition), With<DoubleBarrel>>,
) {
    for event in item_events.read() {
        if event.item_id != DOUBLE_BARREL_ID || event.effect != ItemBarItemEffect::FunctionalActivated
        {
            continue;
        }

        for (weapon, held_by, position) in &weapons {
            if !held_by.is_held || held_by.employee_id != event.employee_id {
                continue;
            }

            trigger_events.send(DoubleBarrelTriggerPulledEvent {
                weapon,
                user: weapon,
                user_stable_id: event.employee_id,
                target: None,
                user_position: *position,
                target_position: *position,
                target_kind: DoubleBarrelTargetKind::NonPlayerEntity,
            });
        }
    }
}

fn double_barrel_trigger_pulled(
    sim_hz: Res<SimHz>,
    tick: Res<SimTick>,
    mut trigger_events: EventReader<DoubleBarrelTriggerPulledEvent>,
    mut fired_events: EventWriter<DoubleBarrelFiredEvent>,
    mut clicked_events: EventWriter<DoubleBarrelClickedEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut weapons: Query<(
        Entity,
        &DoubleBarrel,
        &mut DoubleBarrelAmmo,
        &mut DoubleBarrelFireState,
    )>,
) {
    let cooldown_ticks = seconds_to_ticks(DOUBLE_BARREL_COOLDOWN_SECONDS, sim_hz.0);

    for event in trigger_events.read() {
        let Ok((weapon_entity, weapon, mut ammo, mut fire_state)) = weapons.get_mut(event.weapon)
        else {
            continue;
        };

        if fire_state.cooldown_ticks_remaining > 0 || fire_state.safety_on || ammo.loaded_shells == 0
        {
            fire_state.clicks = fire_state.clicks.wrapping_add(1);
            clicked_events.send(DoubleBarrelClickedEvent {
                weapon: weapon_entity,
                weapon_stable_id: weapon.stable_id,
                user: event.user,
                user_stable_id: event.user_stable_id,
                safety_on: fire_state.safety_on,
                loaded_shells: ammo.loaded_shells,
            });
            continue;
        }

        ammo.loaded_shells -= 1;
        fire_state.shots_fired = fire_state.shots_fired.wrapping_add(1);
        fire_state.last_used_tick = tick.0;
        fire_state.cooldown_ticks_remaining = cooldown_ticks;

        let damage = event
            .target
            .map(|_| {
                let dx = event.target_position.x - event.user_position.x;
                let dy = event.target_position.y - event.user_position.y;
                double_barrel_damage_for_distance(event.target_kind, dx * dx + dy * dy)
            })
            .unwrap_or(I32F32::ZERO);

        if let Some(target) = event.target {
            if damage > I32F32::ZERO {
                damage_events.send(IncomingDamageEvent {
                    target,
                    raw_amount: damage,
                    damage_type: DamageType::Standard,
                    source: weapon_entity,
                });
            }
        }

        fired_events.send(DoubleBarrelFiredEvent {
            weapon: weapon_entity,
            weapon_stable_id: weapon.stable_id,
            user: event.user,
            user_stable_id: event.user_stable_id,
            target: event.target,
            damage,
            target_kind: event.target_kind,
            position: event.user_position,
        });
    }
}

fn double_barrel_reload_requested(
    mut reload_events: EventReader<DoubleBarrelReloadRequestedEvent>,
    mut reloaded_events: EventWriter<DoubleBarrelReloadedEvent>,
    mut weapons: Query<(&DoubleBarrel, &mut DoubleBarrelAmmo)>,
) {
    for event in reload_events.read() {
        let Ok((weapon, mut ammo)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if ammo.loaded_shells >= DOUBLE_BARREL_CHAMBERS || !event.compatible_shells_available {
            continue;
        }

        if ammo.reserve_shells == 0 {
            continue;
        }

        ammo.reserve_shells -= 1;
        ammo.loaded_shells += 1;

        reloaded_events.send(DoubleBarrelReloadedEvent {
            weapon: event.weapon,
            weapon_stable_id: weapon.stable_id,
            user_stable_id: event.user_stable_id,
            loaded_shells: ammo.loaded_shells,
            reserve_shells: ammo.reserve_shells,
        });
    }
}

fn double_barrel_toggle_safety(
    mut toggle_events: EventReader<DoubleBarrelSafetyToggledEvent>,
    mut weapons: Query<(&DoubleBarrel, &mut DoubleBarrelFireState)>,
) {
    for event in toggle_events.read() {
        let Ok((weapon, mut fire_state)) = weapons.get_mut(event.weapon) else {
            continue;
        };

        if weapon.stable_id == event.weapon_stable_id {
            fire_state.safety_on = event.safety_on;
        }
    }
}

fn double_barrel_drop_misfire(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut drop_events: EventReader<DoubleBarrelDroppedEvent>,
    mut misfire_events: EventWriter<DoubleBarrelMisfiredEvent>,
    mut fired_events: EventWriter<DoubleBarrelFiredEvent>,
    mut weapons: Query<(&DoubleBarrel, &mut DoubleBarrelAmmo, &mut DoubleBarrelHeldBy, &mut DoubleBarrelFireState)>,
) {
    for event in drop_events.read() {
        let Ok((weapon, mut ammo, mut held_by, mut fire_state)) = weapons.get_mut(event.weapon)
        else {
            continue;
        };

        if weapon.stable_id != event.weapon_stable_id {
            continue;
        }

        held_by.is_held = false;
        held_by.employee_id = 0;

        if ammo.loaded_shells == 0 {
            continue;
        }

        let salt = DOUBLE_BARREL_MISFIRE_SALT ^ weapon.stable_id;
        let mut rng = tick_rng(seed.0, tick.0, salt);
        let roll = rng.next_u32() % DOUBLE_BARREL_MISFIRE_ROLL_MODULUS;

        if roll < DOUBLE_BARREL_MISFIRE_PERCENT {
            ammo.loaded_shells -= 1;
            fire_state.misfires = fire_state.misfires.wrapping_add(1);
            fire_state.last_used_tick = tick.0;

            misfire_events.send(DoubleBarrelMisfiredEvent {
                weapon: event.weapon,
                weapon_stable_id: weapon.stable_id,
                position: event.position,
            });

            fired_events.send(DoubleBarrelFiredEvent {
                weapon: event.weapon,
                weapon_stable_id: weapon.stable_id,
                user: event.weapon,
                user_stable_id: event.user_stable_id,
                target: None,
                damage: I32F32::ZERO,
                target_kind: DoubleBarrelTargetKind::NonPlayerEntity,
                position: event.position,
            });
        }
    }
}

fn double_barrel_emit_noise(
    mut fired_events: EventReader<DoubleBarrelFiredEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
) {
    for event in fired_events.read() {
        noise_events.send(NoiseEmittedEvent {
            source: event.weapon,
            position: event.position,
            amount: DOUBLE_BARREL_SHOT_NOISE_AMOUNT,
        });
    }
}

fn double_barrel_checksum(
    mut checksum: ResMut<SimChecksumState>,
    weapons: Query<(
        &DoubleBarrel,
        &DoubleBarrelWeapon,
        &SimPosition,
        &DoubleBarrelHeldBy,
        &DoubleBarrelAmmo,
        &DoubleBarrelFireState,
    )>,
) {
    accumulate_str(&mut checksum, 0x1000, DOUBLE_BARREL_ID);
    accumulate_str(&mut checksum, 0x1001, DOUBLE_BARREL_NAME);
    accumulate_str(&mut checksum, 0x1002, DOUBLE_BARREL_TYPE);
    accumulate_str(&mut checksum, 0x1003, DOUBLE_BARREL_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, DOUBLE_BARREL_EFFECTS);
    accumulate_str(&mut checksum, 0x1005, DOUBLE_BARREL_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1006, DOUBLE_BARREL_EXTRACTED_AT);

    checksum.accumulate(DOUBLE_BARREL_SOURCE_REVISION as u64);
    checksum.accumulate(DOUBLE_BARREL_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(DOUBLE_BARREL_WEIGHT.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_CONDUCTIVE as u64);
    checksum.accumulate(DOUBLE_BARREL_SELL_VALUE.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_TWO_HANDED as u64);
    checksum.accumulate(DOUBLE_BARREL_COOLDOWN_SECONDS.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_CHAMBERS as u64);
    checksum.accumulate(DOUBLE_BARREL_SHOT_NOISE_AMOUNT.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_MISFIRE_PERCENT as u64);
    checksum.accumulate(DOUBLE_BARREL_MISFIRE_ROLL_MODULUS as u64);

    for damage in DOUBLE_BARREL_DAMAGE_NON_PLAYER {
        checksum.accumulate(damage.to_bits() as u64);
    }

    for damage in DOUBLE_BARREL_DAMAGE_PLAYER {
        checksum.accumulate(damage.to_bits() as u64);
    }

    for zone in DOUBLE_BARREL_DAMAGE_ZONE_NON_PLAYER_UNITS {
        checksum.accumulate(zone.to_bits() as u64);
    }

    for zone in DOUBLE_BARREL_DAMAGE_ZONE_PLAYER_UNITS {
        checksum.accumulate(zone.to_bits() as u64);
    }

    for spawn_chance in DOUBLE_BARREL_SPAWN_CHANCES {
        accumulate_str(&mut checksum, 0x2000, spawn_chance.moon);
        checksum.accumulate(spawn_chance.chance.to_bits() as u64);
    }

    accumulate_str(
        &mut checksum,
        0x3000,
        DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.name,
    );
    checksum.accumulate(DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.value.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.weight_lbs.to_bits() as u64);
    checksum.accumulate(DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.conductive as u64);
    checksum.accumulate(DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.two_handed as u64);
    accumulate_str(
        &mut checksum,
        0x3001,
        DOUBLE_BARREL_SPECIAL_ITEM_SHOTGUN_SHELLS.info,
    );

    for rule in DOUBLE_BARREL_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (weapon, weapon_data, position, held_by, ammo, fire_state) in &weapons {
        checksum.accumulate(weapon.stable_id);
        checksum.accumulate(weapon_data.weight.to_bits() as u64);
        checksum.accumulate(weapon_data.conductive as u64);
        checksum.accumulate(weapon_data.sell_value.to_bits() as u64);
        checksum.accumulate(weapon_data.two_handed as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(held_by.employee_id);
        checksum.accumulate(held_by.is_held as u64);
        checksum.accumulate(ammo.loaded_shells as u64);
        checksum.accumulate(ammo.reserve_shells as u64);
        checksum.accumulate(fire_state.safety_on as u64);
        checksum.accumulate(fire_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(fire_state.shots_fired);
        checksum.accumulate(fire_state.clicks);
        checksum.accumulate(fire_state.reloads);
        checksum.accumulate(fire_state.misfires);
        checksum.accumulate(fire_state.last_used_tick);
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