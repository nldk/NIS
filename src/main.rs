use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;

static stringInstructionsToU8: [&str; 27] = ["", "mov", "add", "sub", "div", "mul", "and", "or", "xor", "shr", "shl", "store", "load", "push", "pop", "jmp", "jz", "jnz", "eq", "neq", "big", "sm", "hlt", "int", "set","call","ret"];
static stringToReg: [&str; 9] = ["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8","sp"];
#[derive(Debug)]
struct Line {
    instruction: u8,
    arg1: u64,
    arg1IsReg: bool,
    arg2: u64,
    arg2IsReg: bool,
}
fn getLineArgCode(arg: &str) -> (u64, bool) {
    if arg.chars().next().unwrap() == 'r' || arg.chars().next().unwrap() == 's' {
        return (stringToReg.iter().position(|&s| s == arg).unwrap() as u64, true);
    } else {
        return (arg.parse::<u64>().unwrap(), false);
    }
}

fn main() {
    let mut mem:Vec<u64> = vec![];
    let mut carrierbit = false;
    let mut callStack: Vec<usize> = Vec::new();
    let mut ip = 0;
    let mut regristers: [u64; 9] = [0, 0, 0, 0, 0, 0, 0, 0,0];
    let mut labels:HashMap<&str,u64> = HashMap::new();
    let path = Path::new("/home/niel/RustroverProjects/NISinturpriter/test.asm");
    let mut data_file = File::open(path).unwrap();
    let mut file_content = String::new();
    data_file.read_to_string(&mut file_content).unwrap();
    let split: Vec<&str> = file_content.split("\n").collect();
    let filtered: Vec<_> = split.iter()
        .filter(|line| !line.trim().is_empty())
        .cloned()
        .collect();
    let mut instruction_index = 0;
    for line in filtered.iter() {
        let trimmed = line.trim();
        if trimmed.starts_with("#"){
            let instructionsplit = trimmed.split_whitespace().collect::<Vec<&str>>()[1];
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
        if i.trim().ends_with(":") {
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
            1 => { regristers[line.arg1 as usize] = regristers[line.arg2 as usize] }
            2 => {
                if line.arg2IsReg {
                    regristers[line.arg1 as usize] += regristers[line.arg2 as usize]
                } else {
                    regristers[line.arg1 as usize] += line.arg2
                }
            }
            3 => {
                if line.arg2IsReg {
                    regristers[line.arg1 as usize] -= regristers[line.arg2 as usize]
                } else {
                    regristers[line.arg1 as usize] -= line.arg2
                }
            }
            4 => {
                if line.arg2IsReg {
                    regristers[line.arg1 as usize] /= regristers[line.arg2 as usize]
                } else {
                    regristers[line.arg1 as usize] /= line.arg2
                }
            }
            5 => {
                if line.arg2IsReg {
                    regristers[line.arg1 as usize] *= regristers[line.arg2 as usize]
                } else {
                    regristers[line.arg1 as usize] *= line.arg2
                }
            }
            11 =>{
                let addres = if line.arg1IsReg {
                    regristers[line.arg1 as usize]
                }else{
                    line.arg1
                };
                let value = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                mem[addres as usize] = value;
            },
            12 => {
                let addres = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else{
                    line.arg2
                };
                regristers[line.arg1 as usize] = mem[addres as usize];
            },
            13 => {
                let value = if line.arg1IsReg{
                    regristers[line.arg1 as usize]
                }else {
                    line.arg1
                } ;
                regristers[8] += 1;
                mem[regristers[8]as usize] = value;
            },
            14 => {
                regristers[line.arg1 as usize] = mem[regristers[8] as usize];
                regristers[8] -= 1;
            },
            15 =>{
                if cfg!(debug_assertions) {
                    println!("jmping to {}", line.arg1);
                }
                if line.arg1IsReg {
                    ip = regristers[line.arg1 as usize] as usize;
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
                        ip = regristers[line.arg1 as usize] as usize;
                    }else {
                        ip = line.arg1 as usize ;
                    }
                    continue;
                }
            },
            17 => {
                if cfg!(debug_assertions) {
                    println!("jnz:{}",carrierbit);
                }
                if !carrierbit {
                    if line.arg1IsReg {
                        ip = regristers[line.arg1 as usize] as usize;
                    }else {
                        ip = line.arg1 as usize ;
                    }
                    continue;
                }
            },
            18 => {
                let val1 = if line.arg1IsReg {
                    regristers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 == val2
            },
            19 => {
                let val1 = if line.arg1IsReg {
                    regristers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 != val2
            },
            20 => {
                let val1 = if line.arg1IsReg {
                    regristers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 > val2
            },
            21 => {
                let val1 = if line.arg1IsReg {
                    regristers[line.arg1 as usize]
                }else {
                    line.arg1
                };
                let val2 = if line.arg2IsReg {
                    regristers[line.arg2 as usize]
                }else {
                    line.arg2
                };
                carrierbit = val1 < val2
            },
            23 => {
                let mut p:u64 = 0;
                let mut useP:bool = false;
                if line.arg1IsReg {
                    interrupt(regristers[7] as u8, regristers[line.arg1 as usize],&mut mem,&mut p,&mut useP)
                } else {
                    interrupt(regristers[7] as u8, line.arg1,&mut mem,&mut p,&mut useP)
                }
            }
            24 => { regristers[line.arg1 as usize] = line.arg2 },
            25 => {
                //println!("call called",);
                callStack.push(ip+1);
                if line.arg1IsReg {
                    ip = regristers[line.arg1 as usize] as usize;
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
                    panic!("RET with empty call stack");
                }
                //println!("returning to {}",ip);
                continue;
            },
            _ => panic!("invalid instruction {}",stringInstructionsToU8[line.instruction as usize]),
        }
        ip += 1;
    }
}
fn interrupt(intCode: u8, value: u64,mem: &mut Vec<u64>,p:&mut u64,useP:&mut bool) {
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
        _ => panic!("invalid interrupt code {}", intCode),
    }
}