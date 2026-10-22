// generated from a2a v1
// model digest 19f81099dbb59817313f63273ff10776f7d98070eb5de7dda53e5d67514ef5bf
// contract digest b81ca6ca77a699d33385c2f8b48b31221205b1879fc0f96d44d1a695ab4bff58
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
