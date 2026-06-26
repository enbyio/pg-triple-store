use std::collections::{BTreeMap, HashMap, HashSet};

use spargebra::term::TriplePattern;

use crate::db::models::query::{QueryOptions, Solution, SolutionSet, Term};
use crate::store::TripleStore;
use crate::util::{merge, triple_pattern_vars};
use crate::StoreError;

impl TripleStore {
    pub(crate) fn execute_bgp(
        &mut self,
        patterns: Vec<TriplePattern>,
        opts: QueryOptions,
    ) -> Result<SolutionSet, StoreError> {
        if patterns.is_empty() {
            return Ok(SolutionSet {
                vars: vec![],
                rows: vec![],
            });
        }
        let mut acc: Vec<Solution> = vec![];
        let mut acc_vars: HashSet<String> = HashSet::new();
        let mut seeded = false;

        for pattern in patterns {
            log::debug!("acc: {:?}", acc);
            let pattern_vars = triple_pattern_vars(&pattern);

            if !seeded {
                let rows = self.execute_triple_pattern(pattern, opts)?;
                acc = rows;
                acc_vars = pattern_vars;
                seeded = true;
                continue;
            }

            if acc.is_empty() {
                break;
            }

            let shared: HashSet<&String> = pattern_vars.intersection(&acc_vars).collect();
            if pattern_vars.is_empty() {
                // no variables in pattern, constant existence check
                if !self.execute_triple_pattern(pattern, opts)?.is_empty() {
                    acc = vec![]
                }
            } else if shared.is_empty() {
                let new_rows = self.execute_triple_pattern(pattern, opts)?;
                acc = acc
                    .iter()
                    .flat_map(|l| new_rows.iter().map(move |r| merge(l, r)))
                    .collect();
                acc_vars.extend(pattern_vars);
            } else if shared.len() == pattern_vars.len() {
                acc.retain(|acc_row| {
                    // build known bindings from this acc row for the shared vars
                    let known: BTreeMap<String, Vec<Term>> = shared
                        .iter()
                        .filter_map(|v| {
                            acc_row
                                .bindings
                                .get(*v)
                                .map(|t| ((*v).clone(), vec![t.clone()]))
                        })
                        .collect();
                    self.execute_triple_pattern_with_bindings(pattern.clone(), &known)
                        .map(|rows| !rows.is_empty())
                        .unwrap_or(false)
                });
            } else {
                let known_values: BTreeMap<String, Vec<Term>> = shared
                    .iter()
                    .map(|&v| {
                        let vals: Vec<Term> = acc
                            .iter()
                            .filter_map(|s| s.bindings.get(v).cloned())
                            .collect::<HashSet<_>>() // dedup
                            .into_iter()
                            .collect();
                        ((*v).clone(), vals)
                    })
                    .collect();
                // scan pattern with IN-list filters applied in SQL
                let new_rows = self.execute_triple_pattern_with_bindings(pattern, &known_values)?;

                // hash-join: build index on new_rows keyed by shared var values
                let mut index: HashMap<Vec<Option<Term>>, Vec<Solution>> = HashMap::new();
                for row in new_rows {
                    let key: Vec<Option<Term>> = shared
                        .iter()
                        .map(|v| row.bindings.get(*v).cloned())
                        .collect();
                    index.entry(key).or_default().push(row);
                }

                // probe acc against index
                acc = acc
                    .into_iter()
                    .flat_map(|acc_row| {
                        let key: Vec<Option<Term>> = shared
                            .iter()
                            .map(|v| acc_row.bindings.get(*v).cloned())
                            .collect();
                        match index.get(&key) {
                            None => vec![],
                            Some(matches) => matches.iter().map(|r| merge(&acc_row, r)).collect(),
                        }
                    })
                    .collect();

                acc_vars.extend(pattern_vars);
            }
        }

        let mut vars: Vec<String> = acc_vars.into_iter().collect();
        vars.sort();

        Ok(SolutionSet { rows: acc, vars })
    }
}
