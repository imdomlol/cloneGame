//! Headless smoke test: build the full plugin tree and run two Update ticks.
//!
//! Catches runtime panics — Bevy B0001 schedule-ambiguity errors, duplicate
//! resource/event registrations, and system-ordering conflicts — that
//! `cargo build` cannot detect because they fire only when the schedule is
//! first assembled at runtime. The loop driver runs `cargo test --test
//! app_smoke` after every turn that compiles; when this test panics, the
//! repair loop receives the full Bevy error message (which names the
//! conflicting systems and the shared component) and feeds it back to the
//! LLM for a fix-it turn.
//!
//! The plugin list comes from `clone_game::app_plugins::add_all`, which the
//! loop driver regenerates from every `impl Plugin for X` declaration under
//! `src/`. A newly-generated leaf is therefore covered by this test on the
//! next turn with zero hand-edits — that is the contract that closes the
//! pipeline gap where `app_smoke.rs` could silently fall behind the crate.

use bevy::prelude::*;
use clone_game::app_plugins;
use clone_game::sim::{GameSeed, SimHz, SimTick};

fn advance_sim_tick(mut tick: ResMut<SimTick>) {
    tick.0 = tick.0.wrapping_add(1);
}

/// Builds the full plugin tree without a window and runs two ticks.
///
/// Reaching the end of this test without panicking proves the app is free
/// of schedule-ambiguity errors (B0001), duplicate resource/event
/// registrations, and any other runtime-assembly panics. No assertions are
/// needed: a panic is the failure signal.
#[test]
fn app_smoke() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Time::<Fixed>::from_hz(25.0))
        .insert_resource(GameSeed(0xDEAD_BEEF_C0FFEE))
        .insert_resource(SimTick::default())
        .insert_resource(SimHz::default());
    app_plugins::add_all(&mut app);
    app.add_systems(FixedUpdate, advance_sim_tick);

    // One `update()` builds the schedule (catches B0001 ambiguities Bevy
    // detects upfront) and runs the Update schedule. To exercise FixedUpdate
    // systems — where most query conflicts actually fire — explicitly run the
    // FixedUpdate schedule twice via the world. Without this, B0001 errors
    // surfaced only when their owning system runs (e.g. unit `*_checksum`
    // systems, building `*_tick_system`s) would not panic and the gate would
    // pass a runtime bug through.
    app.update();
    app.world_mut().run_schedule(FixedUpdate);
    app.world_mut().run_schedule(FixedUpdate);
}
