use std::path::Path;

/// Converts a path to an absolute path.
///
/// First tries to canonicalize the path (which resolves symlinks and makes it absolute).
/// If that fails, it attempts to make the path absolute by joining with current directory.
/// Falls back to the original path if all attempts fail.
pub fn to_absolute_path(filepath: &str) -> String {
    // Convert to absolute path
    match Path::new(filepath).canonicalize() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => {
            // If canonicalize fails, try to make it absolute manually
            if Path::new(filepath).is_absolute() {
                filepath.to_string()
            } else {
                match std::env::current_dir() {
                    Ok(current_dir) => current_dir.join(filepath).to_string_lossy().to_string(),
                    Err(_) => filepath.to_string(), // Fall back to original if all else fails
                }
            }
        }
    }
}
