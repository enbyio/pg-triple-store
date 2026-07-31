use spargebra::algebra::{Expression, GraphPattern};

use crate::db::models::query::{QueryOptions, QueryResult};
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub(crate) fn add_filter(
        &mut self,
        pattern: GraphPattern,
        expr: Expression,
    ) -> Result<QueryResult, StoreError> {
        log::debug!("Is pushdown: {}", is_pushdown(&expr));
        self.execute_pattern(pattern, QueryOptions::default())
    }
}

//fn build_filter(expr: Box<Expression>) {}

fn is_pushdown(expr: &Expression) -> bool {
    match expr {
        // Always safe leaves
        Expression::NamedNode(_) | Expression::Literal(_) | Expression::Variable(_) => true,

        // Safe if children are safe
        Expression::Or(a, b)
        | Expression::And(a, b)
        | Expression::Equal(a, b)
        | Expression::Greater(a, b)
        | Expression::GreaterOrEqual(a, b)
        | Expression::Less(a, b)
        | Expression::LessOrEqual(a, b)
        | Expression::Add(a, b)
        | Expression::Subtract(a, b)
        | Expression::Multiply(a, b)
        | Expression::Divide(a, b) => is_pushdown(a) && is_pushdown(b),

        Expression::UnaryPlus(a) | Expression::UnaryMinus(a) | Expression::Not(a) => is_pushdown(a),

        Expression::In(a, list) => is_pushdown(a) && list.iter().all(is_pushdown),

        Expression::If(cond, then, els) => {
            is_pushdown(cond) && is_pushdown(then) && is_pushdown(els)
        }

        Expression::Coalesce(list) => list.iter().all(is_pushdown),
        _ => {
            println!("Expression: {:?}", expr);
            false
        }
    }
}
