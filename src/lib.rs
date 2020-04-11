use std::fs;
use std::io;
use std::path::Path;

pub fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    Ok(fs::metadata(path.as_ref())?.len())
}
