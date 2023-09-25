use crate::lexer::{Lexer, LexerError, Token, Keyword};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Integer(i32),
    String(String),
    None
}

impl Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Integer(int) => {
                write!(formatter, "{}", int)
            },
            Value::String(string) => {
                write!(formatter, "{}", string)
            },
            Value::None => {
                write!(formatter, "-")
            }
        }
    }
}


#[derive(PartialEq, Debug)]
pub enum ASTNode {
    UnaryOp {
        expression: Box<ASTNode>,
        token: Token
    },
    BinOp {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        token: Token
    },
    Value {
        value: Value
    },
    FunctionCall {
        function: Box<ASTNode>,
        parameters: Vec<ASTNode>
    },
    Block {
        children: Vec<ASTNode>
    },
    Assign {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
        token: Token
    },
    Variable {
        name: String,
        token: Token
    },
    NoOp
}

pub struct Parser {
    current_token: Token,
    lexer: Lexer
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            current_token: Token::EOF
        };
        parser.current_token = parser.lexer.get_next_token().unwrap_or(Token::EOF);
        return parser;
    }

    fn consume_token(&mut self) -> Result<(), LexerError> {
        self.current_token = self.lexer.get_next_token()?;
        Ok(())
    }

    fn consume(&mut self, token: Token) -> Result<(), LexerError> {
        if self.current_token == token {
            self.consume_token()?;
            Ok(())
        } else {
            Err(LexerError::UnexpectedToken{
                found: self.current_token.clone(),
                expected: token.to_string()
            })
        }
    }

    fn factor(&mut self) -> Result<ASTNode, LexerError> {
        // FACTOR := +|- FACTOR | integer | (EXPR)
        if Token::Plus == self.current_token || Token::Minus == self.current_token {
            let unary_token = self.current_token.clone();
            self.consume_token()?;
            let node = ASTNode::UnaryOp {
                expression: Box::new(self.expr()?),
                token: unary_token
            };
            return Ok(node)
        }

        if let Token::Integer(value) = self.current_token {
            let node = ASTNode::Value {
                value: Value::Integer(value as i32)
            };
            self.consume_token()?;
            Ok(node)
        } else if Token::ParentheseOpen == self.current_token {
            self.consume(Token::ParentheseOpen)?;
            let node = self.expr()?;
            self.consume(Token::ParentheseClose)?;
            Ok(node)
        }  else if let Token::String(string) = &self.current_token {
            let node = ASTNode::Value {
                value: Value::String(string.clone())
            };
            self.consume_token()?;
            Ok(node)
        } else {
            Ok(self.variable()?)
        }
    } 

    fn term(&mut self) -> Result<ASTNode, LexerError> {
        // TERM := FACTOR ((MUL|DIV)FACTOR)*
        let mut node = self.factor()?;
        while self.current_token == Token::Multiply || self.current_token == Token::Divide { 
            let operator_token = self.current_token.clone();
            self.consume_token()?;
            node = ASTNode::BinOp {
                left: Box::new(node), 
                right: Box::new(self.factor()?),
                token: operator_token
            };
        }
        Ok(node)
    }

    fn expr(&mut self) -> Result<ASTNode, LexerError>{
        // EXPR := TERM ((PLUS|MINUS)TERM)*
        let mut node = self.term()?;

        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let operator_token = self.current_token.clone();
            self.consume_token()?;
            node = ASTNode::BinOp {
                left: Box::new(node),
                right: Box::new(self.term()?),
                token: operator_token
            };
        }

        Ok(node)
    }

    fn empty(&mut self) -> ASTNode {
        ASTNode::NoOp {}
    }

    fn variable(&mut self) -> Result<ASTNode, LexerError> {
        match &self.current_token {
            Token::ID{string} => {
                let node = ASTNode::Variable {
                    name: string.clone(),
                    token: self.current_token.clone()
                };
                self.consume_token()?;
                Ok(node)
            },
            _ => {
                Err(LexerError::UnexpectedToken{
                    found: self.current_token.clone(),
                    expected: "Variable".to_string()
                })
            }
        }
    }

    fn assignment_statement(&mut self, left: ASTNode) -> Result<ASTNode, LexerError> {
        self.consume(Token::Assign)?;
        let right = self.expr()?;
        Ok(ASTNode::Assign {
            left: Box::new(left),
            token: Token::Assign,
            right: Box::new(right)
        })
    }

    fn functioncall_statement(&mut self, function: ASTNode) -> Result<ASTNode, LexerError> {
        self.consume(Token::ParentheseOpen)?;
        let parameter = self.variable().or_else(|_| self.expr())?;
        self.consume(Token::ParentheseClose)?;
        Ok(
            ASTNode::FunctionCall {
                function: Box::new(function),
                parameters: vec![parameter]
            }
        )
    }

    fn statement(&mut self) -> Result<ASTNode, LexerError> {
        Ok(match &self.current_token {
            Token::ID{string: _} => {
                let left = self.variable()?;
                if self.current_token == Token::Assign {
                    self.assignment_statement(left)?
                } else if self.current_token == Token::ParentheseOpen {
                    self.functioncall_statement(left)?
                } else {
                    self.empty()
                }
            }
            _ => {self.empty()}
        }) 
    }

    fn statement_list(&mut self) -> Result<Vec<ASTNode>, LexerError> {
        let node = self.statement()?;
        let mut nodes : Vec<ASTNode> = vec![node];
        while self.current_token == Token::EndLine {
            self.consume(Token::EndLine)?;
            nodes.push(self.statement()?);
        }

        return Ok(nodes);
    }

    fn block_statement(&mut self) -> Result<ASTNode, LexerError>{
        let nodes = self.statement_list()?;

        let block_node = ASTNode::Block {
            children: nodes
        };
        Ok(block_node)
    }

    fn program(&mut self) -> Result<ASTNode, LexerError> {
        self.consume(Token::ReservedKeyword(Keyword::Greeting))?;
        self.consume(Token::EndLine)?;
        let node = self.block_statement()?;
        self.consume(Token::ReservedKeyword(Keyword::Farewell))?;
        Ok(node)
    }

    pub fn parse(&mut self) -> Result<ASTNode, LexerError>{
        let program = self.program()?;
        if self.current_token != Token::EOF {
            Err(LexerError::UnexpectedToken {
                found: self.current_token.clone(),
                expected: "EOF".to_string()
            })
        } else {
            Ok(program)
        }
    }
}