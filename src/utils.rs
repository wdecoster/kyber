use std::path::PathBuf;

pub fn is_file(pathname: &str) -> Result<(), String> {
    if pathname == "-" {
        return Ok(());
    }
    let path = PathBuf::from(pathname);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Input file {} is invalid", path.display()))
    }
}
