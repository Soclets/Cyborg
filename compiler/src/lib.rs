//! `OKAY\n`, the Cyborg compiler (D222).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

pub mod build;
pub mod cache;
pub mod check;
pub mod checked;
pub mod codegen;
pub mod diagnostic;
pub mod flow;
pub mod lexer;
pub mod machine;
pub mod manifest;
pub mod maths;
pub mod parser;
pub mod ranges;
pub mod source;
pub mod spacing;
pub mod syntax;
pub mod target;
