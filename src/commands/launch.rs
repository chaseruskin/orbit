use crate::Command;
use crate::FromCli;
use crate::commands::install::Install;
use crate::core::catalog::Catalog;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::manifest::IpManifest;
use crate::core::store::Store;
use crate::core::variable::VariableTable;
use crate::core::version::AnyVersion;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::version::Version;
use crate::core::extgit::ExtGit;
use crate::util;
use std::io::Write;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use git2::Repository;
use colored::Colorize;
use crate::core::manifest;
use crate::util::anyerror::AnyError;

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
    no_install: bool,
}

impl FromCli for Launch {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Launch {
            ready: cli.check_flag(Flag::new("ready"))?,
            next: cli.check_option(Optional::new("next").value("version"))?,
            message: cli.check_option(Optional::new("message").switch('m'))?,
            no_install: cli.check_flag(Flag::new("no-install"))?,
        });
        command
    }
}

impl Command for Launch {
    type Err = Fault;

    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // make sure it is run from an ip directory
        c.goto_ip_path()?;
        // verify the current directory is a git repository
        let repo = Repository::open(c.get_ip_path().unwrap())?;

        // verify the repository has at least one commit
        let latest_commit = ExtGit::find_last_commit(&repo)?;

        if self.message.is_some() && self.next.is_none() {
            return Err(CliError::BrokenRule(format!("option '{}' is only allowed when using option {}", "--message".yellow(), "--next".yellow())))?
        }

        // verify the manifest is checked into version control
        if repo.is_path_ignored("Orbit.toml")? {
            return Err(AnyError(format!("manifest 'Orbit.toml' is ignored by version control")))?
        }

        // grab the version defined in the manifest
        let mut manifest = manifest::IpManifest::from_path(c.get_ip_path().as_ref().unwrap())?;

        // load variables
        let mut vars = VariableTable::new()
            .load_context(&c)?
            .load_pkgid(&manifest.get_pkgid())?
            .load_environment(&Environment::new().from_config(c.get_config())?)?;

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
            println!("info: raising {} -> {}", prev_version, version.to_string().blue());
            // update the manifest and add a new commit to the git repository
            manifest.get_manifest_mut().write("ip", "version", version.to_string());
            true
        } else {
            println!("info: setting {}", version.to_string().blue());
            false
        };

        vars.add("orbit.ip.version", &version.to_string());

        // verify ip dependency graph (also checks for duplicate design unit identifiers)
        print!("info: verifying ip dependency graph ... ");
        std::io::stdout().flush().ok().expect("could not flush stdout");
        {
            let r = Launch::verify_ip_graph(&c, &manifest);
            println!("{}", util::prompt::report_eval(r.is_ok()));
            r?
        }

        // @todo: report if there are unsaved changes in the working directory/staging index?

        // verify the repository's HEAD is up-to-date (git remote update)
        print!("info: checking git repository remotes ... ");
        std::io::stdout().flush().ok().expect("could not flush stdout");
        let extgit = ExtGit::new(None).path(c.get_ip_path().unwrap().clone());
        {
            let r = extgit.remote_update();
            println!("{}", util::prompt::report_eval(r.is_ok()));
            r?
        }

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

        println!("info: pushing to a remote branch ... {}", util::prompt::report_eval(push));

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

        // check there are zero dependencies from the DEV_PATH
        if let Some(dep) = manifest.get_dependencies()
                .inner()
                .into_iter()
                .find_map(|f| if f.1 == &AnyVersion::Dev { Some(f.0) } else { None }) {
            return Err(AnyError(format!("direct dependency '{}' cannot come from a development state", dep)))?
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

        println!("info: create commit for manifest ... {}", util::prompt::report_eval(overwrite));
       
        if overwrite == true {
            println!("info: future commit message \"{}\"", message)
        }

        println!("info: installing to cache ... {}",util::prompt::report_eval(!self.no_install));

        // find the registry by using the vendor
        let registry = c.get_vendors().get(manifest.get_pkgid().get_vendor().as_ref().unwrap());
        let publish = registry.is_some() && manifest.get_repository().is_some() && push;
        println!("info: publishing to registry ... {}", util::prompt::report_eval(publish));

        // --- verify git things

        // verify Orbit.toml to staging area
        let mut index = repo.index()?;

        let manifest_path = std::path::PathBuf::from(IP_MANIFEST_FILE);
        index.add_path(&manifest_path)?;

        // verify a signature exists
        let signature = repo.signature()?;

        // tag if ready
        if self.ready == true {
            let marked_commit = if overwrite == true {
                // save the manifest
                manifest.get_manifest_mut().save()?;
                // add manifest to staging
                index.add_path(&manifest_path)?;
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
                ExtGit::find_last_commit(&repo)?
            } else {
                latest_commit
            };

            // update the HEAD reference
            repo.tag_lightweight(&ver_str, &marked_commit.as_object(), false)?;

            // push to remotes
            if push == true {
                extgit.push()?;
            }

            println!("info: released version {}", version);

            // publish to vendor
            if let Some(reg) = registry {
                reg.publish(&mut manifest, &version, vars)?;
            }

            // store the repository
            let store = Store::new(c.get_store_path());
            store.store(&manifest)?;

            // perform installation to the cache
            if self.no_install == false {
                Install::install(&manifest.get_root(), &AnyVersion::Specific(version.to_partial_version()), c.get_cache_path(), true, &store)?;
            }
        } else {
            println!("info: version {} is ready for launch\n\nhint: include '{}' to proceed", ver_str, "--ready".green());
        }

        self.run()
    }
}

impl Launch {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn verify_ip_graph(c: &Context, target: &IpManifest) -> Result<(), Fault> {
        // gather the catalog
        let catalog = Catalog::new()
            .store(c.get_store_path())
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(c.get_vendors())?;
        // build entire ip graph and resolve with dynamic symbol transformation
        crate::core::ip::compute_final_ip_graph(&target, &catalog)?;
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
    --no-install            skip installing newly launched version

Use 'orbit help launch' to learn more about the command.
";