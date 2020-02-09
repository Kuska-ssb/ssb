extern crate kuska_handshake;
extern crate kuska_ssb;

extern crate base64;
extern crate crossbeam;

use std::fmt::Debug;

use async_std::io::{Read,Write};
use async_std::net::TcpStream;

use kuska_handshake::async_std::{handshake_client,BoxStream};
use kuska_ssb::rpc::{Header,RequestNo,RpcClient};
use kuska_ssb::patchwork::*;
use kuska_ssb::feed::{is_privatebox,privatebox_decipher};

type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn get_async<'a,R,W,T,F> (client: &mut ApiClient<R,W>, req_no : RequestNo, f : F) -> AnyResult<T>
where
    R: Read+Unpin,
    W: Write+Unpin,
    F: Fn(&Header,&[u8])->Result<T>,
    T: Debug
{
    loop {
        let (header,body) = client.rpc().recv().await?;
        if header.req_no == req_no {
            return f(&header,&body).map_err(|err| err.into());
        }
    }
}

async fn print_source_until_eof<'a,R,W,T,F> (client: &mut ApiClient<R,W>, req_no : RequestNo, f : F) -> AnyResult<()>
where
    R: Read+Unpin,
    W: Write+Unpin,
    F: Fn(&Header,&[u8])->Result<T>,
    T: Debug+serde::Deserialize<'a>
{
    loop {
        let (header,body) = client.rpc().recv().await?;
        if header.req_no == req_no {
            if !header.is_end_or_error {
                match f(&header,&body) {
                    Ok(res) => { println!("{:?}",res) },
                    Err(err) => println!(" 😢 Failed :( {:?} {}",err,String::from_utf8_lossy(&body)),
                }
            } else {
                println!("STREAM FINISHED");
                return Ok(())
            }
        }
    }
}

#[async_std::main]
async fn main() -> AnyResult<()> {
    env_logger::init();
    log::set_max_level(log::LevelFilter::max());

    let IdentitySecret{pk,sk,..} = IdentitySecret::from_local_config()
        .expect("read local secret");

    let mut socket = TcpStream::connect("127.0.0.1:8008").await?;

    let handshake = handshake_client(&mut socket, ssb_net_id(), pk, sk.clone(), pk).await?;

    println!("💃 handshake complete");

    let (box_stream_read, box_stream_write) =
        BoxStream::from_handshake(&socket,&socket,handshake, 0x8000)
        .split_read_write();

    let mut client = ApiClient::new(RpcClient::new(box_stream_read, box_stream_write));

    let req_id = client.send_whoami().await?;    
    let whoami = get_async(&mut client,-req_id,parse_whoami).await?.id;

    println!("😊 server says hello to {}",whoami);

    let mut line_buffer = String::new();
    while let Ok(_) = std::io::stdin().read_line(&mut line_buffer) {

        let args : Vec<String> = line_buffer
            .replace("\n", "")
            .split_whitespace()
            .map(|arg| arg.to_string())
            .collect();

        match (args[0].as_str(), args.len()) {
            ("exit",1) => {
                client.rpc().close().await?;
                break;
            }
            ("get",2) => {
                let msg_id = if args[1] == "any" {
                    "%TL34NIX8JpMJN+ubHWx6cRhIwEal8VqHdKVg2t6lFcg=.sha256".to_string()
                } else {
                    args[1].clone()
                };
                let req_id = client.send_get(&msg_id).await?;
                let msg = get_async(&mut client,-req_id,parse_message).await?;
                println!("{:?}",msg);
            }
            ("user",2) => {
                let user_id = if args[1] == "me" {
                    &whoami
                } else {
                    &args[1]
                };

                let args = CreateHistoryStreamArgs::new(&user_id);
                let req_id = client.send_create_history_stream(&args).await?;
                print_source_until_eof(&mut client, -req_id, parse_feed).await?;
            }
            ("feed",1) => {
                let args = CreateStreamArgs::default();
                let req_id = client.send_create_feed_stream(&args).await?;
                print_source_until_eof(&mut client, -req_id, parse_feed).await?;
            }
            ("latest",1) => {
                let req_id = client.send_latest().await?;
                print_source_until_eof(&mut client, -req_id, parse_latest).await?;
            }
            ("private",2) => {
                let user_id = if args[1] == "me" {
                    &whoami
                } else {
                    &args[1]
                };

                let show_private = |header: &Header, body: &[u8]| {
                    let msg = parse_feed(header,body)?.into_message()?;
                    if let serde_json::Value::String(content) = msg.content() {
                        if is_privatebox(&content) {
                            let ret = privatebox_decipher(&content, &sk)?
                                .unwrap_or("".to_string());
                            return Ok(ret);
                        }
                    }
                    return Ok("".to_string());
                };   

                let args = CreateHistoryStreamArgs::new(&user_id);
                let req_id = client.send_create_history_stream(&args).await?;

                print_source_until_eof(&mut client, -req_id, show_private).await?;
            }
            _ => println!("unknown command {}",line_buffer),
        }
        line_buffer.clear();
    }
    Ok(())
}