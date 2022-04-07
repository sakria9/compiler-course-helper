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
    println!("  -j: Print in JSON format");
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
        && [
            "prod",
            "nff",
            "ll1",
            "lr0fsm",
            "lr1fsm",
            "lalrfsm",
            "lr0table",
            "lr1table",
            "lalrtable",
        ]
        .contains(&args[i].as_str())
    {
        outputs.push(args[i].as_str());
        i += 1;
    }

    enum OutputFormat {
        Plain,
        LaTeX,
        JSON,
    }
    let mut output_format = OutputFormat::Plain;

    while i < args.len() && ["-h", "--help", "-l", "-j"].contains(&args[i].as_str()) {
        if args[i] == "-h" || args[i] == "--help" {
            print_help();
            return;
        } else if args[i] == "-l" {
            output_format = OutputFormat::LaTeX;
        } else if args[i] == "-j" {
            output_format = OutputFormat::JSON;
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
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "nff" {
            let t = g.to_non_terminal_output_vec();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "ll1" {
            let t = g.generate_ll1_parsing_table();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lr0fsm" {
            let t = g.to_lr_fsm(LRFSMType::LR0).unwrap();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lr1fsm" {
            let t = g.to_lr_fsm(LRFSMType::LR1).unwrap();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lalrfsm" {
            let t = g.to_lr_fsm(LRFSMType::LALR).unwrap();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lr0table" {
            let t = g.to_lr_fsm(LRFSMType::LR0).unwrap().to_parsing_table();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lr1table" {
            let t = g.to_lr_fsm(LRFSMType::LR1).unwrap().to_parsing_table();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
        if output == "lalrtable" {
            let t = g.to_lr_fsm(LRFSMType::LALR).unwrap().to_parsing_table();
            println!(
                "{}",
                match output_format {
                    OutputFormat::Plain => t.to_plaintext(),
                    OutputFormat::LaTeX => t.to_latex(),
                    OutputFormat::JSON => serde_json::to_string(&t).unwrap(),
                }
            );
        }
    }
}
