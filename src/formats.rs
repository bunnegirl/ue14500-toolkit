use std::path::PathBuf;

pub mod assembly;
pub mod binary;

#[derive(Debug)]
pub enum FileType {
    Assembly,
    Binary,
}

impl TryFrom<PathBuf> for FileType {
    type Error = ();

    fn try_from(path: PathBuf) -> Result<FileType, ()> {
        let str_path = path.display().to_string();

        if str_path.ends_with(".asm") {
            Ok(FileType::Assembly)
        } else if str_path.ends_with(".bin") {
            Ok(FileType::Binary)
        } else {
            Err(())
        }
    }
}
