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

mod query;
pub use query::*;

mod select_query;
pub use select_query::*;

mod select_list;
pub use select_list::*;

mod expression;
pub use expression::*;

mod table_expression;
pub use table_expression::*;

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
