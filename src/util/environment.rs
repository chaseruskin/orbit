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

use crate::core::config::Config;
use crate::core::manifest::PROJECT_MANIFEST_FILE;
use crate::core::swap::StrSwapTable;
use crate::util::anyerror::Fault;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::io::Write;

use crate::core::project::Project;
use crate::util::filesystem::Standardize;
use std::collections::btree_set::IntoIter;
use std::collections::btree_set::Iter;
use std::path::PathBuf;

use serde_derive::Serialize;

use std::collections::btree_set::BTreeSet;

#[derive(Eq, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EnvVar {
    #[serde(skip_serializing)]
    key: String,
    value: String,
    force: bool,
    relative: bool,
}

impl PartialEq for EnvVar {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Ord for EnvVar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for EnvVar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Hash for EnvVar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // only hash by the key name
        self.key.hash(state);
    }
}

use crate::util::filesystem::into_std_str;

impl EnvVar {
    pub fn with(key: &str, value: &str) -> Self {
        Self::new().key(key).value(value)
    }

    pub fn new() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            relative: false,
            force: false,
        }
    }

    /// Sets the environment key.
    pub fn key(mut self, s: &str) -> Self {
        self.set_key(s);
        self
    }

    /// Sets the environment value.
    pub fn value(mut self, s: &str) -> Self {
        self.value = s.to_owned();
        self
    }

    /// Sets the environment force option.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Sets the environment relative option.
    pub fn relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Returns `true` if the environment variable will be set with the value
    /// it is holding.
    ///
    /// If `force` is false and the variable already exists, then this function will
    /// return `false`.
    pub fn can_apply(&self) -> bool {
        std::env::var(&self.key).is_err_and(|x| x == std::env::VarError::NotPresent)
            || self.force == true
    }

    /// Tries to inherit a value from the external environment.
    pub fn try_inherit(&mut self) {
        if self.force == false && std::env::var(&self.key).is_ok() {
            self.value = std::env::var(&self.key).unwrap().clone();
        }
    }

    /// Transforms the string format into a orbit variable format.
    ///
    /// The rules are that the key's '_' become '.' and all letters become lowercase.
    /// The value is left unmodified.
    pub fn to_variable(&self) -> (String, String) {
        (
            self.key.replace("_", ".").to_lowercase(),
            self.value.to_owned(),
        )
    }

    pub fn set_key(&mut self, s: &str) {
        // normalize the key name upon entry
        self.key = s.to_ascii_uppercase().replace('-', "_").replace('.', "_");
    }

    /// Will attempt to resolve a relative path if the environment variable is configured to do such a thing.
    pub fn resolve_relative(&mut self, root: &PathBuf) {
        if self.relative == true {
            let p = PathBuf::from(&self.value);
            if p.is_relative() == true {
                self.value = into_std_str(root.join(&p));
            }
        }
    }
}

impl std::fmt::Debug for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.key, self.value)
    }
}

impl std::fmt::Display for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use std::fmt;

impl<'de> serde::Deserialize<'de> for EnvVar {
    fn deserialize<D>(deserializer: D) -> Result<EnvVar, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        enum Field {
            Value,
            Force,
            Relative,
        }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`value` or `force` or `relative`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "value" => Ok(Field::Value),
                            "force" => Ok(Field::Force),
                            "relative" => Ok(Field::Relative),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LayerVisitor;

        impl<'de> Visitor<'de> for LayerVisitor {
            type Value = EnvVar;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<EnvVar, E>
            where
                E: de::Error,
            {
                Ok(EnvVar::new().value(value))
            }

            fn visit_map<V>(self, mut map: V) -> Result<EnvVar, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut value: Option<String> = None;
                let mut force: Option<bool> = None;
                let mut relative: Option<bool> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                        Field::Force => {
                            if force.is_some() {
                                return Err(de::Error::duplicate_field("force"));
                            }
                            force = Some(map.next_value()?);
                        }
                        Field::Relative => {
                            if relative.is_some() {
                                return Err(de::Error::duplicate_field("relative"));
                            }
                            relative = Some(map.next_value()?);
                        }
                    }
                }
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let force = force.unwrap_or(false);
                let relative = relative.unwrap_or(false);
                Ok(EnvVar::new().value(&value).force(force).relative(relative))
            }
        }

        const FIELDS: &[&str] = &["value", "force", "relative"];
        deserializer.deserialize_struct("EnvVar", FIELDS, LayerVisitor)
    }
}

impl Environment {
    /// Sets environment variables from a '.env' file living at `root`.
    ///
    /// Silently skips text lines that do not have proper delimiter `=` between key and value.
    /// This function will not add any environment variables if the file does not exist.
    pub fn from_env_file(mut self, root: &std::path::PathBuf) -> Result<Self, Fault> {
        // read the .env file
        let env_file = root.join(DOT_ENV_FILE);
        if env_file.exists() == true {
            let mut file = std::fs::File::open(env_file).expect("failed to open .env file");
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("failed to read contents");
            // transform into environment variables
            for line in contents.split_terminator('\n') {
                let result = line.split_once('=');
                // set env variables
                if let Some((name, value)) = result {
                    self = self.overwrite(EnvVar::new().key(name).value(value));
                }
            }
        }
        Ok(self)
    }

    /// Sets environment variables from a [StrSwapTable].
    pub fn from_var_table(mut self, table: &StrSwapTable) -> Result<Self, Fault> {
        for (k, v) in table.inner().iter() {
            self = self.add(EnvVar::new().key(k).value(v));
        }
        Ok(self)
    }

    pub fn into_map(&self) -> HashMap<&String, &String> {
        self.0.iter().map(|v| (&v.key, &v.value)).collect()
    }

    /// Loads environment variables from a target [Project].
    pub fn from_project(mut self, prj: &Project) -> Result<Self, Fault> {
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_PROJECT_NAME)
                .value(&prj.get_man().get_project().get_name().to_string()),
        );
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_PROJECT_DIR)
                .value(PathBuf::standardize(&prj.get_root()).to_str().unwrap()),
        );
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_PROJECT_UUID)
                .value(&prj.get_uuid().to_string()),
        );
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_PROJECT_VERSION)
                .value(&prj.get_man().get_project().get_version().to_string()),
        );
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_PROJECT_LIBRARY)
                .value(&prj.get_hdl_library().to_string()),
        );
        self = self.overwrite(
            EnvVar::new()
                .key(ORBIT_MANIFEST_DIR)
                .value(PathBuf::standardize(&prj.get_root()).to_str().unwrap()),
        );
        self = self.overwrite(
            EnvVar::new().key(ORBIT_MANIFEST_FILE).value(
                PathBuf::standardize(&prj.get_root().join(PROJECT_MANIFEST_FILE))
                    .to_str()
                    .unwrap(),
            ),
        );
        if let Some(sum) = prj.get_checksum() {
            self = self.overwrite(
                EnvVar::new()
                    .key(ORBIT_PROJECT_CHECKSUM)
                    .value(&sum.to_string()),
            );
        }
        Ok(self)
    }

    /// Loads an `Environment` struct from a `Config` document.
    ///
    /// It searches the `[env]` table and collects all env variables.
    pub fn from_config(mut self, config: &Config) -> Result<Self, Fault> {
        // read config.toml for setting any env variables
        if let Some(map) = config.get_env() {
            // ? omit variables that are not set to force and already have the env var set in the environment
            // .filter(|x| x.can_apply())
            map.values().for_each(|var| {
                let mut var = var.clone();
                // Ensure this value is the one to apply
                if var.can_apply() {
                    self.enforce(var);
                // Try to load a value from the environment, otherwise set the value it found in the config
                } else {
                    var.try_inherit();
                    self.insert(var);
                }
            });
        }
        Ok(self)
    }

    /// Sets a set of environment variables, consuming the list.
    pub fn initialize(self) -> () {
        self.into_iter()
            .for_each(|e| unsafe { std::env::set_var(e.key, e.value) });
    }

    pub fn read(key: &str) -> Option<String> {
        match std::env::var(key) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Stores a list of `EnvVar` at root in a file named ".env".
pub fn save_environment(env: &Environment, root: &std::path::PathBuf) -> Result<(), Fault> {
    // create the file
    let mut env_file =
        std::fs::File::create(&root.join(".env")).expect("could not create .env file");
    // prepare the data into a single string for writing
    let contents = env
        .iter()
        .fold(String::new(), |x, y| x + &y.to_string() + &"\n");
    // write the data to the file
    env_file
        .write_all(contents.as_bytes())
        .expect("failed to write data to .env file");
    Ok(())
}

pub struct Environment(BTreeSet<EnvVar>);

impl Environment {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, var: EnvVar) -> bool {
        self.0.insert(var)
    }

    pub fn add(mut self, var: EnvVar) -> Self {
        self.0.insert(var);
        self
    }

    pub fn enforce(&mut self, var: EnvVar) -> () {
        self.0.remove(&var);
        self.0.insert(var);
    }

    pub fn overwrite(mut self, var: EnvVar) -> Self {
        match self.0.contains(&var) {
            true => self.0.remove(&var),
            false => true,
        };
        self.0.insert(var);
        self
    }

    pub fn iter(&self) -> Iter<'_, EnvVar> {
        self.0.iter()
    }

    pub fn into_iter(self) -> IntoIter<EnvVar> {
        self.0.into_iter()
    }

    pub fn from_vec(vec: Vec<EnvVar>) -> Self {
        let mut inner = BTreeSet::new();
        vec.into_iter().for_each(|e| {
            inner.insert(e);
            ()
        });
        Self(inner)
    }

    pub fn get(&self, key: &str) -> Option<&EnvVar> {
        self.0.get(&EnvVar::new().key(key))
    }
}

pub const DOT_ENV_FILE: &str = ".env";

pub const ORBIT_HOME: &str = "ORBIT_HOME";
pub const NO_COLOR: &str = "NO_COLOR";
pub const BROWSER: &str = "BROWSER";
pub const ORBIT_WIN_LITERAL_CMD: &str = "ORBIT_WIN_LITERAL_CMD";

pub const ORBIT: &str = "ORBIT";

pub const ORBIT_MANIFEST_DIR: &str = "ORBIT_MANIFEST_DIR";
pub const ORBIT_MANIFEST_FILE: &str = "ORBIT_MANIFEST_FILE";
pub const ORBIT_PROJECT_DIR: &str = "ORBIT_PROJECT_DIR";
pub const ORBIT_PROJECT_NAME: &str = "ORBIT_PROJECT_NAME";
pub const ORBIT_PROJECT_UUID: &str = "ORBIT_PROJECT_UUID";
pub const ORBIT_PROJECT_VERSION: &str = "ORBIT_PROJECT_VERSION";
pub const ORBIT_PROJECT_LIBRARY: &str = "ORBIT_PROJECT_LIBRARY";
pub const ORBIT_PROJECT_CHECKSUM: &str = "ORBIT_PROJECT_CHECKSUM";

pub const ORBIT_PROTOCOL: &str = "ORBIT_PROTOCOL";
pub const ORBIT_TARGET: &str = "ORBIT_TARGET";

pub const ORBIT_TOP_NAME: &str = "ORBIT_TOP_NAME";
pub const ORBIT_TOP_FILE: &str = "ORBIT_TOP_FILE";

pub const ORBIT_TB_NAME: &str = "ORBIT_TB_NAME";
pub const ORBIT_TB_FILE: &str = "ORBIT_TB_FILE";

pub const ORBIT_DUT_NAME: &str = "ORBIT_DUT_NAME";
pub const ORBIT_DUT_FILE: &str = "ORBIT_DUT_FILE";

pub const ORBIT_BLUEPRINT: &str = "ORBIT_BLUEPRINT";
pub const ORBIT_BLUEPRINT_PLAN: &str = "ORBIT_BLUEPRINT_PLAN";

pub const ORBIT_TARGET_DIR: &str = "ORBIT_TARGET_DIR";
pub const ORBIT_OUT_DIR: &str = "ORBIT_OUT_DIR";

pub const ORBIT_CHANNEL_NAME: &str = "ORBIT_CHANNEL_NAME";
pub const ORBIT_CHANNEL_DIR: &str = "ORBIT_CHANNEL_DIR";
pub const ORBIT_CHANNEL_PROJECT_DIR: &str = "ORBIT_CHANNEL_PROJECT_DIR";
