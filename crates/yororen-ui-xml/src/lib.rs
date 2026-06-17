//! Compile-time XML UI layer for yororen-ui.
//!
//! This crate hosts the schema, parser, and codegen used by
//! `yororen-ui-xml-macro` to expand `xml! { ... }` invocations
//! into equivalent headless builder calls. There is no runtime
//! cost: the macro fully resolves at compile time.
//!
//! Most users should depend on the re-exported
//! `yororen_ui_xml_macro::xml` (or the `yororen-ui` facade's
//! `xml` feature) and never touch this crate directly.
//!
//! ## Module map
//!
//! - [`ast`] — parsed XML node tree
//! - [`parser`] — XML string → AST (roxmltree)
//! - [`schema`] — XML tag/attribute → headless component mapping
//! - [`codegen`] — AST → Rust `TokenStream` (quote)
//! - [`error`] — compile-time diagnostic payload

#![forbid(unsafe_code)]

pub mod ast;
pub mod class;
pub mod codegen;
pub mod error;
pub mod parser;
pub mod runtime;
pub mod schema;

/// Re-exported so `register_xml_component!` (in
/// [`runtime`]) can refer to it via `$crate`.
pub use inventory;

/// Auto-generated schema entries, populated by the
/// `gen-schema` binary. See `yororen-ui-xml/src/bin/gen_schema.rs`
/// for the source of truth. This module is **always
/// included** so the codegen can look up any leaf
/// component; the file is regenerated when the headless
/// API changes.
pub mod schema_generated;
