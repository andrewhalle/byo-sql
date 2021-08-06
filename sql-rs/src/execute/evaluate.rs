use crate::data::{Column, Database, Datatype, Row, Table, Value};
use crate::parse::ast::{self, Expression, Literal};

pub type RowEvaluationContext<'table> = (&'table Vec<Column>, &'table Row);

/// TODO
pub fn evaluate(
    expr: &Expression<'_>,
    row_ctx: Option<RowEvaluationContext>,
    database: Option<&Database>,
) -> Value {
    match expr {
        Expression::ColumnIdentifier(i) => {
            let row_ctx = row_ctx.unwrap();
            let idx = Table::get_column_idx(row_ctx.0, i);

            row_ctx.1 .0[idx].clone()
        }
        Expression::Literal(l) => l.into(),
        Expression::Subquery(sq) => {
            Value::List(database.unwrap().execute_select(sq).unwrap().into())
        }
        Expression::BinaryOp(b) => {
            let v1 = evaluate(&b.left, row_ctx, database);
            let v2 = evaluate(&b.right, row_ctx, database);

            v1.op(b.op, v2)
        }
        _ => unreachable!(),
    }
}

/// TODO
pub fn evaluate_column(expr: &Expression<'_>, columns: &Vec<Column>) -> Vec<Column> {
    match expr {
        Expression::ColumnIdentifier(i) => match &i.name {
            ast::Column::Star => columns
                .iter()
                .filter(|c| match &i.alias {
                    None => true,
                    Some(alias) => {
                        if c.name.contains(".") {
                            alias.0 == c.name.rsplit_once(".").unwrap().0
                        } else {
                            alias.0 == c.name
                        }
                    }
                })
                .map(Clone::clone)
                .collect(),
            _ => {
                let idx = Table::get_column_idx(columns, i);

                vec![columns[idx].clone()]
            }
        },
        Expression::Subquery(_sq) => todo!(),
        Expression::Literal(l) => vec![Column {
            name: String::from("?column?"),
            datatype: <&Literal<'_> as Into<Value>>::into(l).datatype(),
        }],
        Expression::BinaryOp(b) => {
            // for now at least, both sides of a binary op must have the same type
            evaluate_column(&b.left, columns)
        }
        Expression::CountStar => vec![Column {
            name: String::from("count"),
            datatype: Datatype::Number,
        }],
    }
}
