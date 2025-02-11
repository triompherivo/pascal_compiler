// src/interpreter.rs

use crate::ast::{
    BinaryOperator, CaseBranch, Declaration, Expr, ForDirection, PascalType, Program, Statement,
};
use std::collections::HashMap;
use std::io::{self, Write};

use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;


#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Real(f64),
    Str(String),
    Record(HashMap<String, Value>),
    File(Rc<RefCell<File>>),
}

impl Value {
    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Real(r) => *r as i64,
            _ => panic!("Value is not an integer: {:?}", self),
        }
    }
    pub fn as_real(&self) -> f64 {
        match self {
            Value::Real(r) => *r,
            Value::Int(n) => *n as f64,
            _ => panic!("Value is not a real number: {:?}", self),
        }
    }
}

pub struct Interpreter {
    pub globals: HashMap<String, Value>,
    pub functions: HashMap<String, Statement>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Evaluate an expression and return a runtime value.
    pub fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Int(*n),
            Expr::Real(r) => Value::Real(*r),
            Expr::StringLiteral(s) => Value::Str(s.clone()),
            Expr::Variable(name) => self.globals.get(name).cloned().unwrap_or(Value::Int(0)),
            Expr::BinaryOp { left, op, right } => {
                let left_val = self.eval_expr(left);
                let right_val = self.eval_expr(right);
                match op {
                    BinaryOperator::Add => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                        (a, b) => Value::Real(a.as_real() + b.as_real()),
                    },
                    BinaryOperator::Subtract => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        (a, b) => Value::Real(a.as_real() - b.as_real()),
                    },
                    BinaryOperator::Multiply => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        (a, b) => Value::Real(a.as_real() * b.as_real()),
                    },
                    BinaryOperator::Divide => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
                        (a, b) => Value::Real(a.as_real() / b.as_real()),
                    },
                    BinaryOperator::Equal => {
                        if Interpreter::values_equal(&left_val, &right_val) {
                            Value::Int(1)
                        } else {
                            Value::Int(0)
                        }
                    },
                }
            }
            Expr::Call { name, args } => {
                let name_lower = name.to_lowercase();
                let arg_vals: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect();
                if name_lower == "writeln" {
                    if let Some(val) = arg_vals.get(0) {
                        match val {
                            Value::Int(n) => println!("{}", n),
                            Value::Real(r) => println!("{}", r),
                            Value::Str(s) => println!("{}", s),
                            Value::Record(rec) => println!("{:?}", rec),
                            Value::File(_) => println!("<file handle>"),
                        }
                    }
                    Value::Int(0)
                } 
                
                else if name_lower == "fopen" {
                    if arg_vals.len() != 2 {
                        panic!("fopen expects 2 arguments: filename and mode");
                    }
                    let filename = match &arg_vals[0] {
                        Value::Str(s) => s,
                        _ => panic!("fopen expects filename as string"),
                    };
                    let mode = match &arg_vals[1] {
                        Value::Str(s) => s.to_lowercase(),
                        _ => panic!("fopen expects mode as string"),
                    };
                    use std::fs::OpenOptions;
                    let file = if mode == "r" {
                        OpenOptions::new().read(true).open(filename)
                    } else if mode == "w" {
                        OpenOptions::new().write(true).create(true).open(filename)
                    } else {
                        panic!("fopen: unknown mode {}", mode)
                    };
                    match file {
                        Ok(f) => Value::File(Rc::new(RefCell::new(f))),
                        Err(e) => panic!("fopen error: {}", e),
                    }
                }
                // --- New built-in: fread(file) ---
                else if name_lower == "fread" {
                    if arg_vals.len() != 1 {
                        panic!("fread expects 1 argument: file handle");
                    }
                    let file_handle = match &arg_vals[0] {
                        Value::File(f) => f.clone(),
                        _ => panic!("fread expects a file handle"),
                    };
                    use std::io::Read;
                    let mut f = file_handle.borrow_mut();
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)
                        .expect("Failed to read file");
                    Value::Str(contents)
                }
                // --- New built-in: fwrite(file, data) ---
                else if name_lower == "fwrite" {
                    if arg_vals.len() != 2 {
                        panic!("fwrite expects 2 arguments: file handle and data");
                    }
                    let file_handle = match &arg_vals[0] {
                        Value::File(f) => f.clone(),
                        _ => panic!("fwrite expects a file handle as first argument"),
                    };
                    let data = match &arg_vals[1] {
                        Value::Str(s) => s,
                        _ => panic!("fwrite expects data as a string"),
                    };
                    use std::io::Write;
                    let mut f = file_handle.borrow_mut();
                    f.write_all(data.as_bytes()).expect("Failed to write to file");
                    Value::Int(0)
                }
                // --- New built-in: fclose(file) --- (optional)
                else if name_lower == "fclose" {
                    // In Rust, files are closed when they are dropped.
                    // For explicitness, we can remove the file from globals (if stored there) or simply do nothing.
                    // Here, we just pop the file handle off the stack.
                    //self.stack.pop();
                    Value::Int(0)
                }
                
                
                else if name_lower == "readln" {
                    let mut input = String::new();
                    print!("Input> ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input).expect("Failed to read line");
                    if let Ok(n) = input.trim().parse::<i64>() {
                        Value::Int(n)
                    } else if let Ok(r) = input.trim().parse::<f64>() {
                        Value::Real(r)
                    } else {
                        Value::Str(input.trim().to_string())
                    }
                } else if name_lower == "fact" {
                    if let Some(Value::Int(n)) = arg_vals.get(0) {
                        Value::Int(fact(*n))
                    } else {
                        panic!("fact expects an integer argument")
                    }
                } else if name_lower == "printresult" {
                    if let Some(val) = arg_vals.get(0) {
                        println!("Bonjour");
                        match val {
                            Value::Int(n) => println!("{}", n),
                            Value::Real(r) => println!("{}", r),
                            Value::Str(s) => println!("{}", s),
                            Value::Record(rec) => println!("{:?}", rec),
                            Value::File(_) => println!("<file handle>"),
                        }
                    }
                    Value::Int(0)
                } else if name_lower == "sin" {
                    if arg_vals.len() != 1 {
                        panic!("sin expects one argument");
                    }
                    match arg_vals[0] {
                        Value::Real(r) => Value::Real(r.sin()),
                        Value::Int(n) => Value::Real((n as f64).sin()),
                        _ => panic!("sin expects a numeric argument"),
                    }
                } else if name_lower == "cos" {
                    if arg_vals.len() != 1 {
                        panic!("cos expects one argument");
                    }
                    match arg_vals[0] {
                        Value::Real(r) => Value::Real(r.cos()),
                        Value::Int(n) => Value::Real((n as f64).cos()),
                        _ => panic!("cos expects a numeric argument"),
                    }
                } else if name_lower == "uppercase" {
                    if arg_vals.len() != 1 {
                        panic!("uppercase expects one argument");
                    }
                    match &arg_vals[0] {
                        Value::Str(s) => Value::Str(s.to_uppercase()),
                        _ => panic!("uppercase expects a string argument"),
                    }
                } else {
                    // User-defined procedures/functions.
                    if let Some(decl) = self.functions.get(&name_lower).cloned() {
                        match decl {
                            Statement::FunctionDecl { name: _, params, body } |
                            Statement::ProcedureDecl { name: _, params, body } => {
                                if params.len() != args.len() {
                                    panic!(
                                        "Parameter count mismatch in call to {}: expected {}, got {}",
                                        name, params.len(), args.len()
                                    );
                                }
                                let backup = self.globals.clone();
                                for (i, (param_name, _)) in params.iter().enumerate() {
                                    let evaluated_args: Vec<Value> = args.iter().map(|arg| self.eval_expr(arg)).collect();
                                    self.globals.insert(param_name.clone(), evaluated_args[i].clone());
                                }
                                let result = self.execute_statement(&body);
                                self.globals = backup;
                                result
                            },
                            _ => panic!("Expected a function/procedure declaration for {}", name),
                        }
                    } else {
                        panic!("Undefined procedure or function: {}", name);
                    }
                }
            }
            Expr::RecordLiteral(fields) => {
                let mut rec = HashMap::new();
                for (field_name, field_expr) in fields {
                    rec.insert(field_name.clone(), self.eval_expr(field_expr));
                }
                Value::Record(rec)
            }
            Expr::FieldAccess { record, field } => {
                let rec_val = self.eval_expr(record);
                match rec_val {
                    Value::Record(rec) => rec.get(field).cloned().unwrap_or(Value::Int(0)),
                    _ => panic!("Field access on non-record value"),
                }
            }
        }
    }

    /// Execute a statement and return a runtime value.
    pub fn execute_statement(&mut self, stmt: &Statement) -> Value {
        match stmt {
            Statement::Assignment { variable, expr } => {
                let val = self.eval_expr(expr);
                self.globals.insert(variable.clone(), val.clone());
                val
            }
            Statement::Compound(stmts) => {
                let mut last_val = Value::Int(0);
                for s in stmts {
                    last_val = self.execute_statement(s);
                }
                last_val
            }
            Statement::If { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expr(condition);
                if let Value::Int(n) = cond_val {
                    if n != 0 {
                        self.execute_statement(then_branch)
                    } else if let Some(else_stmt) = else_branch {
                        self.execute_statement(else_stmt)
                    } else {
                        Value::Int(0)
                    }
                } else {
                    Value::Int(0)
                }
            }
            Statement::While { condition, body } => {
                let mut last_val = Value::Int(0);
                loop {
                    let cond_val = self.eval_expr(condition);
                    if let Value::Int(n) = cond_val {
                        if n == 0 { break; }
                    } else { break; }
                    last_val = self.execute_statement(body);
                }
                last_val
            }
            Statement::For { variable, start, end, direction, body } => {
                let start_val = self.eval_expr(start);
                let end_val = self.eval_expr(end);
                let mut index = match start_val {
                    Value::Int(n) => n,
                    _ => panic!("For loop start must be an integer"),
                };
                let end_int = match end_val {
                    Value::Int(n) => n,
                    _ => panic!("For loop end must be an integer"),
                };
                self.globals.insert(variable.clone(), Value::Int(index));
                match direction {
                    ForDirection::To => {
                        while index <= end_int {
                            self.execute_statement(body);
                            index += 1;
                            self.globals.insert(variable.clone(), Value::Int(index));
                        }
                    }
                    ForDirection::DownTo => {
                        while index >= end_int {
                            self.execute_statement(body);
                            index -= 1;
                            self.globals.insert(variable.clone(), Value::Int(index));
                        }
                    }
                }
                Value::Int(0)
            }
            Statement::RepeatUntil { body, condition } => {
                loop {
                    self.execute_statement(body);
                    let cond_val = self.eval_expr(condition);
                    if let Value::Int(n) = cond_val {
                        if n != 0 { break; }
                    } else {
                        break;
                    }
                }
                Value::Int(0)
            }
            Statement::FunctionDecl { name, params: _, body } => {
                self.functions.insert(name.to_lowercase(), Statement::FunctionDecl {
                    name: name.clone(),
                    params: Vec::new(), // Not used in interpreter calls
                    body: body.clone(),
                });
                Value::Int(0)
            }
            Statement::ProcedureDecl { name, params: _, body } => {
                self.functions.insert(name.to_lowercase(), Statement::ProcedureDecl {
                    name: name.clone(),
                    params: Vec::new(), // Not used in interpreter calls
                    body: body.clone(),
                });
                Value::Int(0)
            }
            Statement::ProcedureCall { name, args } => {
                let name_lower = name.to_lowercase();
                // Handle built-in procedures (writeln, readln, etc.) separately...
                if name_lower == "writeln"
                    || name_lower == "readln"
                    || name_lower == "fact"
                    || name_lower == "sin"
                    || name_lower == "cos"
                    || name_lower == "uppercase"
                    || name_lower == "addreal"
                    || name_lower == "printreal"
                {
                    self.eval_expr(&Expr::Call { name: name.clone(), args: args.clone() })
                } else {
                    if let Some(decl) = self.functions.get(&name_lower).cloned() {
                        match decl {
                            Statement::FunctionDecl { name: _, params, body } |
                            Statement::ProcedureDecl { name: _, params, body } => {
                                if params.len() != args.len() {
                                    panic!(
                                        "Parameter count mismatch in call to {}: expected {}, got {}",
                                        name, params.len(), args.len()
                                    );
                                }
                                // **Step 1:** Evaluate all actual arguments first.
                                let evaluated_args: Vec<Value> =
                                    args.iter().map(|arg| self.eval_expr(arg)).collect();
            
                                // **Step 2:** Backup the current global environment.
                                let backup = self.globals.clone();
            
                                // **Step 3:** Bind each formal parameter to its corresponding evaluated argument.
                                for (i, (param_name, _)) in params.iter().enumerate() {
                                    self.globals.insert(param_name.clone(), evaluated_args[i].clone());
                                }
            
                                // **Step 4:** Execute the procedure body.
                                let result = self.execute_statement(&body);
            
                                // **Step 5:** Restore the original globals.
                                self.globals = backup;
                                result
                            },
                            _ => panic!("Expected a function/procedure declaration for {}", name),
                        }
                    } else {
                        panic!("Undefined procedure or function: {}", name);
                    }
                }
            }
            
            Statement::Case { expr, branches, else_branch } => {
                let controlling_val = self.eval_expr(expr);
                let mut matched = false;
                for branch in branches {
                    let label_val = self.eval_expr(&branch.label);
                    if Interpreter::values_equal(&controlling_val, &label_val) {
                        matched = true;
                        return self.execute_statement(&branch.stmt);
                    }
                }
                if !matched {
                    if let Some(else_stmt) = else_branch {
                        return self.execute_statement(else_stmt);
                    }
                }
                Value::Int(0)
            }
        }
    }

    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Real(x), Value::Real(y)) => (x - y).abs() < 1e-9,
            (Value::Str(s1), Value::Str(s2)) => s1 == s2,
            (Value::Int(x), Value::Real(y)) | (Value::Real(y), Value::Int(x)) => (*x as f64 - *y).abs() < 1e-9,
            _ => false,
        }
    }
}

fn fact(n: i64) -> i64 {
    if n == 0 { 1 } else { n * fact(n - 1) }
}

/// Process variable declarations (var section) in a Program and initialize globals.
pub fn process_declarations(program: &Program, globals: &mut HashMap<String, Value>) {
    for decl in &program.declarations {
        if let Declaration::VarDeclaration { names, typ } = decl {
            let default_value = match typ {
                PascalType::Integer => Value::Int(0),
                PascalType::Real => Value::Real(0.0),
                PascalType::String => Value::Str(String::new()),
                PascalType::Boolean => Value::Int(0),
                PascalType::RecordType(_) => Value::Record(HashMap::new()),
            };
            for name in names {
                globals.insert(name.clone(), default_value.clone());
            }
        }
    }
}
