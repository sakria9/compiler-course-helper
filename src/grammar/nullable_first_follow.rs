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
                                for (idx, symbol) in
                                    production.iter().map(|i| (*i, &self.symbols[*i]))
                                {
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

    fn calculate_follow(&mut self) {
        fn a_extend_b(a: &HashSet<usize>, b: &HashSet<usize>) {
            if a == b {
                return;
            }
            let r = a as *const _ as *mut HashSet<usize>;
            unsafe {
                (*r).extend(b.iter().cloned());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for left in self.non_terminal_iter() {
                for production in &left.productions {
                    let mut first: HashSet<usize> = HashSet::new();
                    let mut left_follow = Some(&left.follow);

                    for i in (0..production.len()).rev() {
                        match &self.symbols[production[i]] {
                            Symbol::Terminal(_) => {
                                first = HashSet::new();
                                first.insert(production[i]);
                                left_follow = None;
                            }
                            Symbol::NonTerminal(nt) => {
                                left_follow.and_then(|b| Some(a_extend_b(&nt.follow, b)));
                                a_extend_b(&nt.follow, &first);
                                if !nt.nullable {
                                    first = nt.first.clone();
                                    left_follow = None;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
