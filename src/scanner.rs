#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType { // Unify TokenType and Token???
    Plus,
    Minus,
    Star,
    Slash,
    Number,
    RParen,
    LParen
}
use TokenType::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub chars: &'a str
}

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner{
            source: source,
            position: 0,
        }
    }

    fn number(&mut self) -> Option<Token<'a>> {
         // we already advanced the position during Iterator::next(),
         // so decrement by one
        let start_index = self.position - 1;

        while Self::is_digit(self.peek().unwrap_or("_") /* "_" isn't a number */) {
            self.position += 1;
        }

        Some(Token{token_type: Number, chars: self.source.get(start_index..self.position)?})
    }

    fn next_char(&mut self) -> Option<&'a str> {
        if self.position >= self.source.len() {
            None
        } else {
            self.position += 1;
            self.source.get(self.position - 1..self.position)
        }
    }

    fn peek(&self) -> Option<&'a str> {
        if self.position >= self.source.len() {
            None
        } else {
            self.source.get(self.position..self.position + 1)
        }
    }

    pub fn is_digit(s: &'a str) -> bool {
        "0123456789".contains(s)
    }

    pub fn is_whitespace(s: &'a str) -> bool {
        " \t\n".contains(s)
    }

    pub fn make_token(token_type: TokenType, chars: &'a str) -> Token {
        Token{token_type: token_type, chars: chars}
    }

}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.next_char()?;
            match c {
                c if Scanner::is_whitespace(c) => { continue; }
                c if Scanner::is_digit(c) => {
                    return self.number()
                },
                "+" => return Some(Scanner::make_token(Plus, c)),
                "-" => return Some(Scanner::make_token(Minus, c)),
                "*" => return Some(Scanner::make_token(Star, c)),
                "/" => return Some(Scanner::make_token(Slash, c)),
                "(" => return Some(Scanner::make_token(LParen, c)),
                ")" => return Some(Scanner::make_token(RParen, c)),
                _ => return None,
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn whitespace() {
        let tokens = Scanner::new(" \t      \n\n  \n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn numbers() {
        let tokens = Scanner::new("1124\n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![Scanner::make_token(Number, "1124")]);
    }

    #[test]
    fn operators() {
        let tokens = Scanner::new("* - + /\n").collect::<Vec<_>>();

        assert_eq!(tokens, vec![
            Scanner::make_token(Star, "*"),
            Scanner::make_token(Minus, "-"),
            Scanner::make_token(Plus, "+"),
            Scanner::make_token(Slash, "/"),
        ]);
    }
}
