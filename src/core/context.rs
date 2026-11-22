//
//  Copyright (C) 2022-2026  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use crate::core::config::General;
use crate::core::config::{Config, Configs, Locality};
use crate::core::manifest::Toolchain;
use crate::core::target::Target;
use crate::error::{Error, Hint};
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::environment::{self, Environment, ORBIT_WIN_LITERAL_CMD};
use crate::util::filesystem::Standardize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;
use std::path::PathBuf;

use super::lang::Language;
use super::lang::sv::format::SystemVerilogFormat;
use super::lang::vhdl::format::VhdlFormat;

pub const CACHE_TAG_FILE: &str = "CACHEDIR.TAG";

pub const ORBIT_HIDDEN_DIR: &str = ".orbit";

pub const CACHE_TAG: &str = "\
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by orbit.
# For information about cache directory tags see https://bford.info/cachedir/
";

/// Shared attributes about the surrounding user run-time environment.

/// The outcome of resolving which target a verb should run.
#[derive(Debug, PartialEq)]
enum TargetResolution {
    /// a tier explicitly named this target
    Named(String),
    /// no tier named anything, but exactly one candidate exists
    Inferred(String),
    /// nothing to go on
    Missing,
}

/// Resolves the target name for a verb through the tier chain:
/// CLI `--target` > `ORBIT_TARGET` env > manifest `[toolchain]` per-verb
/// key > manifest `default` > config `[build]`/`[test].target` >
/// single-candidate inference.
///
/// The manifest tier sits above the config default because it is
/// in-repo: a project declares "test with nvc, synth with vivado" once.
fn resolve_target_name(
    cli: &Option<String>,
    env: Option<String>,
    config_default: Option<&String>,
    toolchain: Option<&crate::core::manifest::Toolchain>,
    is_build: bool,
    single_candidate: Option<&str>,
) -> TargetResolution {
    if let Some(c) = cli {
        return TargetResolution::Named(c.clone());
    }
    if let Some(e) = env {
        return TargetResolution::Named(e);
    }
    if let Some(tc) = toolchain {
        // per-verb binding: one target owns the whole verb
        let verb = if is_build {
            tc.get_build()
        } else {
            tc.get_test()
        };
        if let Some(v) = verb {
            return TargetResolution::Named(v.clone());
        }
        if let Some(d) = tc.get_default() {
            return TargetResolution::Named(d.clone());
        }
    }
    if let Some(cd) = config_default {
        return TargetResolution::Named(cd.clone());
    }
    if let Some(sc) = single_candidate {
        return TargetResolution::Inferred(String::from(sc));
    }
    TargetResolution::Missing
}

pub struct Context {

    /// File system path directing to root of orbit data and configurations.
    home_path: PathBuf,
    /// File system path directing to project installations
    cache_path: PathBuf,
    /// File system path directing to project downloads
    archive_path: PathBuf,
    /// The parent path to the current project `Orbit.toml` manifest file.
    project_path: Option<PathBuf>,
    /// Directory name for the intermediate build processes and outputs.    
    build_dir: String,
    /// Language support mode.
    languages: Language,
    /// Flattened view of the current configuration settings.
    config: Config,
    /// Entire list of configuration settings.
    all_configs: Configs,
    // @idea: optionally move hashmap out of context and create it from fn to allow dynamic loading
    plugins: HashMap<String, Target>,
}

impl Context {
    pub fn new() -> Context {
        let home = std::env::temp_dir();
        let cache = home.join("cache");
        let downloads = home.join("archive");
        Context {
            home_path: home,
            cache_path: cache,
            archive_path: downloads,
            project_path: None,
            plugins: HashMap::new(),
            all_configs: Configs::new(),
            config: Config::new(),
            build_dir: String::new(),
            languages: Language::default(),
        }
    }

    /// Sets the home directory. By default this is `$HOME/.orbit`. If set by `var`,
    /// it must be an existing directory.
    pub fn home(mut self, key: &str) -> Result<Context, Error> {
        self.home_path = if let Ok(s) = env::var(key) {
            std::path::PathBuf::from(s)
        } else {
            let hp = match home::home_dir() {
                Some(p) => p.join(".orbit"),
                None => return Err(Error::FailedToGetHomeDir),
            };
            // create the directory if does not exist
            if path::Path::exists(&hp) == false {
                std::fs::create_dir(&hp).expect("failed to create .orbit directory");
            }
            hp
        };
        // do not allow a non-existent directory to be set for the home
        if path::Path::exists(&self.home_path) == false {
            return Err(Error::OrbitHomeDoesNotExist(self.home_path.clone()));
        }
        // verify the environment variable is set
        unsafe {
            env::set_var(key, &self.home_path);
        }
        Ok(self)
    }

    /// Sets the cache directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn cache(mut self) -> Result<Context, Fault> {
        self.cache_path = self.folder(None, "cache")?;
        // create a cache tag file if does not exist
        match Self::is_cache_tag_valid(&self.cache_path) {
            Ok(_) => (),
            Err(e) => fs::write(&e, CACHE_TAG)?,
        }
        Ok(self)
    }

    /// Sets the downloads directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn archive(mut self) -> Result<Context, Fault> {
        self.archive_path = self.folder(None, "archive")?;
        // create a cache tag file if does not exist
        match Self::is_cache_tag_valid(&self.archive_path) {
            Ok(_) => (),
            Err(e) => fs::write(&e, CACHE_TAG)?,
        }
        Ok(self)
    }

    /// Checks if the cache tag file is properly configured in the set cache directory.
    ///
    /// Returns an `Err` holding the path to the needed cache file if the path was
    /// not a file or did not exactly contain the content's string.
    pub fn is_cache_tag_valid(dir: &PathBuf) -> Result<(), PathBuf> {
        let tag = dir.join(CACHE_TAG_FILE);
        match tag.is_file() {
            false => Err(tag),
            true => match fs::read_to_string(&tag) {
                Err(_) => Err(tag),
                Ok(text) => match text == CACHE_TAG {
                    false => Err(tag),
                    true => Ok(()),
                },
            },
        }
    }

    /// Checks if windows literal command is enabled.
    pub fn enable_windows_bat_file_match() -> bool {
        if cfg!(target_os = "windows") {
            // by not finding the env var, the windows batch file match is enabled
            std::env::var(ORBIT_WIN_LITERAL_CMD).is_err()
        } else {
            false
        }
    }

    /// Returns an existing filesystem path to be used under `key`.
    ///
    /// Uses `key`'s value if already explicitly set and will set the environment
    /// variable accordingly.
    fn folder(&self, key: Option<&str>, folder: &str) -> Result<PathBuf, Fault> {
        // prioritize explicit variable setting
        let dir = if key.is_some() && env::var(key.unwrap()).is_ok() {
            let ep = PathBuf::from(env::var(key.unwrap()).unwrap());
            // verify the path exists
            if ep.exists() == false {
                return Err(AnyError(format!(
                    "directory {} does not exist for {}",
                    ep.display(),
                    key.unwrap()
                )))?;
            }
            // verify the path is a directory
            if ep.is_dir() == false {
                return Err(AnyError(format!(
                    "{} must be a filesystem directory",
                    key.unwrap()
                )))?;
            }
            ep
        // proceed with default
        } else {
            let ep = self.home_path.join(&folder);
            // create the directory if does not exist
            if ep.exists() == false {
                std::fs::create_dir(&ep).expect(&format!(
                    "failed to create {}/{} directory",
                    ORBIT_HIDDEN_DIR, folder
                ));
            }
            ep
        };
        // set the environment variable
        if let Some(k) = key {
            unsafe {
                env::set_var(k, &PathBuf::standardize(&dir));
            }
        }
        Ok(dir)
    }

    /// References the cache directory.
    pub fn get_cache_path(&self) -> &PathBuf {
        &self.cache_path
    }

    /// References the downloads directory
    pub fn get_downloads_path(&self) -> &PathBuf {
        &self.archive_path
    }

    fn collect_configs(&self, global_path: &PathBuf, name: &str) -> Result<Configs, Fault> {
        // initialize and load the global configuration
        let mut configs = Configs::new().load(global_path.clone(), Locality::Global)?;

        // current working directory and its parent directories
        let mut work_dirs = vec![std::env::current_dir()?];
        while let Some(p) = work_dirs.last().unwrap().parent() {
            work_dirs.push(p.to_path_buf());
        }
        work_dirs.reverse();
        for upper_path in work_dirs {
            let parent_path = upper_path.join(ORBIT_HIDDEN_DIR).join(name);
            if parent_path.exists() == true {
                let locality = match self.get_project_path() {
                    Some(prj_path) => match &parent_path == prj_path {
                        true => Locality::Local,
                        false => Locality::Regional,
                    },
                    None => Locality::Regional,
                };
                // skip global path
                if &parent_path == global_path {
                    continue;
                }
                configs = configs.load(parent_path, locality)?;
            }
        }

        Ok(configs)
    }

    /// Configures and reads data from the settings object to return a `Settings` struct
    /// in the `Context`.
    ///
    /// The settings file `name` must be directly under `$ORBIT_HOME`. It also
    /// checks for a local configuration as `name` under a .orbit/ directory if
    /// the command is invoked from within a project directory.
    ///
    /// Note: the `self.project_path` must already be determined before invocation.
    pub fn settings(mut self, name: &str) -> Result<Context, Fault> {
        // check if global file exists first
        let global_path = self.home_path.join(name);
        if global_path.exists() == false {
            std::fs::write(&global_path, Vec::new())?;
        }

        // produce the layered variant
        self.all_configs = self.collect_configs(&global_path, name)?;

        // produce the flatten variant
        self.config = self.collect_configs(&global_path, name)?.into();

        // @todo: dynamically set from environment variables from configuration data
        // 2025-04-05: is this still needed?
        Ok(self)
    }

    /// Access the compiled configuration data after resolving all precedence issues.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_all_configs(&self) -> &Configs {
        &self.all_configs
    }

    /// Access the configuration data as mutable.
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Access the default protocol.
    pub fn get_default_protocol(&self) -> Option<&String> {
        self.config.get_install()?.get_default_protocol()
    }

    /// Access the build directory data.
    pub fn get_target_dir(&self) -> String {
        match self.config.get_general() {
            Some(g) => g.get_build_dir(),
            None => General::new().get_build_dir(),
        }
    }

    /// Checks if the design units should be private by default when `public`
    /// field is omitted from the local project.
    pub fn are_units_private_by_default(&self) -> bool {
        match self.config.get_general() {
            Some(g) => g.is_public_required(),
            None => General::new().is_public_required(),
        }
    }

    /// Access the project directory detected from the current working directory.
    pub fn get_project_path(&self) -> Option<&path::PathBuf> {
        self.project_path.as_ref()
    }

    /// Access the home path.
    pub fn get_home_path(&self) -> &path::PathBuf {
        &self.home_path
    }

    /// Determines if the directory is within a current project and sets the proper
    /// runtime environment variable.
    pub fn current_project_dir(mut self, s: &str) -> Result<Context, Error> {
        self.project_path = match Context::find_project_path(match &std::env::current_dir() {
            Ok(r) => r,
            Err(_) => return Err(Error::FailedToGetCurDir),
        }) {
            Some(cwd) => {
                unsafe {
                    env::set_var(s, &cwd);
                }
                Some(cwd)
            }
            None => None,
        };
        Ok(self)
    }

    /// Changes current working directory to the detected project path.
    ///
    /// Returns an error if project_path is `None`.
    pub fn jump_to_working_project(&self) -> Result<(), Error> {
        match self.get_project_path() {
            Some(cwd) => {
                // set the current working directory to here
                std::env::set_current_dir(&cwd).expect("could not change directories");
            }
            None => {
                // @IDEA also give information about reading about project-dir sensitive commands as a topic?
                return Err(Error::NoWorkingProjectFound);
            }
        }
        Ok(())
    }

    /// Finds the complete path to the current project's directory.
    ///
    /// This function will recursively backtrack down the current working directory
    /// until finding the first directory with a file named "Orbit.toml".
    pub fn find_project_path(dir: &std::path::PathBuf) -> Option<path::PathBuf> {
        Self::find_target_path(dir, "Orbit.toml")
    }

    /// Finds the complete path to the current directory that hosts the `target_file`.
    ///
    /// This function recursively backtracks from `dir` into its ancestors until
    /// finding the first directory with a file named `target_file`.
    ///
    /// This function has no assumptions on if the directory is readable or not (bypasses read_dir errors).
    pub fn find_target_path(dir: &std::path::PathBuf, target_file: &str) -> Option<path::PathBuf> {
        let mut cur = dir.clone();
        // search for the manifest file
        loop {
            match std::fs::read_dir(&cur) {
                // the directory was able to be read (it exists)
                Ok(mut entries) => {
                    let result = entries.find_map(|p| match p {
                        Ok(file) => {
                            if file.file_name() == target_file {
                                Some(cur.to_path_buf())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });
                    if let Some(r) = result {
                        break Some(r);
                    }
                }
                // failed to read the directory
                Err(_) => {}
            }
            if cur.pop() == false {
                break None;
            }
        }
    }

    /// Sets the project's build directory and the corresponding environment variable.
    pub fn build_dir(self, s: &str) -> Result<Context, Error> {
        unsafe {
            env::set_var(s, &self.get_target_dir());
        }
        Ok(self)
    }

    /// If a target is required, then it will try to fetch the default target
    /// defined from the user's configuration (if `target` is also `None`).
    pub fn select_target(
        &self,
        target: &Option<String>,
        required: bool,
        is_build: bool,
    ) -> Result<Option<&Target>, Error> {
        let env_target = Environment::read(environment::ORBIT_TARGET);
        // allow the list to be printed
        if target.is_none() && required == false {
            return Ok(None);
        }
        let filtered = self.get_config().get_targets(is_build);
        let single_candidate = match filtered.len() {
            1 => filtered.values().next().map(|t| t.get_name().to_string()),
            _ => None,
        };
        let resolution = resolve_target_name(
            target,
            env_target,
            self.get_config().get_default_target(is_build),
            self.read_manifest_toolchain().as_ref(),
            is_build,
            single_candidate.as_deref(),
        );
        let name = match resolution {
            TargetResolution::Named(name) => Some(name),
            TargetResolution::Inferred(name) => Some(name),
            TargetResolution::Missing => None,
        };
        match name {
            // verify the target name matches
            Some(name) => match self.get_config().get_targets(is_build).get(name.as_str()) {
                Some(&t) => Ok(Some(t)),
                None => {
                    // does the target exist but declare the other verb only?
                    // a mismatch is a loud face here, never a mid-drive confusion
                    if self.get_config().get_all_targets().contains_key(name.as_str()) {
                        let verb = if is_build { "build" } else { "test" };
                        let hint = if is_build {
                            Hint::TargetsListBuild
                        } else {
                            Hint::TargetsListTest
                        };
                        Err(Error::TargetWrongVerb(name.to_string(), verb, hint))
                    } else {
                        match is_build {
                            true => Err(Error::TargetNotFoundBuild(
                                name.to_string(),
                                Hint::TargetsListBuild,
                            )),
                            false => Err(Error::TargetNotFoundTest(
                                name.to_string(),
                                Hint::TargetsListTest,
                            )),
                        }
                    }
                }
            },
            None => match required {
                true => Err(Error::MissingRequiredTarget),
                false => Ok(None),
            },
        }
    }

    /// Reads the `[toolchain]` face from the current project's manifest,
    /// if the project is discoverable and the section parses.
    ///
    /// This is a light, extract-only read: the document is not fully
    /// validated here (a broken manifest reports honestly when the
    /// project itself loads); only the `toolchain` table is taken.
    fn read_manifest_toolchain(&self) -> Option<crate::core::manifest::Toolchain> {
        #[derive(serde::Deserialize)]
        struct ToolchainOnly {
            #[serde(default)]
            toolchain: Option<crate::core::manifest::Toolchain>,
        }
        let prj_path = self.get_project_path()?;
        let text = std::fs::read_to_string(prj_path.join(crate::core::manifest::PROJECT_MANIFEST_FILE)).ok()?;
        toml::from_str::<ToolchainOnly>(&text).ok()?.toolchain
    }

    pub fn get_vhdl_format(&self) -> VhdlFormat {
        let default_fmt = VhdlFormat::new();
        let vhdl_fmt = match self.get_config().get_vhdl_formatting() {
            Some(v) => v,
            None => &default_fmt,
        };
        vhdl_fmt.clone()
    }

    pub fn get_sv_format(&self) -> SystemVerilogFormat {
        let default_fmt = SystemVerilogFormat::new();
        let sv_fmt = match self.get_config().get_sv_formatting() {
            Some(sv) => sv,
            None => &default_fmt,
        };
        sv_fmt.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HOME: &str = "./tests";

    /// Builds a toolchain face for the resolution tests.
    fn face(default: Option<&str>, build: Option<&str>, test: Option<&str>) -> Option<Toolchain> {
        Some(Toolchain::new(
            default.map(String::from),
            build.map(String::from),
            test.map(String::from),
        ))
    }

    #[test]
    fn resolution_cli_wins_over_every_tier() {
        let tc = face(Some("m-default"), Some("m-build"), Some("m-test"));
        let r = resolve_target_name(
            &Some(String::from("cli")),
            Some(String::from("env")),
            Some(&String::from("config-default")),
            tc.as_ref(),
            true,
            None,
        );
        assert_eq!(r, TargetResolution::Named(String::from("cli")));
    }

    #[test]
    fn resolution_env_sits_above_the_manifest_face() {
        let tc = face(Some("m-default"), Some("m-build"), Some("m-test"));
        let r = resolve_target_name(
            &None,
            Some(String::from("env")),
            Some(&String::from("config-default")),
            tc.as_ref(),
            false,
            None,
        );
        assert_eq!(r, TargetResolution::Named(String::from("env")));
    }

    #[test]
    fn resolution_manifest_binds_per_verb() {
        let tc = face(Some("m-default"), Some("m-build"), Some("m-test"));
        let r = resolve_target_name(&None, None, None, tc.as_ref(), true, None);
        assert_eq!(r, TargetResolution::Named(String::from("m-build")));
        let r = resolve_target_name(&None, None, None, tc.as_ref(), false, None);
        assert_eq!(r, TargetResolution::Named(String::from("m-test")));
    }

    #[test]
    fn resolution_manifest_default_is_the_verb_fallback() {
        let tc = face(Some("m-default"), None, None);
        let r = resolve_target_name(&None, None, None, tc.as_ref(), true, None);
        assert_eq!(r, TargetResolution::Named(String::from("m-default")));
        // the config default only matters when the manifest face is silent
        let r = resolve_target_name(
            &None,
            None,
            Some(&String::from("config-default")),
            tc.as_ref(),
            false,
            None,
        );
        assert_eq!(r, TargetResolution::Named(String::from("m-default")));
    }

    #[test]
    fn resolution_config_default_is_the_last_named_tier() {
        let r = resolve_target_name(
            &None,
            None,
            Some(&String::from("config-default")),
            None,
            true,
            None,
        );
        assert_eq!(r, TargetResolution::Named(String::from("config-default")));
    }

    #[test]
    fn resolution_infers_a_single_candidate() {
        let r = resolve_target_name(&None, None, None, None, true, Some("solo"));
        assert_eq!(r, TargetResolution::Inferred(String::from("solo")));
        // two or more candidates refuse to infer
        let r = resolve_target_name(&None, None, None, None, true, None);
        assert_eq!(r, TargetResolution::Missing);
        // zero candidates refuse too
        let r = resolve_target_name(&None, None, None, None, true, None);
        assert_eq!(r, TargetResolution::Missing);
    }

    #[test]
    fn resolution_missing_when_no_tier_answers() {
        let r = resolve_target_name(&None, None, None, None, true, None);
        assert_eq!(r, TargetResolution::Missing);
    }

    #[test]
    fn find_target_path() {
        // existing path with target at root
        let home = HOME.to_owned();
        let p = Context::find_target_path(&PathBuf::from(home.clone() + "/t5"), "Orbit.toml");
        assert_eq!(p, Some(PathBuf::from(home.clone() + "/t5")));

        // inner path with target a directory back
        let p = Context::find_target_path(&PathBuf::from("./src"), "Cargo.toml");
        assert_eq!(p, Some(PathBuf::from(".")));

        // imaginary path with target a couple directories back
        let p =
            Context::find_target_path(&PathBuf::from(home.clone() + "/t5/rtl/syn/"), "Orbit.toml");
        assert_eq!(p, Some(PathBuf::from(home.clone() + "/t5")));

        // no existing target
        let p = Context::find_target_path(
            &PathBuf::from(home.clone() + "/t5/rtl/syn/"),
            "HIDDEN-TARGET.TXT",
        );
        assert_eq!(p, None);
    }
}
