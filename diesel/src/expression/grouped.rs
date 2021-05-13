use crate::backend::Backend;
use crate::expression::{Expression, ValidGrouping};
use crate::query_builder::*;
use crate::result::QueryResult;
use crate::sql_types::DieselNumericOps;

#[derive(Debug, Copy, Clone, QueryId, Default, DieselNumericOps, ValidGrouping)]
pub struct Grouped<T>(pub T);

impl<T: Expression> Expression for Grouped<T> {
    type SqlType = T::SqlType;
}

impl<T: QueryFragment<DB>, DB: Backend> QueryFragment<DB> for Grouped<T> {
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_sql("(");
        self.0.walk_ast(out.reborrow())?;
        out.push_sql(")");
        Ok(())
    }
}

impl_selectable_expression!(Grouped<T>);
