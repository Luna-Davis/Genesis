use std::io::Write;
use zip::ZipWriter;
use zip::write::FileOptions;

use crate::bundler::types::AnalysisResult;

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub struct Payload {
    pub bytes: Vec<u8>,
}

const BOOTLOADER_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bootloader_bin"));

/// Build a native bundle: bootloader binary + zip archive containing all analyzed .py files.
pub fn pack_zipapp(analysis: &AnalysisResult) -> Result<Payload, PackError> {
    let mut out = Vec::new();

    // 1. Prepend the native bootloader
    out.extend_from_slice(BOOTLOADER_BIN);

    // 2. Build the zip archive in memory
    let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // Sort for deterministic output
    let mut files = analysis.py_files.clone();
    files.sort_by(|a, b| a.vfs_path.cmp(&b.vfs_path));

    let mut has_root_main = false;
    for f in &files {
        if f.vfs_path == "__main__.py" {
            has_root_main = true;
        }
        let src = std::fs::read(&f.host_path)?;
        zip.start_file(&f.vfs_path, options.clone())?;
        zip.write_all(&src)?;
    }

    // 2b. Synthesize __main__.py if missing
    if !has_root_main {
        let entry_vfs = &analysis.entry_vfs;
        // Convert "src/myapp/main.py" to "myapp.main"
        let module_path = entry_vfs
            .strip_suffix(".py")
            .unwrap_or(entry_vfs)
            .strip_prefix("src/")
            .unwrap_or(entry_vfs)
            .replace('/', ".");

        let stub = format!(
            r#"import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "src"))
from {} import main
if __name__ == "__main__":
    main()
"#,
            module_path
        );
        zip.start_file("__main__.py", options)?;
        zip.write_all(stub.as_bytes())?;
    }

    let cursor = zip.finish()?;

    // 3. Append the ZIP to the bootloader
    out.extend_from_slice(cursor.get_ref());

    // 4. Append the footer: the original offset where ZIP starts
    let zip_start_offset = BOOTLOADER_BIN.len() as u64;
    out.extend_from_slice(&zip_start_offset.to_le_bytes());

    Ok(Payload { bytes: out })
}
