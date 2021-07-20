use crate::data::{Column, Database, Row, Table, Value};
use crate::parse::ast::Expression;

type RowEvaluationContext<'table> = (&'table Vec<Column>, &'table Row);

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
        Expression::BinaryOp(b) => {
            let v1 = evaluate(&b.left, row_ctx, database);
            let v2 = evaluate(&b.right, row_ctx, database);

            v1.op(b.op, v2)
        }
        _ => unreachable!(),
    }
}
