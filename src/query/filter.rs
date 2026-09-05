// In-memory SPARQL expression evaluator.
//
// This is the *authoritative* filter pass. The SQL pushdown in relations.rs /
// property.rs is only an optimisation that reduces the number of rows fetched;
// the in-memory pass always runs afterward and is what actually enforces the
// SPARQL semantics (correct numeric ordering, LANG(), REGEX(), …).
use spargebra::algebra::{Expression, Function, GraphPattern};

use crate::error::StoreError;
use crate::model::triple::{Term, VarKey};
use crate::query::solution::{QueryResult, Solution};
use crate::store::TripleStore;

// ── public entry point ────────────────────────────────────────────────────────

/// Returns `true` iff `expr` evaluates to the effective boolean value `true`
/// over `row`. Unbound variables, type errors, and errors all return `false`
/// (SPARQL spec: filter errors are treated as `false` and the row is dropped).
pub(crate) fn eval_filter(expr: &Expression, row: &Solution) -> bool {
    matches!(eval(expr, row), Some(Value::Bool(true)))
}

// ── internal value type ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Bool(bool),
    Num(f64),
    Str(String),
    Iri(String),
}

fn named_var_key(name: &str) -> VarKey {
    VarKey::Named(name.to_string())
}

// ── XSD IRI constants ─────────────────────────────────────────────────────────

impl TripleStore {
    pub(crate) fn add_filter(
        &self,
        pattern: GraphPattern,
        expr: Expression,
    ) -> Result<QueryResult, StoreError> {
        // let pushable = PushableFilter::try_from(&expr);
        // let opts = QueryOptions {
        //     filter: pushable.map(|f| vec![f]),
        //     ..QueryOptions::default()
        // };
        // log::debug!("Is pushdown: {}", is_pushdown(&expr));
        let res = self.execute_pattern(pattern)?;
        Ok(match res {
            QueryResult::Solutions(mut sol) => {
                sol.rows.retain(|row| eval_filter(&expr, row));
                QueryResult::Solutions(sol)
            }
            other => other,
        })
    }
}

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
const XSD_FLOAT: &str = "http://www.w3.org/2001/XMLSchema#float";
const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
// const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

fn is_numeric_xsd(dt: &str) -> bool {
    matches!(dt, XSD_INTEGER | XSD_DECIMAL | XSD_FLOAT | XSD_DOUBLE)
        || dt.starts_with(XSD)
            && matches!(
                &dt[XSD.len()..],
                "long"
                    | "int"
                    | "short"
                    | "byte"
                    | "unsignedLong"
                    | "unsignedInt"
                    | "unsignedShort"
                    | "unsignedByte"
                    | "nonNegativeInteger"
                    | "nonPositiveInteger"
                    | "positiveInteger"
                    | "negativeInteger"
            )
}

// ── core evaluator ────────────────────────────────────────────────────────────

fn eval(expr: &Expression, row: &Solution) -> Option<Value> {
    match expr {
        // ── leaves ──
        Expression::Literal(lit) => {
            let dt = lit.datatype().as_str();
            let val = lit.value();
            if dt == XSD_BOOLEAN {
                return Some(Value::Bool(matches!(val, "true" | "1")));
            }
            if is_numeric_xsd(dt)
                && let Ok(n) = val.parse::<f64>()
            {
                return Some(Value::Num(n));
            }
            Some(Value::Str(val.to_string()))
        }

        Expression::Variable(var) => row
            .bindings
            .get(&named_var_key(var.as_str()))
            .map(term_to_value),

        Expression::NamedNode(nn) => Some(Value::Iri(nn.as_str().to_string())),

        // ── logical ──
        Expression::And(a, b) => Some(Value::Bool(
            matches!(eval(a, row), Some(Value::Bool(true)))
                && matches!(eval(b, row), Some(Value::Bool(true))),
        )),

        Expression::Or(a, b) => Some(Value::Bool(
            matches!(eval(a, row), Some(Value::Bool(true)))
                || matches!(eval(b, row), Some(Value::Bool(true))),
        )),

        Expression::Not(inner) => Some(Value::Bool(!matches!(
            eval(inner, row),
            Some(Value::Bool(true))
        ))),

        // ── equality / identity ──
        Expression::Equal(a, b) => {
            let va = eval(a, row)?;
            let vb = eval(b, row)?;
            Some(Value::Bool(values_equal(&va, &vb)))
        }

        // SameTerm: term identity (stricter than rdf:equal, but for our Term
        // type — which has no blank nodes and no mixed literal types —
        // value equality is sufficient).
        Expression::SameTerm(a, b) => {
            let va = eval(a, row)?;
            let vb = eval(b, row)?;
            Some(Value::Bool(va == vb))
        }

        // ── ordering comparisons ──
        Expression::Less(a, b) => compare(a, b, row, |o| o.is_lt()),
        Expression::LessOrEqual(a, b) => compare(a, b, row, |o| o.is_le()),
        Expression::Greater(a, b) => compare(a, b, row, |o| o.is_gt()),
        Expression::GreaterOrEqual(a, b) => compare(a, b, row, |o| o.is_ge()),

        // ── arithmetic ──
        Expression::Add(a, b) => num2(a, b, row, |x, y| x + y),
        Expression::Subtract(a, b) => num2(a, b, row, |x, y| x - y),
        Expression::Multiply(a, b) => num2(a, b, row, |x, y| x * y),
        Expression::Divide(a, b) => {
            let y = as_num(eval(b, row)?)?;
            if y == 0.0 {
                return None; // division by zero → error → filter drops row
            }
            Some(Value::Num(as_num(eval(a, row)?)? / y))
        }

        Expression::UnaryPlus(inner) => eval(inner, row),
        Expression::UnaryMinus(inner) => Some(Value::Num(-as_num(eval(inner, row)?)?)),

        // ── IN ──
        Expression::In(val, list) => {
            let v = eval(val, row)?;
            Some(Value::Bool(list.iter().any(|item| {
                eval(item, row)
                    .map(|w| values_equal(&v, &w))
                    .unwrap_or(false)
            })))
        }

        // ── control flow ──
        Expression::If(cond, then_, else_) => {
            if matches!(eval(cond, row), Some(Value::Bool(true))) {
                eval(then_, row)
            } else {
                eval(else_, row)
            }
        }

        Expression::Coalesce(list) => list.iter().find_map(|e| eval(e, row)),

        // ── BOUND(?var) — own Expression variant in spargebra 0.4 ──
        Expression::Bound(var) => Some(Value::Bool(
            row.bindings.contains_key(&named_var_key(var.as_str())),
        )),

        // ── EXISTS / NOT EXISTS — needs sub-query execution; skip for now ──
        Expression::Exists(_) => None,

        // ── built-in function calls ──
        Expression::FunctionCall(func, args) => eval_function(func, args, row),
    }
}

// ── built-in functions ────────────────────────────────────────────────────────

fn eval_function(func: &Function, args: &[Expression], row: &Solution) -> Option<Value> {
    use Function::*;
    match func {
        // ── string tests ──
        Contains => {
            let (s, sub) = str2(args, row)?;
            Some(Value::Bool(s.contains(sub.as_str())))
        }
        StrStarts => {
            let (s, sub) = str2(args, row)?;
            Some(Value::Bool(s.starts_with(sub.as_str())))
        }
        StrEnds => {
            let (s, sub) = str2(args, row)?;
            Some(Value::Bool(s.ends_with(sub.as_str())))
        }
        StrBefore => {
            let (s, sub) = str2(args, row)?;
            Some(Value::Str(
                s.find(sub.as_str())
                    .map(|i| s[..i].to_string())
                    .unwrap_or_default(),
            ))
        }
        StrAfter => {
            let (s, sub) = str2(args, row)?;
            Some(Value::Str(
                s.find(sub.as_str())
                    .map(|i| s[i + sub.len()..].to_string())
                    .unwrap_or_default(),
            ))
        }
        StrLen => {
            let s = as_str(eval(args.first()?, row)?)?;
            Some(Value::Num(s.chars().count() as f64)) // char-length per SPARQL spec
        }
        Str => {
            let v = eval(args.first()?, row)?;
            Some(Value::Str(as_str(v)?))
        }
        UCase => Some(Value::Str(
            as_str(eval(args.first()?, row)?)?.to_uppercase(),
        )),
        LCase => Some(Value::Str(
            as_str(eval(args.first()?, row)?)?.to_lowercase(),
        )),
        Concat => {
            let parts: Option<Vec<String>> =
                args.iter().map(|e| eval(e, row).and_then(as_str)).collect();
            Some(Value::Str(parts?.join("")))
        }
        SubStr => {
            // SPARQL SubStr is 1-based, length optional
            let s = as_str(eval(args.first()?, row)?)?;
            let start = (as_num(eval(args.get(1)?, row)?)? as usize).saturating_sub(1);
            let chars: Vec<char> = s.chars().collect();
            if let Some(len_expr) = args.get(2) {
                let len = as_num(eval(len_expr, row)?)? as usize;
                Some(Value::Str(
                    chars[start..][..len.min(chars.len() - start)]
                        .iter()
                        .collect(),
                ))
            } else {
                Some(Value::Str(chars[start..].iter().collect()))
            }
        }
        Replace => {
            // REPLACE(str, pattern, replacement [, flags])
            let s = as_str(eval(args.first()?, row)?)?;
            let pattern = as_str(eval(args.get(1)?, row)?)?;
            let replacement = as_str(eval(args.get(2)?, row)?)?;
            let flags = args
                .get(3)
                .and_then(|e| eval(e, row))
                .and_then(as_str)
                .unwrap_or_default();
            let re_str = if flags.contains('i') {
                format!("(?i){}", pattern)
            } else {
                pattern.clone()
            };
            regex::Regex::new(&re_str)
                .ok()
                .map(|re| Value::Str(re.replace_all(&s, replacement.as_str()).into_owned()))
        }
        // REGEX(?var, "pattern") or REGEX(?var, "pattern", "flags")
        Regex => {
            let s = as_str(eval(args.first()?, row)?)?;
            let pattern = as_str(eval(args.get(1)?, row)?)?;
            let flags = args
                .get(2)
                .and_then(|e| eval(e, row))
                .and_then(as_str)
                .unwrap_or_default();
            let re_str = if flags.contains('i') {
                format!("(?i){}", pattern)
            } else {
                pattern.clone()
            };
            regex::Regex::new(&re_str)
                .ok()
                .map(|re| Value::Bool(re.is_match(&s)))
        }

        // ── lang / datatype accessors ──
        Lang => {
            // LANG(?x) returns the language tag or "" if none.
            // Our Term::Literal stores the datatype as something like
            //   "@en"  (language-tagged literal) or an XSD IRI.
            // The datatype field conventions used in this codebase:
            //   literal_type column in postgres: spargebra serialises literals
            //   as `"value"@lang` or `"value"^^<type>`. We store the raw type IRI
            //   or "Unknown". Extract the language from the term directly.
            if let Some(Expression::Variable(v)) = args.first()
                && let Some(Term::Literal { datatype, .. }) =
                    row.bindings.get(&named_var_key(v.as_str()))
            {
                // Convention: if the datatype is stored as "@<lang>" it's a
                // language-tagged literal; if it's "Unknown" or an XSD IRI, no lang.
                let lang = if let Some(data_type) = datatype.strip_prefix("@") {
                    data_type.to_string()
                } else {
                    String::new()
                };
                return Some(Value::Str(lang));
            }
            Some(Value::Str(String::new()))
        }
        LangMatches => {
            // LangMatches(lang, range) — case-insensitive prefix match
            let lang = as_str(eval(args.first()?, row)?)?;
            let range = as_str(eval(args.get(1)?, row)?)?;
            if range == "*" {
                return Some(Value::Bool(!lang.is_empty()));
            }
            Some(Value::Bool(
                lang.to_lowercase().starts_with(&range.to_lowercase()),
            ))
        }
        Datatype => {
            if let Some(Expression::Variable(v)) = args.first()
                && let Some(Term::Literal { datatype, .. }) =
                    row.bindings.get(&named_var_key(v.as_str()))
                && !datatype.starts_with('@')
            {
                return Some(Value::Iri(datatype.clone()));
            }
            None
        }

        // ── type-testing functions (as FunctionCall variants in spargebra 0.4) ──
        IsIri => Some(Value::Bool(matches!(
            eval(args.first()?, row)?,
            Value::Iri(_)
        ))),
        IsBlank => Some(Value::Bool(false)), // no blank nodes in our Term
        IsLiteral => Some(Value::Bool(matches!(
            eval(args.first()?, row)?,
            Value::Str(_) | Value::Num(_) | Value::Bool(_)
        ))),
        IsNumeric => Some(Value::Bool(matches!(
            eval(args.first()?, row)?,
            Value::Num(_)
        ))),

        // ── numeric functions ──
        Abs => Some(Value::Num(as_num(eval(args.first()?, row)?)?.abs())),
        Ceil => Some(Value::Num(as_num(eval(args.first()?, row)?)?.ceil())),
        Floor => Some(Value::Num(as_num(eval(args.first()?, row)?)?.floor())),
        Round => Some(Value::Num(as_num(eval(args.first()?, row)?)?.round())),

        // ── IRI construction ──
        Iri => {
            let s = as_str(eval(args.first()?, row)?)?;
            Some(Value::Iri(s))
        }

        // ── anything else (NOW, RAND, SHA*, UUID, date/time parts, …) ──
        // Return None → row is dropped by the filter (conservative correct behaviour).
        _ => None,
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Convert a `Term` from a solution row into an evaluator `Value`.
fn term_to_value(t: &Term) -> Value {
    match t {
        Term::Iri(s) => Value::Iri(s.clone()),
        Term::Literal { value, datatype } => {
            if datatype == XSD_BOOLEAN {
                return Value::Bool(matches!(value.as_str(), "true" | "1"));
            }
            if is_numeric_xsd(datatype)
                && let Ok(n) = value.parse::<f64>()
            {
                return Value::Num(n);
            }
            Value::Str(value.clone())
        }
        _ => panic!("blank node can not be converted to internal value"),
    }
}

/// SPARQL equality: numerics compare by value, strings by lexicographic value,
/// IRIs by string equality. Mixed types (e.g. Num vs Str) are never equal.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Iri(x), Value::Iri(y)) => x == y,
        _ => false,
    }
}

/// Total ordering for comparison operators. Returns None on incompatible types.
fn partial_cmp_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x.partial_cmp(y),
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y),
        (Value::Str(x), Value::Str(y)) => x.partial_cmp(y),
        (Value::Iri(x), Value::Iri(y)) => x.partial_cmp(y),
        _ => None, // type error → None → filter drops the row
    }
}

fn compare(
    a: &Expression,
    b: &Expression,
    row: &Solution,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Option<Value> {
    let va = eval(a, row)?;
    let vb = eval(b, row)?;
    partial_cmp_values(&va, &vb).map(|o| Value::Bool(pred(o)))
}

fn num2(
    a: &Expression,
    b: &Expression,
    row: &Solution,
    op: impl Fn(f64, f64) -> f64,
) -> Option<Value> {
    Some(Value::Num(op(
        as_num(eval(a, row)?)?,
        as_num(eval(b, row)?)?,
    )))
}

fn str2(args: &[Expression], row: &Solution) -> Option<(String, String)> {
    if args.len() < 2 {
        return None;
    }
    Some((as_str(eval(&args[0], row)?)?, as_str(eval(&args[1], row)?)?))
}

fn as_num(v: Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(n),
        Value::Str(s) => s.parse().ok(),
        Value::Bool(b) => Some(if b { 1.0 } else { 0.0 }),
        Value::Iri(_) => None,
    }
}

fn as_str(v: Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s),
        Value::Iri(s) => Some(s),
        Value::Num(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
    }
}
