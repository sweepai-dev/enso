#![feature(strict_provenance)]
#![feature(box_syntax)]
#![feature(box_patterns)]

// === Linter configuration
#![allow(dead_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_numeric_casts)]

pub mod fwd;
pub mod bwd;
pub mod types;