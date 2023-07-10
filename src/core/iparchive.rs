use zip::ZipArchive;
use crate::util::anyerror::Fault;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use crate::util::compress;
use std::str::FromStr;
use super::ip::Ip;
use super::lockfile::LockFile;
use super::manifest::Manifest;
use flate2::read::ZlibDecoder;
use std::io::Read;
use flate2::Compression;
use flate2::write::ZlibEncoder;

/// Increment this number any time the format of the archive changes.
const ARCHIVE_VERSION: u8 = 1;

pub const ARCHIVE_EXT: &str = "ip";

/// Number of bytes to read the archive version.
const VERSION_SIZE: usize = 1;
/// Number of bytes to read a [u32] value.
const U32_SIZE: usize = 4;

#[derive(Debug, PartialEq)]
pub struct IpArchive {
    manifest: Manifest,
    lock: LockFile,
    /// Compressed data containing the [Ip].
    archive: Vec<u8>,
}

impl IpArchive {
    fn slice(buf: &Vec<u8>, offset: usize, size: usize) -> &[u8] {
        &buf[offset..offset+size]
    }

    pub fn read(path: &PathBuf) -> Result<Self, Fault> {
        let contents = fs::read(path)?;
        // read the first byte as the version of the archive
        let version: u8 = u8::from_be_bytes(Self::slice(&contents, 0, VERSION_SIZE).try_into()?);

        match version {
            1 => Self::parse_v1(contents),
            _ => { panic!("unsupported archive version {}", version) }
        }
    }


    /// Parses according to version of [IpArchive] format.
    fn parse_v1(buf: Vec<u8>) -> Result<Self, Fault> {
        // read length of header
        let header_offset = VERSION_SIZE;
        let header_len: usize = u32::from_be_bytes(Self::slice(&buf, header_offset, U32_SIZE).try_into()?) as usize;

        // decompress the header bytes
        let mut d = ZlibDecoder::new(Self::slice(&buf, header_offset+U32_SIZE, header_len));
        let mut header_bytes = Vec::new();
        d.read_to_end(&mut header_bytes)?;

        // decompose the decompressed header bytes
        
        // read length of manifest
        let man_offset: usize = 0;
        let man_len: usize = u32::from_be_bytes(Self::slice(&header_bytes, man_offset, U32_SIZE).try_into()?) as usize;
        // read length of lockfile
        let lock_offset: usize = man_offset+U32_SIZE+man_len;
        let lock_len: usize = u32::from_be_bytes(Self::slice(&header_bytes, lock_offset, U32_SIZE).try_into()?) as usize;

        // slice to the bytes for the relevant zipped archive
        let archive_offset: usize = header_offset+U32_SIZE+header_len;
        let archive = &buf[archive_offset..];

        Ok(Self {
            manifest: Manifest::from_str(&String::from_utf8(Self::slice(&header_bytes, man_offset+U32_SIZE, man_len).to_vec())?)?,
            lock: LockFile::decode(&String::from_utf8(Self::slice(&header_bytes, lock_offset+U32_SIZE, lock_len).to_vec())?)?,
            archive: archive.to_vec(),
        })
    }

    /// Separates the inner data into their own structs.
    /// 
    /// This function is useful for implementing From<[IpArchive]> for [Ip].
    pub fn decouple(self) -> (Manifest, LockFile) {
        (self.manifest, self.lock)
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

        // compress the header bytes
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        let embedded_data = vec![
            // get the manifest bytes
            ip.get_man().to_string(),
            // get the lockfile bytes
            ip.get_lock().to_string(),
        ];
        for data in embedded_data {
            // write the size of the string
            e.write_all(&(data.len() as u32).to_be_bytes())?;
            // write the string contents
            e.write_all(&data.as_bytes())?;
        }
        // complete the compression algorithm
        let header_bytes = e.finish()?;
        // write the number of compressed header bytes to the archive
        file.write(&(header_bytes.len() as u32).to_be_bytes())?;
        // write the compressed bytes
        file.write(&header_bytes)?;

        // write the entire compressed file back
        file.write(&archive_bytes)?;

        Ok(())
    }

    /// Detects all Ip found as archives.
    pub fn detect_all(dir: &PathBuf) -> Result<Vec<Ip>, Fault> {
        // for each .ip file
        fs::read_dir(&dir)?
            .filter_map(|result| { if let Ok(r) = result { Some(r) } else { None } })
            .map(|entry| { entry.path().to_path_buf() })
            .filter(|path| path.extension().is_some() && path.extension().unwrap() == ARCHIVE_EXT)
            .map(|path| {
                match IpArchive::read(&path) {
                    Ok(arc) => Ok(Ip::from(arc)),
                    Err(e) => Err(e)
                }
            })
            .collect()
    }

}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn write_and_read() {

//     }
// }