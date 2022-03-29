pub mod grammar;
pub use grammar::Grammar;
fn main() {
    let mut g = grammar::Grammar::parse(
        "
        S -> A a | b
        A -> A c | S d | ϵ
        ",
    )
    .unwrap();

    g.calculate_nullable_first_follow();
    println!("{}", g.to_non_terminal_output_vec().to_plaintext());
}
