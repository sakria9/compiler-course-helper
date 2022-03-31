use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct NonTerminal {
    pub index: usize,
    pub name: String,
    pub first: HashSet<usize>,
    pub follow: HashSet<usize>,
    pub nullable: bool,
    pub productions: Vec<Vec<usize>>,
}

impl NonTerminal {
    pub fn new(index: usize, name: String) -> Self {
        Self {
            index,
            name,
            first: HashSet::new(),
            follow: HashSet::new(),
            nullable: false,
            productions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    NonTerminal(NonTerminal),
    Terminal(String),
}

impl Symbol {
    pub fn non_terminal(&self) -> Option<&NonTerminal> {
        match self {
            Symbol::NonTerminal(e) => Some(e),
            Symbol::Terminal(_) => None,
        }
    }

    pub fn mut_non_terminal(&mut self) -> Option<&mut NonTerminal> {
        match self {
            Symbol::NonTerminal(e) => Some(e),
            Symbol::Terminal(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub symbols: Vec<Symbol>,
    pub symbol_table: HashMap<String, usize>,
    pub start_symbol: Option<usize>,
}

impl Grammar {
    pub fn new() -> Self {
        let mut g = Self {
            symbols: Vec::new(),
            symbol_table: HashMap::new(),
            start_symbol: None,
        };

        let e_idx = g.add_non_terminal(super::EPSILON);
        g.symbols[e_idx].mut_non_terminal().unwrap().nullable = true;
        g.symbol_table.insert("ε".to_string(), e_idx);

        g.add_terminal(super::END_MARK.to_string());

        g
    }

    pub fn terminal_iter(&self) -> impl Iterator<Item = &String> {
        self.symbols.iter().filter_map(|s| {
            if let Symbol::Terminal(name) = s {
                Some(name)
            } else {
                None
            }
        })
    }

    pub fn non_terminal_iter(&self) -> impl Iterator<Item = &NonTerminal> {
        self.symbols.iter().filter_map(|s| s.non_terminal()).skip(1)
    }

    pub fn non_terminal_iter_mut(&mut self) -> impl Iterator<Item = &mut NonTerminal> {
        self.symbols
            .iter_mut()
            .filter_map(|s| s.mut_non_terminal())
            .skip(1)
    }

    pub fn get_symbol_index(&self, name: &str) -> Option<usize> {
        self.symbol_table.get(name).cloned()
    }

    pub fn add_non_terminal(&mut self, name: &str) -> usize {
        let idx = self.symbols.len();
        self.symbols
            .push(Symbol::NonTerminal(NonTerminal::new(idx, name.to_string())));
        self.symbol_table.insert(name.to_string(), idx);
        idx
    }

    pub fn add_terminal(&mut self, name: String) -> usize {
        let idx = self.symbols.len();
        self.symbols.push(Symbol::Terminal(name.clone()));
        self.symbol_table.insert(name, idx);
        idx
    }

    pub fn add_production(&mut self, left: usize, right: Vec<usize>) {
        self.symbols[left]
            .mut_non_terminal()
            .unwrap()
            .productions
            .push(right);
    }

    pub fn get_symbol_name(&self, index: usize) -> &str {
        match &self.symbols[index] {
            Symbol::NonTerminal(e) => e.name.as_str(),
            Symbol::Terminal(e) => e.as_str(),
        }
    }

    pub fn get_symbol_prime_name(&self, mut name: String) -> String {
        while self.symbol_table.contains_key(&name) {
            name.push('\'');
        }
        name
    }
}
