pub mod grammar;
use grammar::lr_dfa::LRFSMType;
use std::{fs, io::BufRead};

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
}

fn main() {
    let mut actions: Vec<&str> = Vec::new();
    let mut outputs: Vec<&str> = Vec::new();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut i: usize = 0;
    while i < args.len() && args[i] == "elf" {
        actions.push(args[i].as_str());
        i += 1;
    }
    while i < args.len()
        && (args[i] == "prod"
            || args[i] == "nff"
            || args[i] == "ll1"
            || args[i] == "lr0fsm"
            || args[i] == "lr1fsm"
            || args[i] == "lalrfsm"
            || args[i] == "lr0table"
            || args[i] == "lr1table"
            || args[i] == "lalrtable")
    {
        outputs.push(args[i].as_str());
        i += 1;
    }
    let mut is_latex = false;
    while i < args.len() && (args[i] == "-h" || args[i] == "--help" || args[i] == "-l") {
        if args[i] == "-h" || args[i] == "--help" {
            print_help();
            return;
        } else {
            is_latex = true;
        }
        i += 1;
    }

    if i + 1 < args.len() || outputs.len() < 1 {
        print_help();
        return;
    }

    let input: String = if i == args.len() {
        std::io::stdin()
            .lock()
            .lines()
            .map(|l| l.unwrap())
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        fs::read_to_string(args[i].as_str()).expect("Failed to read file")
    };

    let mut g = Grammar::parse(&input).unwrap();

    for action in actions {
        if action == "elf" {
            g.eliminate_left_recursion();
        }
    }

    for output in outputs {
        if output == "prod" {
            let t = g.to_production_output_vec();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "nff" {
            let t = g.to_non_terminal_output_vec();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "ll1" {
            let t = g.generate_ll1_parsing_table();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lr0fsm" {
            let t = g.to_lr_fsm(LRFSMType::LR0).unwrap();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lr1fsm" {
            let t = g.to_lr_fsm(LRFSMType::LR1).unwrap();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lalrfsm" {
            let t = g.to_lr_fsm(LRFSMType::LALR).unwrap();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lr0table" {
            let t = g.to_lr_fsm(LRFSMType::LR0).unwrap().to_parsing_table();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lr1table" {
            let t = g.to_lr_fsm(LRFSMType::LR1).unwrap().to_parsing_table();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
        if output == "lalrtable" {
            let t = g.to_lr_fsm(LRFSMType::LALR).unwrap().to_parsing_table();
            println!(
                "{}",
                if is_latex {
                    t.to_latex()
                } else {
                    t.to_plaintext()
                }
            );
        }
    }
}
