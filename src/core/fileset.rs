use std::str::FromStr;
use ignore::Walk;

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
        self.pattern = glob::Pattern::new(p)?;
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
            if self.pattern.matches_with(f, match_opts) == true {
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

/// Recursively walks the given `path` and ignores files defined in a .gitignore file.
/// 
/// Returns the resulting list of filepath strings. This function silently skips result errors
/// while walking. The collected set of paths are also standardized to use forward slashes '/'.
pub fn gather_current_files(path: &std::path::PathBuf) -> Vec<String> {
    let mut files: Vec<String> = Walk::new(path).filter_map(|result| {
        match result {
            Ok(entry) => {
                if entry.path().is_file() {
                    // replace double backslash \\ with single forward slash /
                    Some(entry.into_path().display().to_string().replace("\\\\", "/"))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }).collect();
    // sort the fileset for reproductibility purposes
    files.sort();
    files
}

#[cfg(test)]
mod test {
    use super::*;

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