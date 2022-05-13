use crate::util::sha256;

/// Given a list of files, compute a single sha256 digest to encapsulate the
/// entire directory state.
/// 
/// Assumes the filepaths are already sorted before entering this function.
fn checksum(files: &[String]) -> sha256::Sha256Hash {
    // determine the amount of bytes required
    let total_hashes = files.len() + 1;
    let mut final_bytes = Vec::<u8>::with_capacity(total_hashes*32);
    
    let mut filename_bytes = Vec::<u8>::new();
    // perform a hash on contents
    for file in files {
        // @NOTE windows uses \r\n for newlines, compared to unix systems using just \n
        let bytes: Vec<u8> = std::fs::read(&file).expect("failed to read as bytes").into_iter().filter(|f| {
            f != &0x0d // \r is 0X0D
        }).collect();
        final_bytes.append(&mut sha256::compute_sha256(&bytes).into_bytes().to_vec());
        filename_bytes.append(&mut file.as_bytes().to_vec());
    }
    // perform hash on filenames
    final_bytes.append(&mut sha256::compute_sha256(&filename_bytes).into_bytes().to_vec());

    // perform hash on all hashes
    sha256::compute_sha256(&final_bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        // use list of files
        let files = vec![
            "test/data/file1.txt".to_owned(),
            "test/data/file2.txt".to_owned(),
            "test/data/file3.txt".to_owned(),
        ];
        let sum1 = checksum(&files);
        assert_eq!(sum1, sha256::Sha256Hash::from_u32s([1192313984, 1124899892, 
            4096760620, 1419010557, 2999754695, 3953725091, 4055090036, 1661318102]));

        // modifying a file name results in a different hash
        let files = vec![
            "test/data/file1.txt".to_owned(),
            "test/data/file2.txt".to_owned(),
            "test/data/file3copy.txt".to_owned(), // same contents as file3.txt
        ];
        assert_eq!(std::fs::read("test/data/file3.txt").unwrap(), std::fs::read("test/data/file3copy.txt").unwrap(), "file3 and file3copy must have same contents");
        assert_ne!(checksum(&files), sum1);

        // taking away a file results in a different hash
        let files = vec![
            "test/data/file1.txt".to_owned(),
            "test/data/file2.txt".to_owned(),
        ];
        let sum2 = checksum(&files);
        assert_ne!(sum2, sum1);


        // adding a file results in a different hash
        let files = vec![
            "test/data/file1.txt".to_owned(),
            "test/data/file2.txt".to_owned(),
            "test/data/file3.txt".to_owned(),
            "Cargo.toml".to_owned(),
        ];
        let sum3 = checksum(&files);
        assert_ne!(sum3, sum1);
        assert_ne!(sum3, sum2);
    }
}