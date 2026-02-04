use rustc_hash::FxHashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct Problem {
    number_variables: usize,
    clauses: Vec<Vec<isize>>,
    primal_graph_edges: FxHashSet<(usize, usize)>,
}

impl Problem {

    pub fn from_file(input: &PathBuf) -> Self {
        let mut clauses: Vec<Vec<isize>> = vec![];
        let mut primal_graph_edges = FxHashSet::default();
        let mut number_variables = 0;
        let file = File::open(input).unwrap();
        let reader = BufReader::new(file);
        for l in reader.lines() {
            match l {
                Err(e) => panic!("Problem while reading file: {}", e),
                Ok(line) => {
                    if line.starts_with("p cnf") {
                        let split = line.trim_end().split_whitespace().collect::<Vec<&str>>();
                        number_variables = split[2].parse::<usize>().unwrap();
                    }
                    if !line.starts_with('c') && !line.starts_with('p') {
                        // Note: the space before the 0 is important so that clauses like "1 -10 0" are correctly splitted
                        for clause in line.trim_end().split(" 0").filter(|cl| !cl.is_empty()) {
                            let cls = clause.split_whitespace().map(|x| x.parse::<isize>().unwrap()).collect::<Vec<isize>>();
                            for x in 0..cls.len() {
                                for y in (x+1)..cls.len() {
                                    let v1 = cls[x].abs() as usize;
                                    let v2 = cls[y].abs() as usize;
                                    let minimum = v1.min(v2);
                                    let maximum = v1.max(v2);
                                    primal_graph_edges.insert((minimum, maximum));
                                }
                            }
                            clauses.push(cls);
                        }
                    }
                }
            }
        }
        Self {
            number_variables,
            clauses,
            primal_graph_edges,
        }
    }

    pub fn primal_graph_to_file(&self, output: &str) {
        let mut file = File::create(output).unwrap();
        writeln!(file, "p tw {} {}", self.number_variables, self.primal_graph_edges.len()).unwrap();
        for (x, y) in self.primal_graph_edges.iter() {
            writeln!(file, "{} {}", x, y).unwrap();
        }
    }
}
