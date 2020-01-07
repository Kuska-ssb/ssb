#![allow(clippy::large_enum_variant)]

use std::collections::HashMap;

pub type SsbHash = String;
pub type SsbId = String;


#[derive(Debug, Deserialize)]
pub struct Mention {
    pub link: SsbId,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    #[serde(rename = "type")]
    pub xtype: String,
    pub text: String,
    pub mentions: Option<Vec<Mention>>,
}

#[derive(Debug, Deserialize)]
pub struct PubAddress {
    pub host: Option<String>,
    pub port: u16,
    pub key: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VoteValue {
    Numeric(i64),
    Boolean(bool),
}

#[derive(Debug, Deserialize)]
pub struct Vote {
    link: SsbHash,
    value: VoteValue,
    expression: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Image {
    OnlyLink(SsbHash),
    Complete {
        link: SsbHash,
        name: Option<String>,
        size: u64,
        width: Option<u32>,
        height: Option<u32>,
        #[serde(rename = "type")]
        content_type: String,
    },
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum FeedTypedContent {
    #[serde(rename = "pub")]
    Pub { address: Option<PubAddress> },
    #[serde(rename = "post")]
    Post {
        text: Option<String>,
        post: Option<String>, // the same than text
        channel: Option<String>,
        mentions: Option<Mentions>,
        root: Option<SsbHash>,
        branch: Option<Branch>,
        reply: Option<HashMap<SsbHash, SsbId>>,
        recps: Option<String>,
    },
    #[serde(rename = "contact")]
    Contact {
        contact: Option<SsbId>,
        blocking: Option<bool>,
        following: Option<bool>,
        autofollow: Option<bool>,
    },
    #[serde(rename = "about")]
    About {
        about: SsbId,
        name: Option<String>,
        title: Option<String>,
        branch: Option<SsbHash>,
        image: Option<Image>,
        description: Option<String>,
        location: Option<String>,
        #[serde(rename = "startDateTime")]
        start_datetime: Option<DateTime>,
    },
    #[serde(rename = "channel")]
    Channel { channel: String, subscribed: bool },
    #[serde(rename = "vote")]
    Vote { vote: Vote },
}

