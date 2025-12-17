use crate::{interrupt, stringInstructionsToU8, Line};

pub struct Interpreter {
    pub lines: Vec<Line>,
}

impl Interpreter {
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
}