use std::io::Write;
pub mod stl;

#[derive(Debug)]
pub enum IoErrorType {
    File(std::io::Error),
    SerdeJson(serde_json::Error),
    SerdeYaml(serde_yaml::Error),
    TomlSer(toml::ser::Error),
    TomlDe(toml::de::Error),
    StringOnly(String),
}
impl std::fmt::Display for IoErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoErrorType::File(error) => write!(f, "- File IO Error:\n{}", error),
            IoErrorType::SerdeJson(error) => write!(f, "- JSON Serialization/Deserialization Error:\n{}", error),
            IoErrorType::SerdeYaml(error) => write!(f, "- YAML Serialization/Deserialization Error:\n{}", error),
            IoErrorType::TomlSer(error) => write!(f, "- TOML Serialization Error:\n{}", error),
            IoErrorType::TomlDe(error) => write!(f, "- TOML Deserialization Error:\n{}", error),
            IoErrorType::StringOnly(error) => write!(f, "- {}", error),
        }
    }
}

/// Custom verbose IO error struct.
#[derive(Debug)]
pub struct IoError {
    /// Filepath facing an error.
    pub file: Option<String>,
    /// Error cause.
    pub cause: IoErrorType,
}
impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.file {
            Some(ref file) => write!(f, "- Error with file: {}\n- {}", file, self.cause),
            None => write!(f, "- {}", self.cause),
        }
    }
}

pub type IoResult<T> = std::result::Result<T, IoError>;

/// Open a file with verbose errors.
pub fn open(path: &str) -> IoResult<std::fs::File> {
    let file = match std::fs::File::open(path){
        Ok(file) => file,
        Err(error) => {
            return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::File(error)});
        },
    };
    Ok(file)
}

/// Create a file with verbose errors.
pub fn create(path: &str) -> IoResult<std::fs::File> {
    let file = match std::fs::File::create(path){
        Ok(file) => file,
        Err(error) => {
            return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::File(error)});
        },
    };
    Ok(file)
}

/// Read from string with verbose errors
pub fn read_to_string(path: &str) -> IoResult<String> {
    match std::fs::read_to_string(path){
        Ok(buffer) => Ok(buffer),
        Err(error) => {
            return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::File(error)});
        },
    }
}

/// Write string to file with verbose errors.
pub fn write_to_file(path: &str, buffer: &str) -> IoResult<()> {
    let mut f = create(path)?;
    match f.write_all(buffer.as_bytes()){
        Ok(_) => Ok(()),
        Err(error) => {
            return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::File(error)});
        },
    }
}

/// Read in cfg files from the supported filetypes.
pub fn read_cfg_file<T>(path: &str) -> IoResult<T> 
where T: serde::de::DeserializeOwned
{
    match path.split('.').last(){
        Some("json") => {
            let cfg: T = match serde_json::from_reader(open(path)?) {
                Ok(cfg) => cfg,
                Err(error) => return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::SerdeJson(error)}),
            };
            Ok(cfg)
        },
        Some("toml") => {
            let cfg: T = match toml::from_str(&read_to_string(path)?) {
                Ok(cfg) => cfg,
                Err(error) => return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::TomlDe(error)}),
            };
            Ok(cfg)
        },
        Some("yaml") | Some("yml") => {
            let cfg: T = match serde_yaml::from_reader(open(path)?) {
                Ok(cfg) => cfg,
                Err(error) => return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::SerdeYaml(error)}),
            };
            Ok(cfg)
        },
        _ => {
            let supported_filetypes = vec!["json", "toml", "yaml", "yml"];
            let error_string = format!("Unsupported filetype for config file: {}\nSupported filetypes: {:?}", path, supported_filetypes);
            Err(IoError{file: Some(path.to_string()), cause: IoErrorType::StringOnly(error_string)})
        },
    }
}

/// Dump a struct to a file with the supported filetypes.
pub fn dump_cfg_to(path: &str, cfg: &impl serde::Serialize) -> IoResult<()> {
    match path.split('.').last(){
        Some("json") => {
            let f = create(path)?;
            match serde_json::to_writer(f, cfg){
                Ok(_) => Ok(()),
                Err(error) => return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::SerdeJson(error)}),
            }
        },
        Some("toml") => {
            let toml_str = match toml::to_string(cfg){
                Ok(toml_str) => toml_str,
                Err(error) => return Err(IoError{file: None, cause: IoErrorType::TomlSer(error)}),
            };
            write_to_file(path, &toml_str)
        },
        Some("yaml") | Some("yml") => {
            let f = create(path)?;
            match serde_yaml::to_writer(f, cfg){
                Ok(_) => Ok(()),
                Err(error) => return Err(IoError{file: Some(path.to_string()), cause: IoErrorType::SerdeYaml(error)}),
            }
        },
        _ => {
            let supported_filetypes = vec!["json", "toml", "yaml", "yml"];
            let error_string = format!("Unsupported filetype for config file: {}\nSupported filetypes: {:?}", path, supported_filetypes);
            Err(IoError{file: Some(path.to_string()), cause: IoErrorType::StringOnly(error_string)})
        },
    }
}
