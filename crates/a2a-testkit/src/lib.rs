//! Checks an A2A endpoint from outside, over its HTTP+JSON binding (A2A v1.0.1 section 11).
//!
//! [`WireTarget`] is an [`a2a_conformance::Target`] over [`a2a_client::Client`]: the interpreter
//! of `a2a-conformance` runs the synthesized suite against any endpoint at a base URL, each
//! command sent as the operation [`OPERATIONS`] binds it to, and each step the wire cannot
//! observe answered `unsupported` with the reason. The `a2a-testkit` binary does that from the
//! command line.
//!
//! The `expect_*` helpers check one behaviour of a connected endpoint and answer a [`Mismatch`]
//! naming what they expected and what the endpoint answered, never panicking.

mod binding;
mod contract;
mod expect;
mod expected;
mod target;

pub use binding::OPERATIONS;
pub use expect::{
    Mismatch, expect_card, expect_error_reason, expect_push_config_roundtrip, expect_stream_states,
    expect_task_state,
};
pub use target::WireTarget;
