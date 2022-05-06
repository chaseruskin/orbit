//! VHDL tokenizer

#[derive(Debug, PartialEq, Clone)]
/// (Line, Col)
struct Position(usize, usize);

impl Position {
    /// Creates a new `Position` struct as line 1, col 0.
    fn new() -> Self {
        Position(1, 0)
    }

    /// Increments the column counter by 1.
    fn next_col(&mut self) {
        self.1 += 1;
    }   

    /// Increments the column counter by 1. If the current char `c` is a newline,
    /// it will then drop down to the next line.
    fn step(&mut self, c: &char) {
        if c == &'\n' {
            self.next_line();
        }
        // @TODO step by +4 if encountered a tab?
        self.next_col();
    }

    /// Increments the line counter by 1.
    /// 
    /// Also resets the column counter to 0.
    fn next_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Access the line (`.0`) number.
    fn line(&self) -> usize {
        self.0
    }

    /// Access the col (`.1`) number.
    fn col(&self) -> usize {
        self.1
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
struct Token<T> {
    position: Position,
    ttype: T,
}

impl<T> Token<T> {
    /// Reveals the token type.
    fn unwrap(&self) -> &T {
        &self.ttype
    }

    /// Transforms the token into its type.
    fn take(self) -> T {
        self.ttype
    }

    /// Returns the position in the file where the token was captured.
    fn locate(&self) -> &Position {
        &self.position
    }

    /// Creates a new token.
    fn new(ttype: T, loc: Position) -> Self {
        Self {
            position: loc,
            ttype: ttype,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Comment {
    Single(String),
    Delimited(String),
}

impl Comment {
    fn as_str(&self) -> &str {
        match self {
            Self::Single(note) => note.as_ref(),
            Self::Delimited(note) => note.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Character(String);

impl Character {
    fn new(c: char) -> Self {
        Self(String::from(c))
    }

    fn as_str(&self) -> &str {
        &self.0.as_ref()
    }
}

#[derive(Debug, PartialEq)]
struct BitStrLiteral {
    width: Option<usize>,
    base: BaseSpec,
    literal: String,
}

impl BitStrLiteral {
    fn as_str(&self) -> &str {
        &self.literal
    }

    fn new(b: BaseSpec) -> Self {
        Self {
            width: None,
            base: b,
            literal: String::new(),
        }
    }

    fn literal(mut self, s: String) -> Self {
        self.literal = s;
        self
    }
}

// B|O|X|UB|UO|UX|SB|SO|SX|D
#[derive(Debug, PartialEq)]
enum BaseSpec {
    B,
    O,
    X,
    UB,
    UO,
    UX,
    SB,
    SO,
    SX,
    D
}

impl std::str::FromStr for BaseSpec {
    type Err = (); // @TODO handle errors
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "b"  => Self::B,
            "o"  => Self::O,
            "x"  => Self::X,
            "ub" => Self::UB,
            "uo" => Self::UO,
            "ux" => Self::UX,
            "sb" => Self::SB,
            "so" => Self::SO,
            "sx" => Self::SX,
            "d"  => Self::D,
            _ => panic!("invalid base specifier {}", s)
        })
    }
}

impl BaseSpec {
    fn as_str(&self) -> &str {
        match self {
            Self::B => "b",
            Self::O => "o",
            Self::X => "x",
            Self::UB => "ub",
            Self::UO => "uo",
            Self::UX => "ux",
            Self::SB => "sb",
            Self::SO => "so",
            Self::SX => "sx",
            Self::D => "d",
        }
    }
}

#[derive(Debug, PartialEq)]
enum Identifier {
    Basic(String),
    Extended(String),
}

impl Identifier {
    fn as_str(&self) -> &str {
        match self {
            Self::Basic(id) => id.as_ref(),
            Self::Extended(id) => id.as_ref(),
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Extended(id) => write!(f, "{}", id),
        }
    }
}

trait Tokenize {
    type TokenType;
    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>>;
} 

#[derive(Debug, PartialEq)]
enum VHDLToken {
    Whitespace,
    Comment(Comment),               // (String) 
    Identifier(Identifier),         // (String) ...can be general or extended (case-sensitive) identifier
    AbstLiteral,                    // (String)
    CharLiteral(Character),         // (char)
    StrLiteral(String),             // (String)
    BitStrLiteral(BitStrLiteral),   // (String)
    EOF,
    DoubleQuote,    // "
    // --- delimiters
    Ampersand,      // &
    SingleQuote,    // '
    ParenL,         // (
    ParenR,         // )
    Star,           // *
    Plus,           // +
    Comma,          // ,
    Dash,           // -
    Dot,            // .
    FwdSlash,       // /
    Colon,          // :
    Terminator,     // ;
    Lt,             // <
    Eq,             // =
    Gt,             // >
    BackTick,       // `
    Pipe,           // |
    BrackL,         // [
    BrackR,         // ]
    Question,       // ?
    AtSymbol,       // @
    Arrow,          // =>
    DoubleStar,     // **
    VarAssign,      // :=
    Inequality,     // /=
    GTE,            // >=
    SigAssign,      // <=
    Box,            // <>
    SigAssoc,       // <=>
    CondConv,       // ??
    MatchEQ,        // ?=
    MatchNE,        // ?/=
    MatchLT,        // ?<
    MatchLTE,       // ?<=
    MatchGT,        // ?>
    MatchGTE,       // ?>=
    DoubleLT,       // <<
    DoubleGT,       // >>
    // --- keywords
    Abs,
    Access,
    After,
    Alias,
    All,
    And,
    Architecture,
    Array,
    Assert,
    Assume,
    // AssumeGuarantee is omitted from VHDL-2019 LRM
    Attribute,
    Begin,
    Block,
    Body,
    Buffer,
    Bus,
    Case, 
    Component,
    Configuration,
    Constant, 
    Context,
    Cover,
    Default,
    Disconnect, 
    Downto,
    Else, 
    Elsif,
    End,
    Entity, 
    Exit,
    Fairness,
    File,
    For, 
    Force,
    Function,
    Generate, 
    Generic, 
    Group, 
    Guarded,
    If,
    Impure, 
    In, 
    Inertial, 
    Inout, 
    Is,
    Label, 
    Library, 
    Linkage, 
    Literal, 
    Loop,
    Map,
    Mod,
    Nand,
    New, 
    Next, 
    Nor, 
    Not, 
    Null,
    Of,
    On,
    Open,
    Or, 
    Others, 
    Out,
    Package, 
    Parameter, 
    Port, 
    Postponed, 
    Private,
    Procedure, 
    Process, 
    Property, 
    Protected, 
    Pure,
    Range,
    Record,
    Register,
    Reject,
    Release,
    Rem,
    Report,
    Restrict, 
    // RestrictGuarantee is omitted from VHDL-2019 LRM
    Return,
    Rol, 
    Ror,
    Select, 
    Sequence, 
    Severity,
    Signal, 
    Shared, 
    Sla,
    Sll,
    Sra,
    Srl, 
    Strong, 
    Subtype,
    Then,
    To, 
    Transport, 
    Type,
    Unaffected, 
    Units,
    Until,
    Use,
    Variable, 
    View,
    Vmode, 
    Vpkg,
    Vprop, 
    Vunit,
    Wait, 
    When, 
    While, 
    With,
    Xnor, 
    Xor,
}

/// Walks through the possible interpretations for capturing a VHDL delimiter.
/// 
/// If it successfully finds a valid VHDL delimiter, it will move the `loc` the number
/// of characters it consumed.
fn collect_delimiter<T>(stream: &mut Peekable<T>, loc: &mut Position, c0: Option<char>) -> Option<VHDLToken> 
    where T: Iterator<Item=char> {

    let mut delim = String::with_capacity(3);
    if let Some(c) = c0 {
        delim.push(c);
    }

    while let Some(c) = stream.peek() {
        match delim.len() {
            0 => match c {
                // ambiguous characters...read another character (could be a len-2 delimiter)
                '?' | '<' | '>' | '/' | '=' | '*' | ':' => {
                    loc.next_col();
                    delim.push(stream.next().unwrap())
                },
                _ => { 
                    let op = VHDLToken::match_delimiter(&String::from(c.clone()));
                    // if it was a delimiter, take the character and increment the location
                    if let Some(r) = op {
                        loc.next_col();
                        stream.next();
                        return Some(r)
                    } else {
                        return None
                    }
                }
            }
            1 => match delim.chars().nth(0).unwrap() {
                '?' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '/' | '<' | '>' => {
                            loc.next_col();
                            delim.push(stream.next().unwrap())
                        }
                        _ => { return Some(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                '<' => {
                    match c {
                        // move on to next round (could be a len-3 delimiter)
                        '=' => {
                            loc.next_col();
                            delim.push(stream.next().unwrap())
                        },
                        _ => { return Some(VHDLToken::match_delimiter(&delim).expect("invalid token")) }
                    }
                }
                _ => {
                    // try with 2
                    delim.push(c.clone());
                    if let Some(op) = VHDLToken::match_delimiter(&delim) {
                        loc.next_col();
                        stream.next();
                        return Some(op)
                    } else {
                        // revert back to 1
                        delim.pop();
                        return VHDLToken::match_delimiter(&delim)
                    }
                }
            }
            2 => {
                // try with 3
                delim.push(c.clone());
                if let Some(op) = VHDLToken::match_delimiter(&delim) {
                    stream.next();
                    loc.next_col();
                    return Some(op)
                } else {
                    // revert back to 2 (guaranteed to exist)
                    delim.pop();
                    return Some(VHDLToken::match_delimiter(&delim).expect("invalid token"))
                }
            }
            _ => panic!("delimiter matching exceeds 3 characters")
        }
    };
    // try when hiting end of stream
    VHDLToken::match_delimiter(&delim)
}

impl VHDLToken {
    /// Attempts to match the given string of characters `s` to a VHDL delimiter.
    fn match_delimiter(s: &str) -> Option<Self> {
        Some(match s {
            "&"     => Self::Ampersand,    
            "'"     => Self::SingleQuote,  
            "("     => Self::ParenL,       
            ")"     => Self::ParenR,       
            "*"     => Self::Star,         
            "+"     => Self::Plus,         
            ","     => Self::Comma,        
            "-"     => Self::Dash,         
            "."     => Self::Dot,          
            "/"     => Self::FwdSlash,     
            ":"     => Self::Colon,        
            ";"     => Self::Terminator,   
            "<"     => Self::Lt,           
            "="     => Self::Eq,           
            ">"     => Self::Gt,           
            "`"     => Self::BackTick,     
            "|"     => Self::Pipe,         
            "["     => Self::BrackL,       
            "]"     => Self::BrackR,       
            "?"     => Self::Question,     
            "@"     => Self::AtSymbol,     
            "=>"    => Self::Arrow,          
            "**"    => Self::DoubleStar,     
            ":="    => Self::VarAssign,      
            "/="    => Self::Inequality,     
            ">="    => Self::GTE,            
            "<="    => Self::SigAssign,      
            "<>"    => Self::Box,            
            "<=>"   => Self::SigAssoc,       
            "??"    => Self::CondConv,       
            "?="    => Self::MatchEQ,        
            "?/="   => Self::MatchNE,      
            "?<"    => Self::MatchLT,        
            "?<="   => Self::MatchLTE,       
            "?>"    => Self::MatchGT,        
            "?>="   => Self::MatchGTE,       
            "<<"    => Self::DoubleLT,       
            ">>"    => Self::DoubleGT,       
            _ => return None,
        })
    }

    /// Attempts to match the given string of characters `s` to a VHDL keyword.
    /// 
    /// Compares `s` against keywords using ascii lowercase comparison.
    fn match_keyword(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_ref() {
            "abs"           => Self::Abs, 
            "access"        => Self::Access, 
            "after"         => Self::After, 
            "alias"         => Self::Alias, 
            "all"           => Self::All, 
            "and"           => Self::And, 
            "architecture"  => Self::Architecture, 
            "array"         => Self::Array, 
            "assert"        => Self::Assert, 
            "assume"        => Self::Assume, 
            "attribute"     => Self::Attribute, 
            "begin"         => Self::Begin, 
            "block"         => Self::Block, 
            "body"          => Self::Body, 
            "buffer"        => Self::Buffer, 
            "bus"           => Self::Bus, 
            "case"          => Self::Case, 
            "component"     => Self::Component, 
            "configuration" => Self::Configuration, 
            "constant"      => Self::Constant, 
            "context"       => Self::Context, 
            "cover"         => Self::Cover, 
            "default"       => Self::Default, 
            "disconnect"    => Self::Disconnect, 
            "downto"        => Self::Downto, 
            "else"          => Self::Else, 
            "elsif"         => Self::Elsif, 
            "end"           => Self::End, 
            "entity"        => Self::Entity, 
            "exit"          => Self::Exit, 
            "fairness"      => Self::Fairness, 
            "file"          => Self::File, 
            "for"           => Self::For, 
            "force"         => Self::Force, 
            "function"      => Self::Function, 
            "generate"      => Self::Generate, 
            "generic"       => Self::Generic, 
            "group"         => Self::Group, 
            "guarded"       => Self::Guarded, 
            "if"            => Self::If, 
            "impure"        => Self::Impure, 
            "in"            => Self::In, 
            "inertial"      => Self::Inertial, 
            "inout"         => Self::Inout, 
            "is"            => Self::Is, 
            "label"         => Self::Label, 
            "library"       => Self::Library, 
            "linkage"       => Self::Linkage, 
            "literal"       => Self::Literal, 
            "loop"          => Self::Loop, 
            "map"           => Self::Map, 
            "mod"           => Self::Mod, 
            "nand"          => Self::Nand, 
            "new"           => Self::New, 
            "next"          => Self::Next, 
            "nor"           => Self::Nor, 
            "not"           => Self::Not, 
            "null"          => Self::Null, 
            "of"            => Self::Of, 
            "on"            => Self::On, 
            "open"          => Self::Open, 
            "or"            => Self::Or, 
            "others"        => Self::Others, 
            "out"           => Self::Out, 
            "package"       => Self::Package, 
            "parameter"     => Self::Parameter, 
            "port"          => Self::Port, 
            "postponed"     => Self::Postponed, 
            "private"       => Self::Private, 
            "procedure"     => Self::Procedure, 
            "process"       => Self::Process, 
            "property"      => Self::Property, 
            "protected"     => Self::Protected, 
            "pure"          => Self::Pure, 
            "range"         => Self::Range, 
            "record"        => Self::Record, 
            "register"      => Self::Register, 
            "reject"        => Self::Reject, 
            "release"       => Self::Release, 
            "rem"           => Self::Rem, 
            "report"        => Self::Report, 
            "restrict"      => Self::Restrict, 
            "return"        => Self::Return, 
            "rol"           => Self::Rol, 
            "ror"           => Self::Ror, 
            "select"        => Self::Select, 
            "sequence"      => Self::Sequence, 
            "severity"      => Self::Severity, 
            "signal"        => Self::Signal, 
            "shared"        => Self::Shared, 
            "sla"           => Self::Sla, 
            "sll"           => Self::Sll, 
            "sra"           => Self::Sra, 
            "srl"           => Self::Srl, 
            "strong"        => Self::Strong, 
            "subtype"       => Self::Subtype, 
            "then"          => Self::Then, 
            "to"            => Self::To, 
            "transport"     => Self::Transport, 
            "type"          => Self::Type, 
            "unaffected"    => Self::Unaffected, 
            "units"         => Self::Units, 
            "until"         => Self::Until, 
            "use"           => Self::Use, 
            "variable"      => Self::Variable, 
            "view"          => Self::View, 
            "vmode"         => Self::Vmode, 
            "vpkg"          => Self::Vpkg, 
            "vprop"         => Self::Vprop, 
            "vunit"         => Self::Vunit, 
            "wait"          => Self::Wait, 
            "when"          => Self::When, 
            "while"         => Self::While, 
            "with"          => Self::With, 
            "xnor"          => Self::Xnor, 
            "xor"           => Self::Xor, 
            _ => return None
        })
    }
}

impl std::fmt::Display for VHDLToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Whitespace    => " ",
            Self::Comment(note) => note.as_str(),
            Self::Identifier(id) => id.as_str(),
            Self::EOF           => "EOF",
            Self::AbstLiteral   => "abstract literal",
            Self::CharLiteral(c) => c.as_str(),
            Self::StrLiteral(s) => s.as_ref(),
            Self::BitStrLiteral(b) => b.as_str(),
            Self::DoubleQuote   => "\"",
            // --- delimiters
            Self::Ampersand     => "&",
            Self::SingleQuote   => "'",
            Self::ParenL        => "(",
            Self::ParenR        => ")",
            Self::Star          => "*",
            Self::Plus          => "+",
            Self::Comma         => ",",
            Self::Dash          => "-",
            Self::Dot           => ".",
            Self::FwdSlash      => "/",
            Self::Colon         => ":",
            Self::Terminator    => ";",
            Self::Lt            => "<",
            Self::Eq            => "=",
            Self::Gt            => ">",
            Self::BackTick      => "`",
            Self::Pipe          => "|",
            Self::BrackL        => "[",
            Self::BrackR        => "]",
            Self::Question      => "?",
            Self::AtSymbol      => "@",
            Self::Arrow         => "=>",
            Self::DoubleStar    => "**",
            Self::VarAssign     => ":=",
            Self::Inequality    => "/=",
            Self::GTE           => ">=",
            Self::SigAssign     => "<=",
            Self::Box           => "<>",
            Self::SigAssoc      => "<=>",
            Self::CondConv      => "??",
            Self::MatchEQ       => "?=",
            Self::MatchNE       => "?/=",
            Self::MatchLT       => "?<",
            Self::MatchLTE      => "?<=",
            Self::MatchGT       => "?>",
            Self::MatchGTE      => "?>=",
            Self::DoubleLT      => "<<",
            Self::DoubleGT      => ">>",
            // --- keywords
            Self::Abs           => "abs",
            Self::Access        => "access",
            Self::After         => "after",
            Self::Alias         => "alias",
            Self::All           => "all",
            Self::And           => "and", 
            Self::Architecture  => "architecture",
            Self::Array         => "array",
            Self::Assert        => "assert",
            Self::Assume        => "assume",
            Self::Attribute     => "attribute",
            Self::Begin         => "begin",
            Self::Block         => "block",
            Self::Body          => "body",
            Self::Buffer        => "buffer",
            Self::Bus           => "bus",
            Self::Case          => "case", 
            Self::Component     => "component",
            Self::Configuration => "configuration",
            Self::Constant      => "constant", 
            Self::Context       => "context",
            Self::Cover         => "cover",
            Self::Default       => "default",
            Self::Disconnect    => "disconnect", 
            Self::Downto        => "downto",
            Self::Else          => "else", 
            Self::Elsif         => "elsif",
            Self::End           => "end",
            Self::Entity        => "entity", 
            Self::Exit          => "exit",
            Self::Fairness      => "fairness",
            Self::File          => "file",
            Self::For           => "for", 
            Self::Force         => "force",
            Self::Function      => "function",
            Self::Generate      => "generate", 
            Self::Generic       => "generic", 
            Self::Group         => "group", 
            Self::Guarded       => "guarded",
            Self::If            => "if",
            Self::Impure        => "impure", 
            Self::In            => "in",     
            Self::Inertial      => "inertial", 
            Self::Inout         => "inout", 
            Self::Is            => "is",
            Self::Label         => "label", 
            Self::Library       => "library", 
            Self::Linkage       => "linkage", 
            Self::Literal       => "literal", 
            Self::Loop          => "loop",
            Self::Map           => "map",
            Self::Mod           => "mod",
            Self::Nand          => "nand",
            Self::New           => "new", 
            Self::Next          => "next", 
            Self::Nor           => "nor", 
            Self::Not           => "not", 
            Self::Null          => "null",
            Self::Of            => "of",
            Self::On            => "on",
            Self::Open          => "open",
            Self::Or            => "or", 
            Self::Others        => "others", 
            Self::Out           => "out",
            Self::Package       => "package", 
            Self::Parameter     => "parameter", 
            Self::Port          => "port", 
            Self::Postponed     => "postponed", 
            Self::Private       => "private",
            Self::Procedure     => "procedure", 
            Self::Process       => "process", 
            Self::Property      => "property", 
            Self::Protected     => "protected", 
            Self::Pure          => "pure",
            Self::Range         => "range",
            Self::Record        => "record",    
            Self::Register      => "register",
            Self::Reject        => "reject",
            Self::Release       => "release",
            Self::Rem           => "rem",
            Self::Report        => "report",
            Self::Restrict      => "restrict", 
            Self::Return        => "return",
            Self::Rol           => "rol", 
            Self::Ror           => "ror",
            Self::Select        => "select", 
            Self::Sequence      => "sequence", 
            Self::Severity      => "severity",
            Self::Signal        => "signal", 
            Self::Shared        => "shared", 
            Self::Sla           => "sla",
            Self::Sll           => "sll",
            Self::Sra           => "sra",
            Self::Srl           => "srl", 
            Self::Strong        => "strong", 
            Self::Subtype       => "subtype",
            Self::Then          => "then",
            Self::To            => "to", 
            Self::Transport     => "transport", 
            Self::Type          => "type",
            Self::Unaffected    => "unaffected", 
            Self::Units         => "units",
            Self::Until         => "until",
            Self::Use           => "use",
            Self::Variable      => "variable", 
            Self::View          => "view",
            Self::Vmode         => "vmode", 
            Self::Vpkg          => "vpkg",
            Self::Vprop         => "vprop", 
            Self::Vunit         => "vunit",
            Self::Wait          => "wait", 
            Self::When          => "when", 
            Self::While         => "while", 
            Self::With          => "with",
            Self::Xnor          => "xnor", 
            Self::Xor           => "xor",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq)]
struct VHDLTokenizer {
    inner: Vec<Token<VHDLToken>>,
}

impl std::fmt::Debug for VHDLTokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tk in &self.inner {
            write!(f, "{} {}\n", tk.locate(), tk.unwrap())?
        }
        Ok(())
    } 
}

/// Compares to string references `s0` and `s1` with case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding case sensivity.
fn cmp_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_lowercase().cmp(s1.next().unwrap().to_lowercase()) != std::cmp::Ordering::Equal {
            return false
        }
    }
    true
}

/// Compares to string references `s0` and `s1` with only ascii case conversion.
/// 
/// Returns `true` if they are deemed equivalent without regarding ascii case sensivity.
fn cmp_ascii_ignore_case(s0: &str, s1: &str) -> bool {
    if s0.len() != s1.len() { return false }
    let mut s0 = s0.chars();
    let mut s1 = s1.chars();
    while let Some(c) = s0.next() {
        if c.to_ascii_lowercase() != s1.next().unwrap().to_ascii_lowercase() {
            return false
        }
    }
    true
}

use std::iter::Peekable;

/// Walks through the stream to gather a `String` literal until finding the 
/// exiting character `br`.
/// 
/// An escape is allowed by double placing the `br`, i.e. """hello"" world".
/// Assumes the first token to parse in the stream is not the `br` character.
/// The `loc` stays up to date on its position in the file.
fn enclose<T>(br: &char, stream: &mut Peekable<T>, loc: &mut Position) -> String 
    where T: Iterator<Item=char> {
        let mut result = String::new();
        while let Some(c) = stream.next() {
            loc.next_col();
            // verify it is a graphic character
            if char_set::is_graphic(&c) == false { panic!("invalid character {}", c) }
            // detect escape sequence
            if br == &c {
                match stream.peek() {
                    Some(c_next) => if br == c_next {
                        loc.next_col();
                        stream.next(); // skip over escape character
                    } else {
                        break;
                    }
                    None => break,
                }
            } 
            result.push(c);
        }
        result
}

mod char_set {
    pub const DOUBLE_QUOTE: char = '\"';
    pub const BACKSLASH: char = '\\';
    pub const STAR: char = '*';
    pub const DASH: char = '-';
    pub const FWDSLASH: char = '/';
    pub const UNDERLINE: char = '_';
    pub const SINGLE_QUOTE: char = '\'';

    /// Checks if `c` is a space according to VHDL-2008 LRM p225.
    /// Set: space, nbsp
    pub fn is_space(c: &char) -> bool {
        c == &'\u{0020}' || c == &'\u{00A0}'
    }

    /// Checks if `c` is a digit according to VHDL-2008 LRM p225.
    pub fn is_digit(c: &char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a graphic character according to VHDL-2008 LRM p230.
    /// - rule ::= upper_case_letter | digit | special_character | space_character 
    /// | lower_case_letter | other_special_character
    pub fn is_graphic(c: &char) -> bool {
        is_lower(&c) || is_upper(&c) || is_digit(&c) || 
        is_special(&c) || is_other_special(&c) || is_space(&c)
    }

    /// Checks if `c` is an upper-case letter according to VHDL-2019 LRM p257.
    /// Set: `ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ`
    pub fn is_upper(c: &char) -> bool {
        match c {
            '\u{00D7}' => false, // reject multiplication sign
            'A'..='Z' | 'À'..='Þ' => true,
            _ => false   
        }
    }

    /// Checks if `c` is a special character according to VHDL-2008 LRM p225.
    /// Set: `"#&'()*+,-./:;<=>?@[]_`|`
    pub fn is_special(c: &char) -> bool {
        match c {
            '"' | '#' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | 
            ':' | ';' | '<'  | '=' | '>' | '?' | '@' | '[' | ']' | '_' | '`' | '|' => true,
            _ => false,
        }
    }

    /// Checks if `c` is an "other special character" according to VHDL-2008 LRM p225.
    /// Set: `!$%\^{} ~¡¢£¤¥¦§ ̈©a«¬® ̄°±23 ́μ¶· ̧1o»1⁄41⁄23⁄4¿×÷-`
    pub fn is_other_special(c: &char) -> bool {
        match c {
            '!' | '$' | '%' | '\\' | '^' | '{' | '}' | ' ' | '~' | '-' | 
            '\u{00A1}'..='\u{00BF}' | '\u{00D7}' | '\u{00F7}' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a lower-case letter according to VHDL-2019 LRM p257.
    /// Set: `abcdefghijklmnopqrstuvwxyzßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ`
    pub fn is_lower(c: &char) -> bool {
        match c {
            '\u{00F7}' => false, // reject division sign
            'a'..='z' | 'ß'..='ÿ' => true,
            _ => false,
        }
    }

    /// Checks if `c` is a letter according to VHDL-2019 LRM p257.
    pub fn is_letter(c: &char) -> bool {
        is_lower(&c) || is_upper(&c)
    }

    /// Checks if the character is a seperator according to VHDL-2019 LRM p259.
    pub fn is_separator(c: &char) -> bool {
        // whitespace: space, nbsp
        c == &'\u{0020}' || c == &'\u{00A0}' ||
        // format-effectors: ht (\t), vt, cr (\r), lf (\n)
        c == &'\u{0009}' || c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}'
    }
}

use std::str::FromStr;

/// Collects a basic identifer ::= letter { [ underline ] letter_or_digit }
fn collect_identifier<T>(stream: &mut Peekable<T>, loc: &mut Position, c0: char) -> Result<VHDLToken, ()>
    where T: Iterator<Item=char> {

    let mut id = String::from(c0);
    let mut bit_lit: Option<BitStrLiteral> = None;

    while let Some(c) = stream.peek() {
        if (bit_lit.is_none() && (char_set::is_letter(&c) || c == &char_set::UNDERLINE)) ||
            (bit_lit.is_some() && c != &char_set::DOUBLE_QUOTE && (char_set::is_graphic(&c) || c == &char_set::UNDERLINE)) {
            loc.next_col();
            let c = stream.next().unwrap();
            id.push(c);
        // handle bit string literals
        } else if c == &char_set::DOUBLE_QUOTE {
            if bit_lit.is_none() {
                let base = BaseSpec::from_str(&id)?;
                // clear id to begin reading string literal
                id.clear();
                // throw away initial " char
                stream.next().unwrap(); 
                loc.next_col();
                bit_lit = Some(BitStrLiteral::new(base));
            } else if bit_lit.is_some() {
                // throw away closing " char
                stream.next().unwrap(); 
                loc.next_col();
                break; // exit loop
            }
        } else {
            break;
        }
    }
    match bit_lit {
        Some(b) => Ok(VHDLToken::BitStrLiteral(b.literal(id))),
        None => {
            // try to transform to key word
            Ok(match VHDLToken::match_keyword(&id) {
                Some(keyword) => keyword,
                None => VHDLToken::Identifier(Identifier::Basic(id))
            })
        }
    }
}

/// Collects a single-line comment (all characters after a `--` up until end-of-line).
fn collect_comment<T>(stream: &mut Peekable<T>, loc: &mut Position) -> VHDLToken
    where T: Iterator<Item=char> { 
    // skip over second '-'
    stream.next(); 
    loc.next_col();
    let mut note = String::new();
    while let Some(c) = stream.peek() {
        // cannot be vt, cr (\r), lf (\n)
        if c == &'\u{000B}' || c == &'\u{000D}' || c == &'\u{000A}' {
            break
        } else {
            loc.next_col();
            note.push(stream.next().unwrap());
        }
    }
    VHDLToken::Comment(Comment::Single(note))
}

/// Collects a delimited comment (all characters after a `/*` up until `*/`).
fn collect_delim_comment<T>(stream: &mut Peekable<T>, loc: &mut Position) -> VHDLToken
    where T: Iterator<Item=char> { 
        
    // skip over opening '*'
    stream.next();
    loc.next_col();
    let mut note = String::new();
    while let Some(c) = stream.next() {
        loc.next_col();
        if c == '\n' {
            loc.next_line();
        }
        // check if we are breaking from the comment
        if c == char_set::STAR {
            if let Some(c_next) = stream.peek() {
                // break from the comment
                if c_next == &char_set::FWDSLASH {
                    loc.next_col();
                    stream.next();
                    break;
                }
            }
        }
        note.push(c);
    }
    VHDLToken::Comment(Comment::Delimited(note))
}

/// Captures an extended identifier token.
/// 
/// Errors if the identifier is empty.
fn collect_extended_identifier<T>(stream: &mut Peekable<T>, loc: &mut Position) -> Result<VHDLToken, ()>
where T: Iterator<Item=char> { 
    let id = enclose(&char_set::BACKSLASH, stream, loc);
    if id.is_empty() { panic!("extended identifier cannot be empty") }
    Ok(VHDLToken::Identifier(Identifier::Extended(id)))
}

/// Captures a character literal according to VHDL-2018 LRM p231.
fn collect_chr_lit<T>(stream: &mut Peekable<T>, loc: &mut Position) -> Result<VHDLToken, ()> 
where T: Iterator<Item=char> {
    let mut char_lit = String::with_capacity(1);
    if let Some(c) = stream.next() {
        // verify the character is a graphic character
        if char_set::is_graphic(&c) == false { panic!("invalid char {}", c) }
        loc.next_col();
        // add to the struct
        char_lit.push(c);
        // expect a closing single-quote 
        // @TODO handle errors
        if stream.next().expect("missing closing char") != char_set::SINGLE_QUOTE {
            panic!("expecting closing '\'' character")
        };
        loc.next_col();
    }
    Ok(VHDLToken::CharLiteral(Character(char_lit)))
}

impl Tokenize for VHDLTokenizer {
    type TokenType = VHDLToken;

    fn tokenize(s: &str) -> Vec<Token<Self::TokenType>> {
        let mut loc = Position::new();
        let mut chars = s.chars().peekable();
        // store results here as we consume the characters
        let mut tokens = Vec::new();
        // consume every character (lexical analysis)
        while let Some(c) = chars.next() {
            loc.next_col();

            let tk_loc = Position(loc.0, loc.1);
            if char_set::is_letter(&c) {
                // collect general identifier (or bit string literal) 
                let tk = collect_identifier(&mut chars, &mut loc, c).expect("failed to read identifier");
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::BACKSLASH {
                // collect extended identifier
                let tk = collect_extended_identifier(&mut chars, &mut loc).unwrap();
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::DOUBLE_QUOTE {
                // collect string literal
                let tk = VHDLToken::StrLiteral(enclose(&c, &mut chars, &mut loc));
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::SINGLE_QUOTE {
                // collect character literal
                let tk = collect_chr_lit(&mut chars, &mut loc).expect("invalid char literal");
                tokens.push(Token::new(tk, tk_loc));

            } else if char_set::is_digit(&c) {
                // collect decimal literal (or bit string literal or based literal)
                // @TODO

            } else if c == char_set::DASH && chars.peek().is_some() && chars.peek().unwrap() == &char_set::DASH {    
                // collect a single-line comment           
                let tk = collect_comment(&mut chars, &mut loc);
                tokens.push(Token::new(tk, tk_loc));

            } else if c == char_set::FWDSLASH && chars.peek().is_some() && chars.peek().unwrap() == &char_set::STAR {
                // collect delimited (multi-line) comment
                let tk = collect_delim_comment(&mut chars, &mut loc);
                tokens.push(Token::new(tk, tk_loc));

            } else {
                // collect delimiter
                if let Some(tk) = collect_delimiter(&mut chars, &mut loc, Some(c)) {
                    tokens.push(Token::new(tk, tk_loc));
                }
            }
            // o.w. collect whitespace
            if c == '\n' {
                loc.next_line();
            }
        }
        // push final EOF token
        loc.next_col();
        tokens.push(Token::new(VHDLToken::EOF, loc));
        tokens
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ignore_case_cmp() {
        let s0 = "ABC";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), true);

        // negative case: different lengths
        let s0 = "ABCD";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // negative case: different letter order
        let s0 = "cba";
        let s1 = "abc";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // VHDL-2008 LRM p226
        let s0 = "ABCDEFGHIJKLMNOPQRSTUVWXYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞ";
        let s1 = "abcdefghijklmnopqrstuvwxyzàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþ";
        assert_eq!(cmp_ignore_case(s0, s1), true);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);

        // these 2 letters do not have upper-case equivalents
        let s0 = "ß";
        let s1 = "ÿ";
        assert_eq!(cmp_ignore_case(s0, s1), false);
        assert_eq!(cmp_ascii_ignore_case(s0, s1), false);
    }

    mod vhdl {
        use super::*;

        #[test]
        fn easy_tokens() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<VHDLToken> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.take() })
                .collect();
            assert_eq!(tokens, vec![
                Entity,
                Identifier(vhdl::Identifier::Basic("fa".to_owned())),
                Is,
                End,
                Entity,
                Terminator,
                EOF,
            ]);
        }

        #[test]
        fn comment_token() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
-- here is a vhdl single-line comment!";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Single(" here is a vhdl single-line comment!".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(1, 39)),
            ]);
        }

        #[test]
        fn comment_token_delim() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
/* here is a vhdl 
    delimited-line comment. Look at all the space! */";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Comment(vhdl::Comment::Delimited(" here is a vhdl 
    delimited-line comment. Look at all the space! ".to_owned())), Position(1, 1)),
                Token::new(EOF, Position(2, 54)),
            ]);
        }

        #[test]
        fn char_literal() {
            use super::VHDLToken::*;
            use crate::core::vhdl::*;
            let s = "\
signal magic_num : std_logic := '1';";
            let tokens: Vec<Token<VHDLToken>> = VHDLTokenizer::tokenize(s);
            assert_eq!(tokens, vec![
                Token::new(Signal, Position(1, 1)),
                Token::new(Identifier(vhdl::Identifier::Basic("magic_num".to_owned())), Position(1, 8)),
                Token::new(Colon, Position(1, 18)),
                Token::new(Identifier(vhdl::Identifier::Basic("std_logic".to_owned())), Position(1, 20)),
                Token::new(VarAssign, Position(1, 30)),
                Token::new(CharLiteral(vhdl::Character("1".to_owned())), Position(1, 33)),
                Token::new(Terminator, Position(1, 36)),
                Token::new(EOF, Position(1, 37)),
            ]);
        }

        #[test]
        fn easy_locations() {
            use crate::core::vhdl::*;
            let s = "\
entity fa is end entity;";
            let tokens: Vec<Position> = VHDLTokenizer::tokenize(s)
                .into_iter()
                .map(|f| { f.locate().clone() })
                .collect();
            assert_eq!(tokens, vec![
                Position(1, 1),  // 1:1 keyword: entity
                Position(1, 8),  // 1:8 basic identifier: fa
                Position(1, 11), // 1:11 keyword: is
                Position(1, 14), // 1:14 keyword: end
                Position(1, 18), // 1:18 keyword: entity
                Position(1, 24), // 1:24 delimiter: ;
                Position(1, 25), // 1:25 eof
            ]);  
        }

        #[test]
        fn read_delimiter_single() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "&";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Ampersand));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 1));

            let mut loc = Position::new();
            let contents = "?";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Question));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 1));

            let mut loc = Position::new();
            let contents = "< MAX_COUNT";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(Lt));
            assert_eq!(stream.collect::<String>(), " MAX_COUNT");
            assert_eq!(loc, Position(1, 1));
        }

        #[test]
        fn read_delimiter_none() {
            let mut loc = Position::new();
            let contents = "fa";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), None);
            assert_eq!(stream.collect::<String>(), "fa");
            assert_eq!(loc, Position(1, 0));
        }

        #[test]
        fn read_delimiter_double() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "<=";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(SigAssign));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 2));

            let mut loc = Position::new();
            let contents = "**WIDTH";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(DoubleStar));
            assert_eq!(stream.collect::<String>(), "WIDTH");
            assert_eq!(loc, Position(1, 2));
        }

        #[test]
        fn read_delimiter_triple() {
            use super::VHDLToken::*;

            let mut loc = Position::new();
            let contents = "<=>";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(SigAssoc));
            assert_eq!(stream.collect::<String>(), "");
            assert_eq!(loc, Position(1, 3));

            let mut loc = Position::new();
            let contents = "?/= MAGIC_NUM";
            let mut stream = contents.chars().peekable();
            assert_eq!(collect_delimiter(&mut stream, &mut loc, None), Some(MatchNE));
            assert_eq!(stream.collect::<String>(), " MAGIC_NUM");
            assert_eq!(loc, Position(1, 3));
        }

        #[test]
        fn match_delimiter() {
            use super::VHDLToken::*;

            let word = "<=";
            assert_eq!(VHDLToken::match_delimiter(word), Some(SigAssign));

            let word = "-";
            assert_eq!(VHDLToken::match_delimiter(word), Some(Dash));

            let word = "<=>";
            assert_eq!(VHDLToken::match_delimiter(word), Some(SigAssoc));

            let word = "^";
            assert_eq!(VHDLToken::match_delimiter(word), None);

            let word = "entity";
            assert_eq!(VHDLToken::match_delimiter(word), None);
        }

        #[test]
        fn match_reserved_idenifier() {
            use super::VHDLToken::*;

            let word = "END";
            assert_eq!(VHDLToken::match_keyword(word), Some(End));

            let word = "EnTITY";
            assert_eq!(VHDLToken::match_keyword(word), Some(Entity));

            let word = "entitys";
            assert_eq!(VHDLToken::match_keyword(word), None);

            let word = "<=";
            assert_eq!(VHDLToken::match_keyword(word), None);
        }

        #[test]
        fn is_sep() {
            let c = ' '; // space
            assert_eq!(char_set::is_separator(&c), true);

            let c = ' '; // nbsp
            assert_eq!(char_set::is_separator(&c), true);

            let c = '\t'; // horizontal tab
            assert_eq!(char_set::is_separator(&c), true);

            let c = '\n'; // new-line
            assert_eq!(char_set::is_separator(&c), true);

            let c = 'c';  // negative case: ascii char
            assert_eq!(char_set::is_separator(&c), false);
        }

        #[test]
        fn read_identifier() {
            let mut loc = Position(1, 1);
            let words = "ntity is";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'e').unwrap(), VHDLToken::Entity);
            assert_eq!(stream.collect::<String>(), " is");
            assert_eq!(loc, Position(1, 6));

            let mut loc = Position(1, 1);
            let words = "eady_OUT<=";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'r').unwrap(), VHDLToken::Identifier(vhdl::Identifier::Basic("ready_OUT".to_owned())));
            assert_eq!(stream.collect::<String>(), "<=");
            assert_eq!(loc, Position(1, 9));
        }

        #[test]
        fn read_bit_str_literal() {
            let mut loc = Position(1, 1);
            let words = "\"1010\"more text";
            let mut stream = words.chars().peekable();
            assert_eq!(collect_identifier(&mut stream, &mut loc, 'b'), Ok(VHDLToken::BitStrLiteral(vhdl::BitStrLiteral { width: None, base: BaseSpec::B, literal: "1010".to_owned() })));
            assert_eq!(stream.collect::<String>(), "more text");
            assert_eq!(loc, Position(1, 7));
        }

        #[test]
        fn wrap_enclose() {
            let mut loc = Position(1, 1);
            let contents = "\"Setup time is too short\"more text";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "Setup time is too short");
            assert_eq!(stream.collect::<String>(), "more text");
            assert_eq!(loc, Position(1, 25));

            let mut loc = Position(1, 1);
            let contents = "\"\"\"\"\"\"";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "\"\"");
            assert_eq!(loc, Position(1, 6));

            let mut loc = Position::new();
            let contents = "\" go \"\"gators\"\" from UF! \"";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), " go \"gators\" from UF! ");
            assert_eq!(loc, Position(1, 25));

            let mut loc = Position::new();
            let contents = "\\VHDL\\";
            let mut stream = contents.chars().peekable();
            assert_eq!(enclose(&stream.next().unwrap(), &mut stream, &mut loc), "VHDL");

            let mut loc = Position::new();
            let contents = "\\a\\\\b\\more text afterward";
            let mut stream = contents.chars().peekable();
            let br = stream.next().unwrap();
            assert_eq!(enclose(&br, &mut stream, &mut loc), "a\\b");
            // verify the stream is left in the correct state
            assert_eq!(stream.collect::<String>(), "more text afterward");
        }
    }

    mod position {
        use super::*;

        #[test]
        fn moving_position() {
            let mut pos = Position::new();
            assert_eq!(pos, Position(1, 0));
            pos.next_col();
            assert_eq!(pos, Position(1, 1));
            pos.next_col();
            assert_eq!(pos, Position(1, 2));
            pos.next_line();
            assert_eq!(pos, Position(2, 0));
            pos.next_line();
            assert_eq!(pos, Position(3, 0));
        }
    }
}