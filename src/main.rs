use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;

static stringInstructionsToU8: [&str; 27] = ["", "mov", "add", "sub", "div", "mul", "and", "or", "xor", "shr", "shl", "store", "load", "push", "pop", "jmp", "jz", "jnz", "eq", "neq", "big", "sm", "hlt", "int", "set","call","ret"];
static stringToReg: [&str; 10] = ["r0","r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8","sp"];
#[derive(Debug)]
struct Line {
    instruction: u8,
    arg1: u64,
    arg1IsReg: bool,
    arg2: u64,
    arg2IsReg: bool,
}
fn getLineArgCode(arg: &str) -> (u64, bool) {
    if arg.len() >0{
        if arg.chars().next().unwrap() == 'r' || arg.chars().next().unwrap() == 's' {
            return (stringToReg.iter().position(|&s| s == arg).unwrap() as u64, true);
        } else {

            if arg.trim().len()>1 {
                //println!("{}", arg);
                if arg.chars().nth(0).unwrap() == '0' || arg.chars().nth(1).unwrap() == 'x' {
                    return match u64::from_str_radix(arg.trim_start_matches("0x"), 16) {
                        Ok(v) => (v, false),
                        Err(_) => {
                            (0, false) // or return an error, or use default
                        }
                    };
                }
            }
            return match arg.parse::<u64>() {
                Ok(v) => (v, false),
                Err(_) => {
                    (0, false) // or return an error, or use default
                }
            };
        }
    }
    return (0, false);

}
fn parse_include_file(path: &str) -> String {
    let include_path = Path::new(path);
    let mut include_file = File::open(include_path).unwrap();
    let mut include_content = String::new();
    include_file.read_to_string(&mut include_content).unwrap();
    include_content
}
fn main() {
    let mut imports:Vec<String> = vec![];
    let mut mem:Vec<u64> = vec![];
    let mut carrierbit = false;
    let mut callStack: Vec<usize> = Vec::new();
    let mut ip = 0;
    let mut registers: [u64; 10] = [0, 0, 0, 0, 0, 0, 0, 0,0,0];
    let mut labels:HashMap<&str,u64> = HashMap::new();
    let path = Path::new("/home/niel/RustroverProjects/NISinturpriter/test.asm");
    let mut data_file = File::open(path).unwrap();
    let mut file_content = String::new();
    data_file.read_to_string(&mut file_content).unwrap();
    let split: Vec<&str> = file_content.split("\n").collect();
    let mut filtered: Vec<&str> = split.iter()
        .filter(|line| !line.trim().is_empty())
        .cloned()
        .collect();
    let mut filtered = filtered.iter().map(|s|s.to_string()).collect::<Vec<String>>();
    let mut instruction_index = 0;
    for line in filtered.clone().iter() {
        let trimmed = line.trim();
        if trimmed.starts_with("#"){
            let preprocessorInstruction = trimmed.split_whitespace().collect::<Vec<&str>>();
            let instruction = preprocessorInstruction[0].trim_start_matches("#");
            match instruction {
                "include" =>{
                    let path_str = preprocessorInstruction[1].to_string();
                    let filename: String = Path::new(&path_str)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    if imports.contains(&filename.clone()){
                        continue
                    }
                    imports.push(filename);
                    let include_content = parse_include_file(preprocessorInstruction[1]);
                    let include_lines: Vec<&str> = include_content
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .collect();
                    let include_lines: Vec<String> = include_lines.iter().map(|line|line.to_string()).collect();
                    filtered.extend(include_lines);
                }
                "/"=>{continue},
                _ => todo!(),
            }
        }
    }
    //println!("{:?}", filtered);
    for line in filtered.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with("#"){
            continue;
        }
        if trimmed.ends_with(":") {
            let name = trimmed.trim_end_matches(":");
            labels.insert(name, instruction_index + 1);
            continue;
        }

        instruction_index += 1;
    }
    let mut lines: Vec<Line> = vec!();
    for (e,i) in filtered.iter().enumerate() {
        if i.trim().starts_with(";"){
            continue;
        }
        if i.trim().ends_with(":") {
            continue;
        }
        if i.trim().starts_with("#"){
            continue;
        }
        let splitLine = i.trim().split(" ").collect::<Vec<&str>>();
        if splitLine[0] == "call" || splitLine[0] == "jmp" || splitLine[0] == "jz" || splitLine[0] == "jnz" {
            if cfg!(debug_assertions) {
                println!("{}:{}",splitLine[1],labels[&splitLine[1]]);
            }

            let (arg1, reg1) = (labels[&splitLine[1]],false);
            let (arg2, reg2) = splitLine.get(2).map_or((0, false), |x| getLineArgCode(x));
            lines.push(Line { instruction: stringInstructionsToU8.iter().position(|&s| s == splitLine[0]).unwrap() as u8, arg1: arg1, arg2: arg2, arg1IsReg: reg1, arg2IsReg: reg2 });
            continue;
        }
        if cfg!(debug_assertions) {
            println!("{:?}", splitLine);
        }
        let (arg1, reg1) = splitLine.get(1).map_or((0, false), |x| getLineArgCode(x));
        let (arg2, reg2) = splitLine.get(2).map_or((0, false), |x| getLineArgCode(x));
        lines.push(Line { instruction: stringInstructionsToU8.iter().position(|&s| s == splitLine[0]).expect(&("invalid instruction: ".to_string() + splitLine[0])) as u8, arg1: arg1, arg2: arg2, arg1IsReg: reg1, arg2IsReg: reg2 });
    }
    let main_index = labels["main"];

    let jmp_main = Line {
        instruction: stringInstructionsToU8.iter().position(|&s| s == "jmp").unwrap() as u8,
        arg1: main_index,
        arg1IsReg: false,
        arg2: 0,
        arg2IsReg: false,
    };

    lines.insert(0, jmp_main);
    if cfg!(debug_assertions) {
        println!("{:?}", lines);
    }
    while ip < lines.len() {
        let line = &lines[ip];
        match line.instruction {
            1 => { registers[line.arg1 as usize] = registers[line.arg2 as usize] }
            2 => {
                if line.arg2IsReg {
                    registers[line.arg1 as usize] += registers[line.arg2 as usize]
                } else {
                    registers[line.arg1 as usize] += line.arg2
                }
            }
            3 => {
                if line.arg2IsReg {
                    registers[line.arg1 as usize] -= registers[line.arg2 as usize]
                } else {
                    registers[line.arg1 as usize] -= line.arg2
                }
            }
            4 => {
                if line.arg2IsReg {
                    registers[line.arg1 as usize] /= registers[line.arg2 as usize]
                } else {
                    registers[line.arg1 as usize] /= line.arg2
                }
            }
            5 => {
                if line.arg2IsReg {
                    registers[line.arg1 as usize] *= registers[line.arg2 as usize]
                } else {
                    registers[line.arg1 as usize] *= line.arg2
                }
            }
            6=>{
                let dest = line.arg1;
                let rhs = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                } else {
                    line.arg2
                };
                registers[dest as usize] &= rhs;
            },
            7=>{
                let dest = line.arg1;
                let rhs = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                } else {
                    line.arg2
                };
                registers[dest as usize] |= rhs;
            },
            8=>{
                let dest = line.arg1;
                let rhs = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                } else {
                    line.arg2
                };
                registers[dest as usize] ^= rhs;
            },
            9=>{
                if line.arg2IsReg {
                    registers[line.arg1 as usize] >>= registers[line.arg2 as usize];
                } else {
                    registers[line.arg1 as usize] >>= line.arg2;
                }
            },
            10=>{
                if line.arg2IsReg {
                    registers[line.arg1 as usize] <<= registers[line.arg2 as usize];
                } else {
                    registers[line.arg1 as usize] <<= line.arg2;
                }
            },
            11 =>{
                let address = if line.arg1IsReg {
                    registers[line.arg1 as usize]
                }else{
                    line.arg1
                };
                let value = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                mem[address as usize] = value;
            },
            12 => {
                let address = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else{
                    line.arg2
                };
                registers[line.arg1 as usize] = mem[address as usize];
            },
            13 => {
                //println!("rv{}", registers[9]);
                let value = if line.arg1IsReg{
                    registers[line.arg1 as usize]
                }else {
                    line.arg1
                } ;
                registers[9] += 1;
                mem[registers[9]as usize] = value;
            },
            14 => {
                registers[line.arg1 as usize] = mem[registers[9] as usize];
                registers[9] -= 1;
            },
            15 =>{
                if cfg!(debug_assertions) {
                    println!("jmping to {}", line.arg1);
                }
                if line.arg1IsReg {
                    ip = registers[line.arg1 as usize] as usize;
                }else {
                    ip = line.arg1 as usize ;
                }
                continue;
            },
            16 => {
                if cfg!(debug_assertions) {
                    println!("jz:{}", carrierbit);
                }
                if carrierbit {
                    if line.arg1IsReg {
                        ip = registers[line.arg1 as usize] as usize;
                    }else {
                        ip = line.arg1 as usize ;
                    }
                    continue;
                }
            },
            17 => {
                if cfg!(debug_assertions) {
                    println!("jnz:{}", carrierbit);
                }
                if !carrierbit {
                    if line.arg1IsReg {
                        ip = registers[line.arg1 as usize] as usize;
                    }else {
                        ip = line.arg1 as usize ;
                    }
                    continue;
                }
            },
            18 => {
                let val1 = if line.arg1IsReg {
                    registers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 == val2
            },
            19 => {
                let val1 = if line.arg1IsReg {
                    registers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 != val2
            },
            20 => {
                let val1 = if line.arg1IsReg {
                    registers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 > val2
            },
            21 => {
                let val1 = if line.arg1IsReg {
                    registers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    registers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 < val2
            },
            22 => {
                let mut p:u64 = 0;
                let mut useP:bool = false;
                interrupt(0, 0, &mut mem, &mut p, &mut useP);
            }
            23 => {
                let mut p:u64 = 0;
                let mut useP:bool = false;
                if line.arg1IsReg {
                    //println!("{}",regristers[7]);
                    interrupt(registers[8] as u8, registers[line.arg1 as usize], &mut mem, &mut p, &mut useP)
                } else {
                    interrupt(registers[8] as u8, line.arg1, &mut mem, &mut p, &mut useP)
                }
                if useP{
                    registers[7] = p
                }
            }
            24 => { registers[line.arg1 as usize] = line.arg2 },
            25 => {
                //println!("call called",);
                callStack.push(ip+1);
                if line.arg1IsReg {
                    ip = registers[line.arg1 as usize] as usize;
                }else {
                    ip = line.arg1 as usize;
                }
                //println!("call going to {}",ip);
                continue;
            }
            26 => {
                //println!("{:?}", callStack);
                if let Some(return_address) = callStack.pop() {
                    ip = return_address;
                    continue;
                } else {
                    panic!("RET with empty call stack. ip: {}",ip);
                }
                //println!("returning to {}",ip);
                continue;
            },
            _ => panic!("invalid instruction {}",stringInstructionsToU8[line.instruction as usize]),
        }
        ip += 1;
    }
}
fn interrupt(intCode: u8, value: u64, mem: &mut Vec<u64>, p: &mut u64, useP:&mut bool) {
    match intCode {
        0 => process::exit(value as i32),
        1 => {
            let start = mem.len();
            mem.resize(start +value as usize, 0);
            *useP = true;
            *p = start as u64;
        },
        2 => print!("{}", std::char::from_u32(value as u32).unwrap()),
        3 => print!("{}", value),
        4 => {
            *p = mem.len() as u64;
            *useP = true;
        }
        _ => panic!("invalid interrupt code {}", intCode),
    }
}