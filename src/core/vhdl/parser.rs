use std::fmt::Display;
use crate::core::lexer::Token;

pub trait Parse<T> {
    type SymbolType;
    type Err;

    fn parse(tokens: Vec<Token<T>>) -> Vec<Result<Symbol<Self::SymbolType>, SymbolError<Self::Err>>> 
        where <Self as Parse<T>>::Err: Display;
}

#[derive(Debug, PartialEq)]
pub struct Symbol<T> {
    stype: T,
}

impl<T> Symbol<T> {
    /// Creates a new token.
    pub fn new(stype: T) -> Self {
        Self {
            stype: stype,
        }
    }

    pub fn take(self) -> T {
        self.stype
    }
}


#[derive(Debug, PartialEq)]
pub struct SymbolError<T: Display> {
    err: T,
}

impl<T: Display> SymbolError<T> {
    /// Creates a new `TokenError` struct at position `loc` with error `T`.
    pub fn new(err: T) -> Self {
        Self { err: err }
    }
}

impl<T: Display> Display for SymbolError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

#[derive(Debug, PartialEq)]
pub struct Package {
    name: Identifier,
    body: Option<PackageBody>,
    refs: Vec<ResReference>,
}

impl Package {
    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> Vec<&ResReference> {
        self.refs.iter().map(|f| f).collect()
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct PackageBody {
    owner: Identifier,
    refs: Vec<ResReference>,
}

impl PackageBody {
    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> Vec<&ResReference> {
        self.refs.iter().map(|f| f).collect()
    }
}

impl Display for PackageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package body for {}", self.owner)
    }
}

#[derive(Debug, PartialEq)]
pub enum VHDLSymbol {
    ContextClause(ContextClause),
    // primary units
    Entity(Entity),
    Package(Package),
    Configuration(Configuration),
    // @TODO context clause
    // secondary units
    Architecture(Architecture),
    PackageBody(PackageBody),
}

impl VHDLSymbol {
    pub fn get_iden(&self) -> Option<&Identifier> {
        match self {
            Self::Entity(e) => Some(&e.name),
            Self::Architecture(a) => Some(&a.name),
            Self::Package(p) => Some(&p.name),
            Self::PackageBody(_) => None,
            Self::Configuration(c) => Some(&c.name),
            Self::ContextClause(_) => todo!("capture context id")
        }
    }

    /// Attempts to reference inner entity.
    pub fn as_entity(&self) -> Option<&Entity> {
        match self {
            Self::Entity(e) => Some(e),
            _ => None
        }
    }

    pub fn add_refs(&mut self, refs: &mut Vec<ResReference>) {
        match self {
            Self::Entity(e) => e.refs.append(refs),
            Self::Architecture(a) => a.refs.append(refs),
            Self::Package(p) => p.refs.append(refs),
            Self::PackageBody(pb) => pb.refs.append(refs),
            _ => (),
        }
        refs.clear();
    }

    pub fn get_refs(&self) -> Vec<&ResReference> {
        match self {
            Self::Entity(e) => e.get_refs(),
            Self::Architecture(a) => a.get_refs(),
            Self::Package(p) => p.get_refs(),
            Self::PackageBody(pb) => pb.get_refs(),
            _ => vec![]
        }
    }
}

impl std::fmt::Display for VHDLSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Entity(e) => format!("entity {} {{ generics={:?} ports={:?} }}", &e.name, e.generics, e.ports),
            Self::PackageBody(pb) => format!("package body- {}", pb),
            Self::Architecture(a) => format!("architecture {} for entity {}", &a.name, &a.owner),
            Self::Package(p) => format!("package {}", &p),
            Self::Configuration(c) => format!("configuration {} for entity {}", &c.name, &c.owner),
            Self::ContextClause(_) => format!("context clause"),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
struct SelectedName(Vec<Identifier>);

impl SelectedName {
    /// Returns the final identifier in the chain.
    fn get_suffix(&self) -> &Identifier {
        self.0.last().unwrap()
    }

    fn take_suffix(mut self) -> Identifier {
        self.0.pop().unwrap()
    }
}

/// A `ResReference` is a pattern in the code that catches <library>.<package>. We
/// assume the pattern can be found anywhere.
#[derive(Debug, PartialEq)]
pub struct ResReference {
    prefix: Identifier,
    suffix: Identifier,
}

impl Display for ResReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.prefix, self.suffix)
    }
}

use std::str::FromStr;

impl ResReference {
    pub fn get_suffix(&self) -> &Identifier {
        &self.suffix
    }

    pub fn get_prefix(&self) -> &Identifier {
        &self.prefix
    }

    fn new() -> Self {
        ResReference { 
            prefix: Identifier::new(), 
            suffix: Identifier::new() 
        }
    }

    /// Creates a reference from a single identifier and makes the prefix=`work`.
    fn from_owner(owner: Identifier) -> Self {
        ResReference { 
            prefix: Identifier::from_str("work").unwrap(), 
            suffix: owner 
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ContextClause {
    LibraryClause,
    UseClause(UseClause),
    // @TODO Context_reference
}

#[derive(Debug, PartialEq)]
pub struct UseClause {
    imports: Vec<SelectedName>,
}

#[derive(Debug, PartialEq)]
pub struct Architecture {
    name: Identifier,
    owner: Identifier,
    dependencies: Vec<Identifier>,
    refs: Vec<ResReference>,
}

impl Architecture {
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn entity(&self) -> &Identifier {
        &self.owner
    }

    pub fn edges(&self) -> &Vec<Identifier> {
        &self.dependencies
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> Vec<&ResReference> {
        self.refs.iter().map(|f| f).collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
    name: Identifier,
    owner: Identifier,
}

#[derive(Debug, PartialEq)]
pub struct Entity {
    name: Identifier,
    ports: Vec<Statement>,
    generics: Vec<Statement>,
    architectures: Vec<Architecture>,
    refs: Vec<ResReference>,
}

impl Entity {
    /// Returns a new blank `Entity` struct.
    pub fn new() -> Self {
        Self { 
            name: Identifier::new(),
            ports: Vec::new(), 
            generics: Vec::new(), 
            architectures: Vec::new(),
            refs: Vec::new(),
        }
    }

    /// Checks if the current `Entity` is a testbench.
    /// 
    /// This is determined by checking if the ports list is empty.
    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }

    /// Accesses the entity's identifier.
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> Vec<&ResReference> {
        self.refs.iter().map(|f| f).collect()
    }

    // Access the signal list
    pub fn get_signals(&self) -> Vec<String> {
        let mut sigs = Vec::with_capacity(self.ports.len());
        for st_port in &self.ports {
            sigs.push(String::from("signal ")+&st_port.to_string()+";");
        }
        sigs
    }

    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self 
    where I: Iterator<Item=Token<VHDLToken>> {
        // take entity name
        let entity_name = tokens.next().take().unwrap().take();
        let (generics, ports) = VHDLSymbol::parse_entity_declaration(tokens);
        Entity { 
            name: match entity_name {
                    VHDLToken::Identifier(id) => id,
                    _ => panic!("expected an identifier")
            },
            architectures: Vec::new(),
            generics: generics,
            ports: ports,
            refs: Vec::new(),
        }
    }
}

/* 
    @NOTE In order to detect if a package was used, the best bet is to just 
    iterate through the the tokens and collect all simple names, i.e. 
    library.name.name. , then try check against the data structure if the name 
    matches anywhere. If so, then it is considered a reference and is needed in 
    that design.
*/

/* 
    @NOTE To check instantiations, check for all identifiers against the
    list of known public identifiers from external API.

    Example: given a ip with public primary design units: adder, adder_pkg
*/

#[derive(Debug, PartialEq)]
pub struct VHDLParser {
    symbols: Vec<Symbol<VHDLSymbol>>,
}

use crate::core::vhdl::vhdl::*;

impl Parse<VHDLToken> for VHDLParser {
    type SymbolType = VHDLSymbol;
    type Err = String;
    
    fn parse(tokens: Vec<Token<VHDLToken>>) -> Vec<Result<Symbol<Self::SymbolType>, SymbolError<Self::Err>>>
        where <Self as Parse<VHDLToken>>::Err: Display {
            
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        let mut global_refs = Vec::new();

        while let Some(t) = tokens.next() {
            // create entity symbol
            if t.as_ref().check_keyword(&Keyword::Entity) {
                let mut ent = VHDLSymbol::parse_entity(&mut tokens);
                ent.add_refs(&mut global_refs);
                // println!("info: detected {}", ent);
                symbols.push(Ok(Symbol::new(ent)));
            // create architecture symbol
            } else if t.as_ref().check_keyword(&Keyword::Architecture) {
                let mut arch = VHDLSymbol::parse_architecture(&mut tokens);
                arch.add_refs(&mut global_refs);
                // println!("info: detected {}", arch);
                symbols.push(Ok(Symbol::new(arch)));
            // create configuration symbol
            } else if t.as_ref().check_keyword(&Keyword::Configuration) {
                let config = VHDLSymbol::parse_configuration(&mut tokens);
                // println!("info: detected {}", config);
                symbols.push(Ok(Symbol::new(config)));
            // create package symbol
            } else if t.as_ref().check_keyword(&Keyword::Package) {
                let mut pack = VHDLSymbol::route_package_parse(&mut tokens);
                pack.add_refs(&mut global_refs);
                // println!("info: detected {}", pack);
                symbols.push(Ok(Symbol::new(pack)));
            // otherwise take a statement (probably by mistake/errors in user's vhdl code or as of now an error in my code)
            } else {
                let mut stmt: Statement = Statement::new();
                stmt.0.push(t);
                let mut res = VHDLSymbol::compose_statement(&mut tokens);
                stmt.0.append(&mut res.0);
                // println!("global statement: {:?}", stmt.0);
                global_refs.append(&mut res.take_refs());
                // println!("global refs: {:?}", global_refs);
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VHDLParser {
    pub fn read(s: &str) -> Self {
        let symbols = VHDLParser::parse(VHDLTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols.into_iter().filter_map(|f| { if f.is_ok() { Some(f.unwrap()) } else { None } }).collect()
        }
    }

    pub fn into_symbols(self) -> Vec<VHDLSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }

}

use std::iter::Peekable;

/// A `Statement` is a vector of tokens, similiar to how a `String` is a vector
/// of characters.
#[derive(PartialEq)]
struct Statement(Vec<Token<VHDLToken>>, Vec<ResReference>);

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // determine which delimiters to not add trailing spaces to
        let is_spaced_token = |d: &Delimiter| {
            match d {
                Delimiter::ParenL | Delimiter::ParenR => false,
                _ => true,
            }
        };
        // iterate through the tokens
        let mut iter = self.0.iter().peekable();
        while let Some(t) = iter.next() {
            let trailing_space = match t.as_ref() {
                VHDLToken::Delimiter(d) => is_spaced_token(d),
                _ => {
                    // make sure the next token is not a tight token (no-spaced)
                    if let Some(m) = iter.peek() {
                        match m.as_ref() {
                            VHDLToken::Delimiter(d) => is_spaced_token(d),
                            _ => true
                        }
                    } else {
                        true
                    }
                }
            };
            write!(f, "{}", t.as_ref().to_string())?;
            if trailing_space == true && iter.peek().is_some() {
                write!(f, " ")?
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.0 {
            write!(f, "{} ", t.as_ref().to_string())?
        }
        Ok(())
    }
}

impl Statement {
    fn new() -> Self {
        Self(Vec::new(), Vec::new())
    }

    /// References the tokens as VHDLTokens
    fn as_types(&self) -> Vec<&VHDLToken> {
        self.0.iter().map(|f| f.as_type() ).collect()
    }

    /// Accesses the resource references.
    fn get_refs(&self) -> Vec<&ResReference> {
        self.1.iter().map(|f| f).collect()
    }

    /// Transforms into only resource references.
    fn take_refs(self) -> Vec<ResReference> {
        self.1
    }
}

impl VHDLSymbol {
    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    fn parse_entity<I>(tokens: &mut Peekable<I>) -> VHDLSymbol 
    where I: Iterator<Item=Token<VHDLToken>>  {
        VHDLSymbol::Entity(Entity::from_tokens(tokens))
    }

    /// Parses a package declaration, from the <package> IS to the END keyword.
    /// 
    /// Assumes the last consumed token was PACKAGE keyword and the next token
    /// is the identifier for the package name.
    fn parse_package_declaration<I>(tokens: &mut Peekable<I>) -> VHDLSymbol 
    where I: Iterator<Item=Token<VHDLToken>>  {
        // take package name
        let pack_name = tokens.next().take().unwrap().take();
        // take the IS keyword
        if tokens.next().take().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            panic!("expecting keyword IS")
        }
        // @TODO check if there is a generic clause

        // compose the declarative items
        while let Some(t) = tokens.peek() {
            // check for nested package declarations
            if t.as_type().check_keyword(&Keyword::Package) {
                // consume PACKAGE keyword
                tokens.next();
                // parse nested package declaration
                Self::parse_package_declaration(tokens);
                // @TODO store nested packages
            // grab component declarations
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp = Self::parse_component(tokens);
                // println!("component declared: {}", comp);
            // grab USE clause
            } else if t.as_type().check_keyword(&Keyword::Use) {
                // consume USE keyword
                tokens.next();
                Self::parse_use_clause(tokens);
                // @TODO store use clauses to check if an external api was called
            } else if t.as_type().check_keyword(&Keyword::End) {
                Self::compose_statement(tokens);
                break;
            } else {
                Self::compose_statement(tokens);
            }
        }

        // println!("*--- unit {}", pack_name);
        VHDLSymbol::Package(Package {
            name: match pack_name {
                VHDLToken::Identifier(id) => id,
                _ => panic!("expected an identifier")
            },
            refs: Vec::new(),
            body: None,
        })
    }

    /// Collects identifiers into a single vector, stopping at a non-identifier token.
    /// 
    /// Assumes the first token to consume is an identifier, and continues to collect
    /// if the next token is a DOT delimiter.
    fn compose_name<I>(tokens: &mut Peekable<I>) -> SelectedName
    where I: Iterator<Item=Token<VHDLToken>>  {
        let mut selected_name = Vec::new();
        // take first token as identifier
        let tk_id = tokens.next().expect("expecting name after '.'");
        if let Some(id) = tk_id.take().take_identifier() {
            selected_name.push(id);
        }
        while let Some(t) = tokens.peek() {
            // consume DOT and expect next identifier
            if t.as_type().check_delimiter(&Delimiter::Dot) {
                // consume DOT
                tokens.next();
                // expect identifier or bail
                let tk_id = tokens.next().expect("expecting name after '.'");

                if tk_id.as_type().check_keyword(&Keyword::All) {
                    // @TODO remember in `name` struct that all was used.
                    break;
                } else if let Some(id) = tk_id.take().take_identifier() {
                    selected_name.push(id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        SelectedName(selected_name)
    }

    /// Parses a package body, taking BODY keyword up until the END keyword.
    /// 
    /// Package declarations within this scope can be ignored because their visibility
    /// is not reached outside of the body.
    fn parse_package_body<I>(tokens: &mut Peekable<I>) -> PackageBody 
    where I: Iterator<Item=Token<VHDLToken>>  {
        // take the 'body' keyword
        tokens.next();
        // take package name
        let pack_name = tokens.next().take().unwrap().take();
        // println!("*--- package {}", pack_name);
        // take the IS keyword
        if tokens.next().take().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            panic!("expecting keyword IS")
        }
        VHDLSymbol::parse_body(tokens, &Self::is_primary_ending);
        PackageBody {
            owner: match pack_name {
                VHDLToken::Identifier(id) => id,
                _ => panic!("expected an identifier")
            },
            refs: Vec::new(),
        }
    }

    /// Detects identifiers instantiated in the architecture statement sections.
    /// 
    /// Assumes the next token to consume is instance name of the instantiation and
    /// the token to follow is the COLON ':' delimiter.
    fn parse_instantiation(statement: Statement) -> Option<Identifier> {
        let mut tokens = statement.0.into_iter().peekable();
        // force identifier (instance name)
        tokens.next()?.take().get_identifier()?;
        // force colon
        if tokens.next()?.take().check_delimiter(&Delimiter::Colon) == false { return None };
        // check what is instantiated
        match tokens.peek()?.as_type() {
            VHDLToken::Identifier(_) => {
                Some(Self::compose_name(&mut tokens).take_suffix())
            }
            VHDLToken::Keyword(kw) => {
                if kw == &Keyword::Component || kw == &Keyword::Entity || kw == &Keyword::Configuration {
                    tokens.next();
                    match tokens.peek()?.as_type() {
                        VHDLToken::Identifier(_) => {
                            Some(Self::compose_name(&mut tokens).take_suffix())
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_configuration<I>(tokens: &mut Peekable<I>) -> VHDLSymbol 
        where I: Iterator<Item=Token<VHDLToken>>  {
        let config_name = match tokens.next().take().unwrap().take() {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        };
        let entity_name = VHDLSymbol::parse_owner_design_unit(tokens);
        VHDLSymbol::parse_primary_declaration(tokens);
        VHDLSymbol::Configuration(Configuration {
            name: config_name,
            owner: entity_name,
        })
    }

    /// Consumes tokens after the USE keyword.
    /// 
    /// Assumes the last token consumed was USE and composes a statement of imports.
    fn parse_use_clause<I>(tokens: &mut Peekable<I>) -> UseClause 
        where I: Iterator<Item=Token<VHDLToken>> {
        // collect first selected_name
        let mut imports = Vec::new();
        imports.push(Self::compose_name(tokens));
        while let Some(t) = tokens.next() {
            // take the comma, then next selected name
            if t.as_type().check_delimiter(&Delimiter::Comma) {
                imports.push(Self::compose_name(tokens));
            }
            if t.as_type().check_delimiter(&Delimiter::Terminator) {
                break;
            }
        }
        UseClause { imports: imports }
    }

    fn parse_architecture<I>(tokens: &mut Peekable<I>) -> VHDLSymbol 
        where I: Iterator<Item=Token<VHDLToken>> {
        let arch_name = match tokens.next().take().unwrap().take() {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        };
        let entity_name = VHDLSymbol::parse_owner_design_unit(tokens);
        // println!("*--- unit {}", arch_name);

        let (deps, refs) =  VHDLSymbol::parse_primary_declaration(tokens);
        VHDLSymbol::Architecture(Architecture {
            name: arch_name,
            owner: entity_name,
            dependencies: deps,
            refs: refs,
        })
    }

    /// Parses together a series of tokens into a single `Statement`.
    /// 
    /// Statements end on a ';' and do not include the ';' token. If the EOF
    /// is reached before completing a statement, it is omitted and a blank
    /// statement is returned.
    fn compose_statement<I>(tokens: &mut Peekable<I>) -> Statement 
        where I: Iterator<Item=Token<VHDLToken>>  {
        let mut statement = Statement::new();
        while let Some(t) = tokens.next() {
            // exit upon encountering terminator ';'
            if t.as_type().check_delimiter(&Delimiter::Terminator) {
                return statement
            // extra keywords to help break up statements early
            } else if t.as_type().check_keyword(&Keyword::Generate) || 
                t.as_type().check_keyword(&Keyword::Begin) || 
                (statement.0.first().is_some() && statement.0.first().unwrap().as_type().check_keyword(&Keyword::When) &&
                t.as_type().check_delimiter(&Delimiter::Arrow)) {
                // add the breaking token to the statement before exiting
                statement.0.push(t);
                return statement
            } else {
                // check for resource references
                let mut took_dot: Option<Token<VHDLToken>> = None;
                if let Some(id) = t.as_type().get_identifier() {
                    // check if next token is a 'dot' delimiter
                    if tokens.peek().is_some() && tokens.peek().unwrap().as_type().check_delimiter(&Delimiter::Dot) {
                        took_dot = tokens.next();
                        if tokens.peek().is_some() {
                            if let Some(id2) = tokens.peek().unwrap().as_type().get_identifier() {
                                // store the resource reference
                                // println!("{} {}", id, id2);
                                statement.1.push(ResReference {
                                    prefix: id.clone(),
                                    suffix: id2.clone(),
                                });
                            }
                        }
                    }
                }
                statement.0.push(t);
                if let Some(dot) = took_dot {
                    statement.0.push(dot);
                }
            }
        }
        Statement::new()
    }

    /// Parses the OF keyword and then returns the following IDENTIFIER.
    /// 
    /// The Identifier should correspond to the architecture's entity name.
    fn parse_owner_design_unit<I>(tokens: &mut Peekable<I>) -> Identifier
    where I: Iterator<Item=Token<VHDLToken>>  {
        // force taking the 'of' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Of) == false {
            panic!("expecting 'of' keyword")
        }
        // return the name of the primary design unit
        match tokens.next().take().unwrap().take() {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        }
    }

    /// @TODO ?
    fn parse_subprogram<I>(_: &mut Peekable<I>) -> Vec<Statement>
    where I: Iterator<Item=Token<VHDLToken>>  {

        vec![]
    }

    /// Returns a list of interface items as `Statements`. 
    /// 
    /// Assumes the last token consumed was either GENERIC or PORT keywords and
    /// stops at the last statement in the respective list.
    fn parse_interface_list<I>(tokens: &mut Peekable<I>) -> Vec<Statement>
    where I: Iterator<Item=Token<VHDLToken>>  {
        // expect the opening '('
        if tokens.next().unwrap().as_type().check_delimiter(&Delimiter::ParenL) == false {
            panic!("expecting '(' delimiter")
        }
        // collect statements until finding the ')', END, BEGIN, or PORT.
        let mut statements: Vec<Statement> = Vec::new();
        while let Some(t) = tokens.peek() {
            if let Some(stmt) = statements.last() {
                if stmt.0.last().unwrap().as_type().check_delimiter(&Delimiter::ParenR) {
                    let balance = stmt.0.iter().fold(0, |acc, x| {
                        if x.as_type().check_delimiter(&Delimiter::ParenL) { acc + 1 }
                        else if x.as_type().check_delimiter(&Delimiter::ParenR) { acc - 1 }
                        else { acc }
                    });
                    if balance == -1 {
                        let index = statements.len()-1;
                        // pop off and exit
                        let last_statement = statements.get_mut(index).unwrap();
                        last_statement.0.pop().expect("expecting closing ')'");
                        break;
                    }
                }
            }
            if t.as_type().check_delimiter(&Delimiter::ParenR) {
                    Self::compose_statement(tokens);
                    break;
            // collect statements
            } else {
                statements.push(Self::compose_statement(tokens));
                // println!("{}", statements.last().unwrap());
            }
        }
        
        // println!("{:?}", statements);
        statements
    }

    /// Consumes tokens after `IS` until finding `BEGIN` or `END`.
    /// 
    /// Assumes the next token to consume is `IS` and throws it away. This will
    /// search for interface lists found after GENERIC and PORT keywords.
    fn parse_entity_declaration<I>(tokens: &mut Peekable<I>) -> (Vec<Statement>, Vec<Statement>)
        where I: Iterator<Item=Token<VHDLToken>> {
        // println!("*--- declaration section");
        // force taking the 'is' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            panic!("expecting 'is' keyword")
        }
        // check entity_header before entering entity declarative part
        // check for generics
        if tokens.peek().is_none() { panic!("expecting END keyword") }
        let generics = if tokens.peek().unwrap().as_type().check_keyword(&Keyword::Generic) {
            tokens.next();
            Self::parse_interface_list(tokens)
        } else {
            Vec::new()
        };
        // check for ports
        if tokens.peek().is_none() { panic!("expecting END keyword") }
        let ports = if tokens.peek().unwrap().as_type().check_keyword(&Keyword::Port) {
            tokens.next();
            Self::parse_interface_list(tokens)
        } else {
            Vec::new()
        };

        while let Some(t) = tokens.peek() {
            // stop the declaration section and enter a statement section
            if t.as_type().check_keyword(&Keyword::Begin) {
                tokens.next();
                Self::parse_body(tokens, &Self::is_primary_ending);
                break;
            // the declaration is over and there is no statement section
            } else if t.as_type().check_keyword(&Keyword::End) {
                let stmt = Self::compose_statement(tokens);
                if Self::is_primary_ending(&stmt) { 
                    break; 
                }
            // find a nested package (throw away for now)
            } else if t.as_type().check_keyword(&Keyword::Package) {
                tokens.next();
                let _pack_name = Self::route_package_parse(tokens);
                // println!("**** INFO: detected nested package \"{}\"", pack_name);
            // build statements to throw away
            } else {
                let _stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
            } 
        }
        (generics, ports)
    }

    /// Consumes tokens after `IS` until finding `BEGIN` or `END`.
    /// 
    /// Assumes the next token to consume is `IS` and throws it away.
    fn parse_primary_declaration<I>(tokens: &mut Peekable<I>) -> (Vec<Identifier>, Vec<ResReference>)
        where I: Iterator<Item=Token<VHDLToken>> {
        // println!("*--- declaration section");
        // force taking the 'is' keyword
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Is) == false {
            panic!("expecting 'is' keyword")
        }
        let mut refs = Vec::new();
        while let Some(t) = tokens.peek() {
            // stop the declaration section and enter a statement section
            if t.as_type().check_keyword(&Keyword::Begin) {
                tokens.next();
                return Self::parse_body(tokens, &Self::is_primary_ending);
            // the declaration is over and there is no statement section
            } else if t.as_type().check_keyword(&Keyword::End) {
                let stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
                if Self::is_primary_ending(&stmt) { 
                    break; 
                }
            // find component names (could be in package or architecture declaration)
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp_name = Self::parse_component(tokens);
                // println!("**** INFO: Found component: \"{}\"", comp_name);
            // find a nested package
            } else if t.as_type().check_keyword(&Keyword::Package) {
                tokens.next();
                let _pack_name = Self::route_package_parse(tokens);
                // println!("**** INFO: detected nested package \"{}\"", pack_name);
            // build statements to throw away
            } else {
                let stmt = Self::compose_statement(tokens);
                // add resource references
                refs.append(&mut stmt.take_refs());
                // println!("{:?}", stmt);
            } 
        }
        (Vec::new(), refs)
    }

    /// Checks if the statement is a valid primary unit END statement.
    /// 
    /// Rejects inferior END statements that require an explicit keyword to following
    /// the END keyword. Primary END statements follow: `END [keyword] [identifier];`.
    fn is_primary_ending(stmt: &Statement) -> bool {
        let keyword = if let Some(t) = stmt.0.get(1) {
            t.as_type()
        } else {
            return true // @TODO make sure "end;" will end up being valid
        };
        match keyword {
            // list mandatory keywords expected after the 'end' keyword for non-primary endings
            VHDLToken::Keyword(kw) => match kw {
                Keyword::Loop | Keyword::Generate | Keyword::Process |
                Keyword::Postponed | Keyword::If | Keyword::Block | 
                Keyword::Protected | Keyword::Record | Keyword::Case | 
                Keyword::Component | Keyword::For => false,
                _ => true,
            },
            _ => true,
        }
    }

    /// Checks if the keyword indicates a subprogram statement.
    fn enter_subprogram(kw: &Keyword) -> bool {
        match kw {
            Keyword::Function | Keyword::Procedure | Keyword::Impure | 
            Keyword::Pure => true,
            _ => false,
        }
    }

    /// Parses a component declaration, consuming the tokens `COMPONENT` until the end.
    /// 
    /// Assumes the first token to consume is `COMPONENT`.
    fn parse_component<I>(tokens: &mut Peekable<I>) -> Identifier
    where I: Iterator<Item=Token<VHDLToken>>  {
        if tokens.next().unwrap().as_type().check_keyword(&Keyword::Component) == false {
            panic!("assumes first token is COMPONENT keyword");
        }
        // take component name
        let comp_name = tokens.next().take().unwrap().take();
        // println!("*--- found component {}", comp_name);
        // take 'is' keyword (optional)
        if tokens.peek().unwrap().as_type().check_keyword(&Keyword::Is) {
            tokens.next();
        }
        // @TODO collect port names and generic names until hitting 'END'
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) {
                let _stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
                break;
            // collect generic statements
            } else if t.as_type().check_keyword(&Keyword::Generic) {
                // take the GENERIC token
                tokens.next();
                let _generics = Self::parse_interface_list(tokens);
            // collect ports
            } else if t.as_type().check_keyword(&Keyword::Port) {
                // take the PORT token
                tokens.next();
                let _ports = Self::parse_interface_list(tokens);
            } else {
                let _stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
            }
        }
        match comp_name {
            VHDLToken::Identifier(id) => id,
            _ => panic!("expected an identifier")
        }
    }

    /// Routes the parsing to either package body or package declaration,
    /// depending on the next token being BODY keyword or identifier.
    fn route_package_parse<I>(tokens: &mut Peekable<I>) -> VHDLSymbol
    where I: Iterator<Item=Token<VHDLToken>> {
        if &VHDLToken::Keyword(Keyword::Body) == tokens.peek().unwrap().as_type() {
            VHDLSymbol::PackageBody(VHDLSymbol::parse_package_body(tokens))
        } else {
            VHDLSymbol::parse_package_declaration(tokens)
        }
    }

    /// Parses a body, consuming tokens from `BEGIN` until `END`.
    /// 
    /// Builds statements and stops after finding the `END` keyword statement. If
    /// the `END` keyword statement is detected, it will have to pass the `eval_exit`
    /// function to properly exit scope. Assumes the last token consumed was `BEGIN`.
    fn parse_body<I>(tokens: &mut Peekable<I>, eval_exit: &dyn Fn(&Statement) -> bool) -> (Vec<Identifier>, Vec<ResReference>)
        where I: Iterator<Item=Token<VHDLToken>> {
        // collect component names
        let mut deps = Vec::new();
        let mut refs = Vec::new();
        // println!("*--- statement section");
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::End) == true {
                let stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
                if eval_exit(&stmt) == true { 
                    break; 
                }
            // enter a subprogram
            } else if t.as_type().check_keyword(&Keyword::Function) || t.as_type().check_keyword(&Keyword::Begin) {
                let _stmt = Self::compose_statement(tokens);
                // println!("ENTERING SUBPROGRAM {:?}", stmt);
                let mut inner = Self::parse_body(tokens, &Self::is_primary_ending);
                refs.append(&mut inner.1);
                // println!("EXITING SUBPROGRAM");
            // find component names (could be in package)
            } else if t.as_type().check_keyword(&Keyword::Component) {
                let _comp_name = Self::parse_component(tokens);
                // println!("**** INFO: Found component: \"{}\"", comp_name);
            // find packages 
            } else if t.as_type().check_keyword(&Keyword::Package) {
                tokens.next();
                let _symbol = Self::route_package_parse(tokens);
                // println!("**** INFO: Detected nested package \"{}\"", symbol);
            // build statements
            } else {
                let stmt = Self::compose_statement(tokens);
                // println!("{:?}", stmt);
                // check if statement is an instantiation
                if let Some(inst) = Self::parse_instantiation(stmt) {
                    // println!("info: detected dependency \"{}\"", inst);
                    deps.push(inst);
                }
                // refs.append(&mut stmt.take_refs()); @TODO
            }
        }
        (deps, refs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_use_clause() {
        let s = "use eel4712c.pkg1, eel4712c.pkg2; entity";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        // take USE
        tokens.next();
        let using_imports = VHDLSymbol::parse_use_clause(&mut tokens);
        assert_eq!(using_imports, UseClause { 
            imports: vec![
                SelectedName(vec![
                    Identifier::Basic("eel4712c".to_owned()),
                    Identifier::Basic("pkg1".to_owned()),
                ]),
                SelectedName(vec![
                    Identifier::Basic("eel4712c".to_owned()),
                    Identifier::Basic("pkg2".to_owned()),
                ]),
        ]});
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Entity));
    }

    #[test]
    fn compose_statement_2() {
        let s = "P1, P2: inout BIT); constant Delay: TIME := 1 ms;";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).as_types(), vec![
            &VHDLToken::Identifier(Identifier::Basic("P1".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Comma),
            &VHDLToken::Identifier(Identifier::Basic("P2".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Colon),
            &VHDLToken::Keyword(Keyword::Inout),
            &VHDLToken::Identifier(Identifier::Basic("BIT".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenR),
        ]);
        assert_eq!(iter.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Constant));
    }

    #[test]
    fn parse_simple_name() {
        let s = "eel4712c.nor_gate port";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let sel_name = VHDLSymbol::compose_name(&mut tokens);
        assert_eq!(sel_name, SelectedName(vec![
            Identifier::Basic("eel4712c".to_owned()),
            Identifier::Basic("nor_gate".to_owned()),
        ]));
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Port));
    }

    #[test]
    fn parse_simple_name_with_all() {
        // @TODO signify within a 'name' struct that the all keyword was used
        let s = "eel4712c.all +";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let sel_name = VHDLSymbol::compose_name(&mut tokens);
        assert_eq!(sel_name, SelectedName(vec![
            Identifier::Basic("eel4712c".to_owned()),
        ]));
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Delimiter(Delimiter::Plus));
    }

    #[test]
    fn parse_port_interface_difficult_ending() {
        let s = "\
port (P1, P2: inout BIT); 
constant Delay: TIME := 1 ms;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        tokens.next(); // take PORT
        let ports = VHDLSymbol::parse_interface_list(&mut tokens);
        let ports: Vec<String> = ports.into_iter().map(|m| m.to_string()).collect();
        assert_eq!(ports, vec![
            "P1 , P2 : inout BIT",
        ]);
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Constant));
    }

    #[test]
    fn parse_ports_both() {
        let s = "\
generic ( N: positive );
port(
    a: in  std_logic_vector(N-1 downto 0);
    b: in  std_logic_vector(N-1 downto 0);
    c: out std_logic_vector(N-1 downto 0)
);
end;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        tokens.next(); // take GENERIC
        let generics = VHDLSymbol::parse_interface_list(&mut tokens);
        // convert to strings for easier verification
        let generics: Vec<String> = generics.into_iter().map(|m| m.to_string()).collect();
        assert_eq!(generics, vec![
            "N : positive",
        ]);
        // take PORT
        tokens.next();
        let ports = VHDLSymbol::parse_interface_list(&mut tokens);
         // convert to strings for easier verification
        let ports: Vec<String> = ports.into_iter().map(|m| m.to_string()).collect();
        assert_eq!(ports, vec![
            "a : in std_logic_vector(N - 1 downto 0)",
            "b : in std_logic_vector(N - 1 downto 0)",
            "c : out std_logic_vector(N - 1 downto 0)",
        ]);
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::End));
    }

    #[test]
    fn parse_generics_only() {
        let s = "\
generic ( N: positive );
begin
end;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        tokens.next(); // take GENERIC
        let generics = VHDLSymbol::parse_interface_list(&mut tokens);
        // convert to strings for easier verification
        let generics: Vec<String> = generics.into_iter().map(|m| m.to_string()).collect();
        assert_eq!(generics, vec![
            "N : positive",
        ]);
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Begin));
    }

    #[test]
    fn parse_component() {
        // ends with 'end component nor_gate;' Statement
        let s = "\
component nor_gate is end component nor_gate;

signal ready: std_logic;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let comp = VHDLSymbol::parse_component(&mut tokens);
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Signal));
        
        // ends with 'end;' statement
        let s = "\
component nor_gate end;

signal ready: std_logic;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let comp = VHDLSymbol::parse_component(&mut tokens);
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Signal));

        // ends with 'end component nor_gate;' statement
        let s = "\
-- declare DUT component
component nor_gate 
    generic( N: positive );
    port(
        a: in  std_logic_vector(N-1 downto 0);
        b: in  std_logic_vector(N-1 downto 0);
        c: out std_logic_vector(N-1 downto 0)
    );
end component nor_gate;

signal ready: std_logic;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let comp = VHDLSymbol::parse_component(&mut tokens);
        assert_eq!(comp.to_string(), "nor_gate");
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::Signal));
    }

    use crate::core::lexer::Position;

    #[test]
    fn is_primary_ending() {
        // 'case' is not a primary ending
        let stmt = Statement(vec![
            Token::new(VHDLToken::Keyword(Keyword::End), Position::new()),
            Token::new(VHDLToken::Keyword(Keyword::Case), Position::new()),
            Token::new(VHDLToken::Identifier(Identifier::Basic("case_label".to_owned())), Position::new()),
        ], vec![]);
        assert_eq!(VHDLSymbol::is_primary_ending(&stmt), false);

        // primary endings can omit keyword and identifier label
        let stmt = Statement(vec![
            Token::new(VHDLToken::Keyword(Keyword::End), Position::new()),
        ], vec![]);
        assert_eq!(VHDLSymbol::is_primary_ending(&stmt), true);

        // primary endings can include their keyword
        let stmt = Statement(vec![
            Token::new(VHDLToken::Keyword(Keyword::End), Position::new()),
            Token::new(VHDLToken::Keyword(Keyword::Architecture), Position::new()),
            Token::new(VHDLToken::Identifier(Identifier::Basic("architecture_name".to_owned())), Position::new()),
        ], vec![]);
        assert_eq!(VHDLSymbol::is_primary_ending(&stmt), true);

        // primary endings can have their keyword omitted and also include the identifier label
        let stmt = Statement(vec![
            Token::new(VHDLToken::Keyword(Keyword::End), Position::new()),
            Token::new(VHDLToken::Identifier(Identifier::Basic("architecture_name".to_owned())), Position::new()),
        ], vec![]);
        assert_eq!(VHDLSymbol::is_primary_ending(&stmt), true);
    }

    #[test]
    fn entity() {
        let s = "\
nor_gate is
    generic(
        N: positive
    );
    port(
        a: in  std_logic_vector(N-1 downto 0);
        b: in  std_logic_vector(N-1 downto 0);
        c: out std_logic_vector(N-1 downto 0)
    );
end entity nor_gate;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let _ = Entity::from_tokens(&mut tokens);

        // @TODO write signals from ports
    }

    use std::str::FromStr;

    #[test]
    fn resource_refs() {
        let s = "a : in std_logic_vector(3 downto 0) := work.pack1.p1;";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).get_refs(), vec![
            &ResReference { prefix: Identifier::from_str("work").unwrap(), suffix: Identifier::from_str("pack1").unwrap()},
            &ResReference { prefix: Identifier::from_str("pack1").unwrap(), suffix: Identifier::from_str("p1").unwrap()},
        ]);

        let s = "use work.package_name;";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).get_refs(), vec![
            &ResReference { prefix: Identifier::from_str("work").unwrap(), suffix: Identifier::from_str("package_name").unwrap()},
        ]);

        let s = "use MKS.MEASUREMENTS, STD.STANDARD;";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).get_refs(), vec![
            &ResReference { prefix: Identifier::from_str("MKS").unwrap(), suffix: Identifier::from_str("MEASUREMENTS").unwrap()},
            &ResReference { prefix: Identifier::from_str("STD").unwrap(), suffix: Identifier::from_str("STANDARD").unwrap()},
        ]);
    }

    #[test]
    fn compose_statement() {
        let s = "a : in std_logic_vector(3 downto 0);";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).as_types(), vec![
            &VHDLToken::Identifier(Identifier::Basic("a".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Colon),
            &VHDLToken::Keyword(Keyword::In),
            &VHDLToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenL),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
            &VHDLToken::Keyword(Keyword::Downto),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenR),
        ]);

        let s = "a : in std_logic_vector(3 downto 0); ready: out std_logic);";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        assert_eq!(VHDLSymbol::compose_statement(&mut iter).as_types(), vec![
            &VHDLToken::Identifier(Identifier::Basic("a".to_owned())),
            &VHDLToken::Delimiter(Delimiter::Colon),
            &VHDLToken::Keyword(Keyword::In),
            &VHDLToken::Identifier(Identifier::Basic("std_logic_vector".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenL),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("3".to_owned())),
            &VHDLToken::Keyword(Keyword::Downto),
            &VHDLToken::AbstLiteral(AbstLiteral::Decimal("0".to_owned())),
            &VHDLToken::Delimiter(Delimiter::ParenR),
        ]);

        let s = "process(all) is begin end process;";
        let mut tokens = VHDLTokenizer::from_source_code(&s).into_tokens().into_iter().peekable();
        let _ = VHDLSymbol::compose_statement(&mut tokens);
        assert_eq!(tokens.next().unwrap().as_type(), &VHDLToken::Keyword(Keyword::End));
    }

    #[test]
    fn print_statement() {
        let s = "a : in std_logic_vector ( 3 downto 0);";
        let tokens = VHDLTokenizer::from_source_code(&s).into_tokens();
        let mut iter = tokens.into_iter().peekable();
        let st = VHDLSymbol::compose_statement(&mut iter);
        assert_eq!(st.to_string(), "a : in std_logic_vector(3 downto 0)");
    }

    #[test]
    #[ignore]
    fn parse_basic() {
        let s = "\
-- design file for a nor_gate
library ieee;
use ieee.std_logic_1164.all;

entity nor_gate is -- comment on this line
    generic(
        N: positive
    );
    port(
        a : in std_logic_vector(N-1 downto 0);
        b : in std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
begin
end entity nor_gate;

architecture rtl of nor_gate is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14;
    constant freq_hz : unsigned := 50_000_000;
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx\"\";
    constant MAGIC_NUM_1 : integer := 2#10101#; -- test constants against tokenizer
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 0; -- 8c\"11\";
begin
    c <= a nor \\In\\;

end architecture rtl; /* long comment */

entity nor_gate_tb is end;

architecture tb of nor_gate_tb is 
-- declare DUT component
component nor_gate 
	generic(
		N: positive
	);
	port(
		a: in  std_logic_vector(N-1 downto 0);
		b: in  std_logic_vector(N-1 downto 0);
		c: out std_logic_vector(N-1 downto 0)
	);
end component nor_gate;
begin 
	DUT : component nor_gate 
    generic map     (
		N   => N
	) port map(
		a => w_a,
		b => w_b,
		c => w_c
	);

end;

package P is

function F return INTEGER;
attribute FOREIGN of F: function is \"implementation-dependent information\"; 

end package P;

package outer is
    package inner is 
        component some_component is
        end component;
    end package;
end package;

entity X is
port (P1, P2: inout BIT); 
constant Delay: TIME := 1 ms;
begin
CheckTiming (P1, P2, 2*Delay); end X ;

package TimeConstants is 
constant tPLH : Time := 9 ns;
constant tPHL : Time := 10 ns;
constant tPLZ :  Time := 12 ns;
constant tPZL :  Time := 7 ns;
constant tPHZ :  Time := 8 ns;
constant tPZH : Time := 8 ns;
end TimeConstants ;

package body TriState is

    package ent is 

    end;
    function BitVal (Value: Tri) return Bit is
        constant Bits : Bit_Vector := \"0100\"; 
    begin
        return Bits(Tri'Pos(Value)); 
    end;

    function TriVal (Value: Bit) return Tri is 
    begin
        return Tri'Val(Bit'Pos(Value)); 
    end;

    function Resolve (Sources: TriVector) return Tri is 
        variable V: Tri := 'Z';
    begin
        for i in Sources'Range loop
            if Sources(i) /= 'Z' then 
                if V = 'Z' then
                    V := Sources(i);
                else 
                    return 'E';
                end if; 
            end if;
        end loop; 
        return V;
    end;

end package body TriState;

architecture test of nor_gate is 

begin 

GEN_ADD: for I in 0 to 7 generate

LOWER_BIT: if I=0 generate
  U0: HALFADD port map
     (A(I),B(I),S(I),C(I));
end generate LOWER_BIT;

UPPER_BITS: if I>0 generate
  UX: FULLADD port map
     (A(I),B(I),C(I-1),S(I),C(I));
end generate UPPER_BITS;

end generate GEN_ADD;

end architecture;

architecture rtl of complex_multiplier is
begin

    mult_structure : case implementation generate 
        when single_cycle => 
            signal real_pp1, real_pp2 : ...;
            ...;
            begin
                real_mult1 : component multiplier
                    port map ( ... ); 
            end;
        when multicycle =>
            signal real_pp1, real_pp2 : ...;
            ...;
            begin
                mult : component multiplier
                    port map ( ... );
            end;
        when pipelined => 
            mult1 : component multiplier
                port map ( ... );
    end generate mutl_structure;

end architecture rtl;
";
        let _ = VHDLParser::parse(VHDLTokenizer::from_source_code(&s).into_tokens());
        panic!("manually inspect token list")
    }
}