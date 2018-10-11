use std::path::Path;
use serde::de::DeserializeOwned;
use std::fs::OpenOptions;
use std::error::Error;
use std::str;

pub fn deserialize_object<T>(source_path: impl AsRef<Path>) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned,
{
    let file = OpenOptions::new().read(true).open(source_path.as_ref())?;
    let mmap = unsafe { ::memmap::Mmap::map(&file) }?;
    let content = str::from_utf8(&mmap)?;
    let object = ::serde_json::from_str(content)?;
    Ok(object)
}

pub fn fast_read(source_path: impl AsRef<Path>) -> Result<String, Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open(source_path.as_ref())?;
    let mmap = unsafe { ::memmap::Mmap::map(&file) }?;
    Ok(str::from_utf8(&mmap)?.to_owned())
}