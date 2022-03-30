use std::collections::{HashMap, HashSet};

use super::{grammar::NonTerminal, Grammar, EPSILON};

impl Grammar {
    pub fn eliminate_left_recursion(&mut self) {
        let epsilon_idx = self.get_symbol_index(EPSILON).unwrap();
        let offset = self.symbols.len();

        let mut non_terminals = self.non_terminal_iter_mut().collect::<Vec<_>>();
        let map: HashMap<usize, usize> =
            non_terminals
                .iter()
                .enumerate()
                .fold(HashMap::new(), |mut map, (i, nt)| {
                    map.insert(nt.index, i);
                    map
                });

        let mut new_non_terminals: Vec<NonTerminal> = Vec::new();

        for i in 0..non_terminals.len() {
            let (replace, b) = non_terminals.split_at_mut(i);
            let (nt, _) = b.split_first_mut().unwrap();
            let replace = &replace[..];

            let old_productions = std::mem::replace(&mut nt.productions, Vec::new());
            let mut recursive_productions: Vec<Vec<usize>> = Vec::new();
            for mut production in old_productions {
                if let Some(idx) = production.first() {
                    if let Some(&arr_idx) = map.get(idx) {
                        match arr_idx.cmp(&i) {
                            std::cmp::Ordering::Less => {
                                for prefix in &replace[arr_idx].productions {
                                    let new_production =
                                        prefix.iter().chain(production.iter().skip(1)).cloned();

                                    if Some(&nt.index) == prefix.first() {
                                        recursive_productions.push(new_production.skip(1).collect())
                                    } else {
                                        nt.productions.push(new_production.collect())
                                    }
                                }
                            }
                            std::cmp::Ordering::Equal => {
                                production.remove(0);
                                recursive_productions.push(production);
                            }
                            std::cmp::Ordering::Greater => {
                                nt.productions.push(production);
                            }
                        };
                    } else {
                        nt.productions.push(production);
                    }
                }
            }

            println!("{:?}", nt.productions);
            println!("{:?}", recursive_productions);

            if recursive_productions.len() > 0 {
                let nt_prime_idx = offset + new_non_terminals.len();
                for production in &mut nt.productions {
                    production.push(nt_prime_idx);
                }
                for production in &mut recursive_productions {
                    production.push(nt_prime_idx);
                }
                recursive_productions.push(vec![epsilon_idx]);
                new_non_terminals.push(NonTerminal {
                    index: nt_prime_idx,
                    nullable: false,
                    name: nt.name.clone(),
                    first: HashSet::new(),
                    follow: HashSet::new(),
                    productions: recursive_productions,
                });
            }
        }

        for mut nt in new_non_terminals {
            nt.name = self.get_symbol_prime_name(nt.name);
            self.symbol_table.insert(nt.name.clone(), nt.index);
            self.symbols.push(super::grammar::Symbol::NonTerminal(nt));
        }
    }
}
