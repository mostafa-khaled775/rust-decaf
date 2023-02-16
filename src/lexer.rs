use crate::{log::format_error, span::*};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    EmptyHexLiteral,
    InvalidChar(u8),
    InvalidEscape(u8),
    UnexpectedChar(u8),
    EmptyChar,
    NonAsciiChars,
    StringLiteral,
    UnterminatedString,
    UnterminatedComment,
    UnterminatedChar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    // keywords
    Import,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Return,
    Int,
    Bool,
    True,
    False,
    Void,
    Len,
    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    And,
    Or,
    Not,
    Question,
    Colon,
    Assign,
    AddAssign,
    SubAssign,
    Increment,
    Decrement,
    // delimiters
    Semicolon,
    Comma,
    LeftParen,
    RightParen,
    SquareLeft,
    SquareRight,
    CurlyLeft,
    CurlyRight,
    // literals
    Identifier,
    DecimalLiteral,
    HexLiteral,
    StringLiteral,
    CharLiteral(u8),

    Space,
    LineComment,
    BlockComment,

    // end of file
    Eof,
}

pub type Result = std::result::Result<Token, Error>;

fn get_string_errors(span: Span) -> impl Iterator<Item = Spanned<Error>> + '_ {
    let mut escape_next = true;
    let mut error_checker = move |s: Span| {
        let c = s[0];
        if escape_next {
            escape_next = false;
            if !is_escaped_char(c) {
                Some(Error::InvalidEscape(c))
            } else {
                None
            }
        } else if c == b'\\' {
            escape_next = true;
            None
        } else if !is_dcf_char(c) {
            Some(Error::InvalidChar(c))
        } else {
            None
        }
    };
    let terminated = if span.ends_with(b"\\\"") || !span.ends_with(b"\"") {
        Some(span.into_spanned(Error::UnterminatedString))
    } else {
        None
    };
    span.spans::<1>()
        .take(span.len() - 1)
        .filter_map(move |s| error_checker(s).map(|e| s.into_spanned(e)))
        .chain(terminated)
}

fn symbol(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    {
        if span.len() > 1 {
            let (ch, rem) = span.split_at(2);
            match &ch[..] {
                b"<=" => Some((ch.into_spanned(Ok(Token::LessEqual)), rem)),
                b">=" => Some((ch.into_spanned(Ok(Token::GreaterEqual)), rem)),
                b"==" => Some((ch.into_spanned(Ok(Token::EqualEqual)), rem)),
                b"!=" => Some((ch.into_spanned(Ok(Token::NotEqual)), rem)),
                b"+=" => Some((ch.into_spanned(Ok(Token::AddAssign)), rem)),
                b"-=" => Some((ch.into_spanned(Ok(Token::SubAssign)), rem)),
                b"&&" => Some((ch.into_spanned(Ok(Token::And)), rem)),
                b"||" => Some((ch.into_spanned(Ok(Token::Or)), rem)),
                b"--" => Some((ch.into_spanned(Ok(Token::Decrement)), rem)),
                b"++" => Some((ch.into_spanned(Ok(Token::Increment)), rem)),
                _ => None,
            }
        } else {
            None
        }
    }
    .or_else(|| {
        let (ch, rem) = span.split_at(1);
        match ch[0] {
            b'+' => Some((ch.into_spanned(Ok(Token::Plus)), rem)),
            b'-' => Some((ch.into_spanned(Ok(Token::Minus)), rem)),
            b'*' => Some((ch.into_spanned(Ok(Token::Star)), rem)),
            b'/' => Some((ch.into_spanned(Ok(Token::Slash)), rem)),
            b'%' => Some((ch.into_spanned(Ok(Token::Percent)), rem)),
            b'!' => Some((ch.into_spanned(Ok(Token::Not)), rem)),
            b';' => Some((ch.into_spanned(Ok(Token::Semicolon)), rem)),
            b'<' => Some((ch.into_spanned(Ok(Token::Less)), rem)),
            b'>' => Some((ch.into_spanned(Ok(Token::Greater)), rem)),
            b'=' => Some((ch.into_spanned(Ok(Token::Assign)), rem)),
            b'{' => Some((ch.into_spanned(Ok(Token::CurlyLeft)), rem)),
            b'}' => Some((ch.into_spanned(Ok(Token::CurlyRight)), rem)),
            b'[' => Some((ch.into_spanned(Ok(Token::SquareLeft)), rem)),
            b']' => Some((ch.into_spanned(Ok(Token::SquareRight)), rem)),
            b',' => Some((ch.into_spanned(Ok(Token::Comma)), rem)),
            b'(' => Some((ch.into_spanned(Ok(Token::LeftParen)), rem)),
            b')' => Some((ch.into_spanned(Ok(Token::RightParen)), rem)),
            b'?' => Some((ch.into_spanned(Ok(Token::Question)), rem)),
            b':' => Some((ch.into_spanned(Ok(Token::Colon)), rem)),
            c if !c.is_ascii_alphanumeric() => {
                Some((ch.into_spanned(Err(Error::UnexpectedChar(c))), rem))
            }
            _ => None,
        }
    })
}

fn identifier<'a>(span: Span<'a>) -> Option<(Spanned<Result>, Span<'a>)> {
    assert!(!span.is_empty());
    if !span[0].is_ascii_alphabetic() && span[0] != b'_' {
        None
    } else {
        let keyword = |(span, rem): (Span<'a>, _)| match span.source() {
            b"import" => (span.into_spanned(Ok(Token::Import)), rem),
            b"void" => (span.into_spanned(Ok(Token::Void)), rem),
            b"int" => (span.into_spanned(Ok(Token::Int)), rem),
            b"bool" => (span.into_spanned(Ok(Token::Bool)), rem),
            b"if" => (span.into_spanned(Ok(Token::If)), rem),
            b"else" => (span.into_spanned(Ok(Token::Else)), rem),
            b"for" => (span.into_spanned(Ok(Token::For)), rem),
            b"while" => (span.into_spanned(Ok(Token::While)), rem),
            b"break" => (span.into_spanned(Ok(Token::Break)), rem),
            b"continue" => (span.into_spanned(Ok(Token::Continue)), rem),
            b"return" => (span.into_spanned(Ok(Token::Return)), rem),
            b"len" => (span.into_spanned(Ok(Token::Len)), rem),
            b"true" => (span.into_spanned(Ok(Token::True)), rem),
            b"false" => (span.into_spanned(Ok(Token::False)), rem),
            _ => (span.into_spanned(Ok(Token::Identifier)), rem),
        };
        Some(
            span.split_once(|&c| !c.is_ascii_alphanumeric() && c != b'_')
                .map(keyword)
                .unwrap_or_else(|| keyword((span, span.split_at(span.len()).1))),
        )
    }
}

fn skip_spaces(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    span[0].is_ascii_whitespace().then(|| {
        span.split_once(|&c| !c.is_ascii_whitespace())
            .map(|(spaces, rem)| (spaces.into_spanned(Ok(Token::Space)), rem))
            .unwrap_or((
                span.into_spanned(Ok(Token::Space)),
                span.split_at(span.len()).1,
            ))
    })
}

fn skip_line_comment(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    span.starts_with(b"//").then(|| {
        let (cmt, rem) = span
            .split_once(|&c| c == b'\n')
            .unwrap_or_else(|| (span, span.split_at(span.len()).1));
        (cmt.into_spanned(Ok(Token::LineComment)), rem)
    })
}

fn skip_block_comment(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    if span.starts_with(b"/*") {
        let split = span
            .split_at(2)
            .1
            .find(b"*/")
            .map(|i| span.split_at(i + 4).1);
        if let Some(split) = split {
            Some((span.into_spanned(Ok(Token::BlockComment)), split))
        } else {
            Some((
                span.into_spanned(Err(Error::UnterminatedComment)),
                span.split_at(span.len()).1,
            ))
        }
    } else {
        None
    }
}

fn int_literal(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    if span[0].is_ascii_digit() {
        if span.starts_with(b"0x") {
            let (lit, _rem) = span
                .split_at(2)
                .1
                .split_once(|c| !c.is_ascii_hexdigit())
                .unwrap_or(span.split_at(span.len() - 2));
            if lit.is_empty() {
                let (err, rem) = span.split_at(2);
                Some((err.into_spanned(Err(Error::EmptyHexLiteral)), rem))
            } else {
                let (lit, rem) = span.split_at(lit.len() + 2);
                Some((lit.into_spanned(Ok(Token::HexLiteral)), rem))
            }
        } else {
            let (lit, rem) = span
                .split_once(|c| !c.is_ascii_digit())
                .unwrap_or(span.split_at(span.len()));
            Some((lit.into_spanned(Ok(Token::DecimalLiteral)), rem))
        }
    } else {
        None
    }
}

const fn is_escaped_char(c: u8) -> bool {
    matches!(c, b'n' | b't' | b'\\' | b'\'' | b'"')
}

fn escaped_char(span: Span) -> Spanned<Result> {
    assert!(span.len() == 4);
    assert!(span.starts_with(b"'\\"));
    if span[3] != b'\'' {
        span.into_spanned(Err(Error::UnterminatedChar))
    } else {
        let c = span[2];
        match c {
            b'n' => span.into_spanned(Ok(Token::CharLiteral(b'\n'))),
            b't' => span.into_spanned(Ok(Token::CharLiteral(b'\t'))),
            b'\\' => span.into_spanned(Ok(Token::CharLiteral(b'\\'))),
            b'\'' => span.into_spanned(Ok(Token::CharLiteral(b'\''))),
            b'"' => span.into_spanned(Ok(Token::CharLiteral(b'"'))),
            c => span.into_spanned(Err(Error::InvalidEscape(c))),
        }
    }
}

const fn is_dcf_char(c: u8) -> bool {
    matches!(c, 32..=33 | 35..=38 | 40..=91 | 93..=126)
}

fn dcf_char(span: Span) -> Spanned<Result> {
    assert!(span.len() == 3);
    let c = span[1];
    match c {
        c if is_dcf_char(c) => span.into_spanned(Ok(Token::CharLiteral(c))),
        _ => span.into_spanned(Err(Error::InvalidChar(c))),
    }
}

fn char_literal(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    if span.len() < 3 || !span.starts_with(b"'") {
        None
    } else if span[1] == b'\\' {
        // escaped char
        if span.len() < 4 {
            Some((
                span.into_spanned(Err(Error::UnterminatedChar)),
                span.split_at(span.len()).1,
            ))
        } else {
            let (lit, rem) = span.split_at(4);
            Some((escaped_char(lit), rem))
        }
    } else if span[1] == b'\'' {
        let (lit, rem) = span.split_at(2);
        Some((lit.into_spanned(Err(Error::EmptyChar)), rem))
    } else if span[2] != b'\'' {
        let (lit, rem) = span.split_at(2);
        Some((lit.into_spanned(Err(Error::UnterminatedChar)), rem))
    } else {
        let (lit, rem) = span.split_at(3);
        Some((dcf_char(lit), rem))
    }
}

fn string_literal(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    if span.starts_with(b"\"") {
        // take the string literal even if it contains errors
        // set `last_char` to '\\' to skip the first quote
        let mut last_char = b'\\';
        let mut break_next = false;
        let (lit, rem) = span.take_while(|c| {
            if break_next {
                false
            } else if *c == b'"' && last_char != b'\\' {
                break_next = true;
                true
            } else {
                last_char = *c;
                true
            }
        });

        // collect errors in the string literal
        if get_string_errors(lit).next().is_some() {
            Some((lit.into_spanned(Err(Error::StringLiteral)), rem))
        } else {
            Some((lit.into_spanned(Ok(Token::StringLiteral)), rem))
        }
    } else {
        None
    }
}

fn is_ascii(c: &u8) -> bool {
    matches!(c, 32..=126 | b'\t' | b'\n' | b'\r')
}

/// collect non-ascii chars
fn non_ascii_graphic_chars(span: Span) -> Option<(Spanned<Result>, Span)> {
    assert!(!span.is_empty());
    // this panics if the span is empty
    let (bad_chars, rem) = span.split_once(is_ascii).unwrap();
    if bad_chars.is_empty() {
        None
    } else {
        Some((bad_chars.into_spanned(Err(Error::NonAsciiChars)), rem))
    }
}

fn token(span: Span) -> Option<(Spanned<Result>, Span)> {
    if span.is_empty() {
        None
    } else {
        // the non_ascii_graphic_chars has to come before spaces
        // skip_spaces skips some of illegal chars
        non_ascii_graphic_chars(span)
            .or_else(|| skip_spaces(span))
            .or_else(|| skip_line_comment(span))
            .or_else(|| skip_block_comment(span))
            .or_else(|| identifier(span))
            .or_else(|| int_literal(span))
            .or_else(|| char_literal(span))
            .or_else(|| string_literal(span))
            .or_else(|| symbol(span))
    }
}

pub fn tokens<L: FnMut(Spanned<Error>)>(
    mut text: Span,
    mut log: L,
) -> impl Iterator<Item = Spanned<Result>> {
    use std::iter;
    iter::from_fn(move || {
        if text.is_empty() {
            None
        } else {
            let (tok, rem) = token(text)?;
            text = rem;
            Some(tok)
        }
    })
    .filter(|t| {
        !matches!(
            t.get(),
            Ok(Token::Space) | Ok(Token::LineComment) | Ok(Token::BlockComment)
        )
    })
    .inspect(move |tok| {
        if let Err(err) = tok.get() {
            log(tok.span().into_spanned(*err))
        }
    })
    .chain(iter::once(
        text.split_at(text.len()).1.into_spanned(Ok(Token::Eof)),
    ))
}

/// creats a log function to the given stream.
/// example:
/// TODO: fix this
/// ```
/// // use dcfrs::lexer::{log_err, tokens};
///
/// // let mut mock_stderr = vec![];
/// // let mut log = log_err(&mut mock_stderr, "test.dcf");
///
/// // tokens(b"'\\a'", log);
/// // assert_eq!(mock_stderr, b"test.dcf:1:2: invalid escape sequence: \\a");
/// ```
pub fn log_err<'a, T: AsRef<str> + 'a>(
    mut write: impl FnMut(String) + 'a,
    input_file: T,
) -> impl FnMut(Spanned<Error>) + 'a {
    move |err| {
        let string = |slice: &[u8]| String::from_utf8(slice.to_vec()).unwrap();
        let mut loge =
            |pos: (usize, usize), msg: &str| write(format_error(input_file.as_ref(), pos, msg));

        fn print_u8(c: u8) -> String {
            if c.is_ascii_digit() {
                format!("{}", c as char)
            } else {
                format!("\\x{:02x}", c)
            }
        }
        let mut handle_single_error = |err: Spanned<Error>| match err.get() {
            Error::EmptyHexLiteral => {
                loge(
                    err.position(),
                    &format!("invalid hex literal: {}", string(err.fragment())),
                );
            }
            Error::EmptyChar => {
                loge(err.position(), "empty char literal");
            }
            Error::InvalidEscape(c) => {
                loge(
                    err.position(),
                    &format!("invalid escape sequence: \\{}", print_u8(*c)),
                );
            }
            Error::InvalidChar(c) => {
                loge(
                    err.position(),
                    &format!("invalid character literal: {}", print_u8(*c)),
                );
            }
            Error::UnexpectedChar(c) => {
                loge(
                    err.position(),
                    &format!("unexpected character: {}", print_u8(*c)),
                );
            }
            Error::UnterminatedString => {
                loge(err.position(), "unterminated string literal");
            }
            Error::UnterminatedChar => {
                loge(err.position(), "unterminated char literal");
            }
            Error::UnterminatedComment => {
                loge(err.position(), "unterminated block comment");
            }
            Error::NonAsciiChars => {
                loge(
                    err.position(),
                    &format!("non-ascii characters: {}", string(err.fragment())),
                );
            }
            _ => unreachable!(),
        };

        match err.get() {
            Error::StringLiteral => {
                get_string_errors(err.span()).for_each(handle_single_error);
            }
            _ => handle_single_error(err),
        };
    }
}

#[cfg(test)]
mod test {
    use super::Token::*;
    use super::*;

    // fn parsed<'a>(opt: Option<(Spanned<'a, Result<'a>>, Span<'a>)>) -> Spanned<'a, Result<'a>> {
    //     opt.unwrap().0
    // }

    fn rem<'a>(opt: Option<(Spanned<'a, Result>, Span<'a>)>) -> Span<'a> {
        opt.unwrap().1
    }

    macro_rules! span {
        ($span:ident, $text:expr) => {
            let span_source = SpanSource::new($text);
            let $span = span_source.source();
        };
    }

    #[test]
    fn identifier() {
        use super::*;
        let text = b"abc";
        span!(span, text);
        let (s1, s2) = identifier(span).unwrap();
        assert_eq!(s1.get().unwrap(), Identifier);
        assert_eq!(s1.fragment(), b"abc");
        assert_eq!(s2.source(), b"");

        let text = b"_abc";
        span!(span, text);
        let (s1, s2) = identifier(span).unwrap();
        assert_eq!(s1.get().unwrap(), Identifier);
        assert_eq!(s1.fragment(), b"_abc");
        assert_eq!(s2.source(), b"");

        let text = b"abc def";
        span!(span, text);
        let (s1, s2) = identifier(span).unwrap();
        assert_eq!(s1.get().unwrap(), Identifier);
        assert_eq!(s1.fragment(), b"abc");
        assert_eq!(s2.source(), b" def");

        let text = b"123abc";
        span!(span, text);
        assert!(identifier(span).is_none());
    }

    #[test]
    fn char_literal() {
        use super::*;
        let text = b"'a'";
        span!(span, text);
        let (s1, s2) = char_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), CharLiteral(b'a'));
        assert_eq!(s1.fragment(), b"'a'");
        assert_eq!(s2.source(), b"");

        let text = b"'\\'";
        span!(span, text);
        let (s1, s2) = char_literal(span).unwrap();
        assert_eq!(s1.get().unwrap_err(), Error::UnterminatedChar);
        assert_eq!(s1.fragment(), b"'\\'");
        assert_eq!(s2.source(), b"");

        let text = b"'	'";
        span!(span, text);
        let (s1, s2) = char_literal(span).unwrap();
        assert_eq!(s1.get().unwrap_err(), Error::InvalidChar(b'\t'));
        assert_eq!(s1.fragment(), b"'	'");
        assert_eq!(s2.source(), b"");

        let text = b"'\\t'";
        span!(span, text);
        let (s1, s2) = char_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), CharLiteral(b'\t'));
        assert_eq!(s1.fragment(), b"'\\t'");
        assert_eq!(s2.source(), b"");
    }

    #[test]
    fn string_literal() {
        use super::*;
        let text = b"\"abc\"";
        span!(span, text);
        let (s1, s2) = string_literal(span).unwrap();
        println!("{:?}", get_string_errors(s1.span()).collect::<Vec<_>>());
        assert_eq!(s1.get().unwrap(), StringLiteral);
        assert_eq!(s1.fragment(), b"\"abc\"");
        assert_eq!(s2.source(), b"");

        let text = br#""\"abcdef\"""#;
        span!(span, text);
        let (s1, s2) = string_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), StringLiteral);
        assert_eq!(s1.fragment(), br#""\"abcdef\"""#);
        assert_eq!(s2.source(), b"");

        let text = b"\"abc alot of text that does not\\\" terminate with a quote";
        span!(span, text);
        let (s1, s2) = string_literal(span).unwrap();
        s1.get().unwrap_err();
        assert_eq!(
            s1.fragment(),
            b"\"abc alot of text that does not\\\" terminate with a quote"
        );
        assert_eq!(s2.source(), b"");
    }

    #[test]
    fn int_literal() {
        use super::*;
        let text = b"123";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), DecimalLiteral);
        assert_eq!(s1.fragment(), b"123");
        assert_eq!(s2.source(), b"");

        let text = b"123abc";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), DecimalLiteral);
        assert_eq!(s1.fragment(), b"123");
        assert_eq!(s2.source(), b"abc");

        let text = b"12a111";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), DecimalLiteral);
        assert_eq!(s1.fragment(), b"12");
        assert_eq!(s2.source(), b"a111");

        let text = b"b1111";
        span!(span, text);
        assert!(int_literal(span).is_none());

        let text = b"0x123";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), HexLiteral);
        assert_eq!(s1.fragment(), b"0x123");
        assert_eq!(s2.source(), b"");

        let text = b"0x123abc";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), HexLiteral);
        assert_eq!(s1.fragment(), b"0x123abc");
        assert_eq!(s2.source(), b"");

        let text = b"0x123abcg";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        // really...
        assert_eq!(s1.get().unwrap(), HexLiteral);
        assert_eq!(s1.fragment(), b"0x123abc");
        assert_eq!(s2.source(), b"g");

        let text = b"0x12gabcg";
        span!(span, text);
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap(), HexLiteral);
        assert_eq!(s1.fragment(), b"0x12");
        assert_eq!(s2.source(), b"gabcg");

        let text = "0xtttt";
        span!(span, text.as_bytes());
        let (s1, s2) = int_literal(span).unwrap();
        assert_eq!(s1.get().unwrap_err(), Error::EmptyHexLiteral);
        assert_eq!(s1.fragment(), b"0x");
        assert_eq!(s2.source(), b"tttt");
    }

    #[test]
    fn skip_spaces() {
        use super::*;
        span!(span, b"    some text");
        assert_eq!(rem(skip_spaces(span)).source(), b"some text");
        span!(span, b"\n\n\t  \n some text");
        assert_eq!(rem(skip_spaces(span)).source(), b"some text");
        span!(span, b"   \n\t");
        assert_eq!(rem(skip_spaces(span)).source(), b"");
    }

    #[test]
    fn skip_line_comment() {
        use super::*;
        span!(span, b"// comment\nsometext");
        let span = skip_line_comment(span);
        assert_eq!(rem(span).source(), b"\nsometext");
        span!(span, b"// comment");
        let span = skip_line_comment(span);
        assert_eq!(rem(span).source(), b"");
    }

    #[test]
    fn skip_block_comment() {
        use super::*;
        span!(span, b"/* comment */sometext");
        let span = skip_block_comment(span);
        assert_eq!(rem(span).source(), b"sometext",);
        span!(span, b"/* comment ");
        let span = skip_block_comment(span);
        assert_eq!(rem(span).source(), b"");

        span!(span, b"/**/");
        let span = skip_block_comment(span);
        assert_eq!(rem(span).source(), b"");

        span!(span, b"/*/");
        let rem = rem(skip_block_comment(span));
        assert!(rem.is_empty())
    }

    #[test]
    fn symbol() {
        use super::*;
        let text = b"==";
        span!(span, text);
        let (s1, s2) = symbol(span).unwrap();
        assert_eq!(s1.get().unwrap(), EqualEqual);
        assert_eq!(s1.fragment(), b"==");
        assert_eq!(s2.source(), b"");

        use std::iter;
        let text = b"=+-*/%&&||!<>?:";
        span!(span, text);
        let mut span = span;
        let symbols = iter::from_fn(move || {
            if span.is_empty() {
                None
            } else {
                let (l, r) = symbol(span).unwrap();
                span = r;
                Some(l.get().unwrap())
            }
        })
        .collect::<Vec<_>>();
        assert_eq!(
            symbols,
            vec![
                Assign, Plus, Minus, Star, Slash, Percent, And, Or, Not, Less, Greater, Question,
                Colon,
            ]
        )
    }

    #[test]
    fn eof() {
        use super::*;
        span!(text, b"some text ***  // comment");
        let mut tokens = tokens(text, |_| {}).skip_while(|tok| tok.get().unwrap() != Token::Eof);
        let eof = tokens.next().unwrap();
        assert_eq!(eof.get().unwrap(), Token::Eof);
        assert_eq!(eof.fragment(), b"");
        assert_eq!(eof.position(), (1, text.len() + 1));
        assert!(tokens.next().is_none());
    }
}
