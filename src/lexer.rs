//std lib imports
use std::collections::HashSet;
use std::{iter::Peekable, str::Chars};
// external imports
use lazy_static::lazy_static;
// internal imports
use crate::tokens::{LiteralKind, Token};

lazy_static! {
    /// extended identification chars
    static ref EXTENDED_IDENT_CHARS: HashSet<char> = HashSet::from(['!', '$', '%', '&', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '^', '_', '~']);
}

/// Directives that designate whether an identifier should use be case agnostic
const DIRECTIVES: [&'static str; 2] = ["#!fold-case", "#!no-fold-case"];

pub const EOF_CHAR: char = '\0';

/// The Lexer. Taking heavy inspiration of the rustc_lexer Cursor struct
pub struct Lexer<'a> {
    len_remaining: usize,
    chars: Chars<'a>,
}

// Here we implement some tooling
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            len_remaining: input.len(),
            chars: input.chars(),
        }
    }

    // peeks the next char
    fn first(&self) -> Option<char> {
        self.chars.clone().next()
    }

    // peeks the second char
    fn second(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
    }

    // peeks the third char
    fn third(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next();
        iter.next()
    }

    fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn take_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut s = String::new();
        // while there exists chars to take that fullfill the predicate, take them.
        while self.first().is_some() && predicate(self.first().unwrap()) {
            s.push(self.bump().unwrap());
        }
        s
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while self.first().is_some() && predicate(self.first().unwrap()) {
            self.bump();
        }
    }
}

// here we define the syntax specific tooling
impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Token {
        if cfg!(debug_assertions) {
            println!(
                "begining 'next_token' call with chars: {:?}",
                self.chars.clone().collect::<Vec<char>>()
            )
        }
        use Token::*;
        // first we consume as much whitespace as we can
        self.eat_while(|c| c.is_whitespace());

        // try consuming a char
        let first_char = match self.bump() {
            Some(c) => c,
            None => return EOF,
        };
        // Based on some char patterns we will opportunistically try to consume more of the input.
        // Every method used to consume further might however return `Token::Error` instead if they were
        // unable to parse the consumed chars as expected.
        let token_kind = match (first_char, self.first(), self.second(), self.third()) {
            // Single char tokens
            ('(', _, _, _) => OpenParen,
            (')', _, _, _) => CloseParen,
            ('[', _, _, _) => OpenSquareParen, // reserved for future syntax extensions
            (']', _, _, _) => CloseSquareParen, // reserved
            ('{', _, _, _) => OpenCurlyParen,  // reserved
            ('}', _, _, _) => CloseCurlyParen, // reserved
            ('\'', _, _, _) => Apost,          // denotes literal data
            ('`', _, _, _) => Grave,           // denotes partially constant data
            // comments
            (';', _, _, _) => self.line_comment(),
            ('#', Some('|'), _, _) => self.block_comment(),
            // directive
            ('#', Some('!'), _, _) => self.directive(),
            ('#', Some(c), _, _) if c == 't' || c == 'f' => self.boolean(),
            // some list types
            ('#', Some('u'), Some('8'), Some('(')) => self.bytevector(),
            ('#', Some('('), _, _) => self.vector(),
            // identifiers
            ('|', _, _, _) => self.pipe_identifier(),
            (i, _, _, _) if is_valid_first_letter_ident(i) => self.identifier(i), // a valid ident may not begin with a number or consist of a single '.'
            _ => Error,
        };

        // if we've been unsuccessfull in  matching some known syntax,
        token_kind
    }

    fn identifier(&mut self, first_letter: char) -> Token {
        let mut content = String::from(first_letter);
        // while the next char is a valid ident char, keep consooooooming
        while self.first().is_some() && is_identifier_char(self.first().unwrap()) {
            content.push(self.bump().unwrap());
        }
        // only include this on debug builds
        if cfg!(debug_assertions) {
            println!(
                "content: {content}, chars left: {:?}",
                self.chars.clone().collect::<Vec<char>>()
            );
        }

        Token::Identifier(content)
    }

    fn vector(&mut self) -> Token {
        self.bump(); // throw away the '('
        Token::OpenVec
    }

    fn bytevector(&mut self) -> Token {
        self.bump(); // throw away the 'u'
        self.bump(); // throw away the '8'
        self.bump(); // throw away the '('
        Token::OpenByteVec
    }

    fn boolean(&mut self) -> Token {
        let content = self.take_while(|c| c.is_whitespace());
        let c = &content;
        if c == "t" || c == "true" || c == "f" || c == "false" {
            Token::Literal(LiteralKind::Boolean(content))
        } else {
            Token::Error
        }
    }

    fn pipe_identifier(&mut self) -> Token {
        let mut content = String::from('|');
        content.push_str(&(self.take_while(|c| c != '|')));
        content.push('|');
        let res = Token::Identifier(content);
        self.bump();
        res
    }

    fn string_literal(&mut self) -> Token {
        let content = self.take_while(|c| !c.is_whitespace());
        let res = Token::Literal(LiteralKind::Str(content));
        // trow away the second '"'
        self.bump();
        res
    }

    fn line_comment(&mut self) -> Token {
        let content = self.take_while(|c| c != '\n');
        Token::Comment(content.trim().into())
    }

    fn block_comment(&mut self) -> Token {
        // throw away the '|'
        self.bump();
        let mut content = String::new();
        while self.first() != Some('|') && self.second() != Some('#') {
            content.push(self.bump().unwrap());
        }
        match (self.first(), self.second()) {
            (Some('|'), Some('#')) => {
                // throw away the '|' and '#'
                self.bump();
                self.bump();
                Token::BlockComment(content.trim().into())
            }
            (_, _) => Token::Error,
        }
    }

    fn directive(&mut self) -> Token {
        // throw away the '!'
        self.bump();
        let content = self.take_while(|c| !c.is_whitespace());
        Token::Directive(content)
    }
}

/// checks whether the letter i is a valid first letter of an identifier
/// (can't be a number or invalid extended char)
fn is_valid_first_letter_ident(c: char) -> bool {
    !c.is_numeric() && (c.is_alphabetic() || EXTENDED_IDENT_CHARS.contains(&c))
}

/// checks whether a letter is a valid ident letter.
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || EXTENDED_IDENT_CHARS.contains(&c)
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    /// Tests that the sequences of tokens produced by the lexer matches the expected sequence.
    fn expected_sequnce(seq: &[Token], input: &str) {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token();
            if token == EOF {
                break;
            } else {
                tokens.push(token);
            }
        }
        println!("{:?}", tokens);

        for (expected_token, actual_token) in seq.iter().zip(tokens) {
            assert_eq!(*expected_token, actual_token);
        }
        assert_eq!(EOF, lexer.next_token());
    }

    #[test]
    fn no_input() {
        expected_sequnce(&[], "");
    }

    #[test]
    fn parens() {
        expected_sequnce(
            &[
                OpenParen, OpenParen, OpenParen, OpenParen, CloseParen, CloseParen, CloseParen,
                CloseParen,
            ],
            "(((())))",
        );
    }
    #[test]
    fn comment1() {
        expected_sequnce(
            &[Comment(String::from("this is a comment"))],
            "; this is a comment",
        );
    }
    #[test]
    fn single_char_ident() {
        expected_sequnce(&[Identifier(String::from("+"))], "+");
    }

    #[test]
    fn all_extended_char_idents() {
        for ident in EXTENDED_IDENT_CHARS.iter() {
            expected_sequnce(&[Identifier(String::from(*ident))], &ident.to_string());
        }
    }

    #[test]
    fn ident1() {
        expected_sequnce(
            &[
                OpenParen,
                Identifier(String::from("+")),
                Identifier(String::from("var1")),
                Identifier(String::from("var2")),
                CloseParen,
            ],
            "(+ var1 var2)",
        );
    }
    #[test]
    fn test_multiple_ients() {
        let idents = [
            "...",
            "+",
            "+soup+",
            "<=?",
            "->string",
            "a34kTMNs",
            "lambda",
            "list->vector",
            "q",
            "V17a",
            "|two words|",
            "|two\x20;words|",
            "the-word-recursion-has-many-meanings",
        ];
        for ident in idents {
            expected_sequnce(&[Identifier(String::from(ident))], ident);
        }
    }
}
