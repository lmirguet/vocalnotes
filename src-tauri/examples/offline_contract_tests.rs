//! Reuse the actual integration suites in a Cargo example test harness.
//! Cargo integration-test targets implicitly build the ordinary app binary;
//! an example harness avoids that while a user keeps an unsaved GUI note open.
//! Run only with `cargo test --offline --locked --example offline_contract_tests`.
//! No copied tests, provider calls, capture, UI startup or filesystem mocks.

#[cfg(test)]
#[path = "../tests/interpreter_operations.rs"]
mod interpreter_operations;
#[cfg(test)]
#[path = "../tests/prototype_audio.rs"]
mod prototype_audio;
#[cfg(test)]
#[path = "../tests/prototype_contracts.rs"]
mod prototype_contracts;
#[cfg(test)]
#[path = "../tests/prototype_envelope.rs"]
mod prototype_envelope;
#[cfg(test)]
#[path = "../tests/prototype_ordered_save.rs"]
mod prototype_ordered_save;
#[cfg(test)]
#[path = "../tests/prototype_pipeline.rs"]
mod prototype_pipeline;
#[cfg(test)]
#[path = "../tests/prototype_services.rs"]
mod prototype_services;
#[cfg(test)]
#[path = "../tests/session_ports.rs"]
mod session_ports;
#[cfg(test)]
#[path = "../tests/session_validity.rs"]
mod session_validity;

#[cfg(test)]
#[path = "../tests/storage_integrity.rs"]
mod storage_integrity;

#[cfg(test)]
#[path = "../tests/close_workflow.rs"]
mod close_workflow;

fn main() {}

#[cfg(test)]
#[path = "../tests/ipc_boundary.rs"]
mod ipc_boundary;

#[cfg(test)]
#[path = "../tests/secrets_policy.rs"]
mod secrets_policy;

#[cfg(test)]
#[path = "../tests/provider_contracts.rs"]
mod provider_contracts;

#[cfg(test)]
#[path = "../tests/diagnostic_privacy.rs"]
mod diagnostic_privacy;
