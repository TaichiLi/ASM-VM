#![allow(dead_code)]
use crate::token::*;
use crate::scanner::*;
use std::collections::HashMap;
use std::vec::Vec;
use std::result::Result;

const MAX: usize = 1024;

pub struct VM {
    stack: [i32; MAX],
    memory: [i32; MAX],
    text: Vec<Token>,
    index: HashMap<String, i32>,
    eax: i32,
    ebx: i32,
    ecx: i32,
    edx: i32,
    esi: i32,
    edi: i32,
    esp: i32,
    ebp: i32,
    eip: usize,
    zf: bool,
    sf: bool,
    scanner: Scanner,
    error_flag_: bool,
}

impl VM {
    pub fn new(source_file_name: String) -> Self {
        VM {
            stack: [0; MAX],
            memory: [0; MAX],
            text: Vec::new(),
            index: HashMap::new(),
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            esi: 0,
            edi: 0,
            esp: (MAX - 1) as i32,
            ebp: 0,
            eip: 0,
            zf: false,
            sf: false,
            scanner: Scanner::new(source_file_name),
            error_flag_: false,
        }
    }

    fn error_syntax(&mut self, msg: &String) {
        eprintln!("{}", msg);
        self.error_flag_ = true;
    }

    fn error_report(&mut self, msg: &String) {
        self.error_syntax(&format!("Syntax Error: {}{}", self.text[self.eip].get_token_location().to_string(),
                    msg));
    }

    fn expect_token_type(&mut self, token_type: TokenType, token_name: String, advance_to_next_token: bool) -> bool {
        if self.text[self.eip].get_token_type() != token_type {
            self.error_report(&format!("Expected \"{}\", but find \"{}\"", token_name,
                        self.text[self.eip].get_token_name()));
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn expect_token_value(&mut self, token_value: TokenValue, token_name: String, advance_to_next_token: bool) -> bool {
        if self.text[self.eip].get_token_value() != token_value {
            self.error_report(&format!("Expected \"{}\", but find \"{}\"", token_name,
                        self.text[self.eip].get_token_name()));
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn validate_token_type(&mut self, token_type: TokenType, advance_to_next_token: bool) -> bool {
        if self.text[self.eip].get_token_type() != token_type {
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn validate_token_value(&mut self, token_value: TokenValue, advance_to_next_token: bool) -> bool {
        if self.text[self.eip].get_token_value() != token_value {
            return false;
        }

        if advance_to_next_token {
            self.go_from_here(1);
        }

        true
    }

    fn go_from_here(&mut self, displacement: i32) {
        self.eip = (self.eip as i32 + displacement) as usize;
    }

    fn preprocess(&mut self) {
        let mut count = 0;
        let mut entrance = 0;

        loop {
            self.scanner.get_next_token();
            count = count + 1;

            let token = self.scanner.get_token();

            match token.get_token_type() {
                TokenType::LABEL => {
                    self.index.insert(token.get_token_name(), count);

                    if token.get_token_name().eq("main") {
                        entrance = count - 1;
                    }

                    self.text.push(token);
                },
                TokenType::END_OF_FILE => break,
                _ => {
                    self.text.push(token.to_owned());

                    if token.get_token_value() == TokenValue::CALL {
                        let mut label = self.scanner.get_next_token();
                        count = count + 1;

                        if label.get_token_type() != TokenType::LABEL {
                                panic!("Expected \"label\", but find \"{}\"", label.get_token_name());
                        }

                        let label_name = label.get_token_name();

                        if !self.index.contains_key(&label_name) {
                            panic!("Unknown label: \"{}\"", label_name);
                        }

                        let label_address = self.index.get(&label_name).unwrap();

                        label.set_token_type(TokenType::IMMEDIATE_DATA);
                        label.set_int_value(label_address - count - 1);

                        self.text.push(label);
                    }
                },
            }
        }

        self.eip = entrance as usize;
    }

    fn parse_source(&mut self) -> Result<i32, &str> {
        if self.validate_token_value(TokenValue::LBRACK, true) {
            if !self.expect_token_type(TokenType::IMMEDIATE_DATA,"immediate data".to_string(), false) {
                return Err("Only support immediate addressing now!");
            }

            let mem_add = self.text[self.eip].get_int_value();

            self.go_from_here(1);

            if !self.expect_token_value(TokenValue::RBRACK, "]".to_string(), true) {
                return Err("Missing right brack \']\'!");
            }

            return Ok(self.memory[mem_add as usize]);
        } else if self.validate_token_type(TokenType::REGISTER, true) {
            match self.text[self.eip - 1].get_token_value() {
                TokenValue::EAX => return Ok(self.eax),
                TokenValue::EBX => return Ok(self.ebx),
                TokenValue::ECX => return Ok(self.ecx),
                TokenValue::EDX => return Ok(self.edx),
                TokenValue::ESI => return Ok(self.esi),
                TokenValue::EDI => return Ok(self.edi),
                TokenValue::ESP => return Ok(self.esp),
                TokenValue::EBP => return Ok(self.ebp),
                TokenValue::EIP => return Ok(self.eip as i32),
                _ => return Err("Flag registers can not be used as source!"),
            }
        } else if self.validate_token_type(TokenType::IMMEDIATE_DATA, true) {
            return Ok(self.text[self.eip - 1].get_int_value());
        } else {
            self.error_report(&format!("parse_source Unexpected token: {}", self.text[self.eip].get_token_name()));
            return Err("Unexpected token: {}");
        }
    }

    fn parse_destination(&mut self) -> Result<*mut i32, &str> {
        if self.validate_token_value(TokenValue::LBRACK, true) {
            if !self.expect_token_type(TokenType::IMMEDIATE_DATA,"immediate data".to_string(), false) {
                return Err("Only support immediate addressing now!");
            }

            let mem_add = self.text[self.eip].get_int_value();

            self.go_from_here(1);

            if !self.expect_token_value(TokenValue::RBRACK, "]".to_string(), true) {
                return Err("Missing right brack \']\'!");
            }

            return Ok(&mut self.memory[mem_add as usize] as *mut i32);
        } else if self.validate_token_type(TokenType::REGISTER, true) {
            match self.text[self.eip - 1].get_token_value() {
                TokenValue::EAX => return Ok(&mut self.eax as *mut i32),
                TokenValue::EBX => return Ok(&mut self.ebx as *mut i32),
                TokenValue::ECX => return Ok(&mut self.ecx as *mut i32),
                TokenValue::EDX => return Ok(&mut self.edx as *mut i32),
                TokenValue::ESI => return Ok(&mut self.esi as *mut i32),
                TokenValue::EDI => return Ok(&mut self.edi as *mut i32),
                TokenValue::ESP => return Ok(&mut self.esp as *mut i32),
                TokenValue::EBP => return Ok(&mut self.ebp as *mut i32),
                _ => return Err("EIP and flag registers can not be used as destination"),
            }
        } else {
            self.error_report(&format!("parse_destination Unexpected token: {}", self.text[self.eip].get_token_name()));
            return Err("Unexpected token: {}");
        }
    }

    fn mov(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let result = self.parse_source().unwrap();

        unsafe {
            *destination = result;
        }
    }

    fn arithmetic(&mut self) {
        let instruction = self.text[self.eip].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap().to_owned();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        unsafe {
            let result = match instruction.get_token_value() {
                TokenValue::ADD => *destination + self.parse_source().unwrap(),
                TokenValue::SUB => *destination - self.parse_source().unwrap(),
                TokenValue::MUL => *destination * self.parse_source().unwrap(),
                TokenValue::DIV => *destination / self.parse_source().unwrap(),
                _ => std::i32::MAX,
            };

            if result > 0 {
                self.sf = false;
                self.zf = false;
            } else if result == 0 {
                self.sf = false;
                self.zf = true;
            } else {
                self.sf = true;
                self.zf = false;
            }

            *destination = result;
        }
    }

    fn push(&mut self) {
        self.go_from_here(1);

        self.stack[(self.ebp + self.esp) as usize] = self.parse_source().unwrap();

        self.esp = self.esp - 1;
    }

    fn pop(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        unsafe {
            *destination = self.stack[(self.ebp + self.esp + 1) as usize];
        }

        self.esp = self.esp + 1;
    }

    fn cmp(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap().to_owned();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let source = self.parse_source().unwrap();

        unsafe {
            if *destination > source{
                self.sf = false;
                self.zf = false;
            } else if *destination == source {
                self.sf = false;
                self.zf = true;
            } else if *destination < source {
                self.sf = true;
                self.zf = false;
            }
        }
    }

    fn jump(&mut self) {
        let instruction = self.text[self.eip].to_owned();

        self.go_from_here(1);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immediate data".to_string(), false) {
            return;
        }

        let displacement = self.text[self.eip].get_int_value();
        self.go_from_here(1);


        match instruction.get_token_value() {
            TokenValue::JE => {
                if self.zf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            TokenValue::JNE => {
                if !self.zf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            TokenValue::JG => {
                if !self.sf && !self.zf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            TokenValue::JGE => {
                if !self.sf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            TokenValue::JL => {
                if self.sf && !self.zf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            TokenValue::JLE => {
                if self.sf {
                    self.go_from_here(displacement);
                } else {
                    self.go_from_here(1);
                }
            },
            _ => {},
        }
    }

    fn call(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immedate data".to_string(), false) {
            return;
        }

        let displacement = self.text[self.eip].get_int_value();
        self.go_from_here(1);

        self.stack[(self.ebp + self.esp) as usize] = self.eip as i32;
        self.esp = self.esp - 1;

        self.go_from_here(displacement);
    }

    fn ret(&mut self) {
        self.eip = self.stack[(self.ebp + self.esp + 1) as usize] as usize;
        self.esp = self.esp + 1;
    }

    pub fn get_eax(&self) -> i32 {
        self.eax
    }

    pub fn get_ebx(&self) -> i32 {
        self.ebx
    }

    pub fn get_ecx(&self) -> i32 {
        self.ecx
    }

    pub fn get_edx(&self) -> i32 {
        self.edx
    }

    pub fn get_text(&self) -> Vec<Token> {
        self.text.to_owned()
    }

    pub fn run(&mut self) {
        self.preprocess();

        loop {
            match self.text[self.eip].get_token_type() {
                TokenType::INSTRUCTION => {
                    match self.text[self.eip].get_token_value() {
                        TokenValue::MOV => self.mov(),
                        TokenValue::ADD | TokenValue::SUB | TokenValue::MUL | TokenValue::DIV => self.arithmetic(),
                        TokenValue::PUSH => self.push(),
                        TokenValue::POP => self.pop(),
                        TokenValue::CMP => self.cmp(),
                        TokenValue::JE | TokenValue::JNE | TokenValue::JG | TokenValue::JGE | TokenValue::JL |
                            TokenValue::JLE => self.jump(),
                        TokenValue::CALL => self.call(),
                        TokenValue::RET => self.ret(),
                        TokenValue::INT => break,
                        _ => {},
                    }
                },
                TokenType::LABEL => {
                    self.go_from_here(1);
                    if !self.expect_token_value(TokenValue::COLON, ":".to_string(), true) {
                    }
                },
                _ => self.error_report(&format!("run Unexpected token: {}", self.text[self.eip].get_token_name())),
            }
        }
    }
}

