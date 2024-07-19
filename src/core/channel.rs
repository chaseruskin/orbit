use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::util::filesystem;

pub type Channels = Vec<Channel>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    command: String,
    args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    name: String,
    description: Option<String>,
    /// The directory located where the channel exists.
    path: Option<String>,
    sync: Option<Sequence>,
    pre: Option<Sequence>,
    post: Option<Sequence>,
    /// Run command sequences from this directory and place manifest data here during launch
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl Channel {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self) -> String {
        format!(
            "{:<24}{}",
            self.name,
            self.description.as_ref().unwrap_or(&String::new())
        )
    }

    /// Resolves the root path according to its path.
    pub fn set_root(&mut self, relative_from: PathBuf) {
        let root = filesystem::resolve_rel_path(
            &relative_from,
            self.path.as_ref().unwrap_or(&String::from(".")),
        );
        self.root = Some(root.into());
    }
}

impl Channel {
    /// Creates a string to display a list of channels.
    ///
    /// The string lists the channels in alphabetical order by `alias`.
    pub fn list_channels(chans: &mut [&&Channel]) -> String {
        let mut list = String::new();
        chans.sort_by(|a, b| a.name.cmp(&b.name));
        for c in chans {
            list += &format!("{}\n", c.quick_info());
        }
        list
    }
}
