//! Abstraction layer for writing a directory to a zip file
//!
//! Reference: https://github.com/zip-rs/zip/blob/master/examples/write_dir.rs

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use zip::result::ZipError;
use zip::write::FileOptions;
use zip::CompressionMethod;

use ignore::{DirEntry, WalkBuilder};
use std::fs::File;
use std::path::{Path, PathBuf};

/// Declare the type of compression algorithm to use.
const COMPRESSION_METHOD: CompressionMethod = CompressionMethod::ZSTD;

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &PathBuf,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn write_zip_dir(src_dir: &PathBuf, dst_file: &PathBuf) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkBuilder::new(src_dir)
        .standard_filters(false)
        .hidden(false)
        .git_ignore(false)
        .build();

    let it = walkdir.into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        src_dir,
        file,
        COMPRESSION_METHOD,
    )?;

    Ok(())
}
