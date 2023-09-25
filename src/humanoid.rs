use rand::Rng;
use crate::parser::{ASTNode, Value, Parser};
use crate::interpreter::{Scope, InterpreterError};
use std::io::prelude::*;
use std::time::Instant;

#[derive(PartialEq, Clone, Debug)]
pub enum Mood {
    Happy = 6,
    Glad = 5,
    Okay = 4,
    Sad = 3,
    Aggressive = 2,
    Deactivated = 0
}

impl std::fmt::Display for Mood {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let smiley = match self {
            Mood::Happy => "=D",
            Mood::Glad => "=)",
            Mood::Okay => "=I",
            Mood::Sad => "=(",
            Mood::Aggressive => "=X",
            Mood::Deactivated => "Xc"
        };
        write!(formatter, "{}", smiley)
    }
}

pub trait Humanoid {
    fn mood_level(&self) -> u32;
    fn mood_changed(&mut self) -> bool;
    fn mood_range(&self) -> [u32; 5];
}

pub struct Worker {
    prev_mood: Mood,
    stress_level: u32,
    user_answer: Option<Value>,
    question_cooldown: Instant,
    cooldown: u128,
    strict_work: bool,
}

pub struct Shouter {
    voice_damage: u32,
    strict_work: bool
}

pub fn read_line(text: &str) -> Option<String> {
    let mut buffer = String::new();
    print!("{}", text);
    std::io::stdout().flush().expect("IO error.");
    match std::io::stdin().read_line(&mut buffer) {
        Ok(_) => {
            Some(buffer)
        },
        Err(_) => {None}
    }
}

pub fn read_value(text: &str) -> Value {
    match read_line(text) {
        Some(buffer) => {
            let mut new_parser = Parser::new(crate::Lexer::new_fill_greeting_farewell(&buffer));
            let node = new_parser.parse();
            match node {
                Ok(ASTNode::Block{children}) => {
                    match children.get(0) {
                        Some(ASTNode::Assign{left:_, right: answer}) => {
                            match &**answer {
                                ASTNode::Value {value: answer} => {
                                    answer.clone()
                                },
                                _ => {Value::None}
                            }
                        },
                        _ => {Value::None}
                    }
                } 
                _ => {
                    Value::None
                }
            }
        },
        None => {Value::None}
    }
}

impl Worker {
    pub fn new(strict_work: bool) -> Self {
        Worker {
            prev_mood: Mood::Happy,
            strict_work,
            stress_level: 0,
            user_answer: None,
            question_cooldown: Instant::now(),
            cooldown: 20
        }
    }

    pub fn call(&mut self, scope: &Scope, node: &ASTNode, correct: &Value) -> Result<(), InterpreterError>{
        if self.strict_work {
            return Ok(());
        }

        self.stress_level = self.stress_level + rand::thread_rng().gen_range(1..10);
        let current_mood = HumanoidControl::mood::<Worker>(&self);
        if self.mood_changed() {
            println!("[ {} ]", current_mood);
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
            if  current_mood == Mood::Deactivated && self.question_cooldown.elapsed().as_nanos() > self.cooldown {
                if let ASTNode::Value{value: _} = node {
                    // Simple value evalution is boring.
                    return Ok(());
                }
                println!("{}, Ich kann nicht mehr... Zu was wertet dieser Ausdruck hier aus?", HumanoidControl::mood::<Worker>(&self));
                println!("{}", "-".repeat(15));
                println!("Symbols: {:?}", scope.symbol_table);
                println!("{:?}", node);
                println!("{}", "-".repeat(15));
                self.user_answer = Some(read_value(">>"));
                
                if let Some(answer) = &self.user_answer {
                    if *answer == *correct {
                        if *correct == Value::None {
                            println!("Wow, gar nichts...");
                        }
                        println!("Danke, du hast recht!");
                        self.stress_level = 0;
                        self.cooldown = rand::thread_rng().gen_range(1000000..1000000000);
                        self.question_cooldown = Instant::now();
                    } else {
                        println!("¿Ehm, nein? Es wäre {}.", correct);
                        return Err(InterpreterError::DisturbedWorker);
                    }
                    self.user_answer = None;
                }
            }
           
        
        Ok(())
    }


}

impl Shouter {
    pub fn new(strict_work: bool) -> Self {
        Shouter {
            voice_damage: 0,
            strict_work
        }
    }

    pub fn shout(&mut self, shout_level: usize, text: String) {
        if self.strict_work  {
            println!("{}", text);
        } else {
        let mut rng = rand::thread_rng();
        if self.voice_damage > 1000 {
            std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(20..500)));
            println!("{} {}", HumanoidControl::mood::<Shouter>(&self), 
            match rng.gen_range(1..4) {
                1 => {
                    "*hust*"
                }, 
                2 => {"*keuch*"},
                3 => {"*arr*"},
                _ => {"*hrrm*"}
            });
            if rand::thread_rng().gen_range(0..1) == 0 {
                println!("Kann ich was zu trinken haben?");
                match read_value("Gebe: ") {
                    Value::String(s) => {
                        match s.to_lowercase().as_str() {
                            "tee"|"wasser" => {
                                println!("Danke!");
                                self.voice_damage = 0;
                            },
                            _ => {
                                println!("Das trinke ich nicht.");
                            }
                        }
                    },
                    _ => {
                        println!("<Du musst in meiner Sprache sprechen>");
                    }
                };
                std::thread::sleep(std::time::Duration::from_millis(800));
            }
        } else {
            let mut s = String::new();
            for c in text.chars() {
                let r = rng.gen_range(0..100);
                if ((shout_level-1)*10) > r {
                    for upper_c in c.to_uppercase() {
                        s.push(upper_c);
                    }
                } else {
                    s.push(c);
                }
            }
            println!("{}", s);
            self.voice_damage = self.voice_damage + shout_level as u32;
            std::thread::sleep(std::time::Duration::from_nanos(self.voice_damage as u64 * 100000));
        }
    }
}
}

pub struct HumanoidControl {}

impl HumanoidControl{
    pub fn mood<T: Humanoid>(humanoid: &T) -> Mood {
        let n = humanoid.mood_level();
        let range = humanoid.mood_range();
        if n < range[0] {
            Mood::Happy
        } else if n < range[1] {
            Mood::Glad
        } else if n < range[2] {
            Mood::Okay
        } else if n < range[3] {
            Mood::Sad
        } else if n < range[4] {
            Mood::Aggressive
        }else {
            Mood::Deactivated
        }
    }
}

impl Humanoid for Worker {
    fn mood_range(&self) -> [u32; 5] {
        [50, 1000, 10000, 100000, 1000000]
    }

    fn mood_level(&self) -> u32 {
        self.stress_level
    }

    fn mood_changed(&mut self) -> bool {
        let last = self.prev_mood.clone();
        let new_mood = HumanoidControl::mood::<Worker>(&self);
        let result = last != new_mood;
        self.prev_mood = HumanoidControl::mood::<Worker>(&self);
        return result
    } 
}

impl Humanoid for Shouter {
    fn mood_range(&self) -> [u32; 5] {
        [20, 30, 40, 100, 10000]
    }

    fn mood_level(&self) -> u32 {
        self.voice_damage
    }

    fn mood_changed(&mut self) -> bool {
        return false
    } 
}