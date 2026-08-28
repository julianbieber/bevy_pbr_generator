#![recursion_limit = "512"]
#![allow(clippy::too_many_arguments)]
//! GPU-generated PBR texture sets: the material definitions read from disk, the
//! compute pipeline that renders them, and the editor built around both.

pub mod app;
pub mod gpu;
pub mod material;
pub mod status;
pub mod ui;
