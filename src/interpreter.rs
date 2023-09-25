
use crate::lexer::{Token, LexerError};
use crate::parser::{Parser, Value, ASTNode, CompareType};
use crate::humanoid::{Shouter, Humanoid, Worker};
use std::collections::HashMap;
use std::string::String;

pub struct Interpreter {
    parser: Parser,
    call_stack: Vec<Scope>,
    worker: Worker,
    shouter: Shouter,
}


#[derive(Clone)]
struct Scope {
    symbol_table: HashMap<String, Value>,
    function_table: HashMap<String, (Vec<String>, ASTNode)>
}

impl Scope {
    pub fn new() ->  Self {
        Scope {
            symbol_table: HashMap::new(),
            function_table: HashMap::new()
        }
    }
}

enum InterpreterError {
    HackyReturn(Value)
}

impl Interpreter {

    pub fn new(parser: Parser) -> Self {
        Interpreter {
            parser,
            call_stack: vec![Scope::new()],
            worker: Worker::new(),
            shouter: Shouter::new()
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

    fn scope(&self) -> &Scope {
        let scope = self.call_stack.last().expect("Empty callstack! :s");
        scope
    }

    fn scope_mut(&mut self) -> &mut Scope {
       let scope = self.call_stack.last_mut().expect("Empty callstack! :s");
       scope
    }

    fn resolve_variable(&self, name: String) -> Value {
        match self.scope().symbol_table.get(&name) {
            Some(value) => {
                return value.clone()
            },
            None => {
                panic!("Unknown variable name: {}", &name);
            }
        }
    }

    fn resolve_function(&self, name: String) -> (Vec<String>, ASTNode) {
        match self.scope().function_table.get(&name) {
            Some(value) => {
                return value.clone()
            },
            None => {
                panic!("Unknown variable name: {}", &name);
            }
        }
    }

    fn visit(&mut self, node: ASTNode) -> Result<Value, InterpreterError> {
        self.worker.call();
        if self.worker.mood_changed() {
            // println!("{}", HumanoidControl::mood(&self.worker));
        }
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
                        match &child {
                            ASTNode::Return{expression: _} => {
                                let result = self.visit(child)?;
                                return Ok(result)
                            },
                            _ => {
                                self.visit(child)?;
                            }
                        }
                        
                    }
                    Value::None
                },
                ASTNode::Variable {name, ..} => {
                    self.resolve_variable(name)
                },
                ASTNode::Assign {left, right} => {
                    match *left {
                        ASTNode::Variable{name,..} => {
                            let value = self.visit(*right)?;
                            self.scope_mut().symbol_table.insert(name, value);
                        }
                        _ => {panic!("Invalid Left Side in Assign.");}
                    }
                    Value::None
                },
                ASTNode::If {condition, execution} => {
                    let result = self.visit(*condition)?;
                    match result {
                        Value::Boolean(true) => {
                            self.visit(*execution)?;
                        },
                        _ => {}
                    }

                    Value::None
                },
                ASTNode::Loop {condition, execution} => {
                    while let Value::Boolean(true) = self.visit(*condition.clone())? {
                        self.visit(*execution.clone())?;
                    }
                    Value::None
                },
                ASTNode::Compare {compare_type, left, right} => {
                    let left_result = self.visit(*left)?;
                    let right_result = self.visit(*right)?;
                    match compare_type {
                        CompareType::Equals => {
                            return Ok(Value::Boolean(left_result == right_result));
                        },
                        CompareType::Less => {
                            return Ok(Value::Boolean(left_result < right_result));
                        },
                        CompareType::Greater => {
                            return Ok(Value::Boolean(left_result > right_result));
                        }
                    }
                },
                ASTNode::FunctionDeclaration {name, parameters, execution_block} => {
                    if None != self.scope_mut().function_table.insert(name.clone(), (parameters, *execution_block)) {
                        panic!("Function {:?} redeclared!", name);
                    }
                    Value::None
                },
                ASTNode::FunctionCall {function, parameters} => {
                    match *function {
                        ASTNode::Variable{name, ..} => {
                            // Hard-coded Output Function
                            if name.starts_with(":O__") {
                                let mut text = String::new(); 
                                for parameter in parameters {
                                    match parameter {
                                        ASTNode::Variable {name, ..} => {
                                            text.push_str(format!("{}", self.resolve_variable(name).to_string()).as_str());
                                        },
                                        _ =>{text.push_str(format!("{}", self.visit(parameter)?).as_str());}
                                    }
                                }
                                self.shouter.shout(name.len() - 3, text);
                            } else {
                                // User-defined Functions

                                let mut new_scope = Scope::new();
                                new_scope.function_table.clone_from(&self.scope().function_table);
                                let (func_parameters, execution_block) = self.resolve_function(name);
                                if func_parameters.len() != parameters.len() {
                                    panic!("Invalid argument count!");
                                }
                                // TODO: There is 100% a Rust Solution for enumerating with an index.
                                let mut i = 0;
                                for parameter in parameters {
                                    let value = self.visit(parameter)?;
                                    new_scope.symbol_table.insert(func_parameters.get(i).expect("Function argument missing").clone(), value);
                                    i = i + 1;
                                }
                                // Push upon callstack new function scope+
                                self.call_stack.push(new_scope);

                                let result = match self.visit(execution_block) {
                                    Ok(value) => {
                                        value
                                    },
                                    Err(InterpreterError::HackyReturn(value)) => {
                                        value
                                    }
                                };
                                self.call_stack.pop();
                                return Ok(result);
                            }
                        },
                        _ => {}
                    }
                    Value::None
                },
                ASTNode::Return{expression} => {
                    // So f...... cursed.
                    return Err(InterpreterError::HackyReturn(self.visit(*expression)?))
                },
                ASTNode::NoOp => {Value::None},
            }
        )
    }

    pub fn interpret(&mut self) -> Result<(), LexerError> {
        let tree = self.parser.parse()?;
        let result = self.visit(tree);
        match result {
            Ok(_) => {

            },
            Err(InterpreterError::HackyReturn(val)) => {
                println!("This program throwed at us a: {}", val);
            }
        }
        //dbg!(&tree);
        //dbg!(&self.symbol_table);
        Ok(())
    }
}