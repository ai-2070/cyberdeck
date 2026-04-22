//! NAT-type classification — collapse observed reflex addresses
//! into the `Open | Cone | Symmetric | Unknown` wire taxonomy.
//!
//! The richer five-way `NatType` lives on
//! `adapter::net::behavior::metadata::NatType` for internal
//! reasoning; on the wire (capability-announcement tags) we only
//! distinguish the four outcomes that matter for punch decisions:
//!
//! - **Open** — reflexive address equals bind address, or a
//!   port-mapping is installed.
//! - **Cone** — reflexive port is consistent across distinct
//!   destinations (punching is reliable).
//! - **Symmetric** — reflexive port differs per destination
//!   (punching is not reliable; cone × symmetric gets one shot
//!   per decision 8 in the plan).
//! - **Unknown** — fewer than two probes, or classification
//!   hasn't run yet.
//!
//! This is **stage 2** of `docs/NAT_TRAVERSAL_PLAN.md`. Stubbed
//! here as part of stage 0; the classification FSM + capability
//! tagging land in the follow-up stage-2 commit.
