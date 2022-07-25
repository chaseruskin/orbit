use std::process::Stdio;
use crate::core::fileset::Fileset;
use crate::core::config::FromToml;

#[derive(Debug, PartialEq)]
pub struct Plugin {
    alias: String,
    command: String,
    args: Vec<String>,
    summary: Option<String>,
    filesets: Vec<Fileset>,
}

impl std::fmt::Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<16}{}", self.alias, self.summary.as_ref().unwrap_or(&String::new()))
    }
}

#[derive(Debug, PartialEq)]
pub enum PluginError {
    ArgsNotArray,
}

impl std::error::Error for PluginError {}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgsNotArray => write!(f, "key 'args' expects an array of strings"),
        }
    }
}

use crate::util::anyerror::{AnyError, Fault};

impl Plugin {
    /// Creates a new `Plugin` struct.
    pub fn new() -> Self {
        Self { 
            alias: String::new(), 
            command: String::new(), 
            args: Vec::new(),
            summary: None,
            filesets: Vec::new(),
        }
    }

    /// Creates a string to display a list of plugins.
    pub fn list_plugins(plugs: &[&Plugin]) -> String {
        let mut list = String::from("Plugins:\n");
        for plug in plugs {
            list += &format!("    {}\n", plug);
        }
        list
    }

    /// Runs the given `command` with the set `args` for the plugin.
    pub fn execute(&self, extra_args: &[String], verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
        let args = [&self.args, extra_args].concat();
        if verbose == true {
            let s = args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " });
            println!("running: {} {}", self.command, s);
        }
        let mut proc = std::process::Command::new(&self.command)
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        let exit_code = proc.wait()?;
        match exit_code.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
            None =>  Err(AnyError(format!("terminated by signal")))?
        }
    }

    /// Accesses the plugin's `alias`.
    pub fn alias(&self) -> &String {
        &self.alias
    }

    /// Accesses the plugin's `filesets`.
    pub fn filesets(&self) -> &Vec<Fileset> {
        &self.filesets
    }

    /// Applies the `resolve_path` fn to the `command` and all `args`.
    /// 
    /// Assumes `root` is the parent directory to the config.toml file that
    /// created this `Plugin` struct.
    pub fn resolve_all_paths(mut self, root: &std::path::PathBuf) -> Self {
        self.command = crate::util::filesystem::resolve_rel_path(&root, self.command);
        self.args = self.args.into_iter()
            .map(|f|  crate::util::filesystem::resolve_rel_path(&root, f) )
            .collect();
        self
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
                    return Err(PluginError::ArgsNotArray)?
                } else {
                    args.as_array().unwrap().into_iter().map(|f| f.as_str().unwrap().to_owned() ).collect()
                }
            } else {
                Vec::new()
            },
            summary: Self::get(table, "summary")?,
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
        // @TODO verify there are no extra keys
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
"#;
        let doc = toml.parse::<toml_edit::Document>().unwrap();
        let plug = Plugin::from_toml(&doc["plugin"].as_array_of_tables().unwrap().get(0).unwrap()).unwrap();
        assert_eq!(plug, Plugin { 
            summary: None,
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