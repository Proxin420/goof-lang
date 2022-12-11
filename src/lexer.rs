use std::fs::File;
use std::process;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Types {
    Int,
    Bool,
    String,
    Pointer,
    Unknown,
}

#[derive(Debug)]
pub enum Arithmetic {
    Plus,
    Minus,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Equality {
    Equal,
    Bigger,
    Smaller,
}

#[derive(Debug)]
pub struct Procedure {
    pub Proc: String,
    pub Location: usize,
    pub ParameterTypes: Vec<Types>,
    pub ReturnTypes: Vec<Types>,
}

#[derive(Debug)]
pub enum ScopeTypes {
    Proc,
    If,
}

#[derive(Debug)]
pub enum Syscalls {
    Read,
    Write,
    Open,
}

#[derive(Debug)]
pub enum OpCodes {
    Push(Types, u64, String),

    Arithmetic(Arithmetic),
    Equality(Equality),

    Cast(Types),

    Ident(String),

    Goto(String),

    Syscalls(Syscalls),

    Return,

    Load,
    Store,

    Dup,
    Swap,
    Rot,
    Drop,

    If,
    EOP,

    Print,
}

pub struct Lexer;

impl Lexer {
    pub fn tokenize(file: &str) -> (Vec<OpCodes>, HashMap<usize, usize>, HashMap<String, Procedure>, HashMap<String, usize>) {
        let mut tokens: Vec<OpCodes> = Vec::new();
        let fd = File::open(file);
        if fd.is_err() {
            println!("Failed to open file: {}", file);
            process::exit(1);
        }
        let buffer = BufReader::new(fd.unwrap());
        let mut token = String::new();

        let mut scope: Vec<(usize, ScopeTypes)> = Vec::new();
        let mut scopes: HashMap<usize, usize> = HashMap::new();
        let mut procedures: (HashMap<String, Procedure>, Vec<Procedure>) = (HashMap::new(), Vec::new());
        let mut labels: HashMap<String, usize> = HashMap::new();

        let mut flag = "";

        for byte in buffer.bytes() {
            let character = String::from_utf8(vec![byte.unwrap()]).unwrap();
            if flag == "string" {
                if character == "\"" {
                    tokens.push(OpCodes::Push(Types::String, 0, token));
                    token = String::new();
                    flag = "";
                    continue;
                }
                token = token + &character;
                continue;
            } else if flag == "use" {
                if character == " " || character == "\n" {
                    let used = Lexer::tokenize(&token);
                    tokens.extend(used.0);
                    scopes.extend(used.1);
                    procedures.0.extend(used.2);
                    labels.extend(used.3);
                    flag = "";
                    token = String::new();
                    continue;
                }
                token = token + &character;
                continue;
            } else if flag == "proc" {
                if character == " " || character == "\n" {
                    procedures.1.push(
                        Procedure {
                            Proc: token,
                            Location: 0,
                            ParameterTypes: Vec::new(),
                            ReturnTypes: Vec::new(),
                        }
                    );
                    flag = "ParameterTypes";
                    token = String::new();
                    continue;
                }
                token = token + &character;
                continue;
            } else if flag == "ParameterTypes" {
                if character == " " || character == "\n" {
                    let mut parameter_type = Types::Int;
                    match token.as_str() {
                        "int" => {
                            parameter_type = Types::Int;
                        },
                        "bool" => {
                            parameter_type = Types::Bool;
                        },
                        "ptr" => {
                            parameter_type = Types::Pointer;
                        },
                        _ => {},
                    }
                    let proc_len = procedures.1.len()-1;
                    procedures.1[proc_len].ParameterTypes.push(parameter_type);
                    token = String::new();
                    continue;
                } else if character == ":" {
                    flag = "ReturnTypes";
                    token = String::new();
                    continue;
                }
                token = token + &character;
                continue;
            } else if flag == "ReturnTypes" {
                if character == " " || character == "\n" {
                    if token == "in" {
                        let proc_len = procedures.1.len()-1;
                        procedures.0.insert(
                            procedures.1[proc_len].Proc.clone(),
                            Procedure {
                                Proc: procedures.1[proc_len].Proc.clone(),
                                Location: tokens.len(),
                                ParameterTypes: procedures.1[proc_len].ParameterTypes.clone(),
                                ReturnTypes: procedures.1[proc_len].ReturnTypes.clone(),
                            }
                        );
                        scope.push((tokens.len(), ScopeTypes::Proc));
                        flag = "";
                        token = String::new();
                        continue;
                    } else if token == "" {
                        continue;
                    }
                    let mut return_type = Types::Int;
                    match token.as_str() {
                        "int" => {
                            return_type = Types::Int;
                        },
                        "bool" => {
                            return_type = Types::Bool;
                        },
                        "ptr" => {
                            return_type = Types::Pointer;
                        },
                        _ => {  },
                    }
                    let proc_len = procedures.1.len()-1;
                    procedures.1[proc_len].ReturnTypes.push(return_type);
                    token = String::new();
                    continue;
                }
                token = token + &character;
                continue;
            } else if flag == "goto" {
                if character == " " || character == "\n" {
                    tokens.push(OpCodes::Goto(token));
                    token = String::new();
                    flag = "";
                    continue;
                }
                token = token + &character;
                continue;
            }
            match character.as_str() {
                "\"" => {
                    flag = "string";
                },
                " " | "\n" => {
                    if token != "" {
                        match token.as_str() {
                            "+" => {
                                tokens.push(OpCodes::Arithmetic(Arithmetic::Plus));
                            },
                            "-" => {
                                tokens.push(OpCodes::Arithmetic(Arithmetic::Minus));
                            },
                            "*" => {
                                tokens.push(OpCodes::Arithmetic(Arithmetic::Mul));
                            },
                            "/" => {
                                tokens.push(OpCodes::Arithmetic(Arithmetic::Div));
                            },
                            "." => {
                                tokens.push(OpCodes::Print);
                            },
                            "=" => {
                                tokens.push(OpCodes::Equality(Equality::Equal));
                            },
                            ">" => {
                                tokens.push(OpCodes::Equality(Equality::Bigger));
                            },
                            "<" => {
                                tokens.push(OpCodes::Equality(Equality::Smaller));
                            },
                            "if" => {
                                tokens.push(OpCodes::If);
                                scope.push((tokens.len(), ScopeTypes::If));
                            },
                            "end" => {
                                let start_scope = scope.pop();
                                if start_scope.is_none() {
                                    println!("Unexpected token: {}", token);
                                    process::exit(1);
                                }
                                let start_scope = start_scope.unwrap();
                                scopes.insert(start_scope.0, tokens.len());
                                match start_scope.1 {
                                    ScopeTypes::Proc => {
                                        tokens.push(OpCodes::Return);
                                    },
                                    _ => {},
                                }
                            },
                            "true" => {
                                tokens.push(OpCodes::Push(Types::Bool, 1, String::new()));
                            },
                            "false" => {
                                tokens.push(OpCodes::Push(Types::Bool, 0, String::new()));
                            },
                            "(int)" => {
                                tokens.push(OpCodes::Cast(Types::Int));
                            },
                            "(ptr)" => {
                                tokens.push(OpCodes::Cast(Types::Pointer));
                            },
                            "(bool)" => {
                                tokens.push(OpCodes::Cast(Types::Bool));
                            },
                            "load" => {
                                tokens.push(OpCodes::Load);
                            },
                            "store" => {
                                tokens.push(OpCodes::Store);
                            },
                            "use" => {
                                flag = "use";
                            },
                            "proc" => {
                                flag = "proc";
                            },
                            "dup" => {
                                tokens.push(OpCodes::Dup);
                            },
                            "swap" => {
                                tokens.push(OpCodes::Swap);
                            },
                            "rot" => {
                                tokens.push(OpCodes::Rot);
                            },
                            "drop" => {
                                tokens.push(OpCodes::Drop);
                            },
                            "goto" => {
                                flag = "goto";
                            },
                            "write" => {
                                tokens.push(OpCodes::Syscalls(Syscalls::Write));
                            },
                            "read" => {
                                tokens.push(OpCodes::Syscalls(Syscalls::Read));
                            },
                            "open" => {
                                tokens.push(OpCodes::Syscalls(Syscalls::Open));
                            },
                            _ => {
                                let int_token = token.parse::<u64>();
                                if int_token.is_ok() {
                                    tokens.push(OpCodes::Push(Types::Int, int_token.unwrap(), String::new()));
                                } else if token.ends_with(":") && token.len() > 1 {
                                    labels.insert(token, tokens.len()-1);
                                } else {
                                    tokens.push(OpCodes::Ident(token));
                                }
                            },
                        }
                        token = String::new();
                    }
                },
                _ => {token = token + &character},
            }
        }
        tokens.push(OpCodes::EOP);
        return (tokens, scopes, procedures.0, labels);
    }
}

