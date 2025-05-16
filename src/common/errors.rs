#[derive(Debug, Clone)]
pub enum Error {
    Io,
    JsonParse,
    DialogClosed,
}
