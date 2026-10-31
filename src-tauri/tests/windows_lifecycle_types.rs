//! Compile-only Windows API-shape check. Not Windows ABI, SDK, linkage or recipe.
#![allow(unused_imports, dead_code)]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[path = "../src/platform/lifecycle_windows.rs"]
mod receiver;
