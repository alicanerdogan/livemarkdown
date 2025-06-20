use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

/// Generates a consistent hash-based ID for a file path.
///
/// This function creates a short, alphanumeric ID that is consistent across runs
/// for the same file path. The ID includes the filename for readability.
///
/// # Arguments
/// * `filepath` - The file path to generate an ID for
///
/// # Returns
/// A string in the format "{filename}-{hash}" where:
/// - filename is the file name with dots replaced by hyphens
/// - hash is a base36 encoded hash of the absolute path (8 characters)
///
/// # Examples
/// ```
/// use livemarkdown::utils::generate_document_id;
/// let id = generate_document_id("/path/to/file.md");
/// // Returns something like "file-md-a1b2c3d4"
/// ```
pub fn generate_document_id(filepath: &str) -> String {
    // Convert to absolute path for consistent hashing
    let absolute_path = to_absolute_path(filepath);

    // Extract filename for readability
    let filename = Path::new(filepath)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .replace('.', "-");

    // Generate consistent hash
    let mut hasher = DefaultHasher::new();
    absolute_path.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert to base36 for shorter, alphanumeric representation
    // Take first 8 characters for reasonable length
    let hash_str = format!("{:x}", hash);
    let short_hash = &hash_str[..8.min(hash_str.len())];

    format!("{}-{}", filename, short_hash)
}
