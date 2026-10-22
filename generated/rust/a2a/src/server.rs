// generated from a2a v1
// model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
// contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
// do not edit: regenerate with `ess synthesize --layout crate`

//! The HTTP surface of `a2a` v1, synthesised.
//!
//! One module per component the specification declares is reached over a network, each holding
//! that component's route table, its listener and the two documents it publishes about itself.
//! The routes are the ones the committed `OpenAPI` document declares, from the same mapping, so a
//! path served here and a path published there cannot be two different answers.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent is absent by
//! decision — no framework, no runtime, no second protocol, no concurrency, no authentication —
//! and each absence is argued in the `TARGET.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod entry;
pub mod http;
pub mod json;
pub mod wire;
pub mod a2a_agent;
pub mod memory;
mod static_assets;
