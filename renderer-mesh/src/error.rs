use std::ffi::OsString;

#[derive(Debug)]
pub enum Error {
    MismatchedVerticesNormals,
    NoSuchFile(OsString),
    NoVerticesFound,
}
