use std::ffi::OsString;

#[derive(Debug)]
pub enum Error {
    LoadError(OsString),
}