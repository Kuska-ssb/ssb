extern crate kuska_handshake;
extern crate kuska_ssb;

extern crate base64;
extern crate crossbeam;
extern crate regex;
extern crate structopt;

use std::fmt::Debug;
use std::io::prelude::*;

use async_std::io::Read;
use async_std::net::{TcpStream, UdpSocket};

use kuska_handshake::async_std::{handshake_client, BoxStream};
use kuska_ssb::api::{
    dto::{CreateHistoryStreamIn, CreateStreamIn, LatestOut, WhoAmIOut},
    ApiCaller,
};
use kuska_ssb::discovery::ssb_net_id;
use kuska_ssb::feed::{is_privatebox, privatebox_decipher, Feed, Message};
use kuska_ssb::keystore::from_patchwork_local;
use kuska_ssb::keystore::OwnedIdentity;
use kuska_ssb::rpc::{RecvMsg, RequestNo, RpcReader, RpcWriter};

use regex::Regex;
use sodiumoxide::crypto::sign::ed25519;
use structopt::StructOpt;

type SolarResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Connect to server
    // format is: server:port:<server_id>
    #[structopt(short, long)]
    connect: Option<String>,
}

pub fn whoami_res_parse(body: &[u8]) -> SolarResult<WhoAmIOut> {
    Ok(serde_json::from_slice(body)?)
}
pub fn message_res_parse(body: &[u8]) -> SolarResult<Message> {
    Ok(Message::from_slice(body)?)
}
pub fn feed_res_parse(body: &[u8]) -> SolarResult<Feed> {
    Ok(Feed::from_slice(&body)?)
}
pub fn latest_res_parse(body: &[u8]) -> SolarResult<LatestOut> {
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

async fn get_async<'a, R, T, F>(
    rpc_reader: &mut RpcReader<R>,
    req_no: RequestNo,
    f: F,
) -> SolarResult<T>
where
    R: Read + Unpin,
    F: Fn(&[u8]) -> SolarResult<T>,
    T: Debug,
{
    loop {
        let (id, msg) = rpc_reader.recv().await?;
        if id == req_no {
            match msg {
                RecvMsg::RpcResponse(_type, body) => {
                    return f(&body).map_err(|err| err.into());
                }
                RecvMsg::ErrorResponse(message) => {
                    return std::result::Result::Err(Box::new(AppError::new(message)));
                }
                _ => { }
            }
        }
    }
}

async fn print_source_until_eof<'a, R, T, F>(
    rpc_reader: &mut RpcReader<R>,
    req_no: RequestNo,
    f: F,
) -> SolarResult<()>
where
    R: Read + Unpin,
    F: Fn(&[u8]) -> SolarResult<T>,
    T: Debug + serde::Deserialize<'a>,
{
    loop {
        let (id, msg) = rpc_reader.recv().await?;
        if id == req_no {
            match msg {
                RecvMsg::RpcResponse(_type, body) => {
                    let display = f(&body)?;
                    println!("{:?}", display);
                }
                RecvMsg::ErrorResponse(message) => {
                    return std::result::Result::Err(Box::new(AppError::new(message)));
                }
                RecvMsg::CancelStreamRespose() => break,
                _ => { }
            }
        }
    }
    Ok(())
}

#[async_std::main]
async fn main() -> SolarResult<()> {
    env_logger::init();
    log::set_max_level(log::LevelFilter::max());

    let OwnedIdentity { pk, sk, id } = from_patchwork_local().await.expect("read local secret");
    println!("connecting with identity {}", id);

    let opt = Opt::from_args();
    let (ip, port, server_pk) = if let Some(connect) = opt.connect {
        let connect: Vec<_> = connect.split(":").collect();
        if connect.len() != 3 {
            panic!("connection string should be server:port:id");
        }
        (
            connect[0].to_string(),
            connect[1].to_string(),
            connect[2].to_string(),
        )
    } else {
        println!("Waiting server broadcast...");

        let socket = UdpSocket::bind("0.0.0.0:8008").await?;
        socket.set_broadcast(true)?;
        let mut buf = [0; 128];
        let (amt, _) = socket.recv_from(&mut buf).await.unwrap();

        let msg = String::from_utf8(buf[..amt].to_vec())?;

        println!("got broadcasted {}", msg);
        let broadcast_regexp =
            r"net:([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+):([0-9]+)~shs:([0-9a-zA-Z=/]+)";
        let captures = Regex::new(broadcast_regexp)
            .unwrap()
            .captures(&msg)
            .unwrap();
        (
            captures[1].to_string(),
            captures[2].to_string(),
            captures[3].to_string(),
        )
    };

    let server_pk =
        ed25519::PublicKey::from_slice(&base64::decode(&server_pk)?).expect("bad public key");
    let server_ipport = format!("{}:{}", ip, port);

    println!("server_ip_port={}", server_ipport);

    let mut socket = TcpStream::connect(server_ipport).await?;

    let handshake = handshake_client(&mut socket, ssb_net_id(), pk, sk.clone(), server_pk).await?;

    println!("💃 handshake complete");

    let (box_stream_read, box_stream_write) =
        BoxStream::from_handshake(&socket, &socket, handshake, 0x8000).split_read_write();

    let mut rpc_reader = RpcReader::new(box_stream_read);
    let mut client = ApiCaller::new(RpcWriter::new(box_stream_write));

    let req_id = client.whoami_req_send().await?;
    let whoami = match get_async(&mut rpc_reader, req_id, whoami_res_parse).await {
       Ok(res) => {
    	  println!("😊 server says hello to {}", res.id);
  	  id
       }
       Err(err) => {
          if !err.to_string().contains("method:whoami is not in list of allowed methods") {
          	println!("Cannot ask for whoami {}",err);
	  }
      	  id  
       }
    };


    loop {
    	let mut line_buffer = String::new();
        print!("> "); std::io::stdout().flush();
        match std::io::stdin().read_line(&mut line_buffer) {
	    Err(_) => break,
            _ => { }
        };
        let args: Vec<String> = line_buffer
            .replace("\n", "")
            .split_whitespace()
            .map(|arg| arg.to_string())
            .collect();
       
        if args.len() == 0 {
	    continue;
        }
        match (args[0].as_str(), args.len()) {
            ("exit", 1) => {
                client.rpc().close().await?;
                break;
            }
            ("whoami", 1) => {
                let req_id = client.whoami_req_send().await?;
                let whoami = get_async(&mut rpc_reader, req_id, whoami_res_parse)
                    .await?
                    .id;
                println!("{}", whoami);
            }
            ("get", 2) => {
                let msg_id = if args[1] == "any" {
                    "%TL34NIX8JpMJN+ubHWx6cRhIwEal8VqHdKVg2t6lFcg=.sha256".to_string()
                } else {
                    args[1].clone()
                };
                let req_id = client.get_req_send(&msg_id).await?;
                let msg = get_async(&mut rpc_reader, req_id, message_res_parse).await?;
                println!("{:?}", msg);
            }
            ("user", 2) => {
                let user_id = if args[1] == "me" { &whoami } else { &args[1] };

                let args = CreateHistoryStreamIn::new(user_id.clone());
                let req_id = client.create_history_stream_req_send(&args).await?;
                print_source_until_eof(&mut rpc_reader, req_id, feed_res_parse).await?;
            }
            ("feed", 1) => {
                let args = CreateStreamIn::default();
                let req_id = client.create_feed_stream_req_send(&args).await?;
                print_source_until_eof(&mut rpc_reader, req_id, feed_res_parse).await?;
            }
            ("latest", 1) => {
                let req_id = client.latest_req_send().await?;
                print_source_until_eof(&mut rpc_reader, req_id, latest_res_parse).await?;
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

                let args = CreateHistoryStreamIn::new(user_id.clone());
                let req_id = client.create_history_stream_req_send(&args).await?;

                print_source_until_eof(&mut rpc_reader, req_id, show_private).await?;
            }
            _ => println!("unknown command {}", line_buffer),
        }
        line_buffer.clear();
    }
    Ok(())
}
