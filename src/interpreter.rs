
use crate::lexer::{Token, LexerError};
use crate::parser::{Parser, Value, ASTNode};
use std::collections::HashMap;

pub struct Interpreter {
    parser: Parser,
    symbol_table: HashMap<String, Value>,
    // function_table: HashMap<String, Value>
}

impl Interpreter {

    pub fn new(parser: Parser) -> Self {
        Interpreter {
            parser,
            symbol_table: HashMap::new()
        }
    }

    fn expect(value: Value) -> i32 {
        match value {
            Value::Integer(v) => {
                v
            },
            _ => {
                panic!("Not a number!");
            }
        }
    }

    fn resolve_variable(&self, name: String) -> Value {
        match self.symbol_table.get(&name) {
            Some(value) => {
                return value.clone()
            },
            None => {
                panic!("Unknown variable name: {}", &name);
            }
        }
    }

    fn visit(&mut self, node: ASTNode) -> Result<Value, LexerError> {
        Ok(
            match node {
                ASTNode::BinOp {left, right, token} => {
                    Value::Integer(
                        match token {
                            Token::Plus => {Interpreter::expect(self.visit(*left)?) + Interpreter::expect(self.visit(*right)?)},
                            Token::Minus => {Interpreter::expect(self.visit(*left)?) - Interpreter::expect(self.visit(*right)?)},
                            Token::Multiply => {Interpreter::expect(self.visit(*left)?) * Interpreter::expect(self.visit(*right)?)},
                            Token::Divide => {Interpreter::expect(self.visit(*left)?) / Interpreter::expect(self.visit(*right)?)},
                            _ => {panic!("Invalid BinaryOp Token: {:?}", token);}
                        }
                     )
                },
                ASTNode::Value {value} => {
                    value
                },
                ASTNode::UnaryOp {expression, token} => {
                    Value::Integer(
                        match token {
                            Token::Plus => {Interpreter::expect(self.visit(*expression)?)},
                            Token::Minus => {-Interpreter::expect(self.visit(*expression)?)},
                            _ => {panic!("Invalid UnaryOp Token")},
                        }
                    )
                },
                ASTNode::Block {children} => {
                    for child in children {
                        self.visit(child)?;
                    }
                    Value::None
                },
                ASTNode::Variable {name, ..} => {
                    self.resolve_variable(name)
                },
                ASTNode::Assign {token: _, left, right} => {
                    match *left {
                        ASTNode::Variable{name,..} => {
                            let value = self.visit(*right)?;
                            self.symbol_table.insert(name, value);
                        }
                        _ => {panic!("Invalid Left Side in Assign.");}
                    }
                    Value::None
                },
                ASTNode::FunctionCall {function, parameters} => {
                    match *function {
                        ASTNode::Variable{name, ..} => {
                            if name == ":O__" {
                                print!(":O__(");
                                for parameter in parameters {
                                    match parameter {
                                        ASTNode::Variable {name, ..} => {
                                            print!("{}", self.resolve_variable(name));
                                        },
                                        _ =>{print!("{}", self.visit(parameter)?);}
                                    }
                                }
                                println!(")");
                            }
                        },
                        _ => {}
                    }
                    Value::None
                },
                ASTNode::NoOp => {Value::None},
            }
        )
    }

    pub fn interpret(&mut self) -> Result<(), LexerError> {
        let tree = self.parser.parse()?;
        self.visit(tree)?;
        //dbg!(&tree);
        //dbg!(&self.symbol_table);
        Ok(())
    }
}