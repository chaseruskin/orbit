use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Fileset {
    name: String,
    pattern: glob::Pattern,
}

#[derive(Debug)]
pub enum FilesetError {
    MissingSeparator(char),
    EmptyPattern,
    EmptyName,
    PatternError(String, glob::PatternError),
}

impl std::error::Error for FilesetError {}

impl std::fmt::Display for FilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::EmptyPattern => write!(f, "empty pattern"),
            Self::EmptyName => write!(f, "empty name"),
            Self::MissingSeparator(c) => write!(f, "missing separator '{}'", c),
            Self::PatternError(p, e) => write!(f, "'{}' {}", p, e.to_string().to_lowercase()),
        }
    }
}

impl FromStr for Fileset {
    type Err = FilesetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split by '=' sign (or ':'?)
        let result = s.split_once('=');
        if result == None {
            return Err(Self::Err::MissingSeparator('='));
        }
        let (name, pattern) = result.unwrap();
        // name cannot be empty
        if name.is_empty() {
            return Err(Self::Err::EmptyName)
        }
        Ok(Fileset {
            pattern: match glob::Pattern::new(pattern) {
                // pattern must not be empty
                Ok(p) => if p.as_str().is_empty() {
                    return Err(Self::Err::EmptyPattern)
                } else {
                    p
                },
                Err(e) => return Err(Self::Err::PatternError(pattern.to_string(), e))
            },
            name: Self::standardize_name(name),
        })
    }
}

impl Fileset {
    /// Create a new `Fileset` structure.
    pub fn new() -> Self {
        Fileset {
            name: String::new(),
            pattern: glob::Pattern::new("*").unwrap(),
        }
    }

    /// Set the `Fileset` name.
    pub fn name(mut self, s: &str) -> Self {
        self.name = Self::standardize_name(s);
        self
    }

    /// Set the `Fileset` glob-style pattern.
    pub fn pattern(mut self, p: &str) -> Result<Self, glob::PatternError>{
        self.pattern = glob::Pattern::new(&("**/".to_owned() + p))?;
        Ok(self)
    }

    /// Standardizes the name to be UPPER-AND-HYPHENS.
    /// 
    /// The returned string is its own data (cloned from `s`).
    fn standardize_name(s: &str) -> String {
        s.to_uppercase().replace('_', "-")
    }

    /// Uses the given pattern to return a set of build files.
    pub fn collect_files<'a>(&self, files: &'a [String]) -> Vec<&'a String> {
        let match_opts = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        files.iter().filter_map(|f| {
            if self.pattern.matches_with(&f, match_opts) == true {
                Some(f)
            } else {
                None
            }
        }).collect()
    }

    /// Access name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Access pattern.
    pub fn get_pattern(&self) -> &glob::Pattern {
        &self.pattern
    }

    /// Loads a fileset from a toml key/value pair
    pub fn from_toml(key: toml_edit::Key, val: toml_edit::Value) -> Self {
        Self {
            name: key.to_string(),
            pattern: glob::Pattern::new(val.as_str().unwrap()).unwrap(),
        }
    }

    /// Creates format for blueprint.tsv file.
    /// 
    /// The format goes FILESET_NAME\tFILE_NAME\tFILE_PATH
    pub fn to_blueprint_string(&self, file: &str) -> String {
        let filename = {
            // removes root path
            let filenode = if let Some((_, node)) = file.rsplit_once('/') {
                node
            } else {
                file
            };
            // removes file extenstion
            match filenode.rsplit_once('.') {
                Some((name, _)) => name,
                None => filenode
            }
        };
        format!("{}\t{}\t{}\n", self.name, filename, file)
    }
}

/// Finds the VHDL files for the proper fileset among the list of available `files`.
/// 
/// Uses glob pattern matching to filter files.
/// __Note__: This function is temporary workaround for avoiding trying to
/// determine in-order dependency lists and files for current VHDL toplevel.
pub fn collect_vhdl_files(files: &[String], is_sim: bool) -> Vec<&String> {
    let match_opts = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let p1 = glob::Pattern::new("*.vhd").unwrap();
    let p2 = glob::Pattern::new("*.vhdl").unwrap();

    let tb1 = glob::Pattern::new("tb_*").unwrap();
    let tb2 = glob::Pattern::new("*_tb.*").unwrap();

    files.iter().filter_map(|f| {
        if (p1.matches_with(f, match_opts) == true || p2.matches_with(f, match_opts) == true) &&
            ((is_sim == false && tb1.matches_with(f, match_opts) == false && tb2.matches_with(f, match_opts) == false) ||
            (is_sim == true && (tb1.matches_with(f, match_opts) == true || tb2.matches_with(f, match_opts) == true))) {
            Some(f)
        } else {
            None
        }
    }).collect()
}

/// Checks if the `file` is a VHDL file (ending with .vhd or .vhdl).
pub fn is_vhdl(file: &str) -> bool {
    if let Some((_, ending)) = file.rsplit_once('.') {
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhd") ||
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhdl")
    } else {
        false
    }
}

/// Checks against file patterns if the file is an rtl file.
pub fn is_rtl(file: &str) -> bool {
    let match_opts = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let p1 = glob::Pattern::new("*.vhd").unwrap();
    let p2 = glob::Pattern::new("*.vhdl").unwrap();

    let tb1 = glob::Pattern::new("tb_*").unwrap();
    let tb2 = glob::Pattern::new("*_tb.*").unwrap();

    (p1.matches_with(file, match_opts) == true || p2.matches_with(file, match_opts) == true) && 
        tb1.matches_with(file, match_opts) == false && tb2.matches_with(file, match_opts) == false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_blueprint_string() {
        let fset = Fileset::new().name("vhdl").pattern("*.sv").unwrap();
        let filepath = "c:/users/chase/develop/project/adder.sv";
        assert_eq!(fset.to_blueprint_string(&filepath), format!("VHDL\tadder\t{}\n", filepath));

        let filepath = "FILE2.sv";
        assert_eq!(fset.to_blueprint_string(&filepath), format!("VHDL\tFILE2\t{}\n", filepath));
    }

    #[test]
    fn detect_vhdl_files() {
        let s = "filename.vhd";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.VHD";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.VHdL";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.vhdl";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.v";
        assert_eq!(is_vhdl(s), false);

        let s = "filename";
        assert_eq!(is_vhdl(s), false);

        let s = "filename.sv";
        assert_eq!(is_vhdl(s), false);
    }

    #[test]
    fn fset_from_str() {
        let s = "xsim-cfg=*.wcfg";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.unwrap(), Fileset {
            name: "XSIM-CFG".to_string(),
            pattern: glob::Pattern::new("*.wcfg").unwrap()
        });

        let s = "xsim-cfg=";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // empty pattern

        let s = "=*.txt";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // empty name

        let s = "xsim-cfg";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // missing separator

        let s = "xsim-cfg=[";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // pattern error
    }

    #[test]
    fn std_name() {
        let s: &str = "VHDL-RTL";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");

        let s: &str = "vhdl-rtl";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");

        let s: &str = "vhdl_rtl";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");
    }

    #[test]
    fn find_vhdl_rtl() {
        let files: Vec<String> = vec![
            "File.vhdl",
            "file2.vhdl",
            "file3.VHDL",
            "file4.vhd",
            "file.txt",
            "tb_file.vhd",
            "TB_file.vhdl",
            "file.vh",
            "file_tb.vhdl",
            "file_Tb.vhd",
            "other_tb_.vhd",
        ].iter().map(|f| f.to_string()).collect();
        let result = collect_vhdl_files(&files, false);
        assert_eq!(result, vec![
            "File.vhdl",
            "file2.vhdl",
            "file3.VHDL",
            "file4.vhd",
            "other_tb_.vhd",
        ]);
    }

    #[test]
    fn find_vhdl_sim() {
        let files: Vec<String> = vec![
            "File.vhdl",
            "file2.vhdl",
            "file3.VHDL",
            "file4.vhd",
            "file.txt",
            "tb_file.vhd",
            "TB_file.vhdl",
            "file.vh",
            "file_tb.VhdL",
            "file_Tb.vhd",
            "other_tb_.vhd",
        ].iter().map(|f| f.to_string()).collect();
        let result = collect_vhdl_files(&files, true);
        assert_eq!(result, vec![
            "tb_file.vhd", 
            "TB_file.vhdl", 
            "file_tb.VhdL", 
            "file_Tb.vhd",
        ]);
    }
}