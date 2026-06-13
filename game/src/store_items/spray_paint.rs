// Sources: vault/store_items/spray_paint.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

pub const BUY_COST: u32 = 50;
pub const SELL_VALUE: u32 = 0;
pub const WEIGHT_LBS: u32 = 0;
pub const CONDUCTIVE: bool = false;
pub const COLOR_COUNT: u32 = 4;
pub const SPRAY_WINDOW_SECONDS: I32F32 = I32F32::lit("1");
pub const TOTAL_PAINT_SECONDS: I32F32 = I32F32::lit("25");
pub const SPRAY_PAINT_COLOR_SALT: u64 = 0x7370_7261_795f_7061;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SprayPaintColor {
    Red,
    Purple,
    Green,
    Yellow,
}

impl SprayPaintColor {
    pub fn from_roll(roll: u32) -> Self {
        match roll % COLOR_COUNT {
            0 => Self::Red,
            1 => Self::Purple,
            2 => Self::Green,
            _ => Self::Yellow,
        }
    }

    pub fn checksum_bits(self) -> u64 {
        match self {
            Self::Red => 0,
            Self::Purple => 1,
            Self::Green => 2,
            Self::Yellow => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SprayPaintSurface {
    ShipInterior,
    ShipExterior,
    FacilityFirstRoom,
    TerminalOrMonitor,
    Other,
}

impl SprayPaintSurface {
    pub fn is_permanent(self) -> bool {
        matches!(
            self,
            Self::ShipInterior | Self::ShipExterior | Self::FacilityFirstRoom
        )
    }

    pub fn obscures_display(self) -> bool {
        matches!(self, Self::TerminalOrMonitor)
    }

    pub fn checksum_bits(self) -> u64 {
        match self {
            Self::ShipInterior => 0,
            Self::ShipExterior => 1,
            Self::FacilityFirstRoom => 2,
            Self::TerminalOrMonitor => 3,
            Self::Other => 4,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct SprayPaintCan {
    pub replicated_id: u64,
    pub paint_remaining_seconds: I32F32,
    pub spray_window_remaining_seconds: I32F32,
    pub depleted: bool,
}

impl Default for SprayPaintCan {
    fn default() -> Self {
        Self {
            replicated_id: 0,
            paint_remaining_seconds: TOTAL_PAINT_SECONDS,
            spray_window_remaining_seconds: SPRAY_WINDOW_SECONDS,
            depleted: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct SprayPaintMark {
    pub color: SprayPaintColor,
    pub surface: SprayPaintSurface,
    pub permanent: bool,
    pub display_obscured: bool,
}

#[derive(Event)]
pub struct SprayPaintPurchasedEvent;

#[derive(Event)]
pub struct SprayPaintUsedEvent {
    pub can: Entity,
    pub target: Option<Entity>,
    pub surface: SprayPaintSurface,
}

#[derive(Event)]
pub struct SprayPaintShakenEvent {
    pub can: Entity,
}

#[derive(Event)]
pub struct SprayPaintAppliedEvent {
    pub can: Entity,
    pub target: Option<Entity>,
    pub color: SprayPaintColor,
    pub surface: SprayPaintSurface,
    pub permanent: bool,
}

#[derive(Event)]
pub struct SprayPaintNeedsShakeEvent {
    pub can: Entity,
}

#[derive(Event)]
pub struct SprayPaintDepletedEvent {
    pub can: Entity,
}

#[derive(Event)]
pub struct SprayPaintDisplayObscuredEvent {
    pub can: Entity,
    pub target: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct SprayPaintState {
    pub purchased_count: u32,
    pub depleted_count: u32,
    pub displays_obscured: u32,
}

#[derive(Bundle, Default)]
pub struct SprayPaintBundle {
    pub can: SprayPaintCan,
}

pub struct SprayPaintPlugin;

impl Plugin for SprayPaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SprayPaintPurchasedEvent>()
            .add_event::<SprayPaintUsedEvent>()
            .add_event::<SprayPaintShakenEvent>()
            .add_event::<SprayPaintAppliedEvent>()
            .add_event::<SprayPaintNeedsShakeEvent>()
            .add_event::<SprayPaintDepletedEvent>()
            .add_event::<SprayPaintDisplayObscuredEvent>()
            .init_resource::<SprayPaintState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_spray_paint_purchase,
                    handle_spray_paint_shake,
                    handle_spray_paint_use,
                    handle_spray_paint_depletion,
                    spray_paint_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_spray_paint_purchase(
    mut events: EventReader<SprayPaintPurchasedEvent>,
    mut state: ResMut<SprayPaintState>,
) {
    for _ in events.read() {
        state.purchased_count = state.purchased_count.saturating_add(1);
    }
}

fn handle_spray_paint_shake(
    mut events: EventReader<SprayPaintShakenEvent>,
    mut cans: Query<&mut SprayPaintCan>,
) {
    for event in events.read() {
        if let Ok(mut can) = cans.get_mut(event.can) {
            if !can.depleted {
                can.spray_window_remaining_seconds = SPRAY_WINDOW_SECONDS;
            }
        }
    }
}

fn handle_spray_paint_use(
    mut events: EventReader<SprayPaintUsedEvent>,
    mut cans: Query<&mut SprayPaintCan>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut applied: EventWriter<SprayPaintAppliedEvent>,
    mut needs_shake: EventWriter<SprayPaintNeedsShakeEvent>,
    mut depleted: EventWriter<SprayPaintDepletedEvent>,
    mut obscured: EventWriter<SprayPaintDisplayObscuredEvent>,
) {
    for event in events.read() {
        let Ok(mut can) = cans.get_mut(event.can) else {
            continue;
        };

        if can.depleted || can.paint_remaining_seconds <= I32F32::lit("0") {
            can.depleted = true;
            depleted.send(SprayPaintDepletedEvent { can: event.can });
            continue;
        }

        if can.spray_window_remaining_seconds <= I32F32::lit("0") {
            needs_shake.send(SprayPaintNeedsShakeEvent { can: event.can });
            continue;
        }

        let salt = SPRAY_PAINT_COLOR_SALT ^ can.replicated_id;
        let mut rng = tick_rng(game_seed.0, tick.0, salt);
        let color = SprayPaintColor::from_roll(rng.next_u32());

        can.paint_remaining_seconds =
            (can.paint_remaining_seconds - SPRAY_WINDOW_SECONDS).max(I32F32::lit("0"));
        can.spray_window_remaining_seconds = I32F32::lit("0");

        if can.paint_remaining_seconds <= I32F32::lit("0") {
            can.depleted = true;
            depleted.send(SprayPaintDepletedEvent { can: event.can });
        }

        if event.surface.obscures_display() {
            obscured.send(SprayPaintDisplayObscuredEvent {
                can: event.can,
                target: event.target,
            });
        }

        applied.send(SprayPaintAppliedEvent {
            can: event.can,
            target: event.target,
            color,
            surface: event.surface,
            permanent: event.surface.is_permanent(),
        });
    }
}

fn handle_spray_paint_depletion(
    mut depleted: EventReader<SprayPaintDepletedEvent>,
    mut obscured: EventReader<SprayPaintDisplayObscuredEvent>,
    mut state: ResMut<SprayPaintState>,
) {
    for _ in depleted.read() {
        state.depleted_count = state.depleted_count.saturating_add(1);
    }

    for _ in obscured.read() {
        state.displays_obscured = state.displays_obscured.saturating_add(1);
    }
}

fn spray_paint_checksum(
    state: Res<SprayPaintState>,
    cans: Query<&SprayPaintCan>,
    marks: Query<&SprayPaintMark>,
    mut cs: ResMut<SimChecksumState>,
) {
    cs.accumulate(state.purchased_count as u64);
    cs.accumulate(state.depleted_count as u64);
    cs.accumulate(state.displays_obscured as u64);

    let mut can_bits = cans
        .iter()
        .map(|can| {
            (
                can.replicated_id,
                can.paint_remaining_seconds.to_bits() as u64,
                can.spray_window_remaining_seconds.to_bits() as u64,
                can.depleted as u64,
            )
        })
        .collect::<Vec<_>>();
    can_bits.sort_by_key(|bits| bits.0);

    for (replicated_id, paint_remaining, spray_window, depleted) in can_bits {
        cs.accumulate(replicated_id);
        cs.accumulate(paint_remaining);
        cs.accumulate(spray_window);
        cs.accumulate(depleted);
    }

    for mark in marks.iter() {
        cs.accumulate(mark.color.checksum_bits());
        cs.accumulate(mark.surface.checksum_bits());
        cs.accumulate(mark.permanent as u64);
        cs.accumulate(mark.display_obscured as u64);
    }
}