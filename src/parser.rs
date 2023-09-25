use crate::lexer::{Lexer, LexerError, Token, Keyword};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Integer(i32),
    String(String),
    Boolean(bool),
    None
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompareType {
    Equals,
    Less,
    Greater
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
            Value::Boolean(b) => {
                write!(formatter, "{}", if *b { ":)" } else { ":("} )
            },
            Value::None => {
                write!(formatter, "-")
            }
        }
    }
}


#[derive(PartialEq, Clone, Debug)]
pub enum ASTNode {
    UnaryOp {
        expression: Rc<ASTNode>,
        token: Token
    },
    BinOp {
        left: Rc<ASTNode>,
        right: Rc<ASTNode>,
        token: Token
    },
    Value {
        value: Value
    },
    FunctionCall {
        function: Rc<ASTNode>,
        parameters: Vec<ASTNode>
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        execution_block: Rc<ASTNode>
    },
    If {
        condition: Rc<ASTNode>,
        execution: Rc<ASTNode>
    },
    Loop {
        condition: Rc<ASTNode>,
        execution: Rc<ASTNode>
    },
    Compare {
        left: Rc<ASTNode>,
        right: Rc<ASTNode>,
        compare_type: CompareType
    },
    Block {
        children: Vec<ASTNode>
    },
    Assign {
        left: Rc<ASTNode>,
        right: Rc<ASTNode>
    },
    Return {
        expression: Rc<ASTNode>,
    },
    Variable {
        name: String
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

    fn function_call_or_variable(&mut self) -> Result<ASTNode, LexerError> {
        let variable = self.variable()?;
        if self.current_token == Token::ParentheseOpen {
            Ok(self.functioncall_statement(variable)?)
        } else {
            Ok(variable)
        }
    }

    fn factor(&mut self) -> Result<ASTNode, LexerError> {
        // FACTOR := +|- FACTOR | integer | (EXPR) | string | boolean | VARIABLE
        if Token::Plus == self.current_token || Token::Minus == self.current_token {
            let unary_token = self.current_token.clone();
            self.consume_token()?;
            let node = ASTNode::UnaryOp {
                expression: Rc::new(self.factor()?),
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
        } else if let Token::Boolean(b) = &self.current_token {
            let node = ASTNode::Value {
                value: Value::Boolean(*b)
            };
            self.consume_token()?;
            Ok(node)
        } else {
            Ok(self.function_call_or_variable()?)
        }
    } 

    fn term(&mut self) -> Result<ASTNode, LexerError> {
        // TERM := FACTOR ((MUL|DIV)FACTOR)*
        let mut node = self.factor()?;
        while self.current_token == Token::Multiply || self.current_token == Token::Divide { 
            let operator_token = self.current_token.clone();
            self.consume_token()?;
            node = ASTNode::BinOp {
                left: Rc::new(node), 
                right: Rc::new(self.factor()?),
                token: operator_token
            };
        }
        while let Token::ReservedKeyword(keyword) = self.current_token  { 
            let compare_type = match keyword {
                Keyword::Equals => {
                    CompareType::Equals
                },
                Keyword::Less => {
                    CompareType::Less
                },
                Keyword::Greater => {
                    CompareType::Greater
                },
                _ => {break;}
            };
            self.consume_token()?;
            node = ASTNode::Compare {
                left: Rc::new(node), 
                right: Rc::new(self.factor()?),
                compare_type
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
                left: Rc::new(node),
                right: Rc::new(self.term()?),
                token: operator_token
            };
        }

        Ok(node)
    }

    fn empty(&mut self) -> ASTNode {
        ASTNode::NoOp {}
    }

    fn variable(&mut self) -> Result<ASTNode, LexerError> {
        match self.current_token.clone() {
            Token::ID{string} => {
                self.consume_token()?;
                let node = ASTNode::Variable {
                    name: string.clone()
                };
                
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
            left: Rc::new(left),
            right: Rc::new(right)
        })
    }

    fn functioncall_statement(&mut self, function: ASTNode) -> Result<ASTNode, LexerError> {
        self.consume(Token::ParentheseOpen)?;
        let mut parameters : Vec<ASTNode> = Vec::new();
        // Check if parameters exist.
        if self.current_token != Token::ParentheseClose {
            loop {
                let parameter = self.expr()?;
                parameters.push(parameter);
                if self.current_token != Token::Comma {
                    break;
                } else {
                    self.consume(Token::Comma)?;
                }
            }
        }
        self.consume(Token::ParentheseClose)?;
        Ok(
            ASTNode::FunctionCall {
                function: Rc::new(function),
                parameters
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
            },
            Token::ReservedKeyword(keyword) => {
                match keyword {
                    Keyword::If | Keyword::Equals => {
                        self.consume_token()?;
                        ASTNode::If {
                            condition: Rc::new(self.expr()?),
                            execution: Rc::new(self.inner_block_statement()?)
                        }
                    },
                    Keyword::Function => {
                        self.consume_token()?;
                        let func_name = match &self.current_token {
                            Token::ID {string} => {
                                string.clone()
                            },
                            _ => {return Err(LexerError::UnexpectedToken{expected: "ID for FunctionName".to_string(), found: self.current_token.clone()});}
                        };
                        self.consume_token()?;
                        self.consume(Token::ParentheseOpen)?;
                        let mut parameters: Vec<String> = Vec::new();
                        if self.current_token != Token::ParentheseClose {
                            while let Token::ID{string} = self.current_token.clone() {
                                self.consume_token()?;
                                parameters.push(string.clone());
                            } 
                        }
                        self.consume(Token::ParentheseClose)?;
                        ASTNode::FunctionDeclaration {
                            name: func_name.clone(),
                            parameters,
                            execution_block: Rc::new(self.inner_block_statement()?)
                        }
                    },
                    Keyword::Loop => {
                        self.consume_token()?;
                        ASTNode::Loop {
                            condition: Rc::new(self.expr()?),
                            execution: Rc::new(self.inner_block_statement()?)
                        }
                    },
                    Keyword::AssignPrefix => {
                        self.consume_token()?;
                        let left = self.variable()?;
                        self.consume(Token::ReservedKeyword(Keyword::AssignInfix))?;
                        let right = self.expr()?;
                        ASTNode::Assign {
                            left: Rc::new(left),
                            right: Rc::new(right)
                        }
                    },
                    Keyword::Return => {
                        self.consume_token()?;
                        ASTNode::Return {
                            expression: Rc::new(self.expr()?)
                        }
                    },
                    _ => {self.empty()}
                }
            },
            _ => {self.empty()}
        }) 
    }

    fn statement_list(&mut self) -> Result<Vec<ASTNode>, LexerError> {
        let node = self.statement()?;
        let mut nodes : Vec<ASTNode> = vec![node];
        while self.current_token == Token::EndLine {
            self.consume(Token::EndLine)?;
            let statement = self.statement()?;
            if statement != ASTNode::NoOp {
                nodes.push(statement);
            }
        }

        return Ok(nodes);
    }

    fn inner_block_statement(&mut self) -> Result<ASTNode, LexerError>{
        if self.current_token == Token::EndLine {
            self.consume_token()?;
        }
        self.consume(Token::ReservedKeyword(Keyword::Avo))?;
        let nodes = self.statement_list()?;
        self.consume(Token::ReservedKeyword(Keyword::Cado))?;

        let block_node = ASTNode::Block {
            children: nodes
        };
        Ok(block_node)
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