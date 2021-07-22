//! A crate for parsing and executing SQL in-memory against a simple database representation.

#[macro_use]
extern crate pest_derive;

/// Data representation.
pub mod data;

/// Executing a query against a database.
pub mod execute;

/// Parsing SQL.
pub mod parse;
