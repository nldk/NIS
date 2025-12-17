mod backends;

use clap::{Arg, ArgAction, Command};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines, Read, Write};
use std::path::Path;
use std::{env, io, process};
use crate::backends::ByteCodeCompiler;

static stringInstructionsToU8: [&str; 27] = [
    "", "mov", "add", "sub", "div", "mul", "and", "or", "xor", "shr", "shl", "store", "load",
    "push", "pop", "jmp", "jz", "jnz", "eq", "neq", "big", "sm", "hlt", "int", "set", "call",
    "ret",
];
static stringToReg: [&str; 10] = ["r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "sp"];
#[derive(Debug)]
struct IntermediateLanguageInstruction{
    instruction: String,
    arg1: String,
    arg2: String,
}
#[derive(Debug)]
struct IntermediateLanguageLabel{
    label: String,
}
#[derive(Debug)]
enum IntermediateLanguageLine{
    Instruction(IntermediateLanguageInstruction),
    Label(IntermediateLanguageLabel),
}
impl IntermediateLanguageLine{
    fn parchLine(line: &str) -> IntermediateLanguageLine {
        if line.ends_with(":") {
            let label = IntermediateLanguageLabel{label: line.to_string()};
            IntermediateLanguageLine::Label(label)
        }else {
            let splitLine = line.trim().split(" ").collect::<Vec<&str>>();
            let instruction = if (stringInstructionsToU8.contains(&splitLine[0])){
                splitLine[0].to_string()
            }else {
                panic!("invalid instruction");
            };
            let arg1 = splitLine.get(1).unwrap_or(&"").to_string();
            let arg2 = splitLine.get(2).unwrap_or(&"").to_string();

            let line = IntermediateLanguageInstruction{instruction: instruction.to_string(), arg1, arg2};
            IntermediateLanguageLine::Instruction(line)
        }

    }
}
#[derive(Debug)]
struct IntermediateLanguage{
    lines: Vec<IntermediateLanguageLine>,
}
#[derive(Debug)]
struct Line {
    instruction: u8,
    arg1: u64,
    arg1IsReg: bool,
    arg2: u64,
    arg2IsReg: bool,
}

fn parse_include_file(path: &str) -> String {
    let include_path = Path::new(path);
    let mut include_file = File::open(include_path).unwrap();
    let mut include_content = String::new();
    include_file.read_to_string(&mut include_content).unwrap();
    include_content
}



struct Parcher {
    lines: Vec<Line>,
    instructionIndex: u64,
    filtered: Vec<String>,
    imports: Vec<String>,
    labels: HashMap<String, u64>,
}
impl Parcher {
    fn new() -> Parcher {
        Parcher {
            lines: vec![],
            instructionIndex: 0,
            filtered: vec![],
            imports: Vec::new(),
            labels: HashMap::new(),
        }
    }


    fn parchFileToIntermediate(&mut self, path: &str)->IntermediateLanguage {
        let mut data_file = File::open(path).unwrap();
        let mut file_content = String::new();
        data_file.read_to_string(&mut file_content).unwrap();
        let split: Vec<&str> = file_content.split("\n").collect();
        let mut filteredStr: Vec<&str> = split
            .iter()
            .filter(|line| !line.trim().is_empty())
            .cloned()
            .collect();
        self.filtered = filteredStr
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        for line in self.filtered.clone().iter() {
            let trimmed = line.trim();
            if trimmed.starts_with("#") {
                let preprocessorInstruction = trimmed.split_whitespace().collect::<Vec<&str>>();
                let instruction = preprocessorInstruction[0].trim_start_matches("#");
                match instruction {
                    "include" => {
                        let path_str = preprocessorInstruction[1].to_string();
                        let filename: String = Path::new(&path_str)
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        if self.imports.contains(&filename.clone()) {
                            continue;
                        }
                        self.imports.push(filename);
                        let include_content = parse_include_file(preprocessorInstruction[1]);
                        let include_lines: Vec<&str> = include_content
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .collect();
                        let include_lines: Vec<String> =
                            include_lines.iter().map(|line| line.to_string()).collect();
                        self.filtered.extend(include_lines);
                    }
                    "/" => continue,
                    _ => todo!(),
                }
            }
        }
        let mut intermediatelanguage: IntermediateLanguage = IntermediateLanguage{lines:vec![]};
        for line in self.filtered.clone().iter() {
            if line.trim().is_empty() || line.trim().starts_with(";") || line.starts_with("#"){
                continue;
            }
            intermediatelanguage.lines.push(IntermediateLanguageLine::parchLine(line))
        }
        intermediatelanguage
    }
}

fn main() {
    let mut parcher = Parcher::new();
    let intermetiate = parcher.parchFileToIntermediate("/home/niel/NIS/test.asm");
    println!("{:?}", intermetiate);
    let mut byteCodeCompiler = ByteCodeCompiler{lines:vec![],labels:HashMap::new(),instructionIndex:0};
    byteCodeCompiler.compileByteCodeFromIntermediate(intermetiate);
    byteCodeCompiler.run();
    /*
    let matches = Command::new("NIS")
        .version("1.0")
        .author("You")
        .about("NIS assembler/interpreter")
        .arg(
            Arg::new("assemble")
                .short('s')
                .long("assemble")
                .value_name("ASM_FILE")
                .help("Assemble an ASM file to a binary")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("BIN_FILE")
                .help("Output file for compiled binary")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("run")
                .short('r')
                .long("run")
                .value_name("BIN_FILE")
                .help("Run a compiled binary file")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("interpret")
                .short('i')
                .long("interpret")
                .value_name("ASM_FILE")
                .help("Interpret an ASM file directly")
                .action(ArgAction::Set),
        )
        .get_matches();

    // Assemble
    if let Some(asm_file) = matches.get_one::<String>("assemble") {
        let output_file = matches
            .get_one::<String>("output")
            .cloned()
            .unwrap_or_else(|| "file.bin".to_string());
        println!("Assembling {} -> {}", asm_file, output_file);

        let mut parcher = Parcher::new();
        parcher.parchFile(asm_file);
        parcher.writeToFile(output_file.as_str());
        println!("Assembled successfully!");
    }
    // Run compiled binary
    else if let Some(bin_file) = matches.get_one::<String>("run") {
        //println!("Running binary {}", bin_file);
        let mut parcher = Parcher::new();
        parcher.readFromFile(bin_file);
        let mut interpreter = Interpreter {
            lines: parcher.lines,
        };
        interpreter.run();
        println!();
    }
    // Interpret ASM file directly
    else if let Some(asm_file) = matches.get_one::<String>("interpret") {
        //println!("Interpreting ASM file {}", asm_file);
        let mut parcher = Parcher::new();
        parcher.parchFile(asm_file);
        let mut interpreter = Interpreter {
            lines: parcher.lines,
        };
        interpreter.run();
    } else {
        println!("No valid option provided. Use -h for help.");
    }
    */
}

