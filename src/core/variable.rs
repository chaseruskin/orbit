use std::collections::HashMap;

use crate::util::{anyerror::Fault, environment::Environment};

use super::{context::Context, pkgid::PkgId};

pub struct VariableTable(HashMap<String, String>);

impl VariableTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn load_environment(mut self, env: &Environment) -> Result<Self, Fault> {
        for entry in env.iter() {
            let (key, value) = entry.to_variable();
            self.0.insert(key, value);
        }
        Ok(self)
    }

    pub fn load_context(self, _c: &Context) -> Result<Self, Fault> {
        // user
        // self.0.insert("orbit.user".to_owned(), c.get_config()
        //     .get_as_str("core", "user")?
        //     .unwrap_or("")
        //     .to_string());
        // date
        // self.0.insert("orbit.date".to_owned(), format!("{}", {
        //     let dt = chrono::offset::Local::now();
        //     let fmt: &str = c.get_config().get_as_str("core", "date-fmt")?.unwrap_or("%Y-%m-%d");
        //     dt.format(fmt).to_string()
        // }));
        // load all env variables

        Ok(self)
    }

    pub fn load_pkgid(mut self, pkgid: &PkgId) -> Result<Self, Fault> {
        self.add("orbit.ip.name", pkgid.get_name().as_ref());
        self.add(
            "orbit.ip.library",
            pkgid.get_library().as_ref().unwrap().as_ref(),
        );
        self.add(
            "orbit.ip.vendor",
            pkgid.get_vendor().as_ref().unwrap().as_ref(),
        );
        self.add("orbit.ip", &pkgid.to_string());
        Ok(self)
    }

    pub fn add(&mut self, key: &str, value: &str) -> Option<String> {
        self.0.insert(key.to_string(), value.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}
