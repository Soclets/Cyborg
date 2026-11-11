//! `OKAY\n`, the Cyborg compiler (D222).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

pub mod build;
pub mod check;
pub mod checked;
pub mod codegen;
pub mod diagnostic;
pub mod lexer;
pub mod manifest;
pub mod parser;
pub mod source;
pub mod spacing;
pub mod syntax;
pub mod target;
