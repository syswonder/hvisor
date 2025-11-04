pub type MpamResult<T> = Result<T, MpamError>;

#[derive(Debug)]
pub enum MpamError {
    Unsupported,
    InvalidValue,
}
