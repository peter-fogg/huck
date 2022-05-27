#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Plus,
    Minus,
    Star,
    Slash,
    Number(&'a str),
    RParen,
    LParen,
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

        Some(Number(self.source.get(start_index..self.position)?))
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
                "+" => return Some(Plus),
                "-" => return Some(Minus),
                "*" => return Some(Star),
                "/" => return Some(Slash),
                "(" => return Some(LParen),
                ")" => return Some(RParen),
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
        assert_eq!(tokens, vec![Number("1124")]);
    }

    #[test]
    fn operators() {
        let tokens = Scanner::new("* - + / ( )\n").collect::<Vec<_>>();
        assert_eq!(tokens, vec![Star, Minus, Plus, Slash, LParen, RParen]);
    }
}
