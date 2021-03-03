// TODO remove this once sqlx releases the fix
// https://github.com/rust-lang/rust-clippy/issues/5849
#![allow(clippy::toplevel_ref_arg)]
pub mod configuration;
pub mod domain;
pub mod routes;
pub mod startup;
pub mod telemetry;
