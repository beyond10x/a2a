// generated from a2a v1
// model digest d761a3fbbc9413d58b2073e5f15bbced42229623312d9101f245c68b57f5021d
// contract digest 91b9b3965c71b5aeb8e0b091924c0a887fe1b698155e248eea0b77ad3859a42a
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
