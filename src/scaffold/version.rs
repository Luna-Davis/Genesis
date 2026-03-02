use semver::Version;

pub fn bump_version_str(version: &str, level: &str) -> Result<String, String> {
    let mut v = Version::parse(version).map_err(|e| e.to_string())?;
    match level {
        "major" => v = Version::new(v.major + 1, 0, 0),
        "minor" => v = Version::new(v.major, v.minor + 1, 0),
        _ => v = Version::new(v.major, v.minor, v.patch + 1),
    }
    Ok(v.to_string())
}
