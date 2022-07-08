use crate::Command;
use crate::FromCli;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::store::Store;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::version::Version;
use crate::core::extgit::ExtGit;

#[derive(Debug, PartialEq)]
enum VersionField {
    Major,
    Minor,
    Patch,
    Version(Version),
}

impl std::str::FromStr for VersionField {
    type Err = crate::core::version::VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            _ => Ok(Self::Version(Version::from_str(s)?)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Launch {
    next: Option<VersionField>,
    ready: bool,
    message: Option<String>,
}

impl FromCli for Launch {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Launch {
            ready: cli.check_flag(Flag::new("ready"))?,
            next: cli.check_option(Optional::new("next").value("version"))?,
            message: cli.check_option(Optional::new("message").switch('m'))?,
        });
        command
    }
}

use git2::Repository;
use colored::Colorize;
use crate::core::manifest;
use crate::util::anyerror::AnyError;

/// Retrieves the latest commit in the current repository using git2 API.
/// 
/// Source: https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html
fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

impl Command for Launch {
    type Err = Box<dyn std::error::Error>;

    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // make sure it is run from an ip directory
        c.goto_ip_path()?;
        // verify the current directory is a git repository
        let repo = Repository::open(c.get_ip_path().unwrap())?;

        // verify the repository has at least one commit
        let latest_commit = find_last_commit(&repo)?;

        if self.message.is_some() && self.next.is_none() {
            return Err(CliError::BrokenRule(format!("option '{}' is only allowed when using option {}", "--message".yellow(), "--next".yellow())))?
        }

        // verify the manifest is checked into version control
        if repo.is_path_ignored("Orbit.toml")? {
            return Err(AnyError(format!("manifest 'Orbit.toml' is ignored by version control")))?
        }

        // grab the version defined in the manifest
        let mut manifest = manifest::IpManifest::from_path(c.get_ip_path().unwrap().to_path_buf().join(manifest::IP_MANIFEST_FILE))?;
        let prev_version = manifest.get_version();

        // at this point it is safe to assume it is a version because manifest will check that
        let mut version = prev_version.clone();
        
        // println!("already defined version: {}", version);
        // check if we applied --next
        let overwrite = if let Some(ver) = &self.next {
            match ver {
                VersionField::Major => version.inc_major(),
                VersionField::Minor => version.inc_minor(),
                VersionField::Patch => version.inc_patch(),
                VersionField::Version(v) => {
                    version = version.major(v.get_major())
                        .minor(v.get_minor())
                        .patch(v.get_patch());
                    // verify version will be larger than the current version
                    if prev_version >= &version {
                        return Err(AnyError(format!("set version {} is not greater than current version {}", version, prev_version)))?
                    }
                }
            }
            println!("info: raising {} --> {}", prev_version, version);
            // update the manifest and add a new commit to the git repository
            manifest.get_manifest_mut().write("ip", "version", version.to_string());
            true
        } else {
            println!("info: setting {}", version);
            false
        };

        // @TODO report if there are unsaved changes in the working directory/staging index?

        // verify the repository's HEAD is up-to-date (git remote update)
        println!("info: updating git repository remotes...");
        let extgit = ExtGit::new().command(None).path(c.get_ip_path().unwrap().clone());
        extgit.remote_update()?;

        let b = git2::Branch::wrap(repo.head()?);
        let local_name = b.name()?.unwrap().to_string();
        println!("info: on local git branch '{}'", local_name);

        let up_b = match b.upstream() {
            Ok(r) => Some(r),
            Err(_) => None,
        };

        let push = if let Some(remote_branch) = up_b {
            let remote_name = remote_branch.name()?.unwrap().to_string();

            // check if a repository field matches the remote url
            let raw_remotes = repo.remotes()?;
            let remotes: Vec<&str> = raw_remotes.into_iter().filter_map(|f| f).collect();
            if remotes.len() > 0 && manifest.get_repository().is_none() {
                let rem = repo.find_remote(remotes.first().unwrap()).unwrap();
                if let Some(url) = rem.url() {
                    return Err(AnyError(format!("manifest {1} is missing repository entry\n\nAdd `repository = \"{}\"` to the [ip] table in {1}", url, IP_MANIFEST_FILE)))?
                }
            }

            // report if the upstream branch does not match with the local branch
            if b.into_reference() != remote_branch.into_reference() {
                return Err(AnyError(format!("git repository's local branch '{}' is not in sync with remote upstream branch '{}'", local_name, remote_name)))?
            } else {
                true
            }
        } else {
            false
        };

        println!("info: pushing to a remote branch ... {}", {if push { "yes" } else { "no" }});

        let ver_str = version.to_string();
        {
            // check if a tag exists for this version
            let tags = repo.tag_names(Some("[0-9]*.[0-9]*.[0-9]*"))?;
            let result = tags.iter()
                .filter_map(|f| f )
                .find(|f| { f == &ver_str });
            
            // the version already exists under a tag
            if let Some(r) = result {
                return Err(AnyError(format!("version \'{}\' is already released", r)))?;
            }
        }
        
        // verify the manifest is committed (not in staging or working directory if not overwriting)
        if overwrite == false {
            let st = repo.status_file(&std::path::PathBuf::from(IP_MANIFEST_FILE))?;
            if st.is_empty() {
                println!("info: manifest is in clean state")
            } else {
                return Err(AnyError(format!("manifest {} is dirty; move changes out of working directory or staging index to enter a clean state", IP_MANIFEST_FILE)))?
            }
        }

        let message = match &self.message {
            Some(m) => m.to_owned(),
            None => format!("releases version {}", version),
        };

        println!("info: create new commit ... {}", match overwrite {
            true => "yes",
            false => "no",
        });
       
        if overwrite == true {
            println!("info: future commit message \"{}\"", message)
        }

        // verify git things

        // verify Orbit.toml to staging area
        let mut index = repo.index()?;
        index.add_path(&std::path::PathBuf::from(IP_MANIFEST_FILE))?;

        // verify a signature exists
        let signature = repo.signature()?;

        // tag if ready
        if self.ready == true {
            let marked_commit = if overwrite == true {
                // save the manifest
                manifest.get_manifest_mut().save()?;
                // add manifest to staging
                index.add_path(&std::path::PathBuf::from(IP_MANIFEST_FILE))?;
                // source: https://github.com/rust-lang/git2-rs/issues/561
                index.write()?;
                // create new commit
                let oid = index.write_tree().unwrap();
                let tree = repo.find_tree(oid)?;
                repo.commit(Some("HEAD"),
                    &signature,
                    &signature,
                    &message,
                    &tree,
                    &[&latest_commit])?;
                // update latest commit to attach with tag
                find_last_commit(&repo)?
            } else {
                latest_commit
            };

            // update the HEAD reference
            repo.tag_lightweight(&ver_str, &marked_commit.as_object(), false)?;

            // push to remotes
            if push == true {
                extgit.push()?;
            }

            // store the repository
            let store = Store::new(c.get_store_path());
            store.store(&manifest)?;

            println!("info: released version {}", version);
        } else {
            println!("info: version {} is ready for launch\n\nhint: include '--ready' flag to proceed", ver_str);
        }

        self.run()
    }
}

impl Launch {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Releases (tags) the current ip's latest commit as the next version.

Usage:
    orbit launch [options]

Options:
    --ready                 proceed with the launch process
    --next <version>        semver version or 'major', 'minor', or 'patch'
    --message, -m <message> message to apply to the commit when using '--next'

Use 'orbit help launch' to learn more about the command.
";