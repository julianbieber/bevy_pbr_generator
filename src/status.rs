//! What the compute pipeline is currently doing, as the status bar reports it.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

/// Outcome of the last attempt to build the active material's compute pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CompileState {
    /// The shader asset has not finished loading. Not an error: nothing is
    /// dispatched, and the previously generated textures stay bound.
    #[default]
    Loading,
    Ok,
    /// Compilation failed, with the message naga produced. The previous
    /// textures stay bound and the session continues.
    Failed(String),
}

/// The pipeline state, written in the render world and read in the main world.
///
/// Behind a mutex because it is the one piece of state both worlds touch: the
/// render world learns whether the pipeline compiled, and only the main world
/// can draw the result.
#[derive(Debug, Default)]
pub struct ShaderStatus {
    pub compile: CompileState,
    pub last_dispatch_micros: u64,
}

/// Handle to the [`ShaderStatus`], inserted as a resource into both worlds.
#[derive(Resource, Clone, Default)]
pub struct SharedStatus(pub Arc<Mutex<ShaderStatus>>);

impl SharedStatus {
    /// Runs `f` against the shared status, doing nothing if the mutex has been
    /// poisoned — a status bar is never worth taking the app down for.
    pub fn with<R>(&self, f: impl FnOnce(&mut ShaderStatus) -> R) -> Option<R> {
        self.0.lock().ok().map(|mut status| f(&mut status))
    }

    /// The current compile state, or [`CompileState::Loading`] if the mutex is
    /// poisoned.
    pub fn compile_state(&self) -> CompileState {
        self.with(|s| s.compile.clone()).unwrap_or_default()
    }
}
