use dcfrs::{error::*, lexer::tokens, span::SpanSource};
use std::{fs, io::Read};

use crate::{App, ExitStatus};

pub struct Lexer;

impl App for Lexer {
    fn run(
        stdout: &mut dyn std::io::Write,
        stderr: &mut dyn std::io::Write,
        input_file: String,
    ) -> ExitStatus {
        /// shadows std's `println` macro
        macro_rules! println {
            ($($arg:tt)*) => ({
                writeln!(stdout, $($arg)*).unwrap();
            });
        }

        let mut buf = String::new();
        fs::File::open(&input_file)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        let code = SpanSource::new(&buf);
        let err_count = tokens(code.source())
            .filter_map(|tok| {
                use dcfrs::lexer::Token::*;
                match tok.get() {
                    Ok(Eof) => None,
                    Ok(
                        Semicolon | And | Or | EqualEqual | NotEqual | Greater | GreaterEqual
                        | Less | LessEqual | Minus | Plus | Assign | SubAssign | AddAssign | Colon
                        | Question | Comma | Void | For | Continue | Break | While | Int | Bool
                        | If | Else | Return | Len | Star | Slash | Percent | Not | LeftParen
                        | RightParen | CurlyLeft | CurlyRight | SquareLeft | SquareRight
                        | Increment | Decrement | Import,
                    ) => {
                        println!("{} {}", tok.line(), tok.fragment());
                        None
                    }
                    Ok(Identifier) => {
                        println!(
                            "{} IDENTIFIER {}",
                            tok.line(),
                            tok.fragment()
                        );
                        None
                    }
                    Ok(DecimalLiteral | HexLiteral) => {
                        println!("{} INTLITERAL {}", tok.line(), tok.fragment());
                        None
                    }
                    Ok(StringLiteral) => {
                        println!("{} STRINGLITERAL {}", tok.line(), tok.fragment());
                        None
                    }
                    Ok(CharLiteral(_)) => {
                        println!("{} CHARLITERAL {}", tok.line(), tok.fragment());
                        None
                    }
                    Ok(True | False) => {
                        println!("{} BOOLEANLITERAL {}", tok.line(), tok.fragment());
                        None
                    }
                    // errors are logged in the lexer module anyways
                    Err(e) => {
                        write!(stderr, "{}", &e.to_error(&input_file)).unwrap();
                        Some(())
                    }
                    _ => unreachable!(),
                }
            })
            .count();
        if err_count == 0 {
            ExitStatus::Success
        } else {
            ExitStatus::Fail
        }
    }
}

#[cfg(test)]
mod test;
