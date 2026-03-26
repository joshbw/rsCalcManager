// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Command enums for the calculator engine.
//!
//! These map directly to the C++ Command.h and CCommand.h definitions.

/// Commands for the unit conversion calculator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitConversionCommand {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Decimal,
    Negate,
    Backspace,
    Clear,
    Reset,
    None,
}

/// Expression command types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandType {
    UnaryCommand,
    BinaryCommand,
    OperandCommand,
    Parentheses,
}

/// Memory-related commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MemoryCommand {
    MemorizeNumber = 330,
    MemorizedNumberLoad = 331,
    MemorizedNumberAdd = 332,
    MemorizedNumberSubtract = 333,
    MemorizedNumberClearAll = 334,
    MemorizedNumberClear = 335,
}

/// Op-code type alias (matches C++ `OpCode` / `int` used for command IDs).
pub type OpCode = i32;

// IDC_* constants matching CCommand.h
pub const IDC_SIGN: OpCode = 80;
pub const IDC_CLEAR: OpCode = 81;
pub const IDC_CENTR: OpCode = 82;
pub const IDC_BACK: OpCode = 83;
pub const IDC_PNT: OpCode = 84;
pub const IDC_AND: OpCode = 86;
pub const IDC_OR: OpCode = 87;
pub const IDC_XOR: OpCode = 88;
pub const IDC_LSHF: OpCode = 89;
pub const IDC_RSHF: OpCode = 90;
pub const IDC_DIV: OpCode = 91;
pub const IDC_MUL: OpCode = 92;
pub const IDC_ADD: OpCode = 93;
pub const IDC_SUB: OpCode = 94;
pub const IDC_MOD: OpCode = 95;
pub const IDC_ROOT: OpCode = 96;
pub const IDC_PWR: OpCode = 97;
pub const IDC_CHOP: OpCode = 98;
pub const IDC_ROL: OpCode = 99;
pub const IDC_ROR: OpCode = 100;
pub const IDC_COM: OpCode = 101;
pub const IDC_SIN: OpCode = 102;
pub const IDC_COS: OpCode = 103;
pub const IDC_TAN: OpCode = 104;
pub const IDC_SINH: OpCode = 105;
pub const IDC_COSH: OpCode = 106;
pub const IDC_TANH: OpCode = 107;
pub const IDC_LN: OpCode = 108;
pub const IDC_LOG: OpCode = 109;
pub const IDC_SQRT: OpCode = 110;
pub const IDC_SQR: OpCode = 111;
pub const IDC_CUB: OpCode = 112;
pub const IDC_FAC: OpCode = 113;
pub const IDC_REC: OpCode = 114;
pub const IDC_DMS: OpCode = 115;
pub const IDC_CUBEROOT: OpCode = 116;
pub const IDC_POW10: OpCode = 117;
pub const IDC_PERCENT: OpCode = 118;
pub const IDC_FE: OpCode = 119;
pub const IDC_PI: OpCode = 120;
pub const IDC_EQU: OpCode = 121;
pub const IDC_MCLEAR: OpCode = 122;
pub const IDC_RECALL: OpCode = 123;
pub const IDC_STORE: OpCode = 124;
pub const IDC_MPLUS: OpCode = 125;
pub const IDC_MMINUS: OpCode = 126;
pub const IDC_EXP: OpCode = 127;
pub const IDC_OPENP: OpCode = 128;
pub const IDC_CLOSEP: OpCode = 129;
pub const IDC_0: OpCode = 130;
pub const IDC_1: OpCode = 131;
pub const IDC_2: OpCode = 132;
pub const IDC_3: OpCode = 133;
pub const IDC_4: OpCode = 134;
pub const IDC_5: OpCode = 135;
pub const IDC_6: OpCode = 136;
pub const IDC_7: OpCode = 137;
pub const IDC_8: OpCode = 138;
pub const IDC_9: OpCode = 139;
pub const IDC_A: OpCode = 140;
pub const IDC_B: OpCode = 141;
pub const IDC_C: OpCode = 142;
pub const IDC_D: OpCode = 143;
pub const IDC_E: OpCode = 144;
pub const IDC_F: OpCode = 145;
pub const IDC_INV: OpCode = 146;
pub const IDC_SET_RESULT: OpCode = 147;

// Radix/mode commands
pub const IDM_HEX: OpCode = 313;
pub const IDM_DEC: OpCode = 314;
pub const IDM_OCT: OpCode = 315;
pub const IDM_BIN: OpCode = 316;
pub const IDM_QWORD: OpCode = 317;
pub const IDM_DWORD: OpCode = 318;
pub const IDM_WORD: OpCode = 319;
pub const IDM_BYTE: OpCode = 320;
pub const IDM_DEG: OpCode = 321;
pub const IDM_RAD: OpCode = 322;
pub const IDM_GRAD: OpCode = 323;
pub const IDM_DEGREES: OpCode = 324;

// Extended unary operators (string-mapped)
pub const IDC_SEC: OpCode = 400;
pub const IDC_CSC: OpCode = 402;
pub const IDC_COT: OpCode = 404;
pub const IDC_SECH: OpCode = 406;
pub const IDC_CSCH: OpCode = 408;
pub const IDC_COTH: OpCode = 410;
pub const IDC_POW2: OpCode = 412;
pub const IDC_ABS: OpCode = 413;
pub const IDC_FLOOR: OpCode = 414;
pub const IDC_CEIL: OpCode = 415;
pub const IDC_ROLC: OpCode = 416;
pub const IDC_RORC: OpCode = 417;

// Extended binary operators
pub const IDC_LOGBASEY: OpCode = 500;
pub const IDC_NAND: OpCode = 501;
pub const IDC_NOR: OpCode = 502;
pub const IDC_RSHFL: OpCode = 505;

// Special commands
pub const IDC_RAND: OpCode = 600;
pub const IDC_EULER: OpCode = 601;

// Binary edit positions
pub const IDC_BINEDITSTART: OpCode = 700;
pub const IDC_BINEDITEND: OpCode = 763;

// Ranges
pub const IDC_FIRSTCONTROL: OpCode = IDC_SIGN;
pub const IDC_UNARYFIRST: OpCode = IDC_CHOP;
pub const IDC_UNARYLAST: OpCode = IDC_PERCENT;
pub const IDC_UNARYEXTENDEDFIRST: OpCode = 400;
pub const IDC_UNARYEXTENDEDLAST: OpCode = IDC_RORC;
pub const IDC_BINARYEXTENDEDFIRST: OpCode = 500;
pub const IDC_BINARYEXTENDEDLAST: OpCode = IDC_RSHFL;

// Engine string ID range
pub const IDS_ENGINESTR_FIRST: OpCode = 0;
pub const IDS_ENGINESTR_MAX: OpCode = 200;

/// High-level calculator command enum matching `CalculationManager::Command`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum Command {
    CommandNULL = 0,

    // Sign, clear, backspace
    CommandSIGN = 80,
    CommandCLEAR = 81,
    CommandCENTR = 82,
    CommandBACK = 83,
    CommandPNT = 84,

    // Binary operators
    CommandAnd = 86,
    CommandOR = 87,
    CommandXor = 88,
    CommandLSHF = 89,
    CommandRSHF = 90,
    CommandDIV = 91,
    CommandMUL = 92,
    CommandADD = 93,
    CommandSUB = 94,
    CommandMOD = 95,
    CommandROOT = 96,
    CommandPWR = 97,

    // Unary operators
    CommandCHOP = 98,
    CommandROL = 99,
    CommandROR = 100,
    CommandNot = 101,

    CommandSIN = 102,
    CommandCOS = 103,
    CommandTAN = 104,
    CommandSINH = 105,
    CommandCOSH = 106,
    CommandTANH = 107,
    CommandLN = 108,
    CommandLOG = 109,
    CommandSQRT = 110,
    CommandSQR = 111,
    CommandCUB = 112,
    CommandFAC = 113,
    CommandREC = 114,
    CommandDMS = 115,
    CommandCUBEROOT = 116,
    CommandPOW10 = 117,
    CommandPERCENT = 118,

    CommandFE = 119,
    CommandPI = 120,
    CommandEQU = 121,

    // Memory
    CommandMCLEAR = 122,
    CommandRECALL = 123,
    CommandSTORE = 124,
    CommandMPLUS = 125,
    CommandMMINUS = 126,

    CommandEXP = 127,
    CommandOPENP = 128,
    CommandCLOSEP = 129,

    // Digits
    Command0 = 130,
    Command1 = 131,
    Command2 = 132,
    Command3 = 133,
    Command4 = 134,
    Command5 = 135,
    Command6 = 136,
    Command7 = 137,
    Command8 = 138,
    Command9 = 139,
    CommandA = 140,
    CommandB = 141,
    CommandC = 142,
    CommandD = 143,
    CommandE = 144,
    CommandF = 145,

    CommandINV = 146,
    CommandSET_RESULT = 147,

    // Mode commands
    ModeBasic = 200,
    ModeScientific = 201,

    // Inverse trig
    CommandASIN = 202,
    CommandACOS = 203,
    CommandATAN = 204,
    CommandPOWE = 205,
    CommandASINH = 206,
    CommandACOSH = 207,
    CommandATANH = 208,

    ModeProgrammer = 209,

    // Radix/width
    CommandHex = 313,
    CommandDec = 314,
    CommandOct = 315,
    CommandBin = 316,
    CommandQword = 317,
    CommandDword = 318,
    CommandWord = 319,
    CommandByte = 320,
    CommandDEG = 321,
    CommandRAD = 322,
    CommandGRAD = 323,
    CommandDegrees = 324,
    CommandHYP = 325,

    // Extended unary
    CommandSEC = 400,
    CommandASEC = 401,
    CommandCSC = 402,
    CommandACSC = 403,
    CommandCOT = 404,
    CommandACOT = 405,
    CommandSECH = 406,
    CommandASECH = 407,
    CommandCSCH = 408,
    CommandACSCH = 409,
    CommandCOTH = 410,
    CommandACOTH = 411,
    CommandPOW2 = 412,
    CommandAbs = 413,
    CommandFloor = 414,
    CommandCeil = 415,
    CommandROLC = 416,
    CommandRORC = 417,

    // Extended binary
    CommandLogBaseY = 500,
    CommandNand = 501,
    CommandNor = 502,
    CommandRSHFL = 505,

    // Special
    CommandRand = 600,
    CommandEuler = 601,

    // Binary edit
    CommandBINEDITSTART = 700,
    CommandBINEDITEND = 763,
}

impl Command {
    /// Convert from an opcode integer.
    #[must_use]
    pub fn from_opcode(op: OpCode) -> Option<Self> {
        // Binary position commands: 700-763
        if (700..=763).contains(&op) {
            return Some(Self::CommandBINEDITSTART);
        }
        // Use a match for the known values
        Some(match op {
            0 => Self::CommandNULL,
            80 => Self::CommandSIGN,
            81 => Self::CommandCLEAR,
            82 => Self::CommandCENTR,
            83 => Self::CommandBACK,
            84 => Self::CommandPNT,
            86 => Self::CommandAnd,
            87 => Self::CommandOR,
            88 => Self::CommandXor,
            89 => Self::CommandLSHF,
            90 => Self::CommandRSHF,
            91 => Self::CommandDIV,
            92 => Self::CommandMUL,
            93 => Self::CommandADD,
            94 => Self::CommandSUB,
            95 => Self::CommandMOD,
            96 => Self::CommandROOT,
            97 => Self::CommandPWR,
            98 => Self::CommandCHOP,
            99 => Self::CommandROL,
            100 => Self::CommandROR,
            101 => Self::CommandNot,
            102 => Self::CommandSIN,
            103 => Self::CommandCOS,
            104 => Self::CommandTAN,
            105 => Self::CommandSINH,
            106 => Self::CommandCOSH,
            107 => Self::CommandTANH,
            108 => Self::CommandLN,
            109 => Self::CommandLOG,
            110 => Self::CommandSQRT,
            111 => Self::CommandSQR,
            112 => Self::CommandCUB,
            113 => Self::CommandFAC,
            114 => Self::CommandREC,
            115 => Self::CommandDMS,
            116 => Self::CommandCUBEROOT,
            117 => Self::CommandPOW10,
            118 => Self::CommandPERCENT,
            119 => Self::CommandFE,
            120 => Self::CommandPI,
            121 => Self::CommandEQU,
            122 => Self::CommandMCLEAR,
            123 => Self::CommandRECALL,
            124 => Self::CommandSTORE,
            125 => Self::CommandMPLUS,
            126 => Self::CommandMMINUS,
            127 => Self::CommandEXP,
            128 => Self::CommandOPENP,
            129 => Self::CommandCLOSEP,
            130 => Self::Command0,
            131 => Self::Command1,
            132 => Self::Command2,
            133 => Self::Command3,
            134 => Self::Command4,
            135 => Self::Command5,
            136 => Self::Command6,
            137 => Self::Command7,
            138 => Self::Command8,
            139 => Self::Command9,
            140 => Self::CommandA,
            141 => Self::CommandB,
            142 => Self::CommandC,
            143 => Self::CommandD,
            144 => Self::CommandE,
            145 => Self::CommandF,
            146 => Self::CommandINV,
            147 => Self::CommandSET_RESULT,
            200 => Self::ModeBasic,
            201 => Self::ModeScientific,
            202 => Self::CommandASIN,
            203 => Self::CommandACOS,
            204 => Self::CommandATAN,
            205 => Self::CommandPOWE,
            206 => Self::CommandASINH,
            207 => Self::CommandACOSH,
            208 => Self::CommandATANH,
            209 => Self::ModeProgrammer,
            313 => Self::CommandHex,
            314 => Self::CommandDec,
            315 => Self::CommandOct,
            316 => Self::CommandBin,
            317 => Self::CommandQword,
            318 => Self::CommandDword,
            319 => Self::CommandWord,
            320 => Self::CommandByte,
            321 => Self::CommandDEG,
            322 => Self::CommandRAD,
            323 => Self::CommandGRAD,
            324 => Self::CommandDegrees,
            325 => Self::CommandHYP,
            400 => Self::CommandSEC,
            401 => Self::CommandASEC,
            402 => Self::CommandCSC,
            403 => Self::CommandACSC,
            404 => Self::CommandCOT,
            405 => Self::CommandACOT,
            406 => Self::CommandSECH,
            407 => Self::CommandASECH,
            408 => Self::CommandCSCH,
            409 => Self::CommandACSCH,
            410 => Self::CommandCOTH,
            411 => Self::CommandACOTH,
            412 => Self::CommandPOW2,
            413 => Self::CommandAbs,
            414 => Self::CommandFloor,
            415 => Self::CommandCeil,
            416 => Self::CommandROLC,
            417 => Self::CommandRORC,
            500 => Self::CommandLogBaseY,
            501 => Self::CommandNand,
            502 => Self::CommandNor,
            505 => Self::CommandRSHFL,
            600 => Self::CommandRand,
            601 => Self::CommandEuler,
            _ => return None,
        })
    }
}

/// Returns true if the opcode is a binary operator.
#[must_use]
pub fn is_binary_operator(op: OpCode) -> bool {
    (IDC_AND..=IDC_PWR).contains(&op)
        || (IDC_BINARYEXTENDEDFIRST..=IDC_BINARYEXTENDEDLAST).contains(&op)
}

/// Returns true if the opcode is a standard unary operator.
#[must_use]
pub fn is_unary_operator(op: OpCode) -> bool {
    (IDC_UNARYFIRST..=IDC_UNARYLAST).contains(&op)
        || (IDC_UNARYEXTENDEDFIRST..=IDC_UNARYEXTENDEDLAST).contains(&op)
}

/// Returns true if the opcode is a digit (0-F).
#[must_use]
pub fn is_digit_op(op: OpCode) -> bool {
    (IDC_0..=IDC_F).contains(&op)
}

/// Returns true if the opcode is a binary edit position toggle.
#[must_use]
pub fn is_bin_edit_op(op: OpCode) -> bool {
    (IDC_BINEDITSTART..=IDC_BINEDITEND).contains(&op)
}
