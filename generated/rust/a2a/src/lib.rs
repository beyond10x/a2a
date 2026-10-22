// generated from a2a v1
// model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
// contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
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
