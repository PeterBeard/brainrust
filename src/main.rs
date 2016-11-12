/// This file is part of brainrust and is copyright Peter Beard
/// Licensed under the GPL v3, see LICENSE for details
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;
use std::env;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Token {
    RAngle,
    LAngle,
    Plus,
    Minus,
    Period,
    Comma,
    LBracket(usize),
    RBracket(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TokenizerState {
    Token,
    TrailingWhitespace,
    LeadingWhitespace,
}

/// Load a program from a file
fn load_from_file(filename: &Path) -> Option<String> {
    let fh = File::open(filename);
    if let Ok(mut f) = fh {
        let mut contents = String::new();
        
        if let Ok(_) = f.read_to_string(&mut contents) {
            Some(contents)
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse a char to a Token
fn char_to_token(ch: &char) -> Option<Token> {
    use Token::*;
    match *ch {
        '>' => Some(RAngle),
        '<' => Some(LAngle),
        '+' => Some(Plus),
        '-' => Some(Minus),
        '.' => Some(Period),
        ',' => Some(Comma),
        '[' => {
            Some(LBracket(0))
        }
        ']' => {
            Some(RBracket(0))
        },
        _ => None,
    }
}

/// Tokenize a string
///
/// Tokenization basically just conists of ignoring characters outside the BF alphabet
fn tokenize(input: &str) -> Vec<Token> {
    use Token::*;

    let mut tokens: Vec<Token> = Vec::with_capacity(input.len());
    let mut state = TokenizerState::LeadingWhitespace;
    for character in input.chars() {
        match state {
            TokenizerState::LeadingWhitespace => {
                if let Some(t) = char_to_token(&character) {
                    state = TokenizerState::Token;
                    tokens.push(t);
                }
            },
            TokenizerState::Token => {
                if let Some(t) = char_to_token(&character) {
                    tokens.push(t);
                } else {
                    state = TokenizerState::TrailingWhitespace;
                }
            },
            TokenizerState::TrailingWhitespace => {
                if character == '\n' || character == '\r' {
                    state = TokenizerState::LeadingWhitespace;
                }
            },
        }
    }

    // Match up brackets
    for (i, t) in tokens.clone().into_iter().enumerate() {
        match t {
            LBracket(_) => {
                let mut depth = 1;
                let mut p = i+1;
                while depth > 0 && p < tokens.len() {
                    if let RBracket(_) = tokens[p] {
                        depth -= 1;
                    } else if let LBracket(_) = tokens[p] {
                        depth += 1;
                    }
                    p += 1;
                }
                if let RBracket(_) = tokens[p-1] {
                    tokens[i] = LBracket(p);
                } else {
                    panic!("Unmatched [ at {}", i);
                }
            },
            RBracket(_) => {
                let mut depth = -1;
                let mut p = i-1;
                while depth < 0 && p > 0 {
                    if let RBracket(_) = tokens[p] {
                        depth -= 1;
                    } else if let LBracket(_) = tokens[p] {
                        depth += 1;
                    }
                    p -= 1;
                }
                if let LBracket(_) = tokens[p+1] {
                    tokens[i] = RBracket(p);
                } else if p == 0 {
                    tokens[i] = RBracket(1);
                } else {
                    panic!("Unmatched ] at {}", i);
                }
            },
            _ => {},
        }
    }
    tokens
}

/// Run a brainfuck program
///
/// A program is just an array of tokens since the language doesn't really
/// require an AST to be generated
fn run_program(program: &[Token]) {
    use Token::*;

    let mut data: Vec<u8> = Vec::new();
    let mut data_pointer: usize = 0;
    let mut instr_pointer: usize = 0;

    while instr_pointer < program.len() {
        // Pretend we have an infinite tape
        if data_pointer >= data.len() {
            data.push(0);
        }

        match program[instr_pointer] {
            RAngle => {
                data_pointer += 1;
            },
            LAngle => {
                if data_pointer == 0 {
                    panic!("Cannot decrement zero data pointer");
                }
                data_pointer -= 1;
            },
            Plus => {
                data[data_pointer] = data[data_pointer].wrapping_add(1);
            },
            Minus => {
                data[data_pointer] = data[data_pointer].wrapping_sub(1);
            },
            Period => {
                print!("{}", data[data_pointer] as char);
            },
            Comma => {
                let mut buf: [u8; 1] = [0];
                let count = io::stdin().read_exact(&mut buf);
                if let Ok(_) = count {
                    data[data_pointer] = buf[0];
                } else {
                    panic!("Error reading from STDIN: {:?}", count);
                }
            },
            LBracket(pointer) => {
                if data[data_pointer] == 0 {
                    instr_pointer = pointer;
                }
            },
            RBracket(pointer) => {
                if data[data_pointer] != 0 {
                    instr_pointer = pointer;
                }
            },
        }
        instr_pointer += 1;
    }
}

/// Entry point
fn main() {
    let program = if let Some(fname) = env::args().nth(1) {
        load_from_file(Path::new(&fname))
    } else {
        panic!("No filename provided.");
    };
    if let Some(p) = program {
        let tokens = tokenize(&p);
        run_program(&tokens);
    } else {
        panic!("Failed to load file");
    }
}
