#[derive(Debug, Clone)]
pub enum Error {
    FontLoadFailed,
    IoError(std::io::ErrorKind),
    JsonParseError,
    DialogClosed,
}
