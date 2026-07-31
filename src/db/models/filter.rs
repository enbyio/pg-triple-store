use spargebra::algebra::Expression;

#[derive(Debug, Clone)]
pub(crate) enum PushableFilter {
    Eq { var: String, value: String },
    NotEq { var: String, value: String },
    Like { var: String, pattern: String }, // SPARQL CONTAINS/STRSTARTS → LIKE
    Lt { var: String, value: String },
    Lte { var: String, value: String },
    Gt { var: String, value: String },
    Gte { var: String, value: String },
    And(Box<PushableFilter>, Box<PushableFilter>),
    Or(Box<PushableFilter>, Box<PushableFilter>),
}

impl PushableFilter {
    pub(crate) fn try_from(expr: &Expression) -> Option<Self> {
        match expr {
            Expression::Equal(a, b) => extract_var_literal_pair(a, b)
                .map(|(var, val)| PushableFilter::Eq { var, value: val }),
            Expression::Less(a, b) => extract_var_literal_pair(a, b)
                .map(|(var, val)| PushableFilter::Lt { var, value: val }),
            Expression::LessOrEqual(a, b) => extract_var_literal_pair(a, b)
                .map(|(var, val)| PushableFilter::Lte { var, value: val }),
            Expression::Greater(a, b) => extract_var_literal_pair(a, b)
                .map(|(var, val)| PushableFilter::Gt { var, value: val }),
            Expression::GreaterOrEqual(a, b) => extract_var_literal_pair(a, b)
                .map(|(var, val)| PushableFilter::Gte { var, value: val }),
            Expression::And(a, b) => {
                // Both sides pushable → push the whole AND
                // One side pushable → push that side, keep the other in-memory
                match (PushableFilter::try_from(a), PushableFilter::try_from(b)) {
                    (Some(fa), Some(fb)) => Some(PushableFilter::And(Box::new(fa), Box::new(fb))),
                    (Some(fa), None) => Some(fa), // partial pushdown
                    (None, Some(fb)) => Some(fb),
                    (None, None) => None,
                }
            }
            Expression::Or(a, b) => {
                // OR is only safe to push if BOTH sides are pushable
                match (PushableFilter::try_from(a), PushableFilter::try_from(b)) {
                    (Some(fa), Some(fb)) => Some(PushableFilter::Or(Box::new(fa), Box::new(fb))),
                    _ => None,
                }
            }
            Expression::FunctionCall(func, args) => {
                use spargebra::algebra::Function;
                // CONTAINS(?var, "str") → LIKE %str%
                // STRSTARTS(?var, "str") → LIKE str%
                if args.len() == 2
                    && let (Expression::Variable(var), Expression::Literal(lit)) =
                        (&args[0], &args[1])
                    {
                        let val = lit.value().to_string();
                        let pattern = match func {
                            Function::Contains => Some(format!("%{}%", val)),
                            Function::StrStarts => Some(format!("{}%", val)),
                            Function::StrEnds => Some(format!("%{}", val)),
                            _ => None,
                        };
                        if let Some(p) = pattern {
                            return Some(PushableFilter::Like {
                                var: var.as_str().to_string(),
                                pattern: p,
                            });
                        }
                    }
                None
            }
            _ => None,
        }
    }
}

fn extract_var_literal_pair(a: &Expression, b: &Expression) -> Option<(String, String)> {
    match (a, b) {
        (Expression::Variable(var), Expression::Literal(lit)) => {
            Some((var.as_str().to_string(), lit.value().to_string()))
        }
        // Also handle reversed: literal = variable
        (Expression::Literal(lit), Expression::Variable(var)) => {
            Some((var.as_str().to_string(), lit.value().to_string()))
        }
        _ => None,
    }
}
