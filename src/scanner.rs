#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Plus,
    Minus,
    Star,
    Slash,
    Number(&'a str),
    True,
    False,
    RParen,
    LParen,
    RBrace,
    LBrace,
    Let,
    SingleEq,
    Semicolon,
    Var(&'a str),
}
use Token::*;

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner{
            source,
            position: 0,
        }
    }

    // TODO: handle non-integers
    fn number(&mut self) -> Option<Token<'a>> {
         // We already advanced the position during Iterator::next(),
         // so decrement by one
        let start_index = self.position - 1;

        // Keep munching characters until we hit EOF or a non-number character
        while Self::is_digit(self.peek().unwrap_or("_") /* "_" isn't a number */) {
            self.position += 1;
        }

        Some(Number(self.source.get(start_index..self.position)?))
    }

    fn identifier(&mut self) -> Option<Token<'a>> {
        let start_index = self.position - 1;

        while let Some(next_char) = self.peek() {
            if Self::is_digit(next_char) || Self::is_alpha(next_char) {
                self.position += 1;
            }
            else { break; }
        }

        let ident = self.source.get(start_index..self.position)?;

        Some(match ident {
            "let" => Let,
            "true" => True,
            "false" => False,
            _ => Var(ident)
        })
    }

    // Get the next character, if it exists, and advance the scanner
    fn next_char(&mut self) -> Option<&'a str> {
        // We're at the end
        if self.position >= self.source.len() {
            None
        } else {
            // This always increments the position to the next character
            self.position += 1;
            self.source.get(self.position - 1..self.position)
        }
    }

    // Get the next character, if it exists, without incrementing
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

    pub fn is_alpha(s: &'a str) -> bool {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_".contains(s)
    }

    pub fn is_whitespace(s: &'a str) -> bool {
        " \t\n".contains(s)
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.next_char()?;
            match c {
                c if Self::is_whitespace(c) => { continue; } // Munch whitespace
                c if Self::is_digit(c) => {
                    return self.number()
                },
                "+" => return Some(Plus),
                "-" => return Some(Minus),
                "*" => return Some(Star),
                "/" => return Some(Slash),
                "(" => return Some(LParen),
                ")" => return Some(RParen),
                "{" => return Some(LBrace),
                "}" => return Some(RBrace),
                "=" => return Some(SingleEq),
                ";" => return Some(Semicolon),
                _ => return self.identifier(),
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
        assert_eq!(tokens, vec![Number("1124")]);
    }

    #[test]
    fn operators() {
        let tokens = Scanner::new("= ; * - + / ( ) { } \n").collect::<Vec<_>>();
        assert_eq!(tokens, vec![SingleEq, Semicolon, Star, Minus, Plus, Slash, LParen, RParen, LBrace, RBrace]);
    }

    #[test]
    fn identifiers() {
        let tokens = Scanner::new("true ident let false ").collect::<Vec<_>>();
        assert_eq!(tokens, vec![True, Var("ident"), Let, False]);
    }
}
