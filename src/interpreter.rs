extern crate libc;
use crate::lexer;
use crate::Args;

use std::process;
use std::collections::HashMap;

pub struct Interpreter;

impl Interpreter {
    pub fn expect(stack: &Vec<(lexer::Types, u64)>, expected_types: Vec<lexer::Types>) {
        let mut ctr = 1;
        for expected in expected_types {
            let stack_len = stack.len();
            if stack_len < 1 {
                println!("Not enough elements on the stack");
                process::exit(1);
            }
            if expected == lexer::Types::Unknown {
                continue;
            }
            let value = &stack[stack_len-ctr];
            if expected != value.0 {
                println!("Expected {:?} but got {:?}", expected, value.0);
                process::exit(1);
            }
            ctr += 1;
        }
    }

    pub fn run(program: (Vec<lexer::OpCodes>,
                         HashMap<usize, usize>,
                         HashMap<String,
                         lexer::Procedure>, HashMap<String, usize>,
                        ),
               args: Args,
               ) {
        let tokens = program.0;
        let scopes = program.1;
        let procedures = program.2;
        let labels = program.3;
        let mut ip = 0;
        let mut stack: Vec<(lexer::Types, u64)> = Vec::new();
        let mut memory: Vec<u64> = Vec::new();
        let mut memory_map: HashMap<u64, usize> = HashMap::new();
        let mut memory_offset = 0;
        let mut return_stack: Vec<(usize, String)> = Vec::new();

        let main = procedures.get("main");
        if main.is_none() {
            println!("No main procedure was provided");
            process::exit(1);
        }
        ip = main.unwrap().Location;

        while ip < tokens.len() {
            let token = &tokens[ip];
            if args.debug {
                println!("Ip: {} Token: {:?}", ip, token);
            }
            match token {
                lexer::OpCodes::Syscalls(syscall) => {
                    match syscall { // TODO: Implement Open syscall
                        lexer::Syscalls::Open => {
                            Interpreter::expect(
                                &stack,
                                vec![],
                            );
                        },
                        lexer::Syscalls::Read => {
                            Interpreter::expect(
                                &stack,
                                vec![lexer::Types::Int, lexer::Types::Int],
                            );
                            let fd = stack.pop().unwrap();
                            let buffer_len = stack.pop().unwrap();
                            if buffer_len.1 > 199 {
                                println!("Currently the read implementation only supports 200 bytes of length");
                                process::exit(1);
                            }
                            let mut buffer: [u8; 200] = [0; 200];

                            unsafe {
                                libc::read(fd.1 as i32, buffer.as_mut_ptr() as *mut libc::c_void, buffer_len.1 as usize);
                            }
                            let base_addr = memory_offset;
                            for byte in buffer {
                                memory.push(byte as u64);
                                memory_map.insert(memory_offset, memory.len());
                                memory_offset += 1;
                            }
                            memory.push(0);
                            memory_map.insert(memory_offset, memory.len());
                            memory_offset += 1;
                            stack.push((lexer::Types::Pointer, base_addr));
                        },
                        lexer::Syscalls::Write => {
                            Interpreter::expect(
                                &stack,
                                vec![lexer::Types::Int, lexer::Types::Pointer, lexer::Types::Int],
                            );
                            let fd = stack.pop().unwrap();
                            let buffer_ptr = stack.pop().unwrap();
                            let buffer_len = stack.pop().unwrap();
                            let mut buffer = String::new();
                            let mut offset = 0;
                            while buffer.len() < buffer_len.1 as usize {
                                let addr = memory_map.get(&(buffer_ptr.1 + offset));
                                if addr.is_none() {
                                    buffer = buffer + &String::from_utf8(vec![0]).unwrap();
                                } else {
                                    buffer = buffer + &String::from_utf8(vec![memory[*addr.unwrap()-1] as u8]).unwrap();
                                }
                                offset += 1;
                            }
                            unsafe {
                                let status = libc::write(fd.1.try_into().unwrap(), buffer.as_ptr() as *const libc::c_void, buffer_len.1.try_into().unwrap());
                                if status == -1 {
                                    println!("Write error");
                                    process::exit(1);
                                }
                            }
                        },
                    }
                },
                lexer::OpCodes::Goto(label) => {
                    let _label = labels.get(label);
                    if _label.is_none() {
                        println!("Unknown label: {}", label);
                        process::exit(1);
                    }
                    ip = *_label.unwrap();
                },
                lexer::OpCodes::Drop => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Unknown],
                    );
                    stack.pop().unwrap();
                },
                lexer::OpCodes::Dup => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Unknown],
                    );
                    let value = stack.pop().unwrap();
                    stack.push(value.clone());
                    stack.push(value);
                },
                lexer::OpCodes::Swap => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Unknown, lexer::Types::Unknown],
                    );
                    let value1 = stack.pop().unwrap();
                    let value2 = stack.pop().unwrap();
                    stack.push(value1);
                    stack.push(value2);
                },
                lexer::OpCodes::Rot => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Unknown, lexer::Types::Unknown, lexer::Types::Unknown],
                    );
                    let value1 = stack.pop().unwrap();
                    let value2 = stack.pop().unwrap();
                    let value3 = stack.pop().unwrap();
                    stack.push(value1);
                    stack.push(value2);
                    stack.push(value3);
                },
                lexer::OpCodes::Ident(ident) => {
                    if procedures.get(ident).is_some() {
                        let procedure = procedures.get(ident).unwrap();
                        return_stack.push((ip, procedure.Proc.clone()));
                        Interpreter::expect(
                            &stack,
                            procedure.ParameterTypes.clone(),
                        );
                        ip = procedure.Location;
                        continue;
                    } else {
                        println!("Unknown ident: {}", ident);
                        process::exit(1);
                    }
                },
                lexer::OpCodes::Return => {
                    let return_location = return_stack.pop();
                    if return_location.is_none() {
                        ip = tokens.len()-1;
                    } else {
                        let procedure = procedures.get(&return_location.as_ref().unwrap().1).unwrap();
                        Interpreter::expect(
                            &stack,
                            procedure.ReturnTypes.clone(),
                        );
                        ip = return_location.unwrap().0;
                    }
                },
                lexer::OpCodes::Load => {
                    let addr = stack.pop();
                    if addr.is_none() {
                        println!("Stack underflow");
                        process::exit(1);
                    }
                    let value = memory_map.get(&addr.unwrap().1);
                    if value.is_none() {
                        stack.push((lexer::Types::Int, 0));
                    } else {
                        stack.push((lexer::Types::Int, memory[*value.unwrap()-1]));
                    }
                },
                lexer::OpCodes::Store => {
                    let addr = stack.pop();
                    let value = stack.pop();
                    if addr.is_none() | value.is_none() {
                        println!("Stack underflow");
                        process::exit(1);
                    }
                    let addr_map = memory_map.get(&addr.as_ref().unwrap().1);
                    if addr_map.is_none() {
                        memory.push(value.unwrap().1);
                        memory_map.insert(addr.as_ref().unwrap().1, memory.len());
                        memory_offset += 1;
                    } else {
                        memory[*addr_map.unwrap()] = value.unwrap().1;
                    }
                },
                lexer::OpCodes::Cast(value_type) => {
                    let value = stack.pop();
                    if value.is_none() {
                        println!("Stack underflow");
                        process::exit(1);
                    }
                    match value_type {
                        lexer::Types::Int => {
                            stack.push((lexer::Types::Int, value.unwrap().1));
                        },
                        lexer::Types::Pointer => {
                            stack.push((lexer::Types::Pointer, value.unwrap().1));
                        },
                        lexer::Types::Bool => {
                            stack.push((lexer::Types::Bool, value.unwrap().1));
                        },
                        _ => {},
                    }
                },
                lexer::OpCodes::Push(push_type, push_int, push_str) => {
                    match push_type {
                        lexer::Types::Int => {
                            stack.push((lexer::Types::Int, *push_int));
                        },
                        lexer::Types::String => {
                            let bytes = push_str.bytes().collect::<Vec<u8>>();
                            let base_addr = memory_offset;
                            let mut flag = "";
                            for byte in bytes {
                                if flag == "escape" {
                                    let character = String::from_utf8(vec![byte]).unwrap();
                                    if character == "\\" {
                                        memory.push(92);
                                        memory_map.insert(memory_offset, memory.len());
                                        memory_offset += 1;
                                        continue;
                                    } else if character == "n" {
                                        memory.push(10);
                                        memory_map.insert(memory_offset, memory.len());
                                        memory_offset += 1;
                                        continue;
                                    }
                                    flag = "";
                                } else if String::from_utf8(vec![byte]).unwrap().as_str() == "\\" {
                                    flag = "escape";
                                    continue;
                                }
                                memory.push(byte.into());
                                memory_map.insert(memory_offset, memory.len());
                                memory_offset += 1;
                            }
                            memory.push(0);
                            memory_map.insert(memory_offset, memory.len());
                            memory_offset += 1;
                            stack.push((lexer::Types::Pointer, base_addr));
                        },
                        lexer::Types::Bool => {
                            stack.push((lexer::Types::Bool, *push_int));
                        },
                        _ => { /* This is only here because you cant push raw pointers */ },
                    }
                },
                lexer::OpCodes::Print => {
                    let value = stack.pop();
                    if value.is_none() {
                        println!("Print: Expected one argument on the stack");
                        process::exit(1);
                    }
                    let value = value.unwrap();
                    match value.0 {
                        lexer::Types::Int => { println!("{}", value.1); },
                        lexer::Types::Pointer => { println!("{}", value.1); },
                        lexer::Types::Bool => {
                            if value.1 == 1 {
                                println!("true");
                            } else {
                                println!("false");
                            }
                        },
                        _ => {  },
                    }
                },
                lexer::OpCodes::Arithmetic(operator) => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Int, lexer::Types::Int]
                    );
                    let value1 = stack.pop();
                    let value2 = stack.pop();
                    match operator {
                        lexer::Arithmetic::Plus => {
                            stack.push((lexer::Types::Int, value2.unwrap().1 + value1.unwrap().1));
                        },
                        lexer::Arithmetic::Minus => {
                            stack.push((lexer::Types::Int, value2.unwrap().1 - value1.unwrap().1));
                        },
                        lexer::Arithmetic::Mul => {
                            stack.push((lexer::Types::Int, value2.unwrap().1 * value1.unwrap().1));
                        },
                        lexer::Arithmetic::Div => {
                            stack.push((lexer::Types::Int, value2.unwrap().1 / value1.unwrap().1));
                        },
                        _ => {},
                    }
                },
                lexer::OpCodes::Equality(operator) => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Int, lexer::Types::Int]
                    );
                    let value1 = stack.pop();
                    let value2 = stack.pop();
                    match operator {
                        lexer::Equality::Equal => {
                            if value1.unwrap().1 == value2.unwrap().1 {
                                stack.push((lexer::Types::Bool, 1));
                            } else {
                                stack.push((lexer::Types::Bool, 0));
                            }
                        },
                        lexer::Equality::Bigger => {
                            if value1.unwrap().1 < value2.unwrap().1 {
                                stack.push((lexer::Types::Bool, 1));
                            } else {
                                stack.push((lexer::Types::Bool, 0));
                            }
                        },
                        lexer::Equality::Smaller => {
                            if value1.unwrap().1 > value2.unwrap().1 {
                                stack.push((lexer::Types::Bool, 1));
                            } else {
                                stack.push((lexer::Types::Bool, 0));
                            }
                        },
                        _ => {},
                    }
                },
                lexer::OpCodes::If => {
                    Interpreter::expect(
                        &stack,
                        vec![lexer::Types::Bool]
                    );
                    let value = stack.pop();

                    if value.unwrap().1 == 0 {
                        ip = scopes.get(&(ip+1)).unwrap()-1;
                    }
                },
                _ => {},
            }
            ip += 1;
        }
        println!("stack: {:?}", stack);
        println!("memory: {:?}", memory);
    }
}


