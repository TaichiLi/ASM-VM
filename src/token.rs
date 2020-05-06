#![allow(dead_code)]

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
/// Type of token
pub enum TokenType {
    /// instruction, such as `mov`
    INSTRUCTION,
    /// register, such as `eax`
    REGISTER,
    ///keyword, such as `ptr`
    KEYWORD,
    /// symbol, such as `+`, `-`, `*`
    SYMBOL,
    /// immediate date, such as `123
    IMMEDIATE_DATA,
    /// label, such as `main`
    LABEL,
    /// eof
    END_OF_FILE,
}

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
/// Value of token
pub enum TokenValue {
    /// instruction
    /// `mov`
    MOV,
    /// `movzx`
    MOVZX,
    /// `movsx`
    MOVSX,
    /// `add`
    ADD,
    /// `sub`
    SUB,
    /// `inc`
    INC,
    /// `dec`
    DEC,
    /// `mul`
    MUL,
    /// `imul`
    IMUL,
    /// `div`
    DIV,
    /// `idiv`
    IDIV,
    /// `and`
    AND,
    /// `or`
    OR,
    /// `xor`
    XOR,
    /// `not`
    NOT,
    /// `neg`
    NEG,
    /// `shl`
    SHL,
    /// `shr`
    SHR,
    /// `sar`
    SAR,
    /// `push`
    PUSH,
    /// `pop`
    POP,
    /// `cmp`
    CMP,
    /// `jmp`
    JMP,
    /// `je`
    JE,
    /// `jne`
    JNE,
    /// `jg`
    JG,
    /// `jge`
    JGE,
    /// `jl`
    JL,
    /// `jle`
    JLE,
    /// `ja`
    JA,
    /// `jae`
    JAE,
    /// `jb`
    JB,
    /// `jbe`
    JBE,
    /// `call`
    CALL,
    /// `ret`
    RET,
    /// `enter`
    ENTER,
    /// `leave`
    LEAVE,
    /// `int`
    INT,

    /// register
    /// `eax`
    EAX,
    /// `ax`
    AX,
    /// `ah`
    AH,
    /// `al`
    AL,
    /// `ebx`
    EBX,
    /// `bx`
    BX,
    /// `bh`
    BH,
    /// `bl`
    BL,
    /// `ecx`
    ECX,
    /// `cx`
    CX,
    /// `ch`
    CH,
    /// `cl`
    CL,
    /// `edx`
    EDX,
    /// `dx`
    DX,
    /// `dh`
    DH,
    /// `dl`
    DL,
    /// `esi`
    ESI,
    /// `si`
    SI,
    /// `edi`
    EDI,
    /// `di`
    DI,
    /// `esp`
    ESP,
    /// `sp`
    SP,
    /// `ebp`
    EBP,
    /// `bp`
    BP,
    /// `eip`
    EIP,

    /// keyword
    /// `ptr`
    PTR,
    /// `byte`
    BYTE,
    /// `word`
    WORD,
    /// `dword`
    DWORD,

    /// symbol
    /// `+`
    PLUS,
    /// `-`
    MINUS,
    /// `*`
    TIMES,
    /// `;`
    SEMICOLON,
    /// `,`
    COMMA,
    /// `[`
    LBRACK,
    /// `]'
    RBRACK,
    /// `:`
    COLON,

    /// immediate data
    INTEGER_LITERAL,
    /// label
    LABEL,

    /// eof
    END_OF_FILE,

    /// unknown token
    UNKNOWN,
}

impl TokenType {
    fn to_string(&self) -> String {
        let buffer = match self {
            TokenType::INSTRUCTION => "instruction",
            TokenType::REGISTER => "register",
            TokenType::KEYWORD => "keyword",
            TokenType::SYMBOL => "symbol",
            TokenType::IMMEDIATE_DATA => "immediate data",
            TokenType::LABEL => "label",
            TokenType::END_OF_FILE => "eof",
        };

        buffer.to_string()
    }
}

#[derive(Default)]
#[derive(Clone)]
/// Location of token
pub struct TokenLocation {
    source_file_name_: String,
    line_: i32,
    column_: i32
}

impl TokenLocation {
    pub fn new(souce_file_name: String, line: i32, column: i32) -> Self {
        TokenLocation {
            source_file_name_: souce_file_name,
            line_: line,
            column_: column,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}:", self.source_file_name_, self.line_, self.column_)
    }
}

#[derive(Clone)]
/// Lexical token
pub struct Token {
    type_: TokenType,
    value_: TokenValue,
    location_: TokenLocation,
    name_: String,
    /// value of integer literal
    int_value_: u32,
    /// precedence of operators, such as `+`, `-`, `*`
    symbol_precedence_: i32,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            type_: TokenType::INSTRUCTION,
            value_: TokenValue::INT,
            location_: Default::default(),
            name_: "int".to_string(),
            int_value_: 0,
            symbol_precedence_: -1,
        }
    }
}

impl Token {
    pub fn new_token(token_type: TokenType, token_value: TokenValue, loc: TokenLocation, name: String) -> Self {
        Token {
            type_: token_type,
            value_: token_value,
            location_: loc,
            name_: name,
            ..Default::default()
        }
    }

    pub fn new_int_token(loc: TokenLocation, name: String, int_value: u32) -> Self {
        Token {
            type_: TokenType::IMMEDIATE_DATA,
            value_: TokenValue::INTEGER_LITERAL,
            location_: loc,
            name_: name,
            int_value_: int_value,
            ..Default::default()
        }
    }

    pub fn new_symbol_token(token_value: TokenValue, loc: TokenLocation, name: String, prcedence: i32) -> Self {
        Token {
            type_: TokenType::SYMBOL,
            value_: token_value,
            location_: loc,
            name_: name,
            symbol_precedence_: prcedence,
            ..Default::default()
        }
    }

    pub fn get_token_location(&self) -> TokenLocation {
        self.location_.to_owned()
    }

    pub fn get_token_type(&self) -> TokenType {
        self.type_
    }

    pub fn get_token_value(&self) -> TokenValue {
        self.value_
    }

    pub fn get_token_name(&self) -> String {
       self.name_.to_owned()
    }

    pub fn get_int_value(&self) -> u32 {
        if self.type_ != TokenType::IMMEDIATE_DATA {
            panic!("{} is not a immediate data token. Only immediate data token have precedence!", self.name_);
        }

        self.int_value_
    }

    pub fn get_precedence(&self) -> i32 {
        if self.type_ != TokenType::SYMBOL {
            panic!("{} is not a symbol token. Only symbol token have precedence!", self.name_);
        }

        self.symbol_precedence_
    }

    pub fn set_token_type(&mut self, token_type: TokenType) {
        self.type_ = token_type;
    }

    pub fn set_int_value(&mut self, int_value: i32) {
        if self.type_ != TokenType::IMMEDIATE_DATA {
            panic!("{} is not a immediate data token. Only immediate data token have precedence!", self.name_);
        }

        self.int_value_ = int_value as u32;
    }

    pub fn to_string(&self) -> String {
        format!("{} Token Type: {}, Token Value: {}", self.location_.to_string(),self.type_.to_string(),
                self.name_)
    }

}
