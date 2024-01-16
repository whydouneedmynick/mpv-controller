use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use anyhow::bail;
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use serde_json::json;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Property {
    Chapter,
    Volume,
    MediaTitle,
    Pause,
    Chapters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

impl TryInto<i32> for PropertyValue {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<i32, Self::Error> {
        match self {
            PropertyValue::Integer(i) => Ok(i),
            PropertyValue::Float(f) => Ok(f as i32),
            v => bail!("Failed to convert {:?} to i8", v),
        }
    }
}

impl TryInto<String> for PropertyValue {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<String, Self::Error> {
        match self {
            PropertyValue::String(s) => Ok(s),
            v => bail!("Failed to convert {:?} to string", v),
        }
    }
}

impl TryInto<bool> for PropertyValue {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<bool, Self::Error> {
        match self {
            PropertyValue::Bool(b) => Ok(b),
            v => bail!("Failed to conver {:?} to string", v),
        }
    }
}

pub enum Command {
    GetProperty(Property),
    SetProperty(Property, PropertyValue),
}

impl Command {
    pub fn execute(&self, stream: &mut UnixStream) -> Result<Response> {
        let cmd = json!({
            "command": self
        })
        .to_string()
            + "\n";
        stream.write_all(cmd.as_bytes())?;

        let stream_copy = stream.try_clone()?;
        let mut wrapped = BufReader::new(stream_copy);

        let mut buf = String::new();
        wrapped.read_line(&mut buf)?;

        let res = serde_json::from_str(&buf)?;
        Ok(res)
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Command::GetProperty(property) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("get_property")?;
                seq.serialize_element(property)?;
                seq.end()
            }
            Command::SetProperty(property, value) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("set_property")?;
                seq.serialize_element(property)?;
                seq.serialize_element(value)?;
                seq.end()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Option<PropertyValue>,
    #[allow(dead_code)]
    pub request_id: u8,
    #[allow(dead_code)]
    pub error: String,
}
