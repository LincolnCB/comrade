/// Custom verbose IO error struct.
#[derive(Debug)]
pub struct IoError {
    /// Filepath facing an error.
    pub file: String,
    /// Error cause.
    pub cause: std::io::Error,
}
impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "- Error with file: {}\n- {}", self.file, self.cause)
    }
}

type IoResult<T> = std::result::Result<T, IoError>;

/// Open a file with verbose errors.
pub fn open(path: &str) -> IoResult<std::fs::File> {
    let file = match std::fs::File::open(path){
        Ok(file) => file,
        Err(error) => {
            return Err(IoError{file: path.to_string(), cause: error});
        },
    };
    Ok(file)
}

/// Create a file with verbose errors.
pub fn create(path: &str) -> IoResult<std::fs::File> {
    let file = match std::fs::File::create(path){
        Ok(file) => file,
        Err(error) => {
            return Err(IoError{file: path.to_string(), cause: error});
        },
    };
    Ok(file)
}
