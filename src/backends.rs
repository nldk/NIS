use std::collections::HashMap;
use std::{io, process};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use crate::{parse_include_file, stringInstructionsToU8, stringToReg, IntermediateLanguage, IntermediateLanguageLine, Line, Parcher};

pub struct ByteCodeCompiler {
    pub lines: Vec<Line>,
    pub labels: HashMap<String, usize>,
    pub instructionIndex: usize,
}

impl ByteCodeCompiler {
    pub fn getLineArgCode(arg: &str) -> (u64, bool) {
        if arg.len() > 0 {
            if arg.chars().next().unwrap() == 'r' || arg.chars().next().unwrap() == 's' {
                return (
                    stringToReg.iter().position(|&s| s == arg).unwrap() as u64,
                    true,
                );
            } else {
                if arg.trim().len() > 1 {
                    //println!("{}", arg);
                    if arg.starts_with("0x") {
                        return match u64::from_str_radix(arg.trim_start_matches("0x"), 16) {
                            Ok(v) => (v, false),
                            Err(_) => {
                                (0, false)
                            }
                        };
                    }
                }
                if arg.trim().len() > 2 {
                    if arg.starts_with("\"") && arg.ends_with("\"") {
                        let value = arg.trim_start_matches("\"").trim_end_matches("\"");
                        return (value.chars().next().unwrap() as u64, false);
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

    pub fn compileByteCodeFromIntermediate(&mut self, intermediateCode:IntermediateLanguage) {
        for i in intermediateCode.lines.iter(){
            match i {
                IntermediateLanguageLine::Instruction(line) => {
                    self.instructionIndex += 1
                },
                IntermediateLanguageLine::Label(label) => {
                    let name = label.label.trim_end_matches(":");
                    self.labels
                        .insert(name.to_string(), self.instructionIndex +1);
                }
            }
        }
        for i in intermediateCode.lines{
            match i {
                IntermediateLanguageLine::Instruction(line) => {
                    if line.instruction == "call"
                        || line.instruction == "jmp"
                        || line.instruction == "jz"
                        || line.instruction == "jnz"
                    {
                        if cfg!(debug_assertions) {
                            println!(
                                "{}:{}",
                                line.arg1,
                                self.labels[&line.arg1.to_string()]
                            );
                        }

                        let (arg1, reg1) = (self.labels[&line.arg1.to_string()], false);
                        let (arg2, reg2) = ByteCodeCompiler::getLineArgCode(line.arg2.as_str());
                        self.lines.push(Line {
                            instruction: stringInstructionsToU8
                                .iter()
                                .position(|&s| s == line.instruction)
                                .unwrap() as u8,
                            arg1: arg1 as u64,
                            arg2: arg2,
                            arg1IsReg: reg1,
                            arg2IsReg: reg2,
                        });
                        continue;
                    }
                    if cfg!(debug_assertions) {
                        println!("{:?}", line);
                    }
                    let (arg1, reg1) = ByteCodeCompiler::getLineArgCode(line.arg1.as_str());
                    let (arg2, reg2) = ByteCodeCompiler::getLineArgCode(line.arg2.as_str());
                    self.lines.push(Line {
                        instruction: stringInstructionsToU8
                            .iter()
                            .position(|&s| s == line.instruction)
                            .expect(&("invalid instruction: ".to_string() + line.instruction.as_str()))
                            as u8,
                        arg1: arg1,
                        arg2: arg2,
                        arg1IsReg: reg1,
                        arg2IsReg: reg2,
                    });
                },
                IntermediateLanguageLine::Label(label) => {

                }
            }
        }
        let main_index = self.labels["main"];

        let jmp_main = Line {
            instruction: stringInstructionsToU8
                .iter()
                .position(|&s| s == "jmp")
                .unwrap() as u8,
            arg1: main_index as u64,
            arg1IsReg: false,
            arg2: 0,
            arg2IsReg: false,
        };

        self.lines.insert(0, jmp_main);
        if cfg!(debug_assertions) {
            println!("{:?}", self.lines);
        }
    }
    pub fn run(&mut self) {
        let mut registers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut mem: Vec<u64> = vec![];
        let mut carrierbit = false;
        let mut callStack: Vec<usize> = Vec::new();
        let mut ip = 0;
        while ip < self.lines.len() {
            let line = &self.lines[ip];
            match line.instruction {
                1 => registers[line.arg1 as usize] = registers[line.arg2 as usize],
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
                6 => {
                    let dest = line.arg1;
                    let rhs = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    registers[dest as usize] &= rhs;
                }
                7 => {
                    let dest = line.arg1;
                    let rhs = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    registers[dest as usize] |= rhs;
                }
                8 => {
                    let dest = line.arg1;
                    let rhs = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    registers[dest as usize] ^= rhs;
                }
                9 => {
                    if line.arg2IsReg {
                        registers[line.arg1 as usize] >>= registers[line.arg2 as usize];
                    } else {
                        registers[line.arg1 as usize] >>= line.arg2;
                    }
                }
                10 => {
                    if line.arg2IsReg {
                        registers[line.arg1 as usize] <<= registers[line.arg2 as usize];
                    } else {
                        registers[line.arg1 as usize] <<= line.arg2;
                    }
                }
                11 => {
                    let address = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    let value = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    mem[address as usize] = value;
                }
                12 => {
                    let address = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    registers[line.arg1 as usize] = mem[address as usize];
                }
                13 => {
                    //println!("rv{}", registers[9]);
                    let value = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    registers[9] += 1;
                    mem[registers[9] as usize] = value;
                }
                14 => {
                    registers[line.arg1 as usize] = mem[registers[9] as usize];
                    registers[9] -= 1;
                }
                15 => {
                    if cfg!(debug_assertions) {
                        println!("jmping to {}", line.arg1);
                    }
                    if line.arg1IsReg {
                        ip = registers[line.arg1 as usize] as usize;
                    } else {
                        ip = line.arg1 as usize;
                    }
                    continue;
                }
                16 => {
                    if cfg!(debug_assertions) {
                        println!("jz:{}", carrierbit);
                    }
                    if carrierbit {
                        if line.arg1IsReg {
                            ip = registers[line.arg1 as usize] as usize;
                        } else {
                            ip = line.arg1 as usize;
                        }
                        continue;
                    }
                }
                17 => {
                    if cfg!(debug_assertions) {
                        println!("jnz:{}", carrierbit);
                    }
                    if !carrierbit {
                        if line.arg1IsReg {
                            ip = registers[line.arg1 as usize] as usize;
                        } else {
                            ip = line.arg1 as usize;
                        }
                        continue;
                    }
                }
                18 => {
                    let val1 = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    let val2 = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    carrierbit = val1 == val2
                }
                19 => {
                    let val1 = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    let val2 = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    carrierbit = val1 != val2
                }
                20 => {
                    let val1 = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    let val2 = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    carrierbit = val1 > val2
                }
                21 => {
                    let val1 = if line.arg1IsReg {
                        registers[line.arg1 as usize]
                    } else {
                        line.arg1
                    };
                    let val2 = if line.arg2IsReg {
                        registers[line.arg2 as usize]
                    } else {
                        line.arg2
                    };
                    carrierbit = val1 < val2
                }
                22 => {
                    let mut p: u64 = 0;
                    let mut useP: bool = false;
                    interrupt(0, 0, &mut mem, &mut p, &mut useP);
                }
                23 => {
                    let mut p: u64 = 0;
                    let mut useP: bool = false;
                    if line.arg1IsReg {
                        //println!("{}",regristers[7]);
                        interrupt(
                            registers[8] as u8,
                            registers[line.arg1 as usize],
                            &mut mem,
                            &mut p,
                            &mut useP,
                        )
                    } else {
                        interrupt(registers[8] as u8, line.arg1, &mut mem, &mut p, &mut useP)
                    }
                    if useP {
                        registers[7] = p
                    }
                }
                24 => registers[line.arg1 as usize] = line.arg2,
                25 => {
                    //println!("call called",);
                    callStack.push(ip + 1);
                    if line.arg1IsReg {
                        ip = registers[line.arg1 as usize] as usize;
                    } else {
                        ip = line.arg1 as usize;
                    }
                    if cfg!(debug_assertions) {
                        println!("call going to {}", ip);
                    }
                    continue;
                }
                26 => {
                    //println!("{:?}", callStack);
                    if let Some(return_address) = callStack.pop() {
                        ip = return_address;
                        continue;
                    } else {
                        panic!("RET with empty call stack. ip: {}", ip);
                    }
                    //println!("returning to {}",ip);
                    continue;
                }
                _ => panic!(
                    "invalid instruction {}",
                    stringInstructionsToU8[line.instruction as usize]
                ),
            }
            ip += 1;
        }
    }
    fn write_instructions(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;

        for line in &self.lines {
            // Write opcode
            file.write_all(&[line.instruction])?;

            // Write arg1 and arg2 as little-endian u64
            file.write_all(&line.arg1.to_le_bytes())?;
            file.write_all(&line.arg2.to_le_bytes())?;

            // Pack booleans into a single byte
            let mut flags: u8 = 0;
            if line.arg1IsReg {
                flags |= 1 << 0;
            }
            if line.arg2IsReg {
                flags |= 1 << 1;
            }
            file.write_all(&[flags])?;
        }
        Ok(())
    }

    fn read_instructions(filename: &str) -> io::Result<Vec<Line>> {
        let mut file = BufReader::new(File::open(filename)?);
        let mut instructions = Vec::new();
        let mut buf = [0u8; 18];

        while file.read_exact(&mut buf).is_ok() {
            let instruction = buf[0];
            let arg1 = u64::from_le_bytes(buf[1..9].try_into().unwrap());
            let arg2 = u64::from_le_bytes(buf[9..17].try_into().unwrap());
            let flags = buf[17];
            let arg1_is_reg = flags & 1 != 0;
            let arg2_is_reg = flags & 2 != 0;

            instructions.push(Line {
                instruction: instruction,
                arg1: arg1,
                arg1IsReg: arg1_is_reg,
                arg2: arg2,
                arg2IsReg: arg2_is_reg,
            });
        }

        Ok(instructions)
    }
    fn readFromFile(&mut self, path: &str) {
        self.lines = ByteCodeCompiler::read_instructions(path).unwrap()
    }
    fn writeToFile(&mut self, path: &str) {
        self.write_instructions(path).unwrap()
    }
}
fn interrupt(intCode: u8, value: u64, mem: &mut Vec<u64>, p: &mut u64, useP: &mut bool) {
    match intCode {
        0 => process::exit(value as i32),
        1 => {
            let start = mem.len();
            mem.resize(start + value as usize, 0);
            *useP = true;
            *p = start as u64;
        }
        2 => print!("{}", std::char::from_u32(value as u32).unwrap()),
        3 => print!("{}", value),
        4 => {
            *p = mem.len() as u64;
            *useP = true;
        }
        _ => panic!("invalid interrupt code {}", intCode),
    }
}


struct x86_64Compiler{

}
impl x86_64Compiler {
    pub fn compileToX86_64FromIntermediate(intermediateLanguage: IntermediateLanguage){

    }
}