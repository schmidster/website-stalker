use std::ffi::OsString;
use std::fs::{create_dir_all, read_dir, read_to_string, remove_file, write};

#[derive(Clone)]
pub struct SiteStore {
    folder: String,
}

impl SiteStore {
    pub fn new(folder: String) -> std::io::Result<Self> {
        create_dir_all(&folder)?;
        Ok(Self { folder })
    }

    pub fn remove_gone(&self, expected_filenames: &[String]) -> anyhow::Result<Vec<OsString>> {
        let mut superfluous = Vec::new();

        for file in read_dir(&self.folder)? {
            let file = file?;
            let is_wanted = file
                .file_name()
                .into_string()
                .map_or(false, |name| expected_filenames.contains(&name));
            if !is_wanted {
                remove_file(file.path())?;
                superfluous.push(file.file_name());
            }
        }

        superfluous.sort();
        Ok(superfluous)
    }

    pub fn write_only_changed(&self, filename: &str, contents: &str) -> std::io::Result<bool> {
        let path = format!("{}/{}", self.folder, filename);
        let contents = contents.trim().to_string() + "\n";

        let current = read_to_string(&path).unwrap_or_default();
        let changed = current != contents;
        if changed {
            write(&path, contents)?;
        }
        Ok(changed)
    }
}