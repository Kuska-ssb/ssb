extern crate kuska_handshake;
extern crate kuska_ssb;

extern crate base64;
extern crate crossbeam;
extern crate structopt;

use std::fmt::Debug;
use structopt::StructOpt;

use async_std::io::{Read, Write};
use async_std::net::TcpStream;

use kuska_handshake::async_std::{handshake_client, BoxStream};
use kuska_ssb::api::{
    ApiHelper, CreateHistoryStreamArgs, CreateStreamArgs, LatestUserMessage, WhoAmI,
};
use kuska_ssb::crypto::ToSodiumObject;
use kuska_ssb::discovery::ssb_net_id;
use kuska_ssb::feed::{is_privatebox, privatebox_decipher, Feed, Message};
use kuska_ssb::keystore::from_patchwork_local;
use kuska_ssb::keystore::OwnedIdentity;
use kuska_ssb::rpc::{RecvMsg, RequestNo, RpcStream};

type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Connect to server
    // format is: server:port:<server_id>
    #[structopt(short, long)]
    connect: String,
}

pub fn whoami_res_parse(body: &[u8]) -> AnyResult<WhoAmI> {
    Ok(serde_json::from_slice(body)?)
}
pub fn message_res_parse(body: &[u8]) -> AnyResult<Message> {
    Ok(Message::from_slice(body)?)
}
pub fn feed_res_parse(body: &[u8]) -> AnyResult<Feed> {
    Ok(Feed::from_slice(&body)?)
}
pub fn latest_res_parse(body: &[u8]) -> AnyResult<LatestUserMessage> {
    Ok(serde_json::from_slice(body)?)
}

#[derive(Debug)]
struct AppError {
    message: String,
}
impl AppError {
    pub fn new(message: String) -> Self {
        AppError { message }
    }
}
impl std::error::Error for AppError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

async fn get_async<'a, R, W, T, F>(
    client: &mut ApiHelper<R, W>,
    req_no: RequestNo,
    f: F,
) -> AnyResult<T>
where
    R: Read + Unpin,
    W: Write + Unpin,
    F: Fn(&[u8]) -> AnyResult<T>,
    T: Debug,
{
    loop {
        let (id, msg) = client.rpc().recv().await?;
        if id == req_no {
            match msg {
                RecvMsg::BodyResponse(body) => {
                    return f(&body).map_err(|err| err.into());
                }
                RecvMsg::ErrorResponse(message) => {
                    return std::result::Result::Err(Box::new(AppError::new(message)));
                }
                _ => unreachable!(),
            }
        } else {
            println!("discarded message {}", id);
        }
    }
}

async fn print_source_until_eof<'a, R, W, T, F>(
    client: &mut ApiHelper<R, W>,
    req_no: RequestNo,
    f: F,
) -> AnyResult<()>
where
    R: Read + Unpin,
    W: Write + Unpin,
    F: Fn(&[u8]) -> AnyResult<T>,
    T: Debug + serde::Deserialize<'a>,
{
    loop {
        let (id, msg) = client.rpc().recv().await?;
        if id == req_no {
            match msg {
                RecvMsg::BodyResponse(body) => {
                    let display = f(&body)?;
                    println!("{:?}", display);
                }
                RecvMsg::ErrorResponse(message) => {
                    return std::result::Result::Err(Box::new(AppError::new(message)));
                }
                RecvMsg::CancelStreamRespose() => break,
                _ => unreachable!(),
            }
        } else {
            println!("discarded message {}", id);
        }
    }
    Ok(())
}

#[async_std::main]
async fn main() -> AnyResult<()> {
    env_logger::init();
    log::set_max_level(log::LevelFilter::max());

    let OwnedIdentity { pk, sk, id } = from_patchwork_local().await.expect("read local secret");
    println!("connecting with identity {}", id);

    let opt = Opt::from_args();
    let connect: Vec<_> = opt.connect.split(":").collect();
    if connect.len() != 3 {
        panic!("connection string should be server:port:id");
    }
    let server_pk = connect[2][1..].to_ed25519_pk()?;

    let mut socket = TcpStream::connect(format!("{}:{}", connect[0], connect[1])).await?;

    let handshake = handshake_client(&mut socket, ssb_net_id(), pk, sk.clone(), server_pk).await?;

    println!("💃 handshake complete");

    let (box_stream_read, box_stream_write) =
        BoxStream::from_handshake(&socket, &socket, handshake, 0x8000).split_read_write();

    let mut client = ApiHelper::new(RpcStream::new(box_stream_read, box_stream_write));

    let req_id = client.whoami_req_send().await?;
    let whoami = get_async(&mut client, req_id, whoami_res_parse).await?.id;

    println!("😊 server says hello to {}", whoami);

    let mut line_buffer = String::new();
    while let Ok(_) = std::io::stdin().read_line(&mut line_buffer) {
        let args: Vec<String> = line_buffer
            .replace("\n", "")
            .split_whitespace()
            .map(|arg| arg.to_string())
            .collect();

        match (args[0].as_str(), args.len()) {
            ("exit", 1) => {
                client.rpc().close().await?;
                break;
            }
            ("whoami", 1) => {
                let req_id = client.whoami_req_send().await?;
                let whoami = get_async(&mut client, req_id, whoami_res_parse).await?.id;
                println!("{}", whoami);
            }
            ("get", 2) => {
                let msg_id = if args[1] == "any" {
                    "%TL34NIX8JpMJN+ubHWx6cRhIwEal8VqHdKVg2t6lFcg=.sha256".to_string()
                } else {
                    args[1].clone()
                };
                let req_id = client.get_req_send(&msg_id).await?;
                let msg = get_async(&mut client, req_id, message_res_parse).await?;
                println!("{:?}", msg);
            }
            ("user", 2) => {
                let user_id = if args[1] == "me" { &whoami } else { &args[1] };

                let args = CreateHistoryStreamArgs::new(user_id.clone());
                let req_id = client.create_history_stream_req_send(&args).await?;
                print_source_until_eof(&mut client, req_id, feed_res_parse).await?;
            }
            ("feed", 1) => {
                let args = CreateStreamArgs::default();
                let req_id = client.send_create_feed_stream(&args).await?;
                print_source_until_eof(&mut client, req_id, feed_res_parse).await?;
            }
            ("latest", 1) => {
                let req_id = client.send_latest().await?;
                print_source_until_eof(&mut client, req_id, latest_res_parse).await?;
            }
            ("private", 2) => {
                let user_id = if args[1] == "me" { &whoami } else { &args[1] };

                let show_private = |body: &[u8]| {
                    let msg = feed_res_parse(body)?.into_message()?;
                    if let serde_json::Value::String(content) = msg.content() {
                        if is_privatebox(&content) {
                            let ret = privatebox_decipher(&content, &sk)?.unwrap_or("".to_string());
                            return Ok(ret);
                        }
                    }
                    return Ok("".to_string());
                };

                let args = CreateHistoryStreamArgs::new(user_id.clone());
                let req_id = client.create_history_stream_req_send(&args).await?;

                print_source_until_eof(&mut client, req_id, show_private).await?;
            }
            _ => println!("unknown command {}", line_buffer),
        }
        line_buffer.clear();
    }
    Ok(())
}
