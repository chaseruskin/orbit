use crate::util::anyerror::Fault;
use std::path::PathBuf;
use std::collections::HashMap;
use ignore;
use ignore::overrides::OverrideBuilder;

use super::config::FromToml;

type VarLUT = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub struct Template {
    alias: String, // required
    root: String, // required
    summary: Option<String>,
    ignores: Vec<String>,
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<16}{}", self.alias, self.summary.as_ref().unwrap_or(&String::new()))
    }
}

impl FromToml for Template {
    type Err = TemplateError;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        // take alias entry
        let alias = match table.get("alias") {
            Some(i) => match i.as_str() {
                Some(s) => s,
                None => return Err(TemplateError::EntryNotString("alias".to_owned()))
            }
            None => return Err(TemplateError::MissingAlias)
        };
        // take path entry
        let path = match table.get("path") {
            Some(i) => match i.as_str() {
                Some(s) => s,
                None => return Err(TemplateError::EntryNotString("path".to_owned()))
            }
            None => return Err(TemplateError::MissingPath)
        };
        // take summary entry
        let summary = match table.get("summary") {
            Some(i) => match i.as_str() {
                Some(s) => Some(s.to_owned()),
                None => return Err(TemplateError::EntryNotString("summary".to_owned()))
            }
            None => None,
        };
        // take ignores entries
        let ignore_list = match table.get("ignore") {
            Some(i) => match i.as_array() {
                Some(arr) => arr.into_iter()
                    .filter_map(|f| f.as_str() )
                    .map(|f| f.to_owned())
                    .collect(),
                None => return Err(TemplateError::IgnoresNotArray),
            }
            None => Vec::new(),
        };
        Ok(Template {
            alias: alias.to_string(),
            root: path.to_string(),
            summary: summary,
            ignores: ignore_list,
        })
    }
}

impl Template {
    /// Creates a new empty `Template` struct.
    pub fn new() -> Self {
        Self { 
            alias: String::new(), 
            root: String::new(),
            summary: None, 
            ignores: Vec::new() 
        }
    }

    /// Copies contents from `self.root` into destination `dest`.
    /// 
    /// Collects all paths to copy. Removes any paths that match with a
    /// pattern provided in the ignores list. Implicitly ignores `Orbit.toml`
    /// files and .git/ directories.
    pub fn import(&self, dest: &PathBuf, vars: &VarLUT) -> Result<(), Fault> {
        std::env::set_current_dir(self.path())?;
        let mut overrides = OverrideBuilder::new(".");
        // add implicit ignores
        overrides.add("!Orbit.toml").unwrap().add("!.git/").unwrap();
        // add user-defined ignores
        for rule in &self.ignores {
            overrides.add(&format!("!{}", rule))?;
        }
        // collect all paths
        let result = ignore::WalkBuilder::new(".")
            .overrides(overrides.build()?)
            .hidden(false)
            .build();
        let paths: Vec<(bool, PathBuf)> = result.into_iter()
            .filter_map(|f| if f.is_ok() { Some((f.as_ref().unwrap().path().is_dir(), f.unwrap().into_path())) } else { None })
            .collect();
        // make destination directory
        std::fs::create_dir_all(&dest)?;
        // change directory to destination
        std::env::set_current_dir(&dest)?;

        // create all directories
        let root = PathBuf::from(self.path());
        for (is_dir, dir) in &paths {
            if is_dir == &true { 
                // perform variable subtitution on the filepath
                let to_dir = substitute(dir.to_str().unwrap().to_owned(), vars);
                std::fs::create_dir_all(&to_dir)?; 
            }
        }
        // create and transform all files
        for (is_dir, file) in &paths {
            if is_dir == &false { 
                // perform variable substitution on the filepath
                let to_file = substitute(file.to_str().unwrap().to_owned(), vars);

                // attempt to read data to string to perform data transformation
                std::fs::copy(root.join(&file), &to_file)?;
                let temp = TemplateFile::path(&PathBuf::from(to_file));
                // silently ignore errors (reading to string or failing to write)
                match temp.substitute(vars) {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
        }
        Ok(())
    }

    /// Applies the `resolve_path` fn to the `path` value.
    /// 
    /// Assumes `root` is the parent directory to the config.toml file that
    /// created this `Plugin` struct.
    pub fn resolve_root_path(mut self, root: &std::path::PathBuf) -> Self {
        self.root = crate::util::filesystem::resolve_rel_path(&root, self.root);
        self
    }

    /// Creates a string to display a list of templates.
    pub fn list_templates(temps: &[&Template]) -> String {
        let mut list = String::from("Templates:\n");
        for temp in temps {
            list += &format!("    {}\n", temp);
        }
        list
    }

    pub fn alias(&self) -> &String {
        &self.alias
    }

    pub fn path(&self) -> &String {
        &self.root
    }
    
    pub fn ignores(&self) -> &Vec<String> {
        &self.ignores
    }

    pub fn summary(&self) -> &Option<String> {
        &self.summary
    }
}

#[derive(Debug, PartialEq)]
pub enum TemplateError {
    EntryNotString(String),
    MissingAlias,
    MissingPath,
    UnknownKey(String),
    IgnoresNotArray,
}

impl std::error::Error for TemplateError {}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntryNotString(k) => write!(f, "key '{}' expects a string", k),
            Self::MissingAlias => write!(f, "key 'alias' holding a string is required for a template"),
            Self::MissingPath => write!(f, "key 'path' holding a string is required for a template"),
            Self::UnknownKey(k) => write!(f, "unknown key '{}' skipped in template array of tables", k),
            Self::IgnoresNotArray => write!(f, "key 'ignore' expects an array of strings"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TemplateFile(PathBuf);

impl TemplateFile {
    pub fn new() -> Self {
        Self(PathBuf::new())
    }

    pub fn path(p: &PathBuf) -> Self {
        Self(p.to_owned())
    }

    /// Performs variable substitution on the file data
    pub fn substitute(&self, code: &VarLUT) -> Result<(), Fault> {
        // read the data
        let contents = std::fs::read_to_string(&self.0)?;
        // transform the data and write it to file
        std::fs::write(&self.0, substitute(contents, code))?;
        Ok(())
    }
}

const L_VAR_DELIMITER: char = '{';
const R_VAR_DELIMITER: char = '}';

/// Performs variable replacement on the given `text`, looking up variables in
/// the `code` to swap with their values.
fn substitute(text: String, code: &VarLUT) -> String {
    let mut result = String::new();
    
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        // check if there is a valid variable replacement
        match c {
            L_VAR_DELIMITER => {
                match gather_variable(&mut chars, c, R_VAR_DELIMITER) {
                    Ok(r) => {
                        // remove delimiters and surrounding whitespace to get key name
                        let key = &r[2..r.len()-2].trim();
                        // look up the key in the code book
                        match code.get(*key) {
                            Some(value) => result.push_str(value),
                            None => result.push_str(&r),
                        }
                    }
                    Err(e) => result.push_str(&e),
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Builds a variable following the syntax `c0c0*c_nc_n`.
/// 
/// Assumes the first token was already consumed and is passed as `c0`.
/// 
/// Errors if the syntax is not fulfilled. 
fn gather_variable<T: Iterator<Item=char>>(chars: &mut T, c0: char, c_n: char) -> Result<String, String> {
    let mut var = String::from(c0);
    let mut last: Option<char> = None;
    // verify next character is also `c0`
    if let Some(c) = chars.next() {
        var.push(c);
        if c != c0 { return Err(var) }
        last = Some(c);
    }
    // collect remaining characters until finding double cN occurrences 
    while let Some(c) = chars.next() {
        var.push(c);
        if c == c_n {
            // check if the last char was also `cN`
            if last.unwrap() == c_n {
                return Ok(var)
            }
        }
        last = Some(c);
    }
    // never was able to close the variable
    Err(var)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::*;

    /// Internal helper test `fn` to generate a sample code book for looking up variables.
    fn create_code() -> HashMap<String, String> {
        let mut code = HashMap::new();
        code.insert("orbit.name".to_owned(), "gates".to_owned());
        code.insert("orbit.library".to_owned(), "rary".to_owned());
        code.insert("orbit.place".to_owned(), "bar".to_owned());
        code
    }

    #[test]
    fn gather_var() {
        let text = "{ variable }}";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Ok("{{ variable }}".to_owned()));

        let text = "{ variable }";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{{ variable }".to_owned()));

        let text = "variable }";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{v".to_owned()));

        let text = "{variable } } ";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{{variable } } ".to_owned()));
    }

    #[test]
    fn replace_variables() {
        let text = "The quick brown fox jumped over the lazy {{ orbit.name }}.";
        let code = create_code();
        assert_eq!(substitute(text.to_owned(), &code), "The quick brown fox jumped over the lazy gates.".to_owned());

        let text = "A duck, a bear, and a {{ animal }} walk into a {{  orbit.place   }}...";
        let code = create_code();
        assert_eq!(substitute(text.to_owned(), &code), "A duck, a bear, and a {{ animal }} walk into a bar...".to_owned());
    }

    #[test]
    fn from_toml() {
        let toml = r#"
[[template]]
alias = "base"
path = "profile/ks-tech/template"
ignore = [
    "build/",
    "extra/",
]
"#;
        let doc = toml.parse::<toml_edit::Document>().unwrap();
        let template = Template::from_toml(&doc["template"].as_array_of_tables().unwrap().get(0).unwrap()).unwrap();
        assert_eq!(template, Template { 
            alias: String::from("base"), 
            summary: None,
            root: String::from("profile/ks-tech/template"), 
            ignores: vec![
                "build/".to_string(),
                "extra/".to_string(),
            ],
        });
    }
}