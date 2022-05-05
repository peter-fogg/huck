use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug)]
pub enum ScanError {
    Fucked,
    Eof
}

#[derive(Debug)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Number
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub chars: String
}

#[derive(Debug)]
pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>
}

type ScanResult = Result<Token, ScanError>;

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner{source: source.chars().peekable()}
    }

    pub fn advance(&mut self) -> ScanResult {
        loop {
            let next = self.source.next();

            match next {
                Some(c) if is_whitespace(c) => continue,
                Some(c) if is_digit(c) => return self.number(c),
                Some('+') => return Ok(Token{token_type: TokenType::Plus, chars: "+".to_string()}),
                Some('-') => return Ok(Token{token_type: TokenType::Minus, chars: "-".to_string()}),
                Some('*') => return Ok(Token{token_type: TokenType::Star, chars: "*".to_string()}),
                Some('/') => return Ok(Token{token_type: TokenType::Slash, chars: "/".to_string()}),
                None => return Err(ScanError::Eof),
                _ => return Err(ScanError::Fucked),
            }
        }
    }

    fn number(&mut self, c: char) -> ScanResult {
        let mut number_string: String = String::from(c);
        loop {
            match self.source.peek() {
                Some(&c) if is_digit(c) => {
                    number_string.push(c);
                    _ = self.source.next();
                },
                _ => break,
            }
        }
        return Ok(Token{token_type: TokenType::Number, chars: number_string});
    }
}

fn is_digit(c: char) -> bool {
    "0123456789".contains(c)
}

fn is_whitespace(c: char) -> bool {
    " \t\n".contains(c)
}
