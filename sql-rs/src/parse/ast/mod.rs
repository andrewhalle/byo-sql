//! Module for holding AST types.
//!
//! Everything in this module with a lifetime parameter has lifetime of the input string being
//! parsed.

mod identifier;
pub use identifier::*;

mod literal;
pub use literal::*;

mod table_identifier;
pub use table_identifier::*;

mod column;
pub use column::*;

mod column_identifier;
pub use column_identifier::*;

// Template
// /// <Quick description.>
// ///
// /// <Long description.>
// #[derive(Debug)]
// pub <decl>
//
// use crate::parse::Rule;
// use pest::iterators::Pair;
//
// impl<'input> From<Pair<'input, Rule>> for <name><'input> {
//     fn from(<name>: Pair<'input, Rule>) -> Self {
//         assert_eq!(<name>.as_rule(), <rule>);
//
//         let mut inner = <name>.into_inner();
//         // impl
//
//         // construct instance
//     }
// }
