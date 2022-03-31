use std::collections::HashSet;

use super::{grammar::Symbol, Grammar, END_MARK};

impl Grammar {
    pub fn calculate_nullable_first_follow(&mut self) {
        if let Some(start_idx) = self.start_symbol {
            self.symbols[start_idx]
                .mut_non_terminal()
                .unwrap()
                .follow
                .insert(self.symbol_table[END_MARK]);
            self.calculate_nullable();
            self.calculate_first();
            self.calculate_follow();
        }
    }

    pub fn reset_nullable_first_follow(&mut self) {
        for nt in self.non_terminal_iter_mut() {
            nt.nullable = false;
            nt.first = HashSet::new();
            nt.follow = HashSet::new();
        }
    }

    fn calculate_nullable(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.symbols.len() {
                let nullable: bool = match &self.symbols[i] {
                    Symbol::Terminal(_) => continue,
                    Symbol::NonTerminal(nt) => {
                        if nt.nullable {
                            continue;
                        }
                        nt.productions.iter().any(|production| {
                            production.iter().all(|s| match &self.symbols[*s] {
                                Symbol::Terminal(_) => false,
                                Symbol::NonTerminal(e) => e.nullable,
                            })
                        })
                    }
                };

                if nullable {
                    self.symbols[i].mut_non_terminal().unwrap().nullable = true;
                    changed = true;
                }
            }
        }
    }

    pub fn calculate_first_for_production(&self, production: &Vec<usize>) -> HashSet<usize> {
        let mut first: HashSet<usize> = HashSet::new();
        for (idx, symbol) in production.iter().map(|i| (*i, &self.symbols[*i])) {
            match symbol {
                Symbol::Terminal(_) => {
                    first.insert(idx);
                    break;
                }
                Symbol::NonTerminal(nt) => {
                    first.extend(nt.first.iter().cloned());
                    if !nt.nullable {
                        break;
                    }
                }
            }
        }
        first
    }

    fn calculate_first(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.symbols.len() {
                let first: HashSet<usize> = match &self.symbols[i] {
                    Symbol::Terminal(_) => continue,
                    Symbol::NonTerminal(nt) => {
                        nt.productions
                            .iter()
                            .fold(HashSet::new(), |mut first, production| {
                                first.extend(
                                    self.calculate_first_for_production(production).into_iter(),
                                );
                                first
                            })
                    }
                };

                let nt = self.symbols[i].mut_non_terminal().unwrap();
                if nt.first.len() != first.len() {
                    changed = true;
                    nt.first = first;
                }
            }
        }
    }

    pub fn calculate_follow_for_production(&self, production: &Vec<usize>) -> HashSet<usize> {
        let mut follow = HashSet::new();
        for idx in production.iter().rev() {
            match &self.symbols[*idx] {
                Symbol::Terminal(_) => {
                    follow.insert(*idx);
                    break;
                }
                Symbol::NonTerminal(nt) => {
                    follow.extend(nt.follow.iter().cloned());
                    if !nt.nullable {
                        break;
                    }
                }
            }
        }
        follow
    }

    fn calculate_follow(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.symbols.len() {
                if let Symbol::Terminal(_) = self.symbols[i] {
                    continue;
                }

                let productions = self.symbols[i].non_terminal().unwrap().productions.clone();
                for production in productions {
                    let mut first: HashSet<usize> = HashSet::new();
                    let mut left_follow =
                        Some(self.symbols[i].non_terminal().unwrap().follow.clone());

                    for i in (0..production.len()).rev() {
                        match &mut self.symbols[production[i]] {
                            Symbol::Terminal(_) => {
                                first = HashSet::new();
                                first.insert(production[i]);
                                left_follow = None;
                            }
                            Symbol::NonTerminal(nt) => {
                                let len = nt.follow.len();

                                if let Some(left_follow) = &left_follow {
                                    nt.follow.extend(left_follow.iter().cloned());
                                }
                                nt.follow.extend(first.iter().cloned());
                                changed |= len != nt.follow.len();

                                if !nt.nullable {
                                    first = nt.first.clone();
                                    left_follow = None;
                                } else {
                                    first.extend(nt.first.iter().cloned());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
