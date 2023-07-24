#![allow(unused)]
extern crate arsenal_globals;

pub mod tokenizer;

use core::panic;
use std::{collections::HashMap, str::FromStr, io::Read};

use arsenal_globals::SysCalls;
use strum::VariantNames;

enum DataObject {
    Byte(u8),
    LabelRequest(String, u32, u32, u64),
    SizeRequest(String, u32, u32),
}

pub fn new_parse(data: Vec<u8>) -> Option<Vec<u8>> {
    let as_str = match String::from_utf8(data) {
        Ok(str) => str,
        Err(_) => return None,
    };
    let mut tokens = tokenizer::tokenize::<tokenizer::ArsenalToken>(&as_str);
    let mut tokens: std::iter::Peekable<std::slice::Iter<'_, tokenizer::ArsenalToken>> = tokens.iter().peekable();

    let mut labels = HashMap::<String, u64>::new();
    let mut sizes = HashMap::<String, u64>::new();
    let mut data: Vec<DataObject> = vec![];
    let mut bytes_count: usize = 0;

    while let Some(token) = &tokens.next() {
        use tokenizer::ArsenalToken::*;
        match *token {
            LineEnd(_) => continue,
            Label(_) => {
                if let Some(Identifier(name)) = tokens.next() {
                    labels.insert(name.clone(), bytes_count as u64);
                    if let Some(Selection(_)) = tokens.next() {} else {
                        panic!("expected line ending after identifier");
                    }
                } else {
                    panic!("expected an identifier after label");
                }
            },
            Identifier(name) => {
                if let Ok(instruction) = arsenal_globals::Instructions::from_str(name) {
                    data.push(DataObject::Byte(instruction as u8));
                    data.push(DataObject::Byte((instruction as u16).wrapping_shr(8) as u8));
                    bytes_count += 2;

                    parse_arg_sequence(&mut tokens, &mut data, &mut bytes_count);
                }
            },
            OpenParen(_) => {
                match tokens.next() {
                    Some(ClosedParen(_)) => {
                        let Identifier(name) = tokens.next().expect(&format!("expected identifier after (), got nothing")) else { panic!("expected identifier after ()"); };
                        assert!(matches!(tokens.next().expect("() name expression require setter =, size cannot be inferred."), VarAssignment(_)));
                        labels.insert(name.clone(), bytes_count as u64);

                        parse_arg_sequence(&mut tokens, &mut data, &mut bytes_count);
                    },
                    Some(Identifier(size)) => {
                        assert!(matches!(tokens.next().expect("expected ) after (capture"), ClosedParen(_)));
                        let Identifier(name) = tokens.next().expect(&format!("expected identifier after (capture), got nothing")) else { panic!("expected identifier after (capture)"); };

                        assert!(matches!(tokens.next().expect("(capture) name expression require setter =, size cannot be inferred."), VarAssignment(_)));
                        labels.insert(name.clone(), bytes_count as u64);
                        let current_count = bytes_count;
                        parse_arg_sequence(&mut tokens, &mut data, &mut bytes_count);
                        sizes.insert(size.clone(), (bytes_count - current_count) as u64);
                    },
                    Some(Number(num)) => {},
                    Some(Hex(num)) => {},

                    Some(unexpected) => {panic!("expected number, identifier, or ) after (, not {:?}", unexpected)}
                    None => panic!("Expected token after ("),
                }
            }

            SpecialIdentifier(_) => todo!(),

            Whitespace(_) => unreachable!(),

            unexpected => panic!("unexpected token {:?}", unexpected),
        }
    }

    let mut return_data: Vec<u8> = vec![];

    for obj in data {
        match obj {
            DataObject::Byte(x) => return_data.push(x),
            DataObject::LabelRequest(name, start, stop, inc) => {
                let location = *labels.get(&name).expect(&format!("unknown label {}", name)) + inc;
                for offset in (start as u32)..=(stop as u32) {
                    unsafe {
                        return_data.push(*(((&location as *const u64 as u64) + (offset) as u64) as *const u64 as *const u8));
                    }
                }
            },
            DataObject::SizeRequest(name, start, stop) => {
                let location = *sizes.get(&name).expect(&format!("unknown size id {}", name));
                for offset in (start as u32)..=(stop as u32) {
                    unsafe {
                        return_data.push(*(((&location as *const u64 as u64) + (offset) as u64) as *const u64 as *const u8));
                    }
                }
            },
        }
    }

    assert!(return_data.len() == bytes_count);

    Some(return_data)
}

fn parse_arg_sequence(tokens: &mut std::iter::Peekable<std::slice::Iter<'_, tokenizer::ArsenalToken>>, data: &mut Vec<DataObject>, bytes_count: &mut usize) {
    'main: while let Some(arg) = tokens.next() {
        use tokenizer::ArsenalToken::*;
        match arg {
            LineEnd(_) => break 'main,
            Separator(_) => todo!(),
            Identifier(syscall) => {
                if let Ok(id) = SysCalls::from_str(&syscall) {
                    data.push(DataObject::Byte(id as u8));
                    *bytes_count += 1;
                } else {
                    panic!("unknown syscall {}", syscall);
                }
            },
            Hex(num) => {
                let num = u8::from_str_radix(num, 16).unwrap();
                data.push(DataObject::Byte(num));
                *bytes_count += 1;
            },
            Number(num) => {
                let num = num.parse().unwrap();
                data.push(DataObject::Byte(num));
                *bytes_count += 1;
            },
            StringLiteral(lit) => {
                let to_put = lit[1..lit.len()-1].to_string();
                for byte in to_put.as_bytes() {
                    data.push(DataObject::Byte(*byte));
                    *bytes_count += 1;
                }
            },
            IDGrab(_) => {
                let Identifier(name) = tokens.next().expect(&format!("expected identifier after &, got nothing")) else { panic!("expected identifier after &"); };
                let (mut start, mut stop, mut extent) = (0, 7, 0);

                if let Some(Selection(_)) = tokens.peek() {
                    tokens.next();
                    let Number(start_str) = tokens.next().expect("expected a number after (&name:)") else { panic!("expected a number after (&name:)"); };
                    start = start_str.parse().expect("(&name:) must be followed by a number.");
                    if let Some(Range(_)) = tokens.peek() {
                        tokens.next();
                        let Number(stop_str) = tokens.next().expect("expected a number after (&name:num->)") else { panic!("expected a number after (&name:num->)"); };
                        stop = stop_str.parse().expect("(&name:num->) must be followed by a number.");
                        if let Some(Shift(_)) = tokens.peek() {
                            tokens.next();
                            let Number(extent_str) = tokens.next().expect("expected a number after (&name:num->num=>)") else { panic!("expected a number after (&name:num->num=>)"); };
                            extent = extent_str.parse().expect("(&name:num->num=>) must be followed by a number.");
                        }
                    }
                }

                *bytes_count += ((stop - start) + 1);
                data.push(DataObject::LabelRequest(name.clone(), start as u32, stop as u32, extent as u64));
            },

            SizeGrab(_) => {
                let Identifier(name) = tokens.next().expect(&format!("expected identifier after $, got nothing")) else { panic!("expected identifier after $"); };
                let (mut start, mut stop, mut extent) = (0, 7, 0);

                if let Some(Selection(_)) = tokens.peek() {
                    tokens.next();
                    let Number(start_str) = tokens.next().expect("expected a number after ($name:)") else { panic!("expected a number after $name:)"); };
                    start = start_str.parse().expect("($name:) must be followed by a number.");
                    if let Some(Range(_)) = tokens.peek() {
                        tokens.next();
                        let Number(stop_str) = tokens.next().expect("expected a number after ($name:num->)") else { panic!("expected a number after ($name:num->)"); };
                        stop = stop_str.parse().expect("($name:num->) must be followed by a number.");
                    }
                }

                *bytes_count += ((stop - start) + 1);
                data.push(DataObject::SizeRequest(name.clone(), start as u32, stop as u32));
            },

            NumericSlice(_) => {
                let Number(num) = tokens.next().expect(&format!("expected number after #, got nothing")) else { panic!("expected number after #"); };
                let num: i64 = num.parse().unwrap();
                let (mut start, mut stop) = (0, 7);

                if let Some(Selection(_)) = tokens.peek() {
                    tokens.next();
                    let Number(start_str) = tokens.next().expect("expected a number after (#num:)") else { panic!("expected a number after #num:)"); };
                    start = start_str.parse().expect("(#num:) must be followed by a number.");
                    if let Some(Range(_)) = tokens.peek() {
                        tokens.next();
                        let Number(stop_str) = tokens.next().expect("expected a number after ($name:num->)") else { panic!("expected a number after ($name:num->)"); };
                        stop = stop_str.parse().expect("($name:num->) must be followed by a number.");
                    }
                }

                *bytes_count += ((stop - start) + 1);
                for offset in start..=stop {
                    unsafe {
                        data.push(DataObject::Byte(*((&num as *const _ as u64 + offset as u64) as *const u8)));
                    }
                }
            }

            Whitespace(_) => unreachable!(),
            token => panic!("unexpected token {:?}", token),
        }
    }
}
