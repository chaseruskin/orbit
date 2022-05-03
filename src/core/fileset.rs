#[derive(Debug, PartialEq)]
pub struct Fileset {
    name: String,
    pattern: glob::Pattern,
}

#[derive(Debug)]
pub enum FilesetError {
    MissingSeparator(char),
    PatternError(String, glob::PatternError),
}

impl std::error::Error for FilesetError {}

impl std::fmt::Display for FilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::MissingSeparator(c) => write!(f, "missing separator '{}'", c),
            Self::PatternError(p, e) => write!(f, "'{}' {}", p, e.to_string().to_lowercase()),
        }
    }
}

impl std::str::FromStr for Fileset {
    type Err = FilesetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split by '=' sign (or ':'?)
        let result = s.split_once('=');
        if result == None {
            return Err(Self::Err::MissingSeparator('='));
        }
        let (name, pattern) = result.unwrap();
        Ok(Fileset {
            pattern: match glob::Pattern::new(pattern) {
                Ok(p) => p,
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

/// Recursively walks the current directory and ignores files defined in a .gitignore file.
/// 
/// Returns the resulting list of filepath strings. This function silently skips result errors
/// while walking.
pub fn gather_current_files() -> Vec<String> {
    Walk::new(std::env::current_dir().unwrap()).filter_map(|result| {
        match result {
            Ok(entry) => {
                if entry.path().is_file() {
                    Some(entry.into_path().display().to_string())
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }).collect()
}

#[derive(Debug, PartialEq)]
pub struct BuildFile {
    path: std::path::PathBuf,
}

#[cfg(test)]
mod test {
    use super::*;

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

use ignore::Walk;