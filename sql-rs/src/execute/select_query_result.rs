use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;

use crate::data::{Column, Datatype, Row, Value};
use crate::parse::ast::{
    self, ColumnIdentifier, Expression, Join, JoinKind, OrderBy, OrderByDirection,
};
use crate::{cmp_column_with_column_identifier, OrdVariants, SortableValue};

#[derive(Debug, Clone)]
pub struct SelectQueryResultColumn {
    pub table: String,
    pub column: String,
    pub datatype: Datatype,
}

impl SelectQueryResultColumn {
    pub fn from(table: String, column: &Column) -> Self {
        SelectQueryResultColumn {
            table,
            column: column.name.clone(),
            datatype: column.datatype,
        }
    }
}

pub struct SelectQueryResult {
    pub columns: Vec<SelectQueryResultColumn>,
    pub rows: Vec<Row>,
    pub table_alias_map: HashMap<String, String>,
}

impl SelectQueryResult {
    // TODO should this clone?
    fn get_column_value_from_row(
        &self,
        column_identifier: &ColumnIdentifier<'_>,
        row: &Row,
    ) -> Value {
        assert!(!matches!(column_identifier.name, ast::Column::Star));

        let idx = self
            .columns
            .iter()
            .enumerate()
            .find(|(_, x)| {
                cmp_column_with_column_identifier(x, &column_identifier, &self.table_alias_map)
            })
            .unwrap()
            .0;

        row.0[idx].clone()
    }

    // TODO re-write using ast::Expression
    fn evaluate(&self, expr: &Expression<'_>, row: &Row) -> Value {
        match expr {
            ast::Expression::ColumnIdentifier(i) => self.get_column_value_from_row(i, row),
            ast::Expression::Literal(l) => l.into(),
            ast::Expression::BinaryOp(b) => {
                let v1 = self.evaluate(&b.left, row);
                let v2 = self.evaluate(&b.right, row);

                v1.op(b.op, v2)
            }
            _ => unreachable!(),
        }
    }

    pub fn select(&mut self, select_list: Vec<Expression<'_>>) -> Self {
        let mut retval = SelectQueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            table_alias_map: HashMap::new(),
        };

        // TODO functions likely needed
        //   * get column description from expression
        //   * get type of expression
        // push columns
        for expr in &select_list {
            if expr.is_count() {
                retval.columns.push(SelectQueryResultColumn {
                    table: "".to_string(),
                    column: "count".to_string(),
                    datatype: Datatype::Number,
                });
                break;
            }

            let column_identifier = match expr {
                ast::Expression::ColumnIdentifier(i) => i,
                _ => unreachable!(),
            };
            if matches!(column_identifier.name, ast::Column::Star) {
                for column in &self.columns {
                    if cmp_column_with_column_identifier(
                        column,
                        &column_identifier,
                        &self.table_alias_map,
                    ) {
                        retval.columns.push(column.clone());
                    }
                }
            } else {
                let column = self
                    .columns
                    .iter()
                    .find(|x| {
                        cmp_column_with_column_identifier(
                            x,
                            &column_identifier,
                            &self.table_alias_map,
                        )
                    })
                    .unwrap();
                retval.columns.push(column.clone());
            }
        }

        // push projected rows
        if select_list[0].is_count() {
            let mut row = Vec::new();
            row.push(Value::Number(self.rows.len() as u32));
            retval.rows.push(Row(row));
        } else {
            for row in &self.rows {
                let mut new_row = Vec::new();
                for expr in &select_list {
                    if expr.is_column_star() {
                        let column_identifier = match expr {
                            ast::Expression::ColumnIdentifier(i) => i,
                            _ => unreachable!(),
                        };
                        for (i, column) in self.columns.iter().enumerate() {
                            if cmp_column_with_column_identifier(
                                column,
                                &column_identifier,
                                &self.table_alias_map,
                            ) {
                                new_row.push(row.0[i].clone());
                            }
                        }
                    } else {
                        new_row.push(self.evaluate(&expr, row));
                    }
                }
                retval.rows.push(Row(new_row));
            }
        }

        retval
    }

    // TODO move this to some sort of TableView once it exists.
    /// Filters a SelectQueryResult by evaluating expression for each row, and keeping it if the
    /// expression evaluates to true.
    pub fn filter(&mut self, expression: &Expression<'_>) {
        let mut rows = Vec::new();
        mem::swap(&mut rows, &mut self.rows);

        for row in rows.into_iter() {
            if self.evaluate(&expression, &row).is_true() {
                self.rows.push(row);
            }
        }
    }

    // TODO move this to some sort of TableView once it exists.
    /// Sorts the rows in a SelectQueryResult by evaluating expression and using it as a key.
    pub fn sort(&mut self, order_by: &OrderBy<'_>) {
        let mut rows = mem::take(&mut self.rows);

        rows.as_mut_slice().sort_unstable_by_key(|row| {
            let sortable = SortableValue(self.evaluate(&order_by.expr, &row));

            match order_by.direction {
                OrderByDirection::Asc => OrdVariants::SortableValue(sortable),
                OrderByDirection::Desc => OrdVariants::Reversed(Reverse(sortable)),
            }
        });

        mem::swap(&mut rows, &mut self.rows);
    }

    pub fn limit(&mut self, rows: u32) {
        self.rows.truncate(rows as usize);
    }

    // TODO nested loop join, considering join.kind. Probably need to re-write self.evaluate
    // so that it can work on a row that's not in self yet (since we need to check if we want to
    // add it).
    pub fn join(&mut self, rhs: &mut Self, join: &Join<'_>) {
        let lhs_column_count = self.columns.len();
        let rhs_column_count = rhs.columns.len();

        self.columns.append(&mut rhs.columns);
        for (k, v) in rhs.table_alias_map.drain() {
            self.table_alias_map.insert(k, v);
        }

        let outer_iter = || match join.kind {
            JoinKind::Right => rhs.rows.iter(),
            _ => self.rows.iter(),
        };
        let inner_iter = || match join.kind {
            JoinKind::Right => self.rows.iter(),
            _ => rhs.rows.iter(),
        };

        let mut rows = Vec::new();
        for i in outer_iter() {
            let mut did_add_row = false;

            for j in inner_iter() {
                let mut row = i.0.clone();
                row.append(&mut j.0.clone());
                let row = Row(row);
                if self.evaluate(&join.condition, &row).is_true() {
                    rows.push(row);
                    did_add_row = true;
                }
            }

            if !did_add_row {
                let row = match join.kind {
                    JoinKind::Left => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(rhs_column_count);
                            for _i in 0..rhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        row.append(&mut nulls);
                        Some(Row(row))
                    }
                    JoinKind::Right => {
                        let mut row = i.0.clone();
                        let mut nulls = {
                            let mut nulls = Vec::with_capacity(lhs_column_count);
                            for _i in 0..lhs_column_count {
                                nulls.push(Value::Null);
                            }
                            nulls
                        };
                        nulls.append(&mut row);
                        Some(Row(nulls))
                    }
                    _ => None,
                };

                if row.is_some() {
                    rows.push(row.unwrap());
                }
            }
        }
        mem::swap(&mut self.rows, &mut rows);
    }
}

impl Display for SelectQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let column_names: Vec<&str> = self.columns.iter().map(|c| c.column.as_str()).collect();
        writeln!(f, "{}", &column_names.join(",")).unwrap();

        let num_rows = self.rows.len();
        for (i, row) in self.rows.iter().enumerate() {
            let values: Vec<String> = row.0.iter().map(|v| v.to_string()).collect();
            write!(f, "{}", &values.join(",")).unwrap();
            if i != num_rows - 1 {
                writeln!(f).unwrap();
            }
        }

        Ok(())
    }
}
