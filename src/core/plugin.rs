use std::process::Stdio;
use crate::core::fileset::Fileset;

pub trait FromToml {
    type Err;

    fn from_toml(doc: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct Plugin {
    alias: String,
    command: String,
    args: Vec<String>,
    summary: Option<String>,
    filesets: Vec<Fileset>,
}

#[derive(Debug, PartialEq)]
pub enum PluginError {
    MissingAlias,
    MissingCommand,
    UnknownKey(String),
    ArgsNotArray,
}

impl std::error::Error for PluginError {}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAlias => write!(f, "key 'alias' holding a string is required for a plugin"),
            Self::MissingCommand => write!(f, "key 'command' holding a string is required for a plugin"),
            Self::UnknownKey(k) => write!(f, "unknown key '{}' skipped in plugin array of tables", k),
            Self::ArgsNotArray => write!(f, "key 'args' expects an array of strings"),
        }
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
            filesets: Vec::new(),
        }
    }

    /// Runs the given `command` with the set `args` for the plugin.
    pub fn execute(&self, extra_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let mut proc = std::process::Command::new(&self.command)
            .args([&self.args, extra_args].concat())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        let _ = proc.wait()?;
        Ok(())
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
        self.command = resolve_path(&root, self.command).unwrap();
        self.args = self.args.into_iter().map(|f| resolve_path(&root, f).unwrap() ).collect();
        self
    }
}

/// Resolves the path into a full path if given relative to some `root` path.
/// 
/// This function is helpful for resolving full paths in plugin arguments,
/// config.toml includes, and template paths.
fn resolve_path(root: &std::path::PathBuf, s: String) -> Result<String, Box<dyn std::error::Error>> {
    let resolved_path = root.join(&s);
    if std::path::Path::exists(&resolved_path) == true {
        // write out full path
        Ok(resolved_path.display().to_string())
    } else {
        Ok(s)
    }
}

impl FromToml for Plugin {
    type Err = PluginError;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err>
    where Self: Sized {
        Ok(Self {
            alias: if let Some(val) = table.get("alias") {
                val.as_str().unwrap().to_string()
            } else {
                return Err(Self::Err::MissingAlias)
            },
            command: if let Some(val) = table.get("command") {
                val.as_str().unwrap().to_string()
            } else {
                return Err(Self::Err::MissingCommand)
            },
            args: if let Some(args) = table.get("args") {
                if args.is_array() == false {
                    return Err(Self::Err::ArgsNotArray)
                } else {
                    args.as_array().unwrap().into_iter().map(|f| f.as_str().unwrap().to_owned() ).collect()
                } 
            } else {
                Vec::new()
            },
            summary: if let Some(val) = table.get("summary") {
                Some(val.as_str().unwrap().to_string())
            } else { 
                None 
            },
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
    fn resolve_path_simple() {
        let rel_root = std::env::current_dir().unwrap();
        // expands relative path to full path
        assert_eq!(resolve_path(&rel_root, String::from("src/lib.rs")).unwrap(), rel_root.join("src/lib.rs").display().to_string());
        // no file or directory named 'orbit' at the relative root
        assert_eq!(resolve_path(&rel_root, String::from("orbit")).unwrap(), String::from("orbit"));
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