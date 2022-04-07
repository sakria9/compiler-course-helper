pub mod grammar;
use compiler_course_helper_lib::{grammar_to_output, Action, Format, LRFSMType, Output};
use std::{collections::HashMap, fs, io::BufRead};

pub use grammar::Grammar;

fn print_help() {
    println!("Usage: compiler-course-helper [actions] outputs [options] [grammar file]");
    println!("actions:");
    println!("  elf: Eliminate left recursion");
    println!("outputs:");
    println!("  prod: Productions");
    println!("  nff: Nullable first and follow");
    println!("  ll1: LL(1) parsing table");
    println!("  lr0fsm: LR(0) Automata");
    println!("  lr1fsm: LR(1) Automata");
    println!("  lalrfsm: LALR Automata");
    println!("  lr0table: LR(0) parsing table");
    println!("  lr1table: LR(1) parsing table");
    println!("  lalrtable: LALR parsing table");
    println!("options:");
    println!("  -h: Print this help");
    println!("  -l: Print in LaTeX format");
    println!("  -j: Print in JSON format");
}

fn main() {
    let mut actions: Vec<Action> = Vec::new();
    let mut outputs: Vec<Output> = Vec::new();
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    let action_map: HashMap<&str, Action> = [("elf", Action::EliminateLeftRecursion)]
        .iter()
        .cloned()
        .collect();
    let output_map: HashMap<&str, Output> = [
        ("prod", Output::Production(Format::Plain)),
        ("nff", Output::NonTerminal(Format::Plain)),
        ("ll1", Output::LL1ParsingTable(Format::Plain)),
        ("lr0fsm", Output::LRFSM(LRFSMType::LR0, Format::Plain)),
        ("lr1fsm", Output::LRFSM(LRFSMType::LR1, Format::Plain)),
        ("lalrfsm", Output::LRFSM(LRFSMType::LALR, Format::Plain)),
        (
            "lr0table",
            Output::LRParsingTable(LRFSMType::LR0, Format::Plain),
        ),
        (
            "lr1table",
            Output::LRParsingTable(LRFSMType::LR1, Format::Plain),
        ),
        (
            "lalrtable",
            Output::LRParsingTable(LRFSMType::LALR, Format::Plain),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let mut i: usize = 0;
    while i < args.len() && action_map.contains_key(args[i].as_str()) {
        actions.push(action_map[args[i].as_str()]);
        i += 1;
    }

    while i < args.len() && output_map.contains_key(args[i].as_str()) {
        outputs.push(output_map[args[i].as_str()]);
        i += 1;
    }

    let mut output_format = Format::Plain;
    while i < args.len() && ["-h", "--help", "-l", "-j"].contains(&args[i].as_str()) {
        if args[i] == "-h" || args[i] == "--help" {
            print_help();
            return;
        } else if args[i] == "-l" {
            output_format = Format::LaTeX;
        } else if args[i] == "-j" {
            output_format = Format::JSON;
        }
        i += 1;
    }
    let outputs: Vec<Output> = outputs
        .into_iter()
        .map(|mut o| {
            o.format(output_format);
            o
        })
        .collect();

    if i + 1 < args.len() || outputs.len() < 1 {
        print_help();
        return;
    }

    let grammar: String = if i == args.len() {
        std::io::stdin()
            .lock()
            .lines()
            .map(|l| l.unwrap())
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        fs::read_to_string(args[i].as_str()).expect("Failed to read file")
    };

    match grammar_to_output(&grammar, &actions, &outputs) {
        Ok(v) => {
            for (i, e) in v.into_iter().enumerate() {
                match e {
                    Ok(o) => println!("{}", o),
                    Err(e) => println!("Error {}-th output: {}", i, e),
                }
            }
        }
        Err(e) => {
            println!("ERROR! {}", e);
        }
    }
}
