// Sources: vault/buildings/command_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const COMMAND_CENTER_HP: I32F32 = I32F32::lit("5000");
const COMMAND_CENTER_DEFENSES_LIFE: I32F32 = I32F32::lit("2500");
const COMMAND_CENTER_WATCH_RANGE: I32F32 = I32F32::lit("16");
const COMMAND_CENTER_ENERGY_COST: i32 = 0;
const COMMAND_CENTER_WOOD_COST: i32 = 0;
const COMMAND_CENTER_STONE_COST: i32 = 0;
const COMMAND_CENTER_IRON_COST: i32 = 0;
const COMMAND_CENTER_OIL_COST: i32 = 0;
const COMMAND_CENTER_GOLD_COST: i32 = 0;
const COMMAND_CENTER_BUILD_TIME_SECONDS: i32 = 0;
const COMMAND_CENTER_WORKERS: i32 = 10;
const COMMAND_CENTER_PFOOD: i32 = 20;
const COMMAND_CENTER_PWOOD: i32 = 0;
const COMMAND_CENTER_PSTONE: i32 = 0;
const COMMAND_CENTER_PIRON: i32 = 0;
const COMMAND_CENTER_POIL: i32 = 0;
const COMMAND_CENTER_PGOLD: i32 = 200;
const COMMAND_CENTER_PENERGY: i32 = 30;
const COMMAND_CENTER_PCOLONISTS: i32 = 10;
const COMMAND_CENTER_STORAGE_CAPACITY: i32 = 50;
const COMMAND_CENTER_STORAGE_EXPANSION_COST_GOLD: i32 = 2000;
const COMMAND_CENTER_ENERGY_TRANSFER_RADIUS: i32 = 10;
const COMMAND_CENTER_SIZE_TILES: i32 = 5;
const COMMAND_CENTER_GOLD_INTERVAL_HOURS: i32 = 8;
const LOGISTICS_GOLD_BONUS: i32 = 40;
const LOGISTICS_STORAGE_BONUS: i32 = 20;
const BASIC_SUPPLIES_FOOD_BONUS: i32 = 20;
const BASIC_SUPPLIES_ENERGY_BONUS: i32 = 20;
const OPTICS_WATCH_RANGE_BONUS: I32F32 = I32F32::lit("5");
const TESLA_COILS_TRANSFER_RADIUS_BONUS: i32 = 4;
const TESLA_COILS_ENERGY_MULTIPLIER_NUM: i32 = 120;
const TESLA_COILS_ENERGY_MULTIPLIER_DEN: i32 = 100;
const SIM_HZ: i32 = 25;
const SIM_SECONDS_PER_HOUR: i32 = 3600;

#[derive(Component, Default)]
pub struct CommandCenter;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct CommandCenterCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for CommandCenterCore {
    fn default() -> Self {
        Self {
            defenses_life: COMMAND_CENTER_DEFENSES_LIFE,
            watch_range: COMMAND_CENTER_WATCH_RANGE,
            health: Health::full(COMMAND_CENTER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CommandCenterEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub pfood: i32,
    pub pwood: i32,
    pub pstone: i32,
    pub piron: i32,
    pub poil: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
}

impl Default for CommandCenterEconomy {
    fn default() -> Self {
        Self {
            energy_cost: COMMAND_CENTER_ENERGY_COST,
            wood_cost: COMMAND_CENTER_WOOD_COST,
            stone_cost: COMMAND_CENTER_STONE_COST,
            iron_cost: COMMAND_CENTER_IRON_COST,
            oil_cost: COMMAND_CENTER_OIL_COST,
            gold_cost: COMMAND_CENTER_GOLD_COST,
            workers: COMMAND_CENTER_WORKERS,
            pfood: COMMAND_CENTER_PFOOD,
            pwood: COMMAND_CENTER_PWOOD,
            pstone: COMMAND_CENTER_PSTONE,
            piron: COMMAND_CENTER_PIRON,
            poil: COMMAND_CENTER_POIL,
            pgold: COMMAND_CENTER_PGOLD,
            penergy: COMMAND_CENTER_PENERGY,
            pcolonists: COMMAND_CENTER_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CommandCenterFootprint {
    pub size_tiles: i32,
    pub energy_transfer_radius_tiles: i32,
}

impl Default for CommandCenterFootprint {
    fn default() -> Self {
        Self {
            size_tiles: COMMAND_CENTER_SIZE_TILES,
            energy_transfer_radius_tiles: COMMAND_CENTER_ENERGY_TRANSFER_RADIUS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CommandCenterBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct CommandCenterStorage {
    pub base_capacity: i32,
    pub bonus_capacity: i32,
    pub expansion_cost_gold: i32,
}

impl Default for CommandCenterStorage {
    fn default() -> Self {
        Self {
            base_capacity: COMMAND_CENTER_STORAGE_CAPACITY,
            bonus_capacity: 0,
            expansion_cost_gold: COMMAND_CENTER_STORAGE_EXPANSION_COST_GOLD,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CommandCenterState {
    pub destroyed: bool,
    pub infected: bool,
    pub repairing: bool,
    pub can_issue_build_orders: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct CommandCenterTechUpgrades {
    pub logistics: bool,
    pub military_training: bool,
    pub basic_supplies: bool,
    pub optics: bool,
    pub tesla_coils: bool,
    pub titanium: bool,
    pub silent_beholder_upgrade_path: bool,
}

#[derive(Component, Clone, Copy)]
pub struct CommandCenterGoldPulseTimer {
    pub ticks_until_next_pulse: i32,
}

impl Default for CommandCenterGoldPulseTimer {
    fn default() -> Self {
        Self {
            ticks_until_next_pulse: gold_interval_ticks(),
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct GameOverState {
    pub lost: bool,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct BuildOrderState {
    pub can_start_construction: bool,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct CommandCenterRegistry {
    pub entity: Option<Entity>,
}

#[derive(Event, Clone, Copy)]
pub struct StartNewGameEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetCommandCenterDestroyedEvent {
    pub entity: Entity,
    pub destroyed: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetCommandCenterInfectedEvent {
    pub entity: Entity,
    pub infected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetCommandCenterRepairingEvent {
    pub entity: Entity,
    pub repairing: bool,
}

#[derive(Event, Clone, Copy)]
pub struct AttemptIssueBuildOrderEvent {
    pub command_center: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct ExpandCommandCenterStorageEvent {
    pub entity: Entity,
    pub paid_gold: i32,
}

#[derive(Event, Clone, Copy)]
pub struct ApplyCommandCenterTechnologyEvent {
    pub entity: Entity,
    pub upgrade: CommandCenterTechnology,
}

#[derive(Event, Clone, Copy)]
pub struct SetCommandCenterSilentBeholderUpgradePathEvent {
    pub entity: Entity,
    pub enabled: bool,
}

#[derive(Event, Clone, Copy)]
pub struct CommandCenterBuildOrderIssuedEvent {
    pub command_center: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct CommandCenterBuildOrderBlockedEvent {
    pub command_center: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct CommandCenterGoldPulseEvent {
    pub command_center: Entity,
    pub gold_amount: i32,
}

#[derive(Clone, Copy)]
pub enum CommandCenterTechnology {
    Logistics,
    MilitaryTraining,
    BasicSupplies,
    Optics,
    TeslaCoils,
    Titanium,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn gold_interval_ticks() -> i32 {
    COMMAND_CENTER_GOLD_INTERVAL_HOURS * SIM_SECONDS_PER_HOUR * SIM_HZ
}

fn start_new_game_spawn_command_center_system(
    mut commands: Commands,
    mut events: EventReader<StartNewGameEvent>,
    mut registry: ResMut<CommandCenterRegistry>,
) {
    for ev in events.read() {
        if registry.entity.is_some() {
            continue;
        }

        let entity = commands
            .spawn((
                CommandCenter,
                BuildingAnchor {
                    x: ev.tile_x,
                    y: ev.tile_y,
                },
                CommandCenterCore::default(),
                CommandCenterEconomy::default(),
                CommandCenterFootprint::default(),
                CommandCenterBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(COMMAND_CENTER_BUILD_TIME_SECONDS),
                    completed: true,
                },
                CommandCenterStorage::default(),
                CommandCenterState {
                    destroyed: false,
                    infected: false,
                    repairing: false,
                    can_issue_build_orders: true,
                },
                CommandCenterTechUpgrades::default(),
                CommandCenterGoldPulseTimer::default(),
            ))
            .id();

        registry.entity = Some(entity);
    }
}

fn command_center_apply_destroyed_event_system(
    mut events: EventReader<SetCommandCenterDestroyedEvent>,
    mut centers: Query<(&mut CommandCenterState, &mut CommandCenterCore), With<CommandCenter>>,
) {
    for ev in events.read() {
        let Ok((mut state, mut core)) = centers.get_mut(ev.entity) else {
            continue;
        };

        state.destroyed = ev.destroyed;
        if state.destroyed {
            core.health.current = I32F32::ZERO;
        }
    }
}

fn command_center_apply_infected_event_system(
    mut events: EventReader<SetCommandCenterInfectedEvent>,
    mut centers: Query<&mut CommandCenterState, With<CommandCenter>>,
) {
    for ev in events.read() {
        let Ok(mut state) = centers.get_mut(ev.entity) else {
            continue;
        };

        state.infected = ev.infected;
    }
}

fn command_center_apply_repairing_event_system(
    mut events: EventReader<SetCommandCenterRepairingEvent>,
    mut centers: Query<&mut CommandCenterState, With<CommandCenter>>,
) {
    for ev in events.read() {
        let Ok(mut state) = centers.get_mut(ev.entity) else {
            continue;
        };

        state.repairing = ev.repairing;
    }
}

fn command_center_apply_silent_beholder_upgrade_path_system(
    mut events: EventReader<SetCommandCenterSilentBeholderUpgradePathEvent>,
    mut centers: Query<&mut CommandCenterTechUpgrades, With<CommandCenter>>,
) {
    for ev in events.read() {
        let Ok(mut upgrades) = centers.get_mut(ev.entity) else {
            continue;
        };

        upgrades.silent_beholder_upgrade_path = ev.enabled;
    }
}

fn command_center_apply_technology_event_system(
    mut events: EventReader<ApplyCommandCenterTechnologyEvent>,
    mut centers: Query<
        (
            &mut CommandCenterEconomy,
            &mut CommandCenterCore,
            &mut CommandCenterFootprint,
            &mut CommandCenterStorage,
            &mut CommandCenterTechUpgrades,
        ),
        With<CommandCenter>,
    >,
) {
    for ev in events.read() {
        let Ok((mut eco, mut core, mut footprint, mut storage, mut upgrades)) =
            centers.get_mut(ev.entity)
        else {
            continue;
        };

        match ev.upgrade {
            CommandCenterTechnology::Logistics => {
                if upgrades.logistics {
                    continue;
                }
                upgrades.logistics = true;
                eco.pgold += LOGISTICS_GOLD_BONUS;
                storage.bonus_capacity += LOGISTICS_STORAGE_BONUS;
            }
            CommandCenterTechnology::MilitaryTraining => {
                if upgrades.military_training {
                    continue;
                }
                upgrades.military_training = true;
            }
            CommandCenterTechnology::BasicSupplies => {
                if upgrades.basic_supplies {
                    continue;
                }
                upgrades.basic_supplies = true;
                eco.pfood += BASIC_SUPPLIES_FOOD_BONUS;
                eco.penergy += BASIC_SUPPLIES_ENERGY_BONUS;
            }
            CommandCenterTechnology::Optics => {
                if upgrades.optics {
                    continue;
                }
                upgrades.optics = true;
                core.watch_range += OPTICS_WATCH_RANGE_BONUS;
            }
            CommandCenterTechnology::TeslaCoils => {
                if upgrades.tesla_coils {
                    continue;
                }
                upgrades.tesla_coils = true;
                footprint.energy_transfer_radius_tiles += TESLA_COILS_TRANSFER_RADIUS_BONUS;
                eco.penergy = (eco.penergy * TESLA_COILS_ENERGY_MULTIPLIER_NUM)
                    / TESLA_COILS_ENERGY_MULTIPLIER_DEN;
            }
            CommandCenterTechnology::Titanium => {
                if upgrades.titanium {
                    continue;
                }
                upgrades.titanium = true;
            }
        }
    }
}

fn command_center_expand_storage_system(
    mut events: EventReader<ExpandCommandCenterStorageEvent>,
    mut centers: Query<&mut CommandCenterStorage, With<CommandCenter>>,
) {
    for ev in events.read() {
        let Ok(mut storage) = centers.get_mut(ev.entity) else {
            continue;
        };

        if ev.paid_gold != storage.expansion_cost_gold {
            continue;
        }

        storage.bonus_capacity += COMMAND_CENTER_STORAGE_CAPACITY;
    }
}

fn command_center_issue_build_order_system(
    mut events: EventReader<AttemptIssueBuildOrderEvent>,
    centers: Query<&CommandCenterState, With<CommandCenter>>,
    mut issued: EventWriter<CommandCenterBuildOrderIssuedEvent>,
    mut blocked: EventWriter<CommandCenterBuildOrderBlockedEvent>,
) {
    for ev in events.read() {
        let Ok(state) = centers.get(ev.command_center) else {
            continue;
        };

        if state.destroyed || state.infected || state.repairing {
            blocked.send(CommandCenterBuildOrderBlockedEvent {
                command_center: ev.command_center,
            });
            continue;
        }

        issued.send(CommandCenterBuildOrderIssuedEvent {
            command_center: ev.command_center,
        });
    }
}

fn command_center_gold_pulse_system(
    mut centers: Query<
        (
            Entity,
            &CommandCenterBuildState,
            &CommandCenterState,
            &CommandCenterEconomy,
            &mut CommandCenterGoldPulseTimer,
        ),
        With<CommandCenter>,
    >,
    mut pulses: EventWriter<CommandCenterGoldPulseEvent>,
) {
    for (entity, build, state, eco, mut timer) in &mut centers {
        if !build.completed || state.destroyed || state.infected {
            continue;
        }

        if timer.ticks_until_next_pulse > 1 {
            timer.ticks_until_next_pulse -= 1;
            continue;
        }

        timer.ticks_until_next_pulse = gold_interval_ticks();
        pulses.send(CommandCenterGoldPulseEvent {
            command_center: entity,
            gold_amount: eco.pgold,
        });
    }
}

fn command_center_derive_state_system(
    mut centers: Query<(&CommandCenterCore, &mut CommandCenterState), With<CommandCenter>>,
    mut game_over: ResMut<GameOverState>,
    mut build_orders: ResMut<BuildOrderState>,
) {
    build_orders.can_start_construction = true;

    for (core, mut state) in &mut centers {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }

        state.can_issue_build_orders = !state.destroyed && !state.infected && !state.repairing;

        if state.destroyed || state.infected {
            game_over.lost = true;
        }

        if state.repairing {
            build_orders.can_start_construction = false;
        }
    }
}

fn command_center_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    centers: Query<
        (
            Entity,
            &BuildingAnchor,
            &CommandCenterCore,
            &CommandCenterEconomy,
            &CommandCenterFootprint,
            &CommandCenterBuildState,
            &CommandCenterStorage,
            &CommandCenterState,
            &CommandCenterTechUpgrades,
            &CommandCenterGoldPulseTimer,
        ),
        With<CommandCenter>,
    >,
    game_over: Res<GameOverState>,
    build_orders: Res<BuildOrderState>,
    registry: Res<CommandCenterRegistry>,
) {
    for (entity, anchor, core, eco, footprint, build, storage, state, upgrades, gold_timer) in &centers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);
        checksum.accumulate(eco.pfood as u64);
        checksum.accumulate(eco.pwood as u64);
        checksum.accumulate(eco.pstone as u64);
        checksum.accumulate(eco.piron as u64);
        checksum.accumulate(eco.poil as u64);
        checksum.accumulate(eco.pgold as u64);
        checksum.accumulate(eco.penergy as u64);
        checksum.accumulate(eco.pcolonists as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.energy_transfer_radius_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(storage.base_capacity as u64);
        checksum.accumulate(storage.bonus_capacity as u64);
        checksum.accumulate(storage.expansion_cost_gold as u64);

        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.infected));
        checksum.accumulate(u64::from(state.repairing));
        checksum.accumulate(u64::from(state.can_issue_build_orders));

        checksum.accumulate(u64::from(upgrades.logistics));
        checksum.accumulate(u64::from(upgrades.military_training));
        checksum.accumulate(u64::from(upgrades.basic_supplies));
        checksum.accumulate(u64::from(upgrades.optics));
        checksum.accumulate(u64::from(upgrades.tesla_coils));
        checksum.accumulate(u64::from(upgrades.titanium));
        checksum.accumulate(u64::from(upgrades.silent_beholder_upgrade_path));

        checksum.accumulate(gold_timer.ticks_until_next_pulse as u64);
    }

    checksum.accumulate(u64::from(game_over.lost));
    checksum.accumulate(u64::from(build_orders.can_start_construction));
    checksum.accumulate(registry.entity.map(Entity::to_bits).unwrap_or(0));
}

pub struct CommandCenterPlugin;

impl Plugin for CommandCenterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameOverState>()
            .init_resource::<BuildOrderState>()
            .init_resource::<CommandCenterRegistry>()
            .add_event::<StartNewGameEvent>()
            .add_event::<SetCommandCenterDestroyedEvent>()
            .add_event::<SetCommandCenterInfectedEvent>()
            .add_event::<SetCommandCenterRepairingEvent>()
            .add_event::<AttemptIssueBuildOrderEvent>()
            .add_event::<ExpandCommandCenterStorageEvent>()
            .add_event::<ApplyCommandCenterTechnologyEvent>()
            .add_event::<SetCommandCenterSilentBeholderUpgradePathEvent>()
            .add_event::<CommandCenterBuildOrderIssuedEvent>()
            .add_event::<CommandCenterBuildOrderBlockedEvent>()
            .add_event::<CommandCenterGoldPulseEvent>()
            .add_systems(
                FixedUpdate,
                (
                    start_new_game_spawn_command_center_system,
                    command_center_apply_destroyed_event_system,
                    command_center_apply_infected_event_system,
                    command_center_apply_repairing_event_system,
                    command_center_apply_silent_beholder_upgrade_path_system,
                    command_center_apply_technology_event_system,
                    command_center_expand_storage_system,
                    command_center_issue_build_order_system,
                    command_center_gold_pulse_system,
                    command_center_derive_state_system,
                    command_center_checksum_system,
                )
                    .chain(),
            );
    }
}