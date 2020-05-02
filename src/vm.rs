use crate::token::*;
use crate::scanner::*;
use std::collections::HashMap;
use std::vec::Vec;
use std::result::Result;
use std::convert::TryInto;

const MAX: usize = 1024;
// const BYTE: u32= 0b1111_1111_1111_1111_1111_1111_0000_0000;
// const WORD: u32 = 0b1111_1111_1111_1111_0000_0000_0000_0000;

/// Visual Machine for x86 assembly
pub struct VM {
    /// simulate the `stack`
    stack: [i32; MAX],
    /// simulate the `memory`
    memory: [i32; MAX],
    /// simulate the `text`
    text: Vec<Token>,
    /// label location table, to implement `call` instruction.
    index: HashMap<String, i32>,
    /// `eax`, accumulator register
    eax: i32,
    /// `ebx`, base register
    ebx: i32,
    /// `ecx`, counter register
    ecx: i32,
    /// `edx`, data register
    edx: i32,
    /// `esi`, source index register
    esi: i32,
    /// `edi`, destination index register
    edi: i32,
    /// `esp`, stack pointer register
    esp: i32,
    /// `ebp`, base pointer register
    ebp: i32,
    /// `eip`, instruction pointer register
    eip: usize,
    /// `zf`, zero flag
    zf: bool,
    /// `sf`, symbol flag
    sf: bool,
    /// lexical scanner
    scanner: Scanner,
    /// error flag
    error_flag_: bool,
}

#[allow(dead_code)]
impl VM {
    /// New VM from a assembly source file.
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
            ebp: (MAX - 1) as i32,
            eip: 0,
            zf: false,
            sf: false,
            scanner: Scanner::new(source_file_name),
            error_flag_: false,
        }
    }

    fn error_syntax(&mut self, msg: &String) {
        self.error_flag_ = true;
        panic!("{}", msg);
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

    /// change `eip`.
    ///
    /// eip += displacement;
    fn go_from_here(&mut self, displacement: i32) {
        self.eip = (self.eip as i32 + displacement) as usize;
    }

    /// Preprocess assembly source code.
    ///
    /// 1. Read all token from source file, and store into `self.text`.
    /// 2. Record the location of `label`, and store into `self.index`.
    /// 3. Replace the the `label` in `call label` instruction with the corresponding displacement.
    fn preprocess(&mut self) {
        let mut count = -1;
        let mut entrance = 0;

        loop {
            let last_token = self.scanner.get_token();

            self.scanner.get_next_token();
            count = count + 1;

            let token = self.scanner.get_token();

            if token.get_token_value() == TokenValue::COLON {
                if last_token.get_token_type() != TokenType::LABEL {
                    panic!("Missing \"label\"");
                }

                self.index.insert(last_token.get_token_name(), count - 1);

                match last_token.get_token_name().as_str() {
                    "main" | "start" | "_main" | "_start" => entrance = count - 1,
                    _ => {},
                }
            }

            match token.get_token_type() {
                TokenType::END_OF_FILE => break,
                _ => self.text.push(token),
            }
        }

        let mut flag = false;
        count = -1;

        for token in &mut self.text {
            count = count + 1;

            if !flag {
                match token.get_token_value() {
                    TokenValue::CALL | TokenValue::JMP | TokenValue::JE | TokenValue::JNE | TokenValue::JG | TokenValue::JGE | TokenValue::JL
                        | TokenValue::JLE => {
                            flag = true;
                    },
                    _ => {},
                }
            } else {
                if token.get_token_type() != TokenType::LABEL {
                    panic!("Expected \"label\", but find \"{}\"", token.get_token_name());
                }

                let label_name = token.get_token_name();

                if !self.index.contains_key(&label_name) {
                    println!("index: {:?}", self.index);
                    panic!("Unknown label: \"{}\"", label_name);
                }

                let label_address = self.index.get(&label_name).unwrap();

                token.set_token_type(TokenType::IMMEDIATE_DATA);
                token.set_int_value(label_address - count - 1);

                flag = false;
            }
        }

        self.eip = entrance as usize;
    }

    fn parse_register(&mut self) -> Result<(*mut i32, u8, u8), String> {
        self.go_from_here(1);

        match self.text[self.eip - 1].get_token_value() {
            TokenValue::EAX => return Ok((&mut self.eax as *mut i32, 0, 32)),
            TokenValue::AX => return Ok((&mut self.eax as *mut i32, 0, 16)),
            TokenValue::AH => return Ok((&mut self.eax as *mut i32, 8, 8)),
            TokenValue::AL => return Ok((&mut self.eax as *mut i32, 0, 8)),
            TokenValue::EBX => return Ok((&mut self.ebx as *mut i32, 0, 32)),
            TokenValue::BX => return Ok((&mut self.ebx as *mut i32, 0, 16)),
            TokenValue::BH => return Ok((&mut self.ebx as *mut i32, 8, 8)),
            TokenValue::BL => return Ok((&mut self.ebx as *mut i32, 0, 8)),
            TokenValue::ECX => return Ok((&mut self.ecx as *mut i32, 0, 32)),
            TokenValue::CX => return Ok((&mut self.ecx as *mut i32, 0, 16)),
            TokenValue::CH => return Ok((&mut self.ecx as *mut i32, 8, 8)),
            TokenValue::CL => return Ok((&mut self.ecx as *mut i32, 0, 8)),
            TokenValue::EDX => return Ok((&mut self.edx as *mut i32, 0, 32)),
            TokenValue::DX => return Ok((&mut self.edx as *mut i32, 0, 16)),
            TokenValue::DH => return Ok((&mut self.edx as *mut i32, 8, 8)),
            TokenValue::DL => return Ok((&mut self.edx as *mut i32, 0, 8)),
            TokenValue::ESI => return Ok((&mut self.esi as *mut i32, 0, 32)),
            TokenValue::SI => return Ok((&mut self.esi as *mut i32, 0, 16)),
            TokenValue::EDI => return Ok((&mut self.edi as *mut i32, 0, 32)),
            TokenValue::DI => return Ok((&mut self.edi as *mut i32, 0, 16)),
            TokenValue::ESP => return Ok((&mut self.esp as *mut i32, 0, 32)),
            TokenValue::SP => return Ok((&mut self.esp as *mut i32, 0, 16)),
            TokenValue::EBP => return Ok((&mut self.ebp as *mut i32, 0, 32)),
            TokenValue::BP => return Ok((&mut self.ebp as *mut i32, 0, 16)),
            _ => return Err("Flag registers can not be used as source!".to_string()),
        }
    }

    fn get_value((pointer, start, size): (*mut i32, u8, u8)) -> i32 {
        unsafe {
            let tmp = *pointer;
            tmp.overflowing_shl((32 - start - size).into()).0.overflowing_shr((32 - size).into()).0
        }
    }

    fn set_value(&self, (pointer, start, size): (*mut i32, u8, u8), value: i32) {
        unsafe {
            let result = match size {
                8 => {
                //0b1111_1111_1111_1111_1111_1111_0000_0000
                    let mut tmp = *pointer;
                    tmp = (tmp.overflowing_shr(start.into()).0 | (tmp.overflowing_shl((32 - start).into()).0)) & !0b0000_0000_0000_0000_0000_0000_1111_1111;
                    (tmp | (value & 0b0000_0000_0000_0000_0000_0000_1111_1111)).overflowing_shl(start.into()).0
                },
                16 => {
                //0b1111_1111_1111_1111_0000_0000_0000_0000
                    let mut tmp = *pointer;
                    tmp = ((tmp.overflowing_shr(start.into()).0) | (tmp.overflowing_shl((32 - start).into()).0)) & !0b0000_0000_0000_0000_1111_1111_1111_1111;
                    (tmp | (value & 0b0000_0000_0000_0000_1111_1111_1111_1111)).overflowing_shl(start.into()).0
                },
                32 => value,
                _ => std::i32::MAX,
            };

            *pointer = result;
        }
    }

    fn parse_immediate_data(&mut self) -> i32 {
        let sign = self.validate_token_value(TokenValue::MINUS, true);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immedidate date".to_string(), false) {
            panic!("{} Missing immediate data! {}", self.text[self.eip].get_token_location().to_string(), self.text[self.eip].get_token_name());
        }

        let mut value = self.text[self.eip].get_int_value();
        self.go_from_here(1);

        if sign {
            value = -value;
        }

        value
    }

    fn parse_binary_operation(&mut self, lhs: i32, precedence: i32) -> i32 {
        let mut result = lhs;

        loop {
            let current_precedence = self.text[self.eip].get_precedence();

            if current_precedence < precedence {
                return result;
            }

            let operation = self.text[self.eip].get_token_value();
            self.go_from_here(1);

            let mut rhs = match self.text[self.eip].get_token_type() {
                TokenType::REGISTER => {
                    let operand = self.parse_register().unwrap();
                    VM::get_value(operand)
                },
                TokenType::IMMEDIATE_DATA => {
                    self.parse_immediate_data()
                },
                _ => {
                    let value;
                    if self.text[self.eip].get_token_value() == TokenValue::MINUS {
                        value = self.parse_immediate_data();
                    } else {
                        self.error_report(&format!("Unexpected token: {}", self.text[self.eip].get_token_name()));
                        value = std::i32::MAX;
                    }

                    value
                },
            };

            let next_precedence = self.text[self.eip].get_precedence();

            if current_precedence < next_precedence {
                rhs = self.parse_binary_operation(rhs, current_precedence + 1);
            }

            result = match operation {
                TokenValue::PLUS => lhs + rhs,
                TokenValue::MINUS => lhs - rhs,
                TokenValue::TIMES => lhs * rhs,
                _ => std::i32::MAX,
            };
        }
    }

    fn parse_address(&mut self) -> i32 {
        let lhs = match self.text[self.eip].get_token_type() {
            TokenType::REGISTER => {
                    let operand = self.parse_register().unwrap();
                    VM::get_value(operand)
            },
            TokenType::IMMEDIATE_DATA => {
                self.parse_immediate_data()
            },
            _ => {
                let value;
                if self.text[self.eip].get_token_value() == TokenValue::MINUS {
                    value = self.parse_immediate_data();
                } else {
                    self.error_report(&format!("Unexpected token: {}", self.text[self.eip].get_token_name()));
                    value = std::i32::MAX;
                }

                value
            },
        };

        self.parse_binary_operation(lhs, 0)
    }

    fn parse_memory(&mut self) -> Result<(*mut i32, u8, u8), String> {
        let size = match self.text[self.eip].get_token_value() {
            TokenValue::BYTE => 8,
            TokenValue::WORD => 16,
            TokenValue::DWORD => 32,
            _ => 0,
        };

        self.go_from_here(1);

        if !self.expect_token_value(TokenValue::PTR, "ptr".to_string(), true) {
            return Err("Missing \"PTR\" !".to_string());
        }

        if !self.expect_token_value(TokenValue::LBRACK, "[".to_string(), true) {
            return Err("Missing left brack '[' !".to_string());
        }

        let mem_add: usize = match self.parse_address().try_into() {
            Ok(mem_add) => mem_add,
            Err(err) => panic!("Invaild memory address: {}", err),
        };

        if !self.expect_token_value(TokenValue::RBRACK, "]".to_string(), true) {
            return Err("Missing right brack ']' !".to_string());
        }

        return Ok((&mut self.memory[mem_add] as *mut i32, 0, size));
    }

    fn parse_source(&mut self) -> Result<i32, String> {
        match self.text[self.eip].get_token_value() {
            TokenValue::BYTE | TokenValue::WORD | TokenValue::DWORD => {
                match self.parse_memory() {
                    Ok(source) => return Ok(VM::get_value(source)),
                    Err(err) => return Err(err),
                }
            },
            _ => {},
        }

        if self.validate_token_type(TokenType::REGISTER, false) {
            match self.parse_register() {
                Ok(source) => return Ok(VM::get_value(source)),
                Err(err) => return Err(err),
            }
        } else if self.validate_token_type(TokenType::IMMEDIATE_DATA, false) || self.validate_token_value(TokenValue::MINUS, false) {
            return Ok(self.parse_immediate_data());
        } else {
            self.error_report(&format!("Unexpected token: {}", self.text[self.eip].get_token_name()));
            return Err(format!("{}: Unexpected token: {}", self.text[self.eip].get_token_location().to_string(), self.text[self.eip].get_token_name()));
        }
    }

    fn parse_destination(&mut self) -> Result<(*mut i32, u8, u8), String> {
        match self.text[self.eip].get_token_value() {
            TokenValue::BYTE | TokenValue::WORD | TokenValue::DWORD => {
                return self.parse_memory();
            },
            _ => {},
        }

        if self.validate_token_type(TokenType::REGISTER, false) {
            return self.parse_register();
        } else {
            self.error_report(&format!("Unexpected token: {}", self.text[self.eip].get_token_name()));
            return Err(format!("{}: Unexpected token: {}", self.text[self.eip].get_token_location().to_string(), self.text[self.eip].get_token_name()));
        }
    }

    /// `mov` instruction
    ///
    /// mov &lt;reg&gt;, &lt;reg&gt;
    ///
    /// mov &lt;reg&gt;, &lt;mem&gt;
    ///
    /// mov &lt;mem&gt;, &lt;reg&gt;
    ///
    /// mov &lt;reg&gt;, &lt;const&gt;
    ///
    /// mov &lt;mem&gt;, &lt;const&gt;
    fn mov(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let result = self.parse_source().unwrap();

/*
        if destination[2] != source[2] {
            panic!("Two operands must have same length!");
        }
*/
        self.set_value(destination, result);
    }

    fn set_flag(&mut self, result: i32) {
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
    }

    /// binary operation, including `add`, `sub`, `and`, `or`, `xor`.
    ///
    /// bop &lt;reg&gt;, &lt;reg&gt;
    ///
    /// bop &lt;reg&gt;, &lt;mem&gt;
    ///
    /// bop &lt;mem&gt;, &lt;reg&gt;
    ///
    /// bop &lt;reg&gt;, &lt;con&gt;
    ///
    /// bop &lt;mem&gt;, &lt;con&gt;
    fn binary_operation(&mut self) {
        let instruction = self.text[self.eip].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let result = match instruction.get_token_value() {
            TokenValue::ADD => VM::get_value(destination) + self.parse_source().unwrap(),
            TokenValue::SUB => VM::get_value(destination) - self.parse_source().unwrap(),
            TokenValue::AND => VM::get_value(destination) & self.parse_source().unwrap(),
            TokenValue::OR => VM::get_value(destination) | self.parse_source().unwrap(),
            TokenValue::XOR => VM::get_value(destination) ^ self.parse_source().unwrap(),
            _ => std::i32::MAX,
        };

        self.set_flag(result);

        self.set_value(destination, result);
    }

    /// `mul` instruction, only support for integer.
    ///
    /// mul &lt;reg32&gt;, &lt;reg32&gt;
    ///
    /// mul &lt;reg32&gt;, &lt;mem&gt;
    ///
    /// mul &lt;reg32&gt;, &lt;reg32&gt;, &lt;con&gt;
    ///
    /// mul &lt;reg32&gt;, &lt;mem&gt;, &lt;con &gt;
    fn multiplication(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::REGISTER, "register".to_string(), false) {
            return;
        }

        let destination = self.parse_register().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let first_operand = self.parse_destination().unwrap();
        let second_operand;
        let result;

        if self.validate_token_value(TokenValue::COMMA, true) {
            if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immedidate data".to_string(), false) {
                return;
            }

            second_operand = self.text[self.eip].get_int_value();
            self.go_from_here(1);

            result = VM::get_value(first_operand) * second_operand;

            self.set_flag(result);

            self.set_value(destination, result);
        } else {
            result = VM::get_value(destination) * VM::get_value(first_operand);

            self.set_flag(result);

            self.set_value(destination, result);
        }
    }

    /// `div` instruction
    ///
    /// div &lt;reg32&gt;
    ///
    /// div &lt;mem&gt;
    fn division(&mut self) {
        self.go_from_here(1);

        let operand = self.parse_destination().unwrap();
        let divisor = VM::get_value(operand);

        let dividend: i64 = (self.edx as i64).overflowing_shl(32).0  + self.eax as i64;

        self.eax = (dividend / divisor as i64) as i32;
        self.edx = (dividend % divisor as i64) as i32;
    }

    /// unary operation, including `inc`, `dec`, `not`, `neg`.
    ///
    /// uop &lt;reg32&gt;
    ///
    /// uop &lt;mem&gt;
    fn unary_operation(&mut self) {
        let instruction = self.text[self.eip].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        let result = match instruction.get_token_value() {
            TokenValue::INC => VM::get_value(destination) + 1,
            TokenValue::DEC => VM::get_value(destination) - 1,
            TokenValue::NOT => !VM::get_value(destination),
            TokenValue::NEG => -VM::get_value(destination),
            _ => std::i32::MAX,
        };

        self.set_flag(result);

        self.set_value(destination, result);
    }

    fn bitshift(&mut self) {
        let instruction = self.text[self.eip].to_owned();
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immediate data".to_string(), false) {
            return;
        }

        let operand = self.text[self.eip].get_int_value();
        self.go_from_here(1);

        if operand > std::u8::MAX as i32 {
            self.error_report(&"Bitshift operand too big!".to_string());
        }

        let result = match instruction.get_token_value() {
            TokenValue::SHL => VM::get_value(destination).overflowing_shl(operand.try_into().unwrap()).0,
            TokenValue::SHR => VM::get_value(destination).overflowing_shr(operand.try_into().unwrap()).0,
            _ => std::i32::MAX,
        };

        self.set_flag(result);

        self.set_value(destination, result);
    }

    /// `push` instruction
    ///
    /// push &lt;reg32&gt;
    ///
    /// push &lt;mem&gt;
    ///
    /// push &lt;con32&gt;
    fn push(&mut self) {
        self.go_from_here(1);

        self.esp = self.esp - 1;
        self.stack[self.esp as usize] = self.parse_source().unwrap();
    }

    /// `pop` instruction
    ///
    /// pop &lt;reg32&gt;
    ///
    /// pop &lt;mem&gt;
    fn pop(&mut self) {
        self.go_from_here(1);

        let destination = self.parse_destination().unwrap();

        self.set_value(destination, self.stack[self.esp as usize]);
        self.esp = self.esp + 1;
    }

    /// `cmp` instruction
    /// cmp &lt;reg&gt;, &lt;reg&gt;
    ///
    /// cmp &lt;reg&gt;, &lt;mem&gt;
    ///
    /// cmp &lt;mem&gt;, &lt;reg&gt;
    ///
    /// cmp &lt;reg&gt;, &lt;con&gt;
    fn cmp(&mut self) {
        self.go_from_here(1);

        let operand = self.parse_destination().unwrap();
        let destination = VM::get_value(operand);

        if !self.expect_token_value(TokenValue::COMMA, ",".to_string(), true) {
            return;
        }

        let source = self.parse_source().unwrap();

        if destination > source{
            self.sf = false;
            self.zf = false;
        } else if destination == source {
            self.sf = false;
            self.zf = true;
        } else if destination < source {
            self.sf = true;
            self.zf = false;
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
            TokenValue::JMP => {
                self.go_from_here(displacement);
            },
            TokenValue::JE => {
                if self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JNE => {
                if !self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JG => {
                if !self.sf && !self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JGE => {
                if !self.sf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JL => {
                if self.sf && !self.zf {
                    self.go_from_here(displacement);
                }
            },
            TokenValue::JLE => {
                if self.sf {
                    self.go_from_here(displacement);
                }
            },
            _ => {},
        }
    }

    /// `call` instruction
    ///
    /// call &lt;label&gt;
    fn call(&mut self) {
        self.go_from_here(1);

        if !self.expect_token_type(TokenType::IMMEDIATE_DATA, "immedate data".to_string(), false) {
            return;
        }

        let displacement = self.text[self.eip].get_int_value();
        self.go_from_here(1);

        //println!("ebp: {}, esp: {}", self.ebp, self.esp);
        self.esp = self.esp - 1;
        self.stack[self.esp as usize] = self.eip as i32;

        self.go_from_here(displacement);
    }

    /// `ret` instruction
    fn ret(&mut self) {
        self.eip = self.stack[self.esp as usize] as usize;
        self.esp = self.esp + 1;
    }

    /// `enter` instruction
    fn enter(&mut self) {
        self.go_from_here(1);

        self.esp = self.esp - 1;
        self.stack[self.esp as usize] = self.ebp;

        self.ebp = self.esp;
    }

    /// `leave` instruction
    fn leave(&mut self) {
        self.go_from_here(1);

        self.esp = self.ebp;

        self.ebp = self.stack[self.esp as usize];
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

    /// Run vm.
    ///
    /// # Examples
    ///
    /// ```
    /// let vm = VM::new("./test.asm".to_string());
    /// vm.run();
    /// ```
    pub fn run(&mut self) {
        self.preprocess();

        loop {
            match self.text[self.eip].get_token_type() {
                TokenType::INSTRUCTION => {
                    match self.text[self.eip].get_token_value() {
                        TokenValue::MOV => self.mov(),
                        TokenValue::ADD | TokenValue::SUB | TokenValue::AND |
                            TokenValue::OR | TokenValue::XOR => self.binary_operation(),
                        TokenValue::MUL => self.multiplication(),
                        TokenValue::DIV => self.division(),
                        TokenValue::INC | TokenValue::DEC | TokenValue::NOT | TokenValue::NEG => self.unary_operation(),
                        TokenValue::SHL | TokenValue::SHR => self.bitshift(),
                        TokenValue::PUSH => self.push(),
                        TokenValue::POP => self.pop(),
                        TokenValue::CMP => self.cmp(),
                        TokenValue::JMP | TokenValue::JE | TokenValue::JNE | TokenValue::JG | TokenValue::JGE | TokenValue::JL |
                            TokenValue::JLE => self.jump(),
                        TokenValue::CALL => self.call(),
                        TokenValue::RET => self.ret(),
                        TokenValue::ENTER => self.enter(),
                        TokenValue::LEAVE => self.leave(),
                        TokenValue::INT => break,
                        _ => {},
                    }
                },
                TokenType::LABEL => {
                    self.go_from_here(1);
                    if !self.expect_token_value(TokenValue::COLON, ":".to_string(), true) {
                    }
                },
                _ => panic!("{} Unexpected token: {}", self.text[self.eip].get_token_location().to_string(), self.text[self.eip].get_token_name()),
            }
            println!("ebp: {}, esp: {}", self.ebp, self.esp);
        }
    }
}

