// generated from a2a v1
// model digest d675090b4779d71e3671099ac4d5efbaa17d7a58ae4e0ddaa6cafd8d5a23f002
// contract digest 3dea9905e0cd94c1255be15d5619996d104b54b00e2b8097da86f0bf80643b7f
// do not edit: regenerate with `ess synthesize --layout crate`

//! Semantic types synthesised from the `a2a` specification, v1.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod actor;
pub mod behaviour;
pub mod discovery;
pub mod json;
pub mod obligation;
pub mod primitives;
pub mod tasks;

pub mod ports;
pub mod system;
#[cfg(feature = "server")]
pub mod server;
