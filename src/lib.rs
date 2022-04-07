extern crate wasm_bindgen;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod grammar;
pub use grammar::lr_fsm::LRFSMType;
pub use grammar::Grammar;

#[derive(Serialize, Deserialize)]
pub struct WasmArgs {
    pub grammar: String,
    pub actions: Vec<Action>,
    pub outputs: Vec<Output>,
}

// This function is intended to be called from JavaScript.
// Example:
// {
//     "grammar": "E -> E + T | T\nT -> T * F | F\nF -> ( E ) | id",
//     "actions": ["EliminateLeftRecursion"],
//     "outputs": [
//         {"NonTerminal": "JSON"},
//         {"Production": "JSON"},
//         {"LL1ParsingTable": "JSON"},
//         {"LRParsingTable": ["LR0", "JSON"]}
//     ]
// }
#[wasm_bindgen]
pub fn wasm_grammar_to_output(json: &str) -> String {
    let args: WasmArgs = serde_json::from_str(json).unwrap();
    let result = grammar_to_output(&args.grammar, &args.actions, &args.outputs);
    serde_json::to_string(&result).unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    EliminateLeftRecursion,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Format {
    Plain,
    LaTeX,
    JSON,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Output {
    Production(Format),
    NonTerminal(Format),
    LL1ParsingTable(Format),
    LRFSM(LRFSMType, Format),
    LRParsingTable(LRFSMType, Format),
}

impl Output {
    pub fn format(&mut self, f: Format) {
        match self {
            Output::Production(format) => *format = f,
            Output::NonTerminal(format) => *format = f,
            Output::LL1ParsingTable(format) => *format = f,
            Output::LRFSM(_, format) => *format = f,
            Output::LRParsingTable(_, format) => *format = f,
        }
    }
}

pub fn grammar_to_output(
    grammar: &str,
    actions: &[Action],
    outputs: &[Output],
) -> Result<Vec<Result<String, String>>, String> {
    let mut ret: Vec<Result<String, String>> = Vec::new();

    let mut g = match Grammar::parse(grammar) {
        Ok(g) => g,
        Err(e) => {
            return Err(e);
        }
    };

    for action in actions {
        match action {
            Action::EliminateLeftRecursion => g.eliminate_left_recursion(),
        }
    }

    for output in outputs {
        match output {
            Output::Production(format) => {
                let t = g.to_production_output_vec();
                ret.push(Ok(match format {
                    Format::Plain => t.to_plaintext(),
                    Format::LaTeX => t.to_latex(),
                    Format::JSON => serde_json::to_string(&t).unwrap(),
                }));
            }
            Output::NonTerminal(format) => {
                let t = g.to_non_terminal_output_vec();
                ret.push(Ok(match format {
                    Format::Plain => t.to_plaintext(),
                    Format::LaTeX => t.to_latex(),
                    Format::JSON => serde_json::to_string(&t).unwrap(),
                }));
            }
            Output::LL1ParsingTable(format) => {
                let t = g.generate_ll1_parsing_table();
                ret.push(Ok(match format {
                    Format::Plain => t.to_plaintext(),
                    Format::LaTeX => t.to_latex(),
                    Format::JSON => serde_json::to_string(&t).unwrap(),
                }));
            }
            Output::LRFSM(typ, format) => ret.push(g.to_lr_fsm(*typ).and_then(|t| {
                Ok(match format {
                    Format::Plain => t.to_plaintext(),
                    Format::LaTeX => t.to_latex(),
                    Format::JSON => serde_json::to_string(&t).unwrap(),
                })
            })),
            Output::LRParsingTable(typ, format) => ret.push(g.to_lr_fsm(*typ).and_then(|t| {
                let t = t.to_parsing_table();
                Ok(match format {
                    Format::Plain => t.to_plaintext(),
                    Format::LaTeX => t.to_latex(),
                    Format::JSON => serde_json::to_string(&t).unwrap(),
                })
            })),
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod parse_tests {
    use crate::grammar::EPSILON;

    #[test]
    fn simple_parse() {
        let g = crate::Grammar::parse("S -> a").unwrap();

        let s = g.symbol_table.get("S").unwrap().clone();
        let a = g.symbol_table.get("a").unwrap().clone();
        let epsilon = g.symbol_table.get(EPSILON).unwrap().clone();

        assert_eq!(g.get_symbol_name(s), "S");
        assert_eq!(g.get_symbol_name(a), "a");

        assert_eq!(g.symbols[epsilon].non_terminal().unwrap().nullable, true);

        assert_eq!(g.symbols[s].non_terminal().unwrap().productions[0], vec![a]);
    }

    #[test]
    fn simple_parse_with_space() {
        let g = crate::Grammar::parse("  S -> a ").unwrap();

        let s = g.symbol_table.get("S").unwrap().clone();
        let a = g.symbol_table.get("a").unwrap().clone();

        assert_eq!(g.get_symbol_name(s), "S");
        assert_eq!(g.get_symbol_name(a), "a");

        assert_eq!(g.symbols[s].non_terminal().unwrap().productions[0], vec![a]);
    }

    #[test]
    fn simple_parse_with_space_and_newline() {
        let g = crate::Grammar::parse("  S -> a \n | b c").unwrap();

        let s = g.symbol_table.get("S").unwrap().clone();
        let a = g.symbol_table.get("a").unwrap().clone();
        let b = g.symbol_table.get("b").unwrap().clone();
        let c = g.symbol_table.get("c").unwrap().clone();

        assert_eq!(g.get_symbol_name(s), "S");
        assert_eq!(g.get_symbol_name(a), "a");
        assert_eq!(g.get_symbol_name(b), "b");
        assert_eq!(g.get_symbol_name(c), "c");
        assert_eq!(g.symbols[s].non_terminal().unwrap().productions[0], vec![a]);
        assert_eq!(
            g.symbols[s].non_terminal().unwrap().productions[1],
            vec![b, c]
        );
    }

    #[test]
    fn empty_parse() {
        let _g = crate::Grammar::parse("  \n  ").unwrap();
    }

    #[test]
    #[should_panic]
    fn two_rightarrows_parse() {
        let _g = crate::Grammar::parse("S -> a -> b").unwrap();
    }

    #[test]
    #[should_panic]
    fn no_left_parse() {
        let _g = crate::Grammar::parse("-> a -> b").unwrap();
    }

    #[test]
    #[should_panic]
    fn no_previous_left_parse() {
        let _g = crate::Grammar::parse("| a b\n S -> a").unwrap();
    }

    #[test]
    #[should_panic]
    fn left_contain_space() {
        let _g = crate::Grammar::parse("S a S -> x").unwrap();
    }
}

#[cfg(test)]
mod nullable_first_follow_test {}

#[cfg(test)]
mod generate_ll1_parsing_table_test {
    #[test]
    fn expression_test() {
        let mut g = crate::Grammar::parse(
            "
            E -> T E'
            E' -> + T E' | ε
            T -> F T'
            T' -> * F T' | ε
            F -> ( E ) | id
            ",
        )
        .unwrap();

        g.calculate_nullable_first_follow();
        let result = g.generate_ll1_parsing_table();
        println!("{}", result.to_plaintext());
    }
}
