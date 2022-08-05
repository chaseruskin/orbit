use crate::util::{anyerror::Fault, filesystem};
use std::{path::PathBuf, collections::HashSet};
use ignore;
use ignore::overrides::OverrideBuilder;

use super::{config::{FromToml, FromTomlError}, variable::VariableTable};

#[derive(Debug, PartialEq)]
pub struct Template {
    alias: String,
    root: String,
    summary: Option<String>,
    ignores: Vec<String>,
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<16}{}", self.alias, self.summary.as_ref().unwrap_or(&String::new()))
    }
}

impl FromToml for Template {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Template {
            alias: Self::require(table, "alias")?,
            root: Self::require(table, "path")?,
            summary: Self::get(table, "summary")?,
            // take ignore entries
            ignores: match table.get("ignore") {
                Some(i) => match i.as_array() {
                    Some(arr) => arr.into_iter()
                        .filter_map(|f| f.as_str() )
                        .map(|f| f.to_owned())
                        .collect(),
                    None => return Err(FromTomlError::ExpectingStringArray("ignore".to_owned()))?,
                }
                None => Vec::new(),
            },
        })
    }
}

impl Template {
    /// Creates a string to display a list of templates.
    pub fn list_templates(temps: &[&Template]) -> String {
        let mut list = String::from("Templates:\n");
        for temp in temps {
            list += &format!("    {}\n", temp);
        }
        list
    }

    /// Creates a new empty `Template` struct.
    pub fn new() -> Self {
        Self { 
            alias: String::new(), 
            root: String::new(),
            summary: None, 
            ignores: Vec::new() 
        }
    }

    /// Collects the template's files while respecting implicit ignores to
    /// the Orbit.toml, Orbit.lock, and .git/ paths.
    /// 
    /// If `allow_hidden` is false, then it will use the globs defined in the `ignore` entry
    /// to filter out files.
    /// 
    /// Returns a list of files paths, and if they are a directory (`true`) or file (`false`).
    fn gather_files(&self, src: &PathBuf, allow_hidden: bool) -> Result<Vec<(bool, PathBuf)>, Fault> {
        let mut overrides = OverrideBuilder::new(src);
        // add implicit ignores
        overrides
            .add("!Orbit.toml").unwrap()
            .add("!.git/").unwrap()
            .add("!Orbit.lock").unwrap();

        // add user-defined ignores if they are not allowed
        if allow_hidden == false {
            for rule in &self.ignores {
                overrides.add(&format!("!{}", rule))?;
            }
        }

        // collect all paths
        let result = ignore::WalkBuilder::new(src)
            .overrides(overrides.build()?)
            .hidden(false)
            .build();
        let paths: Vec<(bool, PathBuf)> = result.into_iter()
            .filter_map(|f| if f.is_ok() { Some((f.as_ref().unwrap().path().is_dir(), f.unwrap().into_path())) } else { None })
            .collect();
        Ok(paths)
    }

    /// Copies contents from `self.root` into destination `dest`.
    /// 
    /// Collects all paths to copy. Removes any paths that match with a
    /// pattern provided in the ignores list. Implicitly ignores `Orbit.toml`
    /// files and .git/ directories.
    pub fn import(&self, dest: &PathBuf, vars: &VariableTable) -> Result<(), Fault> {
        std::env::set_current_dir(self.path())?;
        let paths = self.gather_files(&PathBuf::from("."), false)?;
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
                let dest = PathBuf::from(to_file);
                let temp = TemplateFile::new(&dest);
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

    pub fn display_files(&self) -> () {
        let header = format!("\
{:<46}{:<10}
{2:->46}{2:->10}\n",
            "Relative Path", "Hidden", " ");
        // collect files
        let root = PathBuf::from(self.path());
        let all: Vec<PathBuf> = self.gather_files(&root, true).unwrap()
            .into_iter()
            .filter_map(|(d, f)| if d == false { Some(f) } else { None })
            .collect();
        // collect hashset of files that were not ignored
        let imports_set = {
            let mut set = HashSet::new();
            self.gather_files(&root, false).unwrap()
                .into_iter()
                .for_each(|(d, f)| if d == false { set.insert(f); () } else { () });
            set
        }; 
        // format each file's line in the list
        let s = all.iter()
            .fold(String::new(), |x, y| { 
                x + &format!("{:<46}{:<10}\n",
                // pretty-print the filepath as relative with forward-slashes 
                &filesystem::normalize_path(filesystem::remove_base(&root, &y)).display().to_string(),
                // denote if the file was ignored or not
                { if imports_set.contains(y) == false { "y" } else { "" }})
            });
        println!("{}{}", header, s)
    }
}

#[derive(Debug, PartialEq)]
pub struct TemplateFile<'a>(&'a PathBuf);

impl<'a> TemplateFile<'a> {

    pub fn new(p: &'a PathBuf) -> Self {
        Self(p)
    }

    /// Performs variable substitution on the file data.
    /// 
    /// Writes the contents back to the path it read from.
    pub fn substitute(&self, code: &VariableTable) -> Result<(), Fault> {
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
pub fn substitute(text: String, code: &VariableTable) -> String {
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
    use super::*;

    /// Internal helper test `fn` to generate a sample code book for looking up variables.
    fn create_code() -> VariableTable {
        let mut code = VariableTable::new();
        code.add("orbit.name", "gates");
        code.add("orbit.library", "rary");
        code.add("orbit.place", "bar");
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