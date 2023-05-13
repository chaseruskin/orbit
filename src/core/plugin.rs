use std::path::PathBuf;
use std::error::Error;
use crate::core::fileset::Fileset;
use crate::core::config::FromToml;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::filesystem::Standardize;
use super::config::FromTomlError;
use super::context::Context;

#[derive(Debug, PartialEq)]
pub struct Plugin {
    alias: String,
    command: String,
    args: Vec<String>,
    filesets: Vec<Fileset>,
    summary: Option<String>,
    details: Option<String>,
    root: Option<PathBuf>,
}

impl Process for Plugin {
    fn get_root(&self) -> &PathBuf {
        &self.root.as_ref().unwrap()
    }

    fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    fn get_command(&self) -> &String {
        &self.command
    }
}

impl Plugin {
    /// Creates a new `Plugin` struct.
    pub fn new() -> Self {
        Self { 
            alias: String::new(), 
            command: String::new(), 
            args: Vec::new(),
            summary: None,
            details: None,
            root: None,
            filesets: Vec::new(),
        }
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self) -> String {
        format!("{:<16}{}", self.alias, self.summary.as_ref().unwrap_or(&String::new()))
    }

    /// Creates a string to display a list of plugins.
    /// 
    /// The string lists the plugins in alphabetical order by `alias`.
    pub fn list_plugins(plugs: &mut [&Plugin]) -> String {
        let mut list = String::from("Plugins:\n");
        plugs.sort_by(|a, b| a.alias.cmp(&b.alias));
        for plug in plugs {
            list += &format!("    {}\n", plug.quick_info());
        }
        list
    }

    /// References the plugin's `alias`.
    pub fn alias(&self) -> &String {
        &self.alias
    }

    /// References the plugin's `filesets`.
    pub fn filesets(&self) -> &Vec<Fileset> {
        &self.filesets
    }

    /// Sets the root directory from where the command should reference paths from.
    pub fn set_root(mut self, root: &PathBuf) -> Self {
        self.root = Some(root.to_path_buf());
        self
    }
}

impl std::fmt::Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\
alias:   {}
command: {} {}
root:    {}
filesets:
{}{}{}", 
            self.alias,
            self.command, self.args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " }),
            PathBuf::standardize(self.root.as_ref().unwrap()).display(),
            { if self.filesets.is_empty() { String::from("    None\n") } else { self.filesets.iter().fold(String::new(), |x, y| { x + &format!("    {:<16}{}\n", y.get_name(), y.get_pattern())}) } },
            { if let Some(text) = &self.summary { format!("\n{}\n", text) } else { String::new() } },
            { if let Some(text) = &self.details { format!("\n{}", text) } else { String::new() } },
        )
    }
}

impl FromToml for Plugin {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err>
    where Self: Sized {
        Ok(Self {
            alias: Self::require(table, "alias")?,
            command: Self::require(table, "command")?,
            args: if let Some(args) = table.get("args") {
                if args.is_array() == false {
                    return Err(FromTomlError::ExpectingStringArray(String::from("args")))?
                } else {
                    args.as_array().unwrap().into_iter().map(|f| f.as_str().unwrap().to_owned() ).collect()
                }
            } else {
                Vec::new()
            },
            root: None,
            summary: Self::get(table, "summary")?,
            details: Self::get(table, "details")?,
            filesets: {
                if let Some(inner_table) = table.get("fileset") {
                    // grab every key and value to transform into a fileset
                    let inner_table = inner_table.as_table_like().expect("fileset must be a table");
                    let mut iter = inner_table.iter();
                    let mut filesets = Vec::new();
                    while let Some((key, value)) = iter.next() {
                        let value = value.as_str().unwrap(); 
                        filesets.push(Fileset::new().name(key).pattern(value).unwrap())
                    }
                    filesets
                } else {
                    Vec::new()
                }
            }
        })
        // @todo: verify there are no extra keys
    }
}

pub trait Process {
    fn get_root(&self) -> &PathBuf;

    fn get_command(&self) -> &String;

    fn get_args(&self) -> &Vec<String>;

    /// Runs the given `command` with the set `args` for the plugin.
    fn execute(&self, extra_args: &[String], verbose: bool) -> Result<(), Fault> {
        // resolve the relative paths in the command and arguments defined in original configuration
        let root_path = self.get_root();
        let command = crate::util::filesystem::resolve_rel_path(root_path, &self.get_command());
        let arguments: Vec<String> = self.get_args().iter()
            .map(|f| crate::util::filesystem::resolve_rel_path(root_path, f) )
            .collect();

        // append args set on the command-line to the base-line of arguments
        let args = [&arguments, extra_args].concat();
        // display the literal command being ran
        if verbose == true {
            let s = args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " });
            println!("running: {} {}", command, s);
        }
        let mut proc = crate::util::filesystem::invoke(&command, &args, Context::enable_windows_bat_file_match())?;
        let exit_code = proc.wait()?;
        match exit_code.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
            None =>  Err(AnyError(format!("terminated by signal")))?
        }
    }

}

#[derive(Debug, PartialEq)]
pub enum PluginError {
    Missing(String)
}

impl Error for PluginError {}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "no plugin found as '{}'\n\nTry `orbit plan --list` to see available plugins", name)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let plug = Plugin::new();
        assert_eq!(plug, Plugin { 
            summary: None,
            details: None,
            root: None,
            alias: String::new(), 
            command: String::new(), 
            args: Vec::new(),
            filesets: Vec::new(),
        });
    }

    #[test]
    fn from_toml() {
        let toml = r#"
[[plugin]]
alias = "ghdl"
command = "python"
args = ["orbit-ghdl.py"]
fileset.py-model = "*_mdl.py"
details = "more info"
"#;
        let doc = toml.parse::<toml_edit::Document>().unwrap();
        let plug = Plugin::from_toml(&doc["plugin"].as_array_of_tables().unwrap().get(0).unwrap()).unwrap();
        assert_eq!(plug, Plugin { 
            summary: None,
            root: None,
            details: Some(String::from("more info")),
            alias: String::from("ghdl"), 
            command: String::from("python"), 
            args: vec![
                "orbit-ghdl.py".to_string()
            ],
            filesets: vec![
                Fileset::new().name("py-model").pattern("*_mdl.py").unwrap(),
            ],
        });
    }
}