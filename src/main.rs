use std::fs::{File, read_to_string};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Action {
    Shift(usize),
    Reduce(usize),
    Accept,
    Goto(usize),
    Error,
}

struct Parser {
    stack: Vec<usize>,
    state: usize,
    table: (Vec<char>, Vec<Vec<Action>>),
    reducer: Vec<(char, String)>,
}

impl Parser {
    fn new(table_path: &str, reducer_path: &str) -> Self {
        Parser {
            stack: vec![],
            state: 0,
            table: from_table(table_path),
            reducer: from_reducer(reducer_path),
        }
    }

    fn action(&self, input: char) -> Action {
        let idx = self.state;
        let (symbols, table) = self.table.clone();
        let actions = table[idx].clone();
        let (idx, _) = symbols
            .iter()
            .enumerate()
            .find(|(_i, s)| **s == input)
            .unwrap();
        actions[idx]
    }

    fn parse(&mut self, mut input: String) -> Vec<usize> {
        println!("case: {}", input.clone());
        self.stack.clear();
        self.stack.push(0);
        self.state = 0;
        let mut chars: Vec<char> = input.chars().collect();
        input.clear();
        let mut output = vec![];

        while let Some(head) = chars.first().cloned() {
            let action = self.action(head);
            println!(
                "head: {} || state: {} || action{:?} || stack : {:?}",
                head,
                self.state,
                action,
                self.stack.clone()
            );
            match action {
                Action::Shift(id) => {
                    self.stack.push(id);
                    chars.remove(0);
                    self.state = id;
                }
                Action::Reduce(id) => {
                    output.push(id);
                    let (num_pop, result) = self.apply_reducer(id);
                    for _ in 0..num_pop {
                        let _ = self.stack.pop();
                    }
                    self.state = *self.stack.last().unwrap();
                    if let Action::Goto(next) = self.action(result) {
                        self.stack.push(next);
                    }
                    self.state = *self.stack.last().unwrap();
                }
                Action::Accept => {
                    break;
                }
                _ => return vec![0],
            };
        }
        output
    }

    fn apply_reducer(&self, id: usize) -> (usize, char) {
        let (after, before) = &self.reducer[id - 1];
        (before.len(), *after)
    }
}

fn from_table(path: &str) -> (Vec<char>, Vec<Vec<Action>>) {
    let file = File::open(path).unwrap();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = reader.headers();
    let headers: Vec<char> = headers
        .unwrap()
        .iter()
        .map(|field| field.trim().chars().next().unwrap())
        .collect();

    let rows = reader.records();
    let data: Vec<Vec<Action>> = rows
        .map(|row| {
            row.unwrap()
                .iter()
                .map(|field| {
                    let mut f = field.trim().to_string();
                    let (prefix, id): (char, usize) =
                        (f.remove(0), f.parse::<usize>().unwrap_or(0));
                    match (prefix, id) {
                        ('S', id) => Action::Shift(id),
                        ('R', id) => Action::Reduce(id),
                        ('A', _) => Action::Accept,
                        ('G', id) => Action::Goto(id),
                        (_, _) => Action::Error,
                    }
                })
                .collect()
        })
        .collect();

    (headers, data)
}

fn from_reducer(path: &str) -> Vec<(char, String)> {
    let mut reducer = Vec::new();
    let content = read_to_string(path).unwrap();
    for line in content.lines() {
        let condition: Vec<&str> = line.trim().split("->").collect();
        assert!(condition.len() == 2);
        let (before, after) = (
            condition[0].chars().next().unwrap(),
            condition[1]
                .trim()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
        );
        reducer.push((before, after));
    }

    reducer
}

fn main() {
    let mut parser = Parser::new("./group.csv", "./reducer");
    dbg!(parser.parse(String::from("1+1$")));
    let mut parser = Parser::new("./paren.csv", "./paren_reducer");
    dbg!(parser.parse(String::from("<><<>><>$")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_table() {
        let (elems, actions) = from_table("./group.csv");
        assert_eq!(elems, vec!['*', '+', '0', '1', '$', 'E', 'B']);
        assert_eq!(actions[0][0], Action::Error);

        let (elems, actions) = from_table("./test.csv");
        assert_eq!(elems, vec!['1']);
        assert_eq!(actions[0][0], Action::Shift(10));
    }

    #[test]
    fn test_from_reducer() {
        let reducer = from_reducer("./test_reducer");
        assert_eq!(reducer[0], ('A', String::from("A+A")));
    }

    #[test]
    fn test_parse() {
        let mut parser = Parser::new("./group.csv", "./reducer");
        assert_eq!(parser.parse(String::from("1+1$")), vec![5, 3, 5, 2]);
        assert_eq!(parser.parse(String::from("1*1$")), vec![5, 3, 5, 1]);
        assert_eq!(parser.parse(String::from("1*0+1$")), vec![5, 3, 4, 1, 5, 2]);
        assert_eq!(parser.parse(String::from("1+1*0$")), vec![5, 3, 5, 2, 4, 1]);
    }

    #[test]
    fn test_paren_parse() {
        let mut parser = Parser::new("./paren.csv", "./paren_reducer");
        assert_eq!(parser.parse(String::from("<>$")), vec![1]);
        assert_eq!(parser.parse(String::from("<<>><>$")), vec![1, 2, 1, 3]);
        assert_eq!(
            parser.parse(String::from("<><><><><><>$")),
            vec![1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3]
        );
    }
}
