use std::io::{BufWriter, StdoutLock, Write};

use crate::*;

pub fn run_line(line: &str, tl: &mut TodoList, writer: &mut BufWriter<StdoutLock>) {
    if let Ok((_, q)) = parser::query(line) {
        match run_query(q, tl) {
            Ok(r) => {
                writeln!(writer, "{}", r).unwrap();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn run_query(q: Query, tl: &mut TodoList) -> Result<QueryResult, QueryError> {
    match q {
        Query::Add(desc, tags) => {
            Ok(QueryResult::Added(tl.push(desc, tags)))
        }
        Query::Done(idx) => {
            match tl.done_with_index(idx) {
                None => Err(QueryError("Index does not exist".to_string())),
                Some(idx) => Ok(QueryResult::Done)
            }
        }
        Query::Search(params) => {
            Ok(QueryResult::Found(tl.search(params)))
        }
    }
}
