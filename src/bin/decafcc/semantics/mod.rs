use super::App;
use dcfrs::{ast::*, error::*, lexer::*, span::*};

use std::fs::read_to_string;

pub struct Semantics;

impl App for Semantics {
    fn run(
        _stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
        input_file: String,
    ) -> crate::ExitStatus {
        let text = read_to_string(&input_file).unwrap();
        let code = SpanSource::new(&text);
        let mut parser =
            dcfrs::parser::Parser::new(tokens(code.source()).map(|s| s.map(|t| t.unwrap())), |e| {
                write!(stderr, "{}", e.to_error(&input_file)).unwrap();
            });
        let proot = parser.doc_elems().collect();
        let hirtree = Root::from_proot(proot);
        match hirtree {
            Ok(_) => {
                println!("{hirtree:#?}");
                crate::ExitStatus::Success
            }
            Err(errs) => {
                errs.into_iter()
                    .try_for_each(|err| write!(stderr, "{}", err.to_error(&input_file)))
                    .unwrap();
                crate::ExitStatus::Fail
            }
        }
    }
}

#[cfg(test)]
mod test;
