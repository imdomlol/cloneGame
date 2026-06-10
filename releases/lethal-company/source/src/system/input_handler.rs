// Sources: vault/item_index_pages/items.md, vault/version_pages/version_4.md

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// Emitted each Update tick while movement keys are held.
/// Direction is a unit vector in the XZ-plane (Y = forward).
#[derive(Event, Debug, Clone)]
pub struct InputMoveEvent {
    pub direction: Vec2,
}

/// Emitted on RMB press — activates held item (weapon fire, tool use).
/// Maps the Version-4 "RMB to use item" control.
#[derive(Event, Debug, Clone)]
pub struct InputUseItemEvent;

/// Emitted on LMB press — interacts with doors and ladders.
/// Maps the Version-4 "LMB to interact" control.
#[derive(Event, Debug, Clone)]
pub struct InputInteractEvent;

/// Emitted on E press — picks up nearby scrap or store item.
#[derive(Event, Debug, Clone)]
pub struct InputPickupEvent;

/// Emitted on mouse-wheel scroll — cycles the item bar slot.
/// Positive delta = scroll up (next slot), negative = scroll down (previous slot).
#[derive(Event, Debug, Clone)]
pub struct InputScrollItemBarEvent {
    pub delta: i32,
}

/// Emitted on Tab press — activates the echo scanner.
#[derive(Event, Debug, Clone)]
pub struct InputScanEvent;

/// Emitted on Q press — drops the currently held item.
#[derive(Event, Debug, Clone)]
pub struct InputDropItemEvent;

pub struct InputHandlerPlugin;

impl Plugin for InputHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_event::<MouseWheel>()
            .add_event::<InputMoveEvent>()
            .add_event::<InputUseItemEvent>()
            .add_event::<InputInteractEvent>()
            .add_event::<InputPickupEvent>()
            .add_event::<InputScrollItemBarEvent>()
            .add_event::<InputScanEvent>()
            .add_event::<InputDropItemEvent>()
            .add_systems(
                Update,
                (
                    movement_input_system,
                    use_item_input_system,
                    interact_input_system,
                    pickup_input_system,
                    drop_item_input_system,
                    item_bar_scroll_system,
                    scanner_input_system,
                )
                    .chain(),
            );
    }
}

fn movement_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut move_events: EventWriter<InputMoveEvent>,
) {
    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        move_events.send(InputMoveEvent {
            direction: direction.normalize(),
        });
    }
}

/// RMB activates the held item — weapon firing or tool use.
/// Weapons alter entity_targeting threat-level calculations while held (items.md rules).
fn use_item_input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut use_events: EventWriter<InputUseItemEvent>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        use_events.send(InputUseItemEvent);
    }
}

/// LMB interacts with doors and ladders (version_4 control mapping).
fn interact_input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut interact_events: EventWriter<InputInteractEvent>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        interact_events.send(InputInteractEvent);
    }
}

fn pickup_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut pickup_events: EventWriter<InputPickupEvent>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        pickup_events.send(InputPickupEvent);
    }
}

fn drop_item_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut drop_events: EventWriter<InputDropItemEvent>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        drop_events.send(InputDropItemEvent);
    }
}

/// Scroll wheel cycles the item bar. Each wheel tick emits one event with delta ±1.
fn item_bar_scroll_system(
    mut scroll_reader: EventReader<MouseWheel>,
    mut scroll_events: EventWriter<InputScrollItemBarEvent>,
) {
    for ev in scroll_reader.read() {
        let delta = if ev.y > 0.0 {
            1_i32
        } else if ev.y < 0.0 {
            -1_i32
        } else {
            continue;
        };
        scroll_events.send(InputScrollItemBarEvent { delta });
    }
}

fn scanner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut scan_events: EventWriter<InputScanEvent>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        scan_events.send(InputScanEvent);
    }
}