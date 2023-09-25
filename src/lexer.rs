use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keyword {
    Greeting,
    Farewell,
    Avo,
    Cado,
    Function,
    Return,
    Loop,
    Equals,
    Less,
    Greater,
    AssignPrefix,
    AssignInfix,
    If
}


#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ReservedKeyword(Keyword),
    ID{string: String},
    Integer(u32),
    String(String),
    Boolean(bool),
    Comma,
    Plus,
    Minus,
    Multiply,
    Divide,
    ParentheseOpen,
    ParentheseClose,
    EndLine,
    Assign,
    EOF
}

impl Display for Token {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub struct Lexer {
    text: String,
    position: usize,
    reserved_keywords: HashMap<String, Token>
}

#[derive(Debug)]
pub enum LexerError {
    InvalidSyntax(String),
    UnexpectedToken {
        found: Token,
        expected: String
    }
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        Lexer {
            text: text.to_string(), 
            position: 0,
            reserved_keywords: [
                ("hallo".to_string(), Token::ReservedKeyword(Keyword::Greeting)),
                ("reicht dann auch mal".to_string(), Token::ReservedKeyword(Keyword::Farewell)),
                ("avo".to_string(), Token::ReservedKeyword(Keyword::Avo)),
                ("cado".to_string(), Token::ReservedKeyword(Keyword::Cado)),
                ("funny".to_string(), Token::ReservedKeyword(Keyword::Function)),
                ("wenn".to_string(), Token::ReservedKeyword(Keyword::If)),
                ("wirf".to_string(), Token::ReservedKeyword(Keyword::Return)),
                ("schleif".to_string(), Token::ReservedKeyword(Keyword::Loop)),
                ("is".to_string(), Token::ReservedKeyword(Keyword::Equals)),
                ("kleina".to_string(), Token::ReservedKeyword(Keyword::Less)),
                ("krasser".to_string(), Token::ReservedKeyword(Keyword::Greater)),
                ("machma".to_string(), Token::ReservedKeyword(Keyword::AssignPrefix)),
                ("uf".to_string(), Token::ReservedKeyword(Keyword::AssignInfix)),
                ].iter().cloned().collect()
        }
    }

    fn current_char(&self) -> Option<char> {
        self.text.chars().nth(self.position)
    }

    fn peek(&self) -> Option<char> {
        self.text.chars().nth(self.position + 1)
    }

    fn goto_next_position(&mut self) {
        self.position = self.position + 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(current_char) = self.current_char() {
            if current_char == ' ' {
                self.goto_next_position();
            } else {
                break;
            }
        }
    }

    fn integer(&mut self) -> u32 {
        let mut number = String::new();
        number.push(self.current_char().unwrap());

        while let Some(next_char) = self.peek() {
            if next_char.is_digit(10) {
                number.push(next_char);
                self.goto_next_position();
            } else {
                break;
            }
        }
        number.parse::<u32>().unwrap()
    }

    fn keyword_or_string(&mut self) -> Result<Token, LexerError> {
        let mut result = String::new();
        let current_char = self.current_char().unwrap();

        // String
        if current_char == '<' {
            while let Some(next_char) = self.peek() {
                if next_char != '>' {
                    result.push(next_char);
                    self.goto_next_position();
                } else {
                    break;
                }
            }
            self.goto_next_position();
            if self.current_char() == Some('>') {
                return Ok(
                    Token::String(result)
                )
            } else {
                return Err(
                    LexerError::InvalidSyntax("Missing string closure: >".to_string())
                )
            }
        }
        result.push(current_char);
        let start_position = self.position;
        // Keywords  
        while let Some(next_char) = &mut self.peek() {
            if next_char.is_alphanumeric() || *next_char == ' ' || *next_char == '_' {
                result.push(*next_char);
                self.goto_next_position();

                match self.reserved_keywords.get(&result) {
                    Some(keyword_token) => {
                        // self.position = self.position + 1;
                        return Ok(keyword_token.clone().clone())
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
        result = result.get(0..1).unwrap().to_string();
        self.position = start_position;
        // Variable. TODO: Cut of on first space.
        while let Some(next_char) = &mut self.peek() {
            if next_char.is_alphanumeric() || *next_char == '_' {
                result.push(*next_char);
                self.goto_next_position();
            } else {
                break;
            }
        }
        Ok(Token::ID {
            string: result
        })
        
    }

    fn smiley(&mut self) -> Option<Token> {
        match &self.peek() {
            Some(current_char) => {
                match current_char {
                    ')' => {
                        self.goto_next_position();
                        Some(Token::Boolean(true))
                    },
                    '(' => {
                        self.goto_next_position();
                        Some(Token::Boolean(false))
                    },
                    _ => {None}
                }
            },
            None => {
                None
            }
        }
    }

    // Break text into token.
    pub fn get_next_token(&mut self) -> Result<Token, LexerError> {
        if self.position > self.text.len() - 1 {
            return Ok(Token::EOF)
        }

        let mut token : Option<Token> = None;

        

        if let Some(current_char) = self.current_char() {
            if current_char.is_digit(10) {
                token = Some(Token::Integer(self.integer()));
            } else if current_char == '+' {
                token = Some(Token::Plus);
            } else if current_char == '-' {
                token = Some(Token::Minus);
            } else if current_char == '*' {
                token = Some(Token::Multiply);
            } else if current_char == '/' {
                token = Some(Token::Divide);
            } else if current_char == '(' {
                token = Some(Token::ParentheseOpen);
            } else if current_char == ')' {
                token = Some(Token::ParentheseClose);
            } else if current_char == '=' {
                token = Some(Token::Assign);  
            } else if current_char == '\n' {
                token = Some(Token::EndLine);
            } else if current_char == ',' {
                token = Some(Token::Comma);
            } else if current_char == ':' {
                token = self.smiley();  
            } 

            if token == None {
                token = Some(self.keyword_or_string()?);
            }
        }

        if let Some(token) = token {
            self.goto_next_position();
            self.skip_whitespace();
            Ok(token)
        } else {
            Err(LexerError::InvalidSyntax(String::from("No suitable token.")))
        }
    }

}