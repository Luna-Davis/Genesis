use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sha2::{Sha256, Digest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let mut file = fs::File::open(&current_exe)?;
    
    // 1. Read the ZIP start offset from the last 8 bytes of the file
    let file_len = file.metadata()?.len();
    if file_len < 8 {
        return Err("File too small to contain a bundle".into());
    }
    
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::End(-8))?;
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes)?;
    let zip_start = u64::from_le_bytes(offset_bytes);
    
    // 2. Calculate hash based only on the ZIP payload (for stable caching)
    file.seek(SeekFrom::Start(zip_start))?;
    let mut zip_bytes = Vec::new();
    file.read_to_end(&mut zip_bytes)?;
    
    let mut hasher = Sha256::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    let cache_dir = cache_dir(&hash)?;
    
    // 3. Extract if not cached (or incomplete)
    let need_extract = !cache_dir.join("__main__.py").exists();
    if need_extract {
        fs::create_dir_all(&cache_dir)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&zip_bytes))?;
        
        for i in 0..archive.len() {
            let mut zip_file = archive.by_index(i)?;
            let outpath = match zip_file.enclosed_name() {
                Some(path) => cache_dir.join(path),
                None => continue,
            };

            if (*zip_file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut zip_file, &mut outfile)?;
            }
        }
    }
    
    // 4. Find entrance (always __main__.py since packer synthesizes it)
    let main_py = cache_dir.join("__main__.py");
    
    // 5. Run Python
    let mut cmd = Command::new("python3");
    cmd.arg(main_py)
       .args(env::args().skip(1))
       .env("PYTHONPATH", format!("{}:{}", cache_dir.display(), cache_dir.join("src").display()));
    
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    std::process::exit(status.code().unwrap_or(1));
}

fn cache_dir(hash: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Priority: explicit override → XDG cache → /tmp
    if let Ok(path) = env::var("GENESIS_CACHE_DIR") {
        return Ok(PathBuf::from(path).join(hash));
    }

    if let Some(base) = dirs::cache_dir() {
        let path = base.join("genesis-apps").join(hash);
        if ensure_dir(&path).is_ok() {
            return Ok(path);
        }
    }

    let fallback = Path::new("/tmp").join("genesis-apps").join(hash);
    ensure_dir(&fallback)?;
    Ok(fallback)
}

fn ensure_dir(p: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(p)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(p, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
