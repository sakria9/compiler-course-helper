# Compiler Course Helper

Support:
- eliminate left recursion (require grammar with no cycles or ϵ-production)
- calculate nullable, first sets, follow, sets
- generate LL(1) parsing table
- generate LR(0) automata, parsing table
- generate LR(1) automata, parsing table
- generate LALR automata, parsing table
- **output format: plaintext JSON LaTeX**
- **WebAssembly**

## Build

```
$ cargo run
$ cargo build --release
```

```
$ wasm-pack build --help
```

## CLI

### Usage

```
$ ./compiler-course-helper
Usage: compiler-course-helper [action]... output... [option] [grammar file]
action:
  elf: Eliminate left recursion
output:
  prod: Productions
  nff: Nullable first and follow
  ll1: LL(1) parsing table
  lr0fsm: LR(0) Automata
  lr1fsm: LR(1) Automata
  lalrfsm: LALR Automata
  lr0table: LR(0) parsing table
  lr1table: LR(1) parsing table
  lalrtable: LALR parsing table
option:
  -h: Print this help
  -l: Print in LaTeX format
  -j: Print in JSON format
```

### Example

```
$ ./compiler-course-helper elf prod ll1 -l
E -> E a | a (this is input)
\[\begin{array}{cll}\\
E & \rightarrow & \text{a} \  E'\\
E' & \rightarrow & \text{a} \  E' \mid \epsilon\\
\end{array}\]
\[\begin{array}{c|l|l}
 & \text{\$} & \text{a}\\\hline
E &  & E \rightarrow \text{a} \  E'\\
E' & E' \rightarrow \epsilon & E' \rightarrow \text{a} \  E'
\end{array}\]
```

```
$ ./compiler-course-helper lr0table ../../testcase/expr.txt (read grammar from file)
   |             $ |             + |             * |  ( |             ) | id | E |  T |  F
 0 |               |               |               | s1 |               | s5 | 2 |  4 |  3
 1 |               |               |               | s1 |               | s5 | 6 |  4 |  3
 2 |           acc |            s7 |               |    |               |    |   |    |
 3 |     r(T -> F) |     r(T -> F) |     r(T -> F) |    |     r(T -> F) |    |   |    |
 4 |     r(E -> T) |     r(E -> T) |            s8 |    |     r(E -> T) |    |   |    |
 5 |    r(F -> id) |    r(F -> id) |    r(F -> id) |    |    r(F -> id) |    |   |    |
 6 |               |            s7 |               |    |            s9 |    |   |    |
 7 |               |               |               | s1 |               | s5 |   | 10 |  3
 8 |               |               |               | s1 |               | s5 |   |    | 11
 9 | r(F -> ( E )) | r(F -> ( E )) | r(F -> ( E )) |    | r(F -> ( E )) |    |   |    |
10 | r(E -> E + T) | r(E -> E + T) |            s8 |    | r(E -> E + T) |    |   |    |
11 | r(T -> T * F) | r(T -> T * F) | r(T -> T * F) |    | r(T -> T * F) |    |   |    |
```

## WebAssembly Library

```rust
#[wasm_bindgen]
pub fn wasm_grammar_to_output(json: &str) -> String {
    let args: WasmArgs = serde_json::from_str(json).unwrap();
    let result = grammar_to_output(&args.grammar, &args.actions, &args.outputs);
    serde_json::to_string(&result).unwrap()
}
```

Example argument:

```json
{
    "grammar": "E -> E + a | a",
    "actions": ["EliminateLeftRecursion"],
    "outputs": [
        {"Production": "Plain"},
        {"LL1ParsingTable": "LaTeX"},
        {"LRParsingTable": ["LR0", "JSON"]}
    ]
}
```

Example outputs:

```json
{
    "Ok": [
        {
            "Ok": " E -> a E'\nE' -> + a E'\n    | ϵ"
        },
        {
            "Ok": "\\[\\begin{array}{c|l|l|l}\n & \\text{\\$} & \\text{+} & \\text{a}\\\\\\hline\nE &  &  & E \\rightarrow \\text{a} \\  E'\\\\\nE' & E' \\rightarrow \\epsilon & E' \\rightarrow \\text{+} \\  \\text{a} \\  E' & \n\\end{array}\\]"
        },
        {
            "Ok": "{\"t\":\"LR0\",\"terminals\":[\"$\",\"+\",\"a\"],\"non_terminals\":[\"E\",\"E'\"],\"action\":[[[],[],[{\"Shift\":2}]],[[\"Accept\"],[],[]],[[{\"Reduce\":[\"E'\",[\"ϵ\"]]}],[{\"Shift\":3}],[]],[[],[],[{\"Shift\":5}]],[[{\"Reduce\":[\"E\",[\"a\",\"E'\"]]}],[],[]],[[{\"Reduce\":[\"E'\",[\"ϵ\"]]}],[{\"Shift\":3}],[]],[[{\"Reduce\":[\"E'\",[\"+\",\"a\",\"E'\"]]}],[],[]]],\"goto\":[[1,null],[null,null],[null,4],[null,null],[null,null],[null,6],[null,null]]}"
        }
    ]
}
```

## Rust Library

```rust
use compiler_course_helper::{Grammar, LRFSMType};

fn main() {
    let mut g = Grammar::parse(
        "
    E -> E + T | T
    T -> T * F | F
    F -> ( E ) | id",
    )
    .unwrap();
    
    g.eliminate_left_recursion();

    println!("{}", g.to_production_output_vec().to_plaintext());
    println!("{}", g.to_production_output_vec().to_latex());

    println!("{}", g.to_non_terminal_output_vec().to_plaintext());
    println!("{}", g.to_non_terminal_output_vec().to_latex());

    println!("{}", g.generate_ll1_parsing_table().to_latex());
    println!("{}", g.generate_ll1_parsing_table().to_plaintext());

    let fsm = g.to_lr_fsm(LRFSMType::LR0).unwrap();
    println!("{}", fsm.to_plaintext());
    println!("{}", fsm.to_latex());

    let table = fsm.to_parsing_table();
    println!("{}", table.to_plaintext());
    println!("{}", table.to_latex());
}
```