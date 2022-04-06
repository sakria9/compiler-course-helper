use crowbook_text_processing::escape::tex as escape_tex;
use std::collections::{HashMap, HashSet};

use crate::Grammar;

use super::{pretty_print::ProductionOutput, EPSILON};

pub struct LL1ParsingTable<'a> {
    terminals: Vec<&'a str>,
    rows: Vec<(&'a str, Vec<ProductionOutput<'a>>)>,
}

impl LL1ParsingTable<'_> {
    pub fn to_plaintext(&self) -> String {
        let mut header: Vec<String> = vec![String::new()];
        header.extend(self.terminals.iter().map(|&t| t.to_string()));
        let mut output: Vec<Vec<String>> = vec![header];
        for (left, row) in &self.rows {
            let mut line: Vec<String> = vec![left.to_string()];
            line.extend(
                row.iter()
                    .map(|productions| productions.to_plaintext(left.len(), false)),
            );
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

    pub fn to_latex(&self) -> String {
        let mut header: Vec<String> = vec![format!(
            "\\[\\begin{{array}}{{c{}}}\n",
            "|l".repeat(self.terminals.len()),
        )];
        header.extend(
            self.terminals
                .iter()
                .map(|&t| format!("\\text{{{}}}", escape_tex(t))),
        );
        let header = header.join(" & ");

        let mut output: Vec<String> = Vec::new();
        let termintal_set: HashSet<&str> = self.terminals.iter().cloned().collect();
        for (left, row) in &self.rows {
            let mut line: Vec<String> = vec![format!("{}", escape_tex(*left))];
            line.extend(
                row.iter()
                    .map(|productions| productions.to_latex(false, &termintal_set)),
            );
            output.push(line.join(" & "));
        }

        let output = output.join("\\\\\n");

        header + "\\\\\\hline\n" + &output + "\n\\end{array}\\]"
    }
}

impl Grammar {
    pub fn generate_ll1_parsing_table(&mut self) -> LL1ParsingTable {
        if !self.is_nullable_first_follow_valid() {
            self.calculate_nullable_first_follow();
        }

        let terminals: Vec<&str> = self.terminal_iter().map(|t| t.as_str()).collect();
        let map: HashMap<usize, usize> = terminals
            .iter()
            .enumerate()
            .map(|(i, t)| (self.get_symbol_index(t).unwrap(), i))
            .collect();

        let mut rows: Vec<(&str, Vec<ProductionOutput>)> = Vec::new();
        for nt in self.non_terminal_iter() {
            let left = nt.name.as_str();
            let mut row: Vec<ProductionOutput> = vec![
                ProductionOutput {
                    left,
                    rights: Vec::new()
                };
                terminals.len()
            ];
            for production in &nt.productions {
                let first = self.calculate_first_for_production(production);

                let production_string_iter =
                    production.iter().map(|idx| self.get_symbol_name(*idx));

                for col in first.iter().map(|idx| map[idx]) {
                    row[col]
                        .rights
                        .push(production_string_iter.clone().collect::<Vec<_>>());
                }
            }

            if nt.nullable {
                for idx in &nt.follow {
                    row[map[idx]].rights.push(vec![EPSILON]);
                }
            }

            rows.push((left, row));
        }

        LL1ParsingTable { terminals, rows }
    }
}
