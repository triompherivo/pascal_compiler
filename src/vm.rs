// src/vm.rs

use crate::codegen::Instruction;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;

#[derive(Debug, Clone)]
pub enum VMValue {
    Int(i64),
    Real(f64),
    Str(String),
    Record(HashMap<String, VMValue>),  // New variant to represent record values.
    File(Rc<RefCell<File>>), // New variant for file handles.
}

impl VMValue {
    fn as_int(&self) -> i64 {
        match self {
            VMValue::Int(n) => *n,
            VMValue::Real(r) => *r as i64,
            _ => panic!("Value is not an integer: {:?}", self),
        }
    }
    fn as_real(&self) -> f64 {
        match self {
            VMValue::Real(r) => *r,
            VMValue::Int(n) => *n as f64,
            _ => panic!("Value is not a real number: {:?}", self),
        }
    }
}

fn fact(n: i64) -> i64 {
    if n == 0 { 1 } else { n * fact(n - 1) }
}

pub struct VM {
    pub instructions: Vec<Instruction>,
    pub ip: usize,
    pub stack: Vec<VMValue>,
    pub globals: HashMap<String, VMValue>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        VM {
            instructions,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        while self.ip < self.instructions.len() {
            match self.instructions[self.ip].clone() {
                Instruction::Push(n) => {
                    self.stack.push(VMValue::Int(n));
                    self.ip += 1;
                },
                Instruction::PushReal(r) => {
                    self.stack.push(VMValue::Real(r));
                    self.ip += 1;
                },
                Instruction::PushString(s) => {
                    self.stack.push(VMValue::Str(s));
                    self.ip += 1;
                },
                Instruction::Load(var) => {
                    let val = self.globals.get(&var).cloned().unwrap_or(VMValue::Int(0));
                    self.stack.push(val);
                    self.ip += 1;
                },
                Instruction::Store(var) => {
                    if let Some(val) = self.stack.pop() {
                        self.globals.insert(var, val);
                    }
                    self.ip += 1;
                },
                Instruction::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => VMValue::Int(x + y),
                        (a, b) => VMValue::Real(a.as_real() + b.as_real()),
                    };
                    self.stack.push(result);
                    self.ip += 1;
                },
                Instruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => VMValue::Int(x - y),
                        (a, b) => VMValue::Real(a.as_real() - b.as_real()),
                    };
                    self.stack.push(result);
                    self.ip += 1;
                },
                Instruction::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => VMValue::Int(x * y),
                        (a, b) => VMValue::Real(a.as_real() * b.as_real()),
                    };
                    self.stack.push(result);
                    self.ip += 1;
                },
                Instruction::PushRecord => {
                    // Push an empty record (represented as an empty HashMap) onto the stack.
                    self.stack.push(VMValue::Record(HashMap::new()));
                    self.ip += 1;
                },
                Instruction::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let result = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => VMValue::Int(x / y),
                        (a, b) => VMValue::Real(a.as_real() / b.as_real()),
                    };
                    self.stack.push(result);
                    self.ip += 1;
                },
                Instruction::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let res = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => if x == y { 1 } else { 0 },
                        (VMValue::Real(x), VMValue::Real(y)) => if (x - y).abs() < 1e-9 { 1 } else { 0 },
                        (VMValue::Str(s1), VMValue::Str(s2)) => if s1 == s2 { 1 } else { 0 },
                        (a, b) => if (a.as_real() - b.as_real()).abs() < 1e-9 { 1 } else { 0 },
                    };
                    self.stack.push(VMValue::Int(res));
                    self.ip += 1;
                },
                Instruction::LE => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let res = match (a, b) {
                        (VMValue::Int(x), VMValue::Int(y)) => if x <= y { 1 } else { 0 },
                        (a, b) => if a.as_real() <= b.as_real() { 1 } else { 0 },
                    };
                    self.stack.push(VMValue::Int(res));
                    self.ip += 1;
                },
                // New: FieldAccess
                Instruction::FieldAccess(field) => {
                    let rec = self.stack.pop().expect("Stack underflow in FieldAccess");
                    match rec {
                        VMValue::Record(ref hm) => {
                            if let Some(val) = hm.get(&field) {
                                self.stack.push(val.clone());
                            } else {
                                panic!("Field '{}' not found in record", field);
                            }
                        },
                        _ => panic!("FieldAccess expected a record, found {:?}", rec),
                    }
                    self.ip += 1;
                },
                Instruction::Call(name, num_args) => {
                    let name_lower = name.to_lowercase();
                    if name_lower == "writeln" {
                        if let Some(val) = self.stack.pop() {
                            match val {
                                VMValue::Int(n) => println!("{}", n),
                                VMValue::Real(r) => println!("{}", r),
                                VMValue::Str(s) => println!("{}", s),
                                VMValue::Record(rec) => println!("{:?}", rec),
                                VMValue::File(_) => println!("<file handle>"),
                            }
                        }
                        self.ip += 1;
                    } else if name_lower == "readln" {
                        let mut input = String::new();
                        print!("Input> ");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut input).expect("Failed to read line");
                        if let Ok(n) = input.trim().parse::<i64>() {
                            self.stack.push(VMValue::Int(n));
                        } else if let Ok(r) = input.trim().parse::<f64>() {
                            self.stack.push(VMValue::Real(r));
                        } else {
                            self.stack.push(VMValue::Str(input.trim().to_string()));
                        }
                        self.ip += 1;
                    } else if name_lower == "fact" {
                        let arg = self.stack.pop().unwrap().as_int();
                        self.stack.push(VMValue::Int(fact(arg)));
                        self.ip += 1;
                    } else if name_lower == "printresult" {
                        if let Some(val) = self.stack.pop() {
                            match val {
                                VMValue::Int(n) => println!("{}", n),
                                VMValue::Real(r) => println!("{}", r),
                                VMValue::Str(s) => println!("{}", s),
                                VMValue::Record(rec) => println!("{:?}", rec),
                                VMValue::File(_) => println!("<file handle>"),
                            }
                        }
                        self.ip += 1;
                    } else if name_lower == "addreal" {
                        let b = self.stack.pop().unwrap().as_real();
                        let a = self.stack.pop().unwrap().as_real();
                        self.stack.push(VMValue::Real(a + b));
                        self.ip += 1;
                    } else if name_lower == "printreal" {
                        if let Some(val) = self.stack.pop() {
                            match val {
                                VMValue::Real(r) => println!("{}", r),
                                VMValue::Int(n) => println!("{}", n as f64),
                                VMValue::Str(s) => println!("{}", s),
                                VMValue::Record(rec) => println!("{:?}", rec),
                                VMValue::File(_) => println!("<file handle>"),
                            }
                        }
                        self.ip += 1;
                    } 
                    else if name_lower == "uppercase" {
                        if let Some(val) = self.stack.pop() {
                            match val {
                                VMValue::Str(s) => {
                                    self.stack.push(VMValue::Str(s.to_uppercase()));
                                },
                                _ => panic!("uppercase expects a string argument"),
                            }
                        }
                        self.ip += 1;
                    } 
                    else if name_lower == "makerecord" {
                        // num_args is the total number of arguments (should be even)
                        if num_args % 2 != 0 {
                            panic!("makerecord expects an even number of arguments (value, field name pairs)");
                        }
                        let num_fields = num_args / 2;
                        let mut args_vec = Vec::new();
                        // Pop all arguments from the stack.
                        for _ in 0..num_args {
                            args_vec.push(self.stack.pop().expect("Not enough arguments for makerecord"));
                        }
                        // The arguments are typically in reverse order, so reverse them.
                        args_vec.reverse();
                        let mut rec = HashMap::new();
                        for i in 0..num_fields {
                            // Expect the first argument in the pair to be the field value,
                            // and the second to be the field name (as a string).
                            let value = args_vec[2 * i].clone();
                            let field_name_val = &args_vec[2 * i + 1];
                            let field_name = match field_name_val {
                                VMValue::Str(s) => s.clone(),
                                _ => panic!("makerecord expects field names to be strings"),
                            };
                            rec.insert(field_name, value);
                        }
                        self.stack.push(VMValue::Record(rec));
                        self.ip += 1;
                    }
                    
                    else if name_lower == "main" {
                        // main should be inlined; do nothing here.
                        self.ip += 1;
                    } else {
                        self.ip += 1;
                    }
                },
                Instruction::Jump(target) => {
                    self.ip = target;
                },
                Instruction::JumpIfFalse(target) => {
                    let cond = self.stack.pop().unwrap();
                    match cond {
                        VMValue::Int(x) => {
                            if x == 0 {
                                self.ip = target;
                            } else {
                                self.ip += 1;
                            }
                        },
                        _ => panic!("JumpIfFalse expects an integer"),
                    }
                },
                Instruction::Ret => {
                    break;
                },
            }
        }
    }
}
