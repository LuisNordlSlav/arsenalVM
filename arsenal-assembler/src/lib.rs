#![allow(unused)]
extern crate arsenal_globals;

use core::panic;
use std::{collections::HashMap, str::FromStr, io::Read};

use arsenal_globals::SysCalls;
use strum::VariantNames;

const IDENTIFIER_STARTER: &str = "qwertyuiopasdfghjklzxcvbnm_QWERTYUIOPASDFGHJKLZXCVBNM";
const IDENTIFIER: &str = "qwertyuiopasdfghjklzxcvbnm_QWERTYUIOPASDFGHJKLZXCVBNM1234567890";

enum DataObject {
    Byte(u8),
    LabelRequest(String, u32, u32, u64),
    SizeRequest(String, u32, u32),
}

fn custom_split(input: &str) -> Vec<String> {
    enum State {
        Normal,
        InsideString,
    }

    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut state = State::Normal;
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        match (c, &state) {
            ('"', State::Normal) => { state = State::InsideString; current_word += "\""},
            ('"', State::InsideString) => { state = State::Normal; current_word += "\""},
            ('\\', State::InsideString) => {
                if let Some(char) = iter.peek() {
                    if char == &'"' {
                        current_word.push('"');
                        iter.next();
                    }
                }
            }
            (_, State::InsideString) => current_word.push(c),
            (',', State::Normal) => {
                if !current_word.is_empty() {
                    result.push(current_word.clone());
                    current_word.clear();
                }
            },
            (' ', State::Normal) => {
                if !current_word.is_empty() {
                    result.push(current_word.clone());
                    current_word.clear();
                }
            },
            (_, _) => current_word.push(c),
        }
    }

    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}

fn next_identifier(slice: &str) -> String {
    let mut result: String = "".to_string();
    let mut index = 0;
    loop {
        if let Some(char) = slice.chars().nth(index) {
            if IDENTIFIER.contains(char) {
                result += &char.to_string();
            } else {
                break;
            }
        } else {
            break;
        }
        index += 1;
    };
    result
}

fn next_sequence_of(acceptable: &str, slice: &str) -> String {
    let mut result: String = "".to_string();
    let mut index = 0;
    loop {
        if let Some(char) = slice.chars().nth(index) {
            if acceptable.contains(char) {
                result += &char.to_string();
            } else {
                break;
            }
        } else {
            break;
        }
        index += 1;
    };
    result
}

fn next_number(slice: &str) -> (isize, usize) {
    let mut result: String = "".to_string();
    let mut index = 0;
    let mut num_decimal_places = 0;
    if slice.starts_with(|x| {"+-".contains(x)}) {
        result += &slice.chars().nth(index).unwrap().to_string();
        index += 1;
    }
    loop {
        if let Some(char) = slice.chars().nth(index) {
            if "1234567890.".contains(char) {
                if &char.to_string() == "." {
                    if num_decimal_places == 1 {
                        break; 
                    }
                    num_decimal_places = 1;
                }
                result += &char.to_string();
            } else {
                break;
            }
        } else {
            break;
        }
        index += 1;
    };

    (result.parse().expect(&format!("tried to treat {} as a number", result)), result.len())
}

fn parse_data(data: &str, package: &mut Vec<DataObject>, length: &mut usize) {
    let data_blobs = custom_split(data.trim_end_matches(";").trim_start_matches(":"));
    for mut data in data_blobs {
        if data.starts_with("\"") {
            for char in data.to_string()[1..(data.to_string().len() - 1)].chars() {
                package.push(DataObject::Byte(char as u8));
                *length += 1;
            }
        } else if data.starts_with("&") {
            data = data[1..].to_string();
            let identifier = next_identifier(&data);
            data = data[identifier.len()..].trim_start()[1..].to_string();
            let start = next_number(&data);
            data = data[start.1..].to_string();
            let mut end = (0, 0);
            let mut shift = (0, 0);
            if data.starts_with("->") {
                data = data[2..].to_string();
                end = next_number(&data);
                data = data[end.1..].to_string();
                if data.starts_with("=>") {
                    data = data[2..].to_string();
                    shift = next_number(&data);
                    data = data[shift.1..].to_string();
                    println!("{}", shift.0);
                    // panic!("=> is currently unimplemented :(");
                }
            }
            package.push(DataObject::LabelRequest(identifier, start.0 as u32, end.0 as u32, shift.0 as u64));
            *length += ((end.0 + 1) - start.0) as usize;
        } else if data.starts_with("$") {
            data = data[1..].to_string();
            let identifier = next_identifier(&data);
            data = data[identifier.len()..].trim_start()[1..].to_string();
            let start = next_number(&data);
            data = data[start.1..].to_string();
            let mut end = (0, 0);
            if data.starts_with("->") {
                data = data[2..].to_string();
                end = next_number(&data);
                data = data[end.1..].to_string();
            }
            package.push(DataObject::SizeRequest(identifier, start.0 as u32, end.0 as u32));
            *length += ((end.0 + 1) - start.0) as usize;
        } else if data.starts_with(|x| { IDENTIFIER_STARTER.contains(x) }) {
            let syscall = next_identifier(&data);
            if let Ok(id) = SysCalls::from_str(&syscall) {
                package.push(DataObject::Byte(id as u8));
                *length += 1;
            } else {
                panic!("unknown syscall {}", syscall);
            }
        } else if data.starts_with("0x") {
            let hex = next_sequence_of("1234567890abcdefABCDEF", &data[2..]);
            let data = &data[hex.len()..];
            package.push(DataObject::Byte(u64::from_str_radix(&hex, 16).unwrap() as u8));
            *length += 1;
        } else {
            let number = next_number(&data);
            package.push(DataObject::Byte(number.0 as u8));
            *length += 1;
        }
    }
}

pub fn parse(data: Vec<u8>) -> Option<Vec<u8>> {
    let string_data = match String::from_utf8(data) {
        Ok(data) => data,
        Err(_) => return None,
    }.split("\n").into_iter()
        .map(|x| {
            let index = x.find("//");
            let mut return_value = x;
            if let Some(idx) = index {
                return_value = x.get(..idx).unwrap();
            }
            return_value.to_string()
        }).collect::<Vec<String>>().concat();
    let mut lines: Vec<String> = string_data.split(";").into_iter()
        .map(|x| x.trim().to_string()+";" )
        .collect();
    lines.pop();

    let mut labels = HashMap::<String, u64>::new();
    let mut sizes = HashMap::<String, u64>::new();
    let mut data: Vec<DataObject> = vec![];
    let mut bytes_count: usize = 0;

    for mut line in lines {

        if line.starts_with("label") {
            line = line[5..].trim_start().to_string();
            let name = next_identifier(&line);
            labels.insert(name, bytes_count as u64);
        } else if line.starts_with("(") {
            line = line[1..].to_string();

            if line.starts_with(|x| { IDENTIFIER_STARTER.contains(x) }) {
                let length = next_identifier(&line);
                line = line[length.len()..].trim_start().to_string();
                assert!(line.starts_with(")"));
                line = line[1..].trim_start().to_string();
                let name = next_identifier(&line);
                line = line[name.len()..].trim_start().to_string();
                assert!(line.starts_with("="));
                line = line[1..].trim_start().to_string();
                
                labels.insert(name, bytes_count as u64);

                let previous_length = bytes_count;
                parse_data(&line, &mut data, &mut bytes_count);
                sizes.insert(length, (bytes_count-previous_length) as u64);
            } else if line.starts_with(|x| "1234567890-+".contains(x) ) {
                let length = next_number(&line);
                line = line[length.1..].trim_start().to_string();
                assert!(line.starts_with(")"));
                line = line[1..].trim_start().to_string();

                let name = next_identifier(&line);
                line = line[name.len()..].trim_start().to_string();

                let previous_length = bytes_count;
                if line.starts_with("=") {
                    line = line[1..].trim_start().to_string();
                    parse_data(&line, &mut data, &mut bytes_count);
                }
                assert!((bytes_count - previous_length) <= length.0 as usize, "arguments({}) are to many for setter({})", bytes_count - previous_length, length.0);
                while (bytes_count - previous_length) < length.0 as usize {
                    bytes_count += 1;
                    data.push(DataObject::Byte(0));
                }

            } else {
                line = line.trim_start().to_string();
                assert!(line.starts_with(")"), "expected ')'");
                line = line[1..].trim_start().to_string();
                let name = next_identifier(&line);
                line = line[name.len()..].trim_start().to_string();
                assert!(line.starts_with("="), "cannot infer size from empty capture ()");
                line = line[1..].trim_start().to_string();

                labels.insert(name, bytes_count as u64);

                parse_data(&line, &mut data, &mut bytes_count);
            }
        } else if line.starts_with(|x| { IDENTIFIER_STARTER.contains(x) }) {
            let name = next_identifier(&line);
            line = (&line[name.len()..line.len()]).trim_start().to_string();

            if let Ok(instruction) = arsenal_globals::Instructions::from_str(&name) {
                data.push(DataObject::Byte(instruction as u8));
                data.push(DataObject::Byte((instruction as u16).wrapping_shr(8) as u8));
                bytes_count += 2;

                if line.starts_with(":") {
                    parse_data(&line, &mut data, &mut bytes_count);
                }
            } else {
                panic!("unknown instruction {}", name);
            }
        }
    }

    let mut return_data: Vec<u8> = vec![];

    let mut general_offset: u32 = 0;

    for obj in data {
        match obj {
            DataObject::Byte(x) => return_data.push(x),
            DataObject::LabelRequest(name, start, stop, inc) => {
                let location = *labels.get(&name).unwrap() + inc;
                for offset in (start as u32)..=(stop as u32) {
                    unsafe {
                        return_data.push(*(((&location as *const u64 as u64) + (offset + general_offset) as u64) as *const u64 as *const u8));
                    }
                }
            },
            DataObject::SizeRequest(name, start, stop) => {
                let location = *sizes.get(&name).unwrap();
                for offset in (start as u32)..=(stop as u32) {
                    unsafe {
                        return_data.push(*(((&location as *const u64 as u64) + (offset + general_offset) as u64) as *const u64 as *const u8));
                    }
                }
            },
        }
    }

    assert!(return_data.len() == bytes_count);

    Some(return_data)
}
