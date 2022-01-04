#![allow(clippy::large_enum_variant)]

use std::collections::HashMap;

pub type SsbHash = String;
pub type SsbId = String;
pub type SsbMsgType = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mention {
    pub link: SsbId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "type")]
    pub xtype: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<Mention>>,
}

impl Post {
    pub fn new(text: String, mentions: Option<Vec<Mention>>) -> Self {
        Post {
            xtype: String::from("post"),
            text,
            mentions,
        }
    }
    pub fn to_msg(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PubAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub port: u16,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VoteValue {
    Numeric(i64),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    link: SsbHash,
    value: VoteValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    expression: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Image {
    OnlyLink(SsbHash),
    Complete {
        link: SsbHash,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        size: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        #[serde(rename = "type")]
        content_type: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateTime {
    epoch: u64,
    tz: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Branch {
    One(SsbHash),
    Many(Vec<SsbHash>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Mentions {
    Link(SsbHash),
    One(Mention),
    Vector(Vec<Mention>),
    Map(HashMap<String, Mention>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypedMessage {
    #[serde(rename = "pub")]
    Pub { address: Option<PubAddress> },
    #[serde(rename = "post")]
    Post {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mentions: Option<Vec<Mention>>,
    },
    #[serde(rename = "contact")]
    Contact {
        contact: Option<SsbId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        blocking: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        following: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        autofollow: Option<bool>,
    },
    #[serde(rename = "about")]
    About {
        about: SsbId,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<SsbHash>,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<Image>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        location: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "startDateTime")]
        start_datetime: Option<DateTime>,
    },
    #[serde(rename = "channel")]
    Channel { channel: String, subscribed: bool },
    #[serde(rename = "vote")]
    Vote { vote: Vote },
}

/// An ssb-ql-1 query as defined by the 'Subset replication for SSB'
/// specification.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubsetQuery {
    Type { op: String, string: SsbMsgType },
    Author { op: String, feed: SsbId },
    And { op: String, args: Vec<SubsetQuery> },
    Or { op: String, args: Vec<SubsetQuery> },
}
