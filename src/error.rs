use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConvertError<'a> {
    #[error("Cannot process unit '{0}' use either 'lb', 'g', 'kg', or 'metric ton'")]
    UnknownUnit(&'a str),
}
