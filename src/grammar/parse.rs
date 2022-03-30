use crate::Grammar;

impl Grammar {
    pub fn parse(grammar: &str) -> Result<Self, String> {
        let mut g = Self::new();

        let mut raw_productions: Vec<(usize, &str)> = Vec::new();

        let mut previous_left: Option<usize> = None;
        for (i, line) in grammar.lines().enumerate() {
            if line.chars().all(|c| c.is_whitespace()) {
                continue;
            }
            let parts: Vec<&str> = line.split("->").collect();
            if parts.len() > 2 {
                return Err(format!("Line {}: too many \"->\"", i + 1));
            }
            let (left, rights): (usize, &str) = if parts.len() == 2 {
                let left_str = parts[0].trim();
                if left_str.split_whitespace().count() != 1 {
                    return Err(format!("Line {}: left side contains whitespace", i + 1));
                } else if left_str.is_empty() {
                    return Err(format!("Line {}: empty left side", i + 1));
                } else {
                    (
                        if let Some(idx) = g.get_symbol_index(left_str) {
                            idx
                        } else {
                            g.add_non_terminal(left_str)
                        },
                        parts[1].trim(),
                    )
                }
            } else {
                if let Some(idx) = previous_left {
                    (idx, parts[0].trim()[1..].trim())
                } else {
                    return Err(format!("Line {}: cannot find left side", i + 1));
                }
            };

            previous_left = Some(left);

            raw_productions.push((left, rights));
        }

        for (left, rights) in raw_productions {
            for right in rights.split("|") {
                let symbols = right
                    .split_whitespace()
                    .map(|s| {
                        if let Some(idx) = g.get_symbol_index(s) {
                            idx
                        } else {
                            g.add_terminal(s.to_string())
                        }
                    })
                    .collect();
                g.add_production(left, symbols);
            }
        }

        let start_symbol: Option<usize> = if let Some(nt) = g.non_terminal_iter().next() {
            Some(g.symbol_table[&nt.name])
        } else {
            None
        };
        g.start_symbol = start_symbol;

        Ok(g)
    }
}
