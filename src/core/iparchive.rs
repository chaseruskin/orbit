// #[derive(Debug, PartialEq)]
// pub struct Ip {
//     /// The base directory for the entire [Ip] structure.
//     root: PathBuf,
//     /// The metadata for the [Ip].
//     data: Manifest,
//     /// The lockfile for the [Ip].
//     lock: LockFile,
//     /// The UUID for the [Ip].
//     uuid: Uuid,
// }

use zip::ZipArchive;

use crate::util::anyerror::Fault;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::util::compress;

use super::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct IpArchive {
    version: u8,
    /// Metadata about the [Ip].
    header: Vec<u8>,
    /// Compressed data containing the [Ip].
    archive: Vec<u8>,
}

/// Increment this number any time the format of the archive changes.
const ARCHIVE_VERSION: u8 = 1;

impl IpArchive {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, Fault> {
        let contents = fs::read(path)?;
        // read the first byte as the version of the archive
        let version: u8 = u8::from_be_bytes(contents[0..1].try_into()?);

        match version {
            1 => Self::parse_v1(contents),
            _ => { panic!("unsupported archive version {}", version) }
        }
    }

    /// Parses according to version of [IpArchive] format.
    fn parse_v1(buf: Vec<u8>) -> Result<Self, Fault> {
        // read length of manifest
        let man_offset: usize = 1;
        let man_len: usize = u32::from_be_bytes(buf[man_offset..man_offset+4].try_into()?) as usize;
        // read length of lockfile
        let lock_offset: usize = man_offset+4+man_len;
        let lock_len: usize = u32::from_be_bytes(buf[lock_offset..lock_offset+4].try_into()?) as usize;

        let archive_offset: usize = lock_offset+4+lock_len;

        let lock_str = String::from_utf8(buf[lock_offset+4..lock_offset+4+lock_len].to_vec())?;
        println!("{}", lock_str);

        let (header, archive) = buf.split_at(archive_offset);
        Ok(Self {
            version: 1,
            header: header.to_vec(),
            archive: archive.to_vec(),
        })
    }

    /// Unzips the archive and places it at `dest`. The `dest` path will be
    /// the root folder of the decompressed archive.
    /// 
    /// Assumes `dest` does not exist.
    pub fn extract(self, dest: &PathBuf) -> Result<(), Fault> {
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&self.archive)?;
        let mut zip_archive = ZipArchive::new(temp_file)?;
        zip_archive.extract(dest)?;

        Ok(())
    } 

    /// Stores the project's state and additional metadata into a .zip archive.
    pub fn write(ip: &Ip, dest: &PathBuf) -> Result<(), Fault> {

        compress::write_zip_dir(ip.get_root(), &dest)?;
        // read back the bytes
        let archive_bytes = fs::read(&dest)?;
        // write the header bytes first
        let mut file = File::options().write(true).truncate(true).open(&dest)?;

        // write the version of the writer
        file.write(&ARCHIVE_VERSION.to_be_bytes())?;

        let embedded_files = vec![
            // get the manifest bytes
            ip.get_man().to_string(),
            // get the lockfile bytes
            ip.get_lock().to_string(),
        ];

        for data in embedded_files {
            // write the size of the string
            file.write(&(data.len() as u32).to_be_bytes())?;
            // write the string contents
            file.write(&data.as_bytes())?;
        }

        // write the entire compressed file back
        file.write(&archive_bytes)?;

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn write_and_read() {

//     }
// }