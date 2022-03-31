use std::collections::HashMap;

use crate::Grammar;

use super::{pretty_print::ProductionOutput, EPSILON};

pub struct LL1ParsingTable<'a> {
    terminals: Vec<&'a str>,
    rows: Vec<(&'a str, Vec<Vec<ProductionOutput<'a>>>)>,
}

impl LL1ParsingTable<'_> {
    pub fn to_plaintext(&self) -> String {
        let mut header: Vec<String> = vec![String::new()];
        header.extend(self.terminals.iter().map(|&t| t.to_string()));
        let mut output: Vec<Vec<String>> = vec![header];
        for (left, row) in &self.rows {
            let mut line: Vec<String> = vec![left.to_string()];
            line.extend(row.iter().map(|productions| {
                productions
                    .iter()
                    .map(|production| production.to_plaintext(left.len()))
                    .collect::<Vec<_>>()
                    .join(", ")
            }));
            output.push(line);
        }

        let mut width = vec![0; self.terminals.len() + 1];
        for j in 0..output[0].len() {
            width[j] = output.iter().map(|line| line[j].len()).max().unwrap();
        }
        output
            .iter()
            .map(|line| {
                line.iter()
                    .enumerate()
                    .map(|(i, s)| format!("{:>width$}", s, width = width[i]))
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Grammar {
    pub fn generate_ll1_parsing_table(&self) -> LL1ParsingTable {
        let terminals: Vec<&str> = self.terminal_iter().map(|t| t.as_str()).collect();
        let map: HashMap<usize, usize> = terminals
            .iter()
            .enumerate()
            .map(|(i, t)| (self.get_symbol_index(t).unwrap(), i))
            .collect();

        let mut rows: Vec<(&str, Vec<Vec<ProductionOutput>>)> = Vec::new();
        for nt in self.non_terminal_iter() {
            let left = nt.name.as_str();
            let mut row: Vec<Vec<ProductionOutput>> = vec![Vec::new(); terminals.len()];
            for production in &nt.productions {
                let first = self.calculate_first_for_production(production);

                let production_string_iter =
                    production.iter().map(|idx| self.get_symbol_name(*idx));

                for col in first.iter().map(|idx| map[idx]) {
                    row[col].push(ProductionOutput {
                        left,
                        rights: vec![production_string_iter.clone().collect::<Vec<_>>()],
                    });
                }
            }

            if nt.nullable {
                println!("{} is nullable", left);
                println!("follow of {}: {:?}", left, nt.follow);
                for idx in &nt.follow {
                    row[map[idx]].push(ProductionOutput {
                        left,
                        rights: vec![vec![EPSILON]],
                    });
                }
            }

            rows.push((left, row));
        }

        LL1ParsingTable { terminals, rows }
    }
}
