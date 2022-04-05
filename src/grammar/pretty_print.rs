use crowbook_text_processing::escape;
use serde::Serialize;

use super::{
    lr_dfa::{DotProduction, LRItem, LRParsingTable, LRParsingTableAction},
    Grammar, EPSILON,
};

#[derive(Debug, Clone)]
pub struct ProductionOutput<'a> {
    pub left: &'a str,
    pub rights: Vec<Vec<&'a str>>,
}

impl ProductionOutput<'_> {
    pub fn to_plaintext(&self, left_width: usize, multiline: bool) -> String {
        self.rights
            .iter()
            .map(|right| right.join(" "))
            .enumerate()
            .map(|(i, right)| {
                if i == 0 {
                    format!("{:>width$} -> {}", self.left, right, width = left_width)
                } else {
                    if multiline {
                        format!("{:>width$}  | {}", "", right, width = left_width)
                    } else {
                        format!(" | {}", right)
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(if multiline { "\n" } else { "" })
    }
    pub fn to_latex(&self, and_sign: bool) -> String {
        if self.rights.len() == 0 {
            return String::new();
        }

        let left = if and_sign {
            format!("{} & \\rightarrow &", escape::tex(self.left)).to_string()
        } else {
            format!("{} \\rightarrow ", escape::tex(self.left)).to_string()
        };
        let right = self
            .rights
            .iter()
            .map(|right| {
                right
                    .iter()
                    .map(|s| escape::tex(*s))
                    .collect::<Vec<_>>()
                    .join(" \\ ")
            })
            .collect::<Vec<_>>()
            .join(" \\mid ");

        let output = left + &right;
        output.replace(super::EPSILON, "\\epsilon")
    }
}

pub struct ProductionOutputVec<'a> {
    productions: Vec<ProductionOutput<'a>>,
}

impl ProductionOutputVec<'_> {
    pub fn to_plaintext(&self) -> String {
        let left_max_len = self.productions.iter().map(|p| p.left.len()).max().unwrap();
        self.productions
            .iter()
            .map(|s| s.to_plaintext(left_max_len, true))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_latex(&self) -> String {
        std::iter::once("\\[\\begin{array}{cll}".to_string())
            .chain(self.productions.iter().map(|s| s.to_latex(true)))
            .chain(std::iter::once("\\end{array}\\]".to_string()))
            .collect::<Vec<String>>()
            .join("\\\\\n")
    }
}

impl Grammar {
    pub fn to_production_output_vec(&self) -> ProductionOutputVec {
        let mut productions = Vec::new();
        for symbol in self.symbols.iter().skip(1) {
            // skip(1): skip epsilon
            if let Some(non_terminal) = symbol.non_terminal() {
                let mut rights = Vec::new();
                for production in &non_terminal.productions {
                    rights.push(self.production_to_vec_str(&production));
                }
                productions.push(ProductionOutput {
                    left: non_terminal.name.as_str(),
                    rights,
                });
            }
        }
        ProductionOutputVec { productions }
    }
}

#[derive(Serialize)]
struct NonTerminalOutput<'a> {
    name: &'a str,
    nullable: bool,
    first: Vec<&'a str>,
    follow: Vec<&'a str>,
}

impl NonTerminalOutput<'_> {
    fn to_plaintext(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            self.name,
            self.nullable,
            self.first.join(", "),
            self.follow.join(", ")
        )
    }
    fn to_latex(&self) -> String {
        fn f(a: &Vec<&str>) -> String {
            a.iter()
                .map(|s| escape::tex(*s))
                .collect::<Vec<_>>()
                .join(r"\ ")
                .replace(EPSILON, r"$\epsilon$")
        }

        format!(
            "{} & {} & {} & {}",
            escape::tex(self.name),
            self.nullable,
            f(&self.first),
            f(&self.follow)
        )
    }
}

#[derive(Serialize)]
pub struct NonTerminalOutputVec<'a> {
    data: Vec<NonTerminalOutput<'a>>,
}

impl NonTerminalOutputVec<'_> {
    pub fn to_plaintext(&self) -> String {
        self.data
            .iter()
            .map(|s| s.to_plaintext())
            .collect::<Vec<String>>()
            .join("\n")
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn to_latex(&self) -> String {
        let content = self
            .data
            .iter()
            .map(|e| e.to_latex())
            .collect::<Vec<_>>()
            .join("\\\\\n ");

        "\\begin{tabular}{c|c|c|c}\n".to_string()
            + "Symbol & Nullable & First & Follow\\\\\\hline\n"
            + &content
            + "\\\\\n\\end{tabular}"
    }
}

impl Grammar {
    pub fn to_non_terminal_output_vec(&self) -> NonTerminalOutputVec {
        let mut data = Vec::new();
        for symbol in self.symbols.iter().skip(1) {
            // skip(1): skip epsilon
            if let Some(non_terminal) = symbol.non_terminal() {
                let mut t = NonTerminalOutput {
                    name: non_terminal.name.as_str(),
                    nullable: non_terminal.nullable,
                    first: non_terminal
                        .first
                        .iter()
                        .map(|idx| self.get_symbol_name(*idx))
                        .collect(),
                    follow: non_terminal
                        .follow
                        .iter()
                        .map(|idx| self.get_symbol_name(*idx))
                        .collect(),
                };
                t.first.sort();
                t.follow.sort();

                if non_terminal.nullable {
                    t.first.push(EPSILON);
                }
                data.push(t);
            }
        }
        NonTerminalOutputVec { data }
    }
}

impl DotProduction {
    pub fn to_plaintext(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.left);
        output.push_str(" -> ");
        for (i, s) in self.production.iter().enumerate() {
            if i != 0 {
                output.push_str(" ");
            }

            if i == self.position {
                output.push_str(".");
            }
            output.push_str(s);
        }
        if self.position == self.production.len() {
            output.push_str(".");
        }
        if let Some(lookahead) = &self.lookahead {
            output.push_str(", ");
            output.push_str(&lookahead.join("/"));
        }

        output
    }
    pub fn to_latex(&self) -> String {
        let mut right: Vec<String> = Vec::new();
        for (i, s) in self.production.iter().enumerate() {
            if i == self.position {
                right.push("\\ldots".to_string());
            }
            right.push(escape::tex(s).to_string());
        }
        let right = right.join(" ").replace(super::EPSILON, "\\epsilon");

        if let Some(lookahead) = &self.lookahead {
            let lookahead = lookahead
                .iter()
                .map(|s| escape::tex(s))
                .collect::<Vec<_>>()
                .join(" ");
            format!("${} \\rightarrow {}$, {}", self.left, right, lookahead)
        } else {
            format!("${} \\rightarrow {}$", self.left, right)
        }
    }
}

impl LRItem {
    pub fn to_plaintext(&self) -> String {
        let kernel = self
            .kernel
            .iter()
            .map(|c| c.to_plaintext())
            .collect::<Vec<_>>()
            .join("\n");

        let extend = if self.extend.len() > 0 {
            format!(
                "\n---\n{}",
                self.extend
                    .iter()
                    .map(|c| c.to_plaintext())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        let edges = if self.edges.len() > 0 {
            format!(
                "\n===\n{}",
                self.edges
                    .iter()
                    .map(|(k, v)| format!("- {} -> {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        format!("{}{}{}", kernel, extend, edges)
    }
}

impl LRParsingTableAction {
    pub fn to_plaintext(&self) -> String {
        match self {
            LRParsingTableAction::Reduce(r) => {
                format!("r({} -> {})", r.0, r.1.join(" "))
            }
            LRParsingTableAction::Shift(s) => {
                format!("s{}", s)
            }
            LRParsingTableAction::Accept => "acc".to_string(),
        }
    }

    pub fn to_latex(&self) -> String {
        match self {
            LRParsingTableAction::Reduce(r) => {
                format!(
                    "reduce ${} \\rightarrow {}$",
                    escape::tex(&r.0),
                    r.1.iter()
                        .map(|s| escape::tex(s))
                        .collect::<Vec<_>>()
                        .join(" \\  ")
                        .replace(super::EPSILON, "\\epsilon")
                )
            }
            LRParsingTableAction::Shift(s) => {
                format!("shift {}", s)
            }
            LRParsingTableAction::Accept => "accept".to_string(),
        }
    }
}

impl LRParsingTable {
    pub fn to_plaintext(&self) -> String {
        let mut output: Vec<Vec<String>> = Vec::new();

        output.push(vec![String::new()]);
        for s in self.terminals.iter().chain(self.non_terminals.iter()) {
            output[0].push(s.clone());
        }

        for (r1, r2) in self.action.iter().zip(self.goto.iter()) {
            let i = output.len() - 1;
            let row: Vec<String> = std::iter::once(i.to_string())
                .chain(r1.iter().map(|actions| {
                    actions
                        .iter()
                        .map(|action| action.to_plaintext())
                        .collect::<Vec<_>>()
                        .join("; ")
                }))
                .chain(r2.iter().map(|gotos| {
                    gotos
                        .iter()
                        .map(|goto| goto.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                }))
                .collect::<Vec<_>>();
            output.push(row);
        }

        let width: Vec<usize> = (0..output[0].len())
            .map(|j| output.iter().map(|row| row[j].len()).max().unwrap())
            .collect();

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
        let header: String = format!(
            "\\begin{{tabular}}{{c{}}}\n & \\multicolumn{{{}}}{{c}}{{action}} & \\multicolumn{{{}}}{{|c}}{{goto}}\\\\",
            "|l".repeat(self.terminals.len() + self.non_terminals.len()),
            self.terminals.len(),
            self.non_terminals.len(),
        );

        let mut content: Vec<Vec<String>> = Vec::new();

        let mut first_row: Vec<String> = Vec::new();
        for s in self.terminals.iter().chain(self.non_terminals.iter()) {
            first_row.push(escape::tex(s).to_string());
        }
        let first_row = first_row.join(" & ");

        for (r1, r2) in self.action.iter().zip(self.goto.iter()) {
            let i = content.len();
            let row: Vec<String> = std::iter::once(i.to_string())
                .chain(r1.iter().map(|actions| {
                    let r = actions
                        .iter()
                        .map(|action| action.to_latex())
                        .collect::<Vec<_>>()
                        .join("; ");
                    if actions.len() > 1 {
                        format!("{{\\color{{red}}{}}}", r)
                    } else {
                        r
                    }
                }))
                .chain(r2.iter().map(|gotos| {
                    gotos
                        .iter()
                        .map(|goto| goto.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                }))
                .collect::<Vec<_>>();
            content.push(row);
        }

        let content = content
            .iter()
            .map(|row| row.join(" & "))
            .collect::<Vec<_>>();
        let content = content.join(" \\\\\n");

        format!(
            "{}\n{} \\\\\\hline\n{}\n\\end{{tabular}}",
            header, first_row, content
        )
    }
}
