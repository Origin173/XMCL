use serde::Serialize;
use std::error::Error;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct XMCLError(pub String);

pub type XMCLResult<T> = Result<T, XMCLError>;

impl<T> From<T> for XMCLError
where
  T: Error,
{
  fn from(err: T) -> Self {
    XMCLError(err.to_string())
  }
}
