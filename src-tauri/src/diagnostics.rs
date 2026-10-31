//! Console-only technical diagnostics requested for native validation.
//! Local static labels and numeric metadata, never provider payloads or user text.
use std::{io::Write, time::SystemTime};

pub(crate) fn event(message: &'static str) {
    numbers(message, &[]);
}

pub(crate) fn numbers(message: &'static str, fields: &[(&'static str, u64)]) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut line = format!("[vocalnotes {timestamp}] {message}");
    for (key, value) in fields {
        use std::fmt::Write;
        let _ = write!(line, " {key}={value}");
    }
    // A closed console must not panic or affect note/capture correctness.
    let _ = writeln!(std::io::stderr().lock(), "{line}");
}
