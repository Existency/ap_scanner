use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use rocket::request::FromParam;

use crate::SCAN_PATH as scan_path;

#[derive(UriDisplayPath)]
pub struct ReadingID<'a>(Cow<'a, str>);

impl ReadingID<'_> {
    pub fn new() -> ReadingID<'static> {
        ReadingID(Cow::Owned(uuid::Uuid::new_v4().to_string()))
    }

    pub fn path(&self) -> PathBuf {
        Path::new(scan_path.as_str()).join(format!("{}.json", self.0.as_ref()))
    }
}

impl<'a> FromParam<'a> for ReadingID<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
            .then(|| ReadingID(param.into()))
            .ok_or(param)
    }
}
