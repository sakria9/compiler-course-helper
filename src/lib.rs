extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod grammar;
pub use grammar::Grammar;

#[wasm_bindgen]
pub fn nullable_first_follow_to_json(grammar: &str) -> String {
    let g = crate::Grammar::parse(grammar);
    match g {
        Ok(mut g) => {
            g.calculate_nullable_first_follow();
            g.to_non_terminal_output_vec().to_json()
        }
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    }
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
