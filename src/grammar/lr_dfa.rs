use std::collections::{HashMap, HashSet, VecDeque};

use crate::Grammar;

use super::{grammar::Symbol, END_MARK, EPSILON};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DotProduction {
    pub left: String,
    pub production: Vec<String>,
    pub position: usize,
    pub lookahead: Option<Vec<String>>,
}

impl DotProduction {
    pub fn new(left: String, production: Vec<String>, lookahead: Option<Vec<String>>) -> Self {
        let mut i = 0;
        while i < production.len() && production[i] == EPSILON {
            i += 1;
        }
        Self {
            left,
            production,
            position: i,
            lookahead,
        }
    }

    pub fn generate_next(&self) -> Self {
        let mut i = self.position + 1;
        while i < self.production.len() && self.production[i] == EPSILON {
            i += 1;
        }

        Self {
            left: self.left.clone(),
            production: self.production.clone(),
            position: i,
            lookahead: self.lookahead.clone(),
        }
    }

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
}

#[derive(PartialEq, Eq, Debug)]
pub struct LRItem {
    pub core: HashSet<DotProduction>,
    pub extend: HashSet<DotProduction>,
    pub edges: HashMap<String, usize>,
}

impl LRItem {
    fn calculate_extend(&mut self, g: &Grammar) {
        let is_lr1 = self.core.iter().next().unwrap().lookahead.is_some();
        let mut extend: HashMap<usize, Option<HashSet<usize>>> = HashMap::new();
        let mut q: VecDeque<usize> = VecDeque::new();

        let calculate_first = |production: &[String]| -> Vec<usize> {
            g.calculate_first_for_production(
                &production
                    .iter()
                    .map(|s| g.get_symbol_index(s).unwrap())
                    .collect::<Vec<_>>(),
            )
            .into_iter()
            .collect()
        };

        // use self.core as the initial self.extend
        for c in &self.core {
            if let Some(symbol) = c.production.get(c.position) {
                if let Symbol::NonTerminal(nt) = g.get_symbol_by_name(symbol.as_str()) {
                    if !extend.contains_key(&nt.index) {
                        extend.insert(nt.index, if is_lr1 { Some(HashSet::new()) } else { None });
                        q.push_back(nt.index);
                    }

                    if is_lr1 {
                        let lookahead = if c.position + 1 < c.production.len() {
                            calculate_first(&c.production[c.position + 1..])
                        } else {
                            c.lookahead
                                .as_ref()
                                .unwrap()
                                .iter()
                                .map(|s| g.get_symbol_index(s).unwrap())
                                .collect()
                        };
                        extend
                            .get_mut(&nt.index)
                            .unwrap()
                            .as_mut()
                            .unwrap()
                            .extend(lookahead.into_iter());
                    }
                }
            }
        }

        // iteratively calculate self.extend
        while let Some(s_idx) = q.pop_front() {
            for production in &g.symbols[s_idx].non_terminal().unwrap().productions {
                if let Symbol::NonTerminal(nt) = &g.symbols[production[0]] {
                    if !extend.contains_key(&nt.index) {
                        extend.insert(nt.index, if is_lr1 { Some(HashSet::new()) } else { None });
                        q.push_back(nt.index);
                    }

                    if is_lr1 {
                        let lookahead = if production.len() > 1 {
                            g.calculate_first_for_production(&production[1..])
                        } else {
                            extend[&s_idx].as_ref().unwrap().clone()
                        };
                        extend
                            .get_mut(&nt.index)
                            .unwrap()
                            .as_mut()
                            .unwrap()
                            .extend(lookahead);
                    }
                }
            }
        }

        for (nt_idx, lookahead) in extend {
            let nt = g.symbols[nt_idx].non_terminal().unwrap();

            let lookahead: Option<Vec<String>> = lookahead.and_then(|lookahead| {
                let mut lookahead = lookahead
                    .iter()
                    .map(|&i| g.get_symbol_name(i).to_string())
                    .collect::<Vec<_>>();
                lookahead.sort();
                Some(lookahead)
            });

            for production in &nt.productions {
                self.extend.insert(DotProduction::new(
                    nt.name.clone(),
                    g.production_to_vec_str(production)
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    lookahead.clone(),
                ));
            }
        }
    }
}

impl LRItem {
    fn new(core: HashSet<DotProduction>) -> Self {
        Self {
            core,
            extend: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    fn to_plaintext(&self) -> String {
        let core = self
            .core
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

        format!("{}{}{}", core, extend, edges)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LRFSMType {
    LR0,
    LR1,
}

#[derive(Debug)]
pub struct LRFSM {
    pub t: LRFSMType,
    terminals: Vec<String>,
    non_terminals: Vec<String>,

    pub states: Vec<LRItem>,
    pub start: usize,
    pub end: usize,
    pub follow: Option<HashMap<String, Vec<String>>>,
}

impl LRFSM {
    pub fn to_plaintext(&self) -> String {
        let states = self
            .states
            .iter()
            .enumerate()
            .map(|(i, s)| format!("I{}\n{}", i, s.to_plaintext()))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!("{}\n\nstart: {}", states, self.start)
    }
}

impl Grammar {
    pub fn to_lr_fsm(&mut self, t: LRFSMType) -> Result<LRFSM, String> {
        if self.start_symbol.is_none() {
            return Err("start symbol is not set".to_string());
        }

        if t == LRFSMType::LR0 && !self.is_nullable_first_follow_valid() {
            self.calculate_nullable_first_follow();
        }

        let real_start = self.get_symbol_name(self.start_symbol.unwrap()).to_string();
        let dummy_start = self.get_symbol_prime_name(real_start.clone());
        let mut start_state = LRItem::new(HashSet::from([DotProduction::new(
            dummy_start.clone(),
            vec![real_start, END_MARK.to_string()],
            if t == LRFSMType::LR1 {
                Some(Vec::new())
            } else {
                None
            },
        )]));
        start_state.calculate_extend(self);
        let mut states = vec![start_state];
        let mut q: VecDeque<usize> = VecDeque::new();
        q.push_back(0);

        let mut end: usize = 0;

        while let Some(u) = q.pop_front() {
            let mut edges: HashMap<String, LRItem> = HashMap::new();

            let productions = states[u].core.iter().chain(states[u].extend.iter());
            for production in productions {
                if production.position < production.production.len() {
                    let e = production.production[production.position].clone();
                    let item = edges.entry(e).or_insert(LRItem::new(HashSet::new()));
                    item.core.insert(production.generate_next());
                }
            }

            for (_, item) in edges.iter_mut() {
                item.calculate_extend(self);
            }

            for (e, v) in edges {
                if e == END_MARK {
                    end = u;
                    continue;
                }

                let mut entry_or_insert = |s: LRItem| {
                    for (i, state) in states.iter().enumerate() {
                        if state.core == s.core && state.extend == s.extend {
                            return i;
                        }
                    }
                    states.push(s);
                    q.push_back(states.len() - 1);
                    states.len() - 1
                };

                let v_idx = entry_or_insert(v);
                states[u].edges.insert(e.clone(), v_idx);
            }
        }

        Ok(LRFSM {
            t,
            terminals: self.terminal_iter().cloned().collect(),
            non_terminals: self.non_terminal_iter().map(|nt| nt.name.clone()).collect(),
            states,
            start: 0,
            end,
            follow: if t == LRFSMType::LR0 {
                let mut r: HashMap<String, Vec<String>> = HashMap::new();
                r.insert(dummy_start, vec![]);
                for nt in self.non_terminal_iter() {
                    r.insert(
                        nt.name.clone(),
                        nt.follow
                            .iter()
                            .map(|i| self.get_symbol_name(*i).to_string())
                            .collect(),
                    );
                }
                Some(r)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum LRParsingTableAction {
    Shift(usize),
    Reduce((String, Vec<String>)),
    Accept,
}

pub struct LRParsingTable {
    pub t: LRFSMType,
    pub terminals: Vec<String>,
    pub non_terminals: Vec<String>,
    pub action: Vec<Vec<Vec<LRParsingTableAction>>>,
    pub goto: Vec<Vec<Vec<usize>>>,
}

impl LRFSM {
    pub fn to_parsing_table(&self) -> LRParsingTable {
        let mut terminal_idx_map: HashMap<&str, usize> = HashMap::new();
        for (i, s) in self.terminals.iter().enumerate() {
            terminal_idx_map.insert(s, i);
        }

        let mut non_terminal_idx_map: HashMap<&str, usize> = HashMap::new();
        for (i, s) in self.non_terminals.iter().enumerate() {
            non_terminal_idx_map.insert(s, i);
        }

        let mut table = LRParsingTable {
            t: self.t,
            terminals: self.terminals.clone(),
            non_terminals: self.non_terminals.clone(),
            action: Vec::new(),
            goto: Vec::new(),
        };

        for state in &self.states {
            let mut action_row: Vec<Vec<LRParsingTableAction>> =
                vec![Vec::new(); self.terminals.len()];
            let mut goto_row: Vec<Vec<usize>> = vec![Vec::new(); self.non_terminals.len()];
            for prodcution in state.core.iter().chain(state.extend.iter()) {
                if prodcution.production.len() == prodcution.position {
                    let lookahead = if let Some(lookahead) = &prodcution.lookahead {
                        lookahead
                    } else {
                        &self.follow.as_ref().unwrap()[&prodcution.left]
                    };
                    for terminal in lookahead {
                        action_row[terminal_idx_map[terminal.as_str()]].push(
                            LRParsingTableAction::Reduce((
                                prodcution.left.clone(),
                                prodcution.production.clone(),
                            )),
                        );
                    }
                }
            }
            for (e, v) in &state.edges {
                if let Some(idx) = terminal_idx_map.get(e.as_str()) {
                    action_row[*idx].push(LRParsingTableAction::Shift(*v));
                }
                if let Some(idx) = non_terminal_idx_map.get(e.as_str()) {
                    goto_row[*idx].push(*v);
                }
            }
            table.action.push(action_row);
            table.goto.push(goto_row);
        }

        table.action[self.end][terminal_idx_map[END_MARK]].push(LRParsingTableAction::Accept);

        table
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
}
