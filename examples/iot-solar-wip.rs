#[macro_use]
extern crate lazy_static;

use async_std::sync::{Receiver,Sender,channel};
use async_std::task;
use std::fmt::Debug;
use std::cell::{RefCell};
use std::sync::{Arc,Mutex};

use std::time::Duration;  
use signal_hook::{iterator::Signals, SIGTERM, SIGINT, SIGHUP, SIGQUIT};

use actix_web::{get, web, App, HttpServer, Responder};

use async_std::io;
use async_std::io::{Read,Write};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};

use kuska_handshake::async_std::{BoxStream,handshake_client,TokioCompatExt,TokioCompatExtRead,TokioCompatExtWrite};
use kuska_ssb::rpc::{Header,RequestNo,RpcClient};
use kuska_ssb::util::to_ioerr;
use kuska_ssb::patchwork::*;

use tokio::net::TcpStream;
use tokio::net::tcp::{ReadHalf,WriteHalf};

async fn get_async<'a,R,W,T,F> (client: &mut ApiClient<R,W>, req_no : RequestNo, f : F) -> io::Result<T>
where
    R: Read+Unpin,
    W: Write+Unpin,
    F: Fn(&Header,&Vec<u8>)->io::Result<T>,
    T: Debug
{
    loop {
        let (header,body) = client.rpc().recv().await?;
        if header.req_no == req_no {
            return f(&header,&body);
        }
    }
}

async fn run_task<R:Read+Unpin,W:Write+Unpin>(api : &mut ApiClient<R,W>, command: &str) -> Result<bool,io::Error> {
  let req_id = api.send_whoami().await?;
  let whoami = get_async(api,-req_id,parse_whoami).await?.id;

  println!("{}",whoami);

  Ok(false)
}

async fn sync_loop(command_receiver: Receiver<String>, stop_receiver : Receiver<bool>) -> Result<(),io::Error>{
  
  /*
    sync loop functionality
    friend_list = []
    discovered_peers = []
    connected_peers =  []
    loop {
      foreach peer in discoverd_peer not in conected_peers {
        peer.handshake.createboxstream
        peer.feed_callback = { |msg| 
          process_feed_message(msg)
        }
        for each friend in friend_list {
          peer.createUserStream(friend)
        }
        conected_peers.push(peer)
      }
    } 
  */


  let IdentitySecret{pk,sk,..} = IdentitySecret::from_local_config()
  .expect("read local secret");
  
  let tokio_socket : TcpStream = TcpStream::connect("127.0.0.1:8008").await?;
  let asyncstd_socket = TokioCompatExt::wrap(tokio_socket);

  let (asyncstd_socket,handshake) = handshake_client(asyncstd_socket, ssb_net_id(), pk, sk.clone(), pk).await
    .map_err(to_ioerr)?;

  println!("💃 handshake complete");
 
  let mut tokio_socket = asyncstd_socket.into_inner();
  let (read,write) = tokio_socket.split();

  let read = TokioCompatExtRead::wrap(read);
  let write = TokioCompatExtWrite::wrap(write);

  let (box_stream_read, box_stream_write) =
    BoxStream::from_handhake(read, write, handshake, 0x8000)
    .split_read_write();

  let rpc = RpcClient::new(box_stream_read, box_stream_write);
  let mut api = ApiClient::new(rpc);

  let mut commands_queue : Vec<String> = Vec::new();

  loop {

    if !stop_receiver.is_empty() {
      stop_receiver.recv().await;
      println!("finished loop");
      return Ok(());
    }
    
    // read all pending requests
    while !command_receiver.is_empty() {
      if let Some(msg) = command_receiver.recv().await {
        commands_queue.push(msg);
      }
    }

    if let Some(command) = commands_queue.pop() {
      run_task(&mut api,&command).await?;
    } else {
      task::sleep(Duration::from_secs(1)).await;
      println!("waiting!");  
    }

  }

}


async fn command_loop(command_receiver: Receiver<String>, stop_receiver : Receiver<bool>) -> Result<(),io::Error>{
  
  let IdentitySecret{pk,sk,..} = IdentitySecret::from_local_config()
  .expect("read local secret");
  
  let tokio_socket : TcpStream = TcpStream::connect("127.0.0.1:8008").await?;
  let asyncstd_socket = TokioCompatExt::wrap(tokio_socket);

  let (asyncstd_socket,handshake) = handshake_client(asyncstd_socket, ssb_net_id(), pk, sk.clone(), pk).await
    .map_err(to_ioerr)?;

  println!("💃 handshake complete");
 
  let mut tokio_socket = asyncstd_socket.into_inner();
  let (read,write) = tokio_socket.split();

  let read = TokioCompatExtRead::wrap(read);
  let write = TokioCompatExtWrite::wrap(write);

  let (box_stream_read, box_stream_write) =
    BoxStream::from_handhake(read, write, handshake, 0x8000)
    .split_read_write();

  let rpc = RpcClient::new(box_stream_read, box_stream_write);
  let mut api = ApiClient::new(rpc);

  let mut commands_queue : Vec<String> = Vec::new();

  loop {

    if !stop_receiver.is_empty() {
      stop_receiver.recv().await;
      println!("finished loop");
      return Ok(());
    }
    
    // read all pending requests
    while !command_receiver.is_empty() {
      if let Some(msg) = command_receiver.recv().await {
        commands_queue.push(msg);
      }
    }

    if let Some(command) = commands_queue.pop() {
      run_task(&mut api,&command).await?;
    } else {
      task::sleep(Duration::from_secs(1)).await;
      println!("waiting!");  
    }

  }

}

lazy_static! {
  static ref COMMAND_SENDER : Arc<Mutex<RefCell<Option<Sender<String>>>>> = Arc::new(Mutex::new(RefCell::new(None)));
}


#[get("/{id}/{name}/index.html")]
async fn index(info: web::Path<(u32, String)>) -> impl Responder {
  COMMAND_SENDER.lock().unwrap().borrow().as_ref().unwrap().send("hola".to_owned()).await;
  format!("Hello {}! id:{}", info.1, info.0)
}


async fn web_handler() -> std::io::Result<()> {
  HttpServer::new(|| App::new().service(index))
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn sigterm_handler(stop_sender : Sender<bool>, count : usize, ) {
  let signals = Signals::new(&[SIGTERM, SIGHUP, SIGINT, SIGQUIT]).expect("cannot capture SIGTERM");
  loop {
    if signals.pending().next().is_some() {
      for _ in 0..count {
        stop_sender.send(true).await;
      }
      return;
    }
    task::sleep(Duration::from_secs(1)).await;  
  }
}
 
#[actix_rt::main]
async fn main() {
  println!("started");

  let (stop_sender, stop_receiver) = channel::<bool>(1);
  let (command_sender, command_receiver) = channel::<String>(1);
  COMMAND_SENDER.lock().unwrap().replace(Some(command_sender));

  let future_sigterm = sigterm_handler(stop_sender,1);
  let future_loop = command_loop(command_receiver,stop_receiver.clone());
  let future_web = web_handler();

  let (_,loop_res,_) = futures::join!(future_sigterm,future_loop,future_web);
  loop_res.expect("main loop failed");
}

