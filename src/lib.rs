#![feature(impl_trait_in_bindings, never_type)]

use crate::network::{
    ClientPoll, NetworkClient,
    messages::{ClientRequest, ServerMessage},
};
use std::ffi::CStr;
use tokio::{
    net::ToSocketAddrs,
    sync::mpsc::{Receiver, Sender},
};

mod args;
mod game;
mod network;

#[unsafe(no_mangle)]
pub extern "C" fn start_host_loop(port: u16) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
        let _enter = rt.enter();
        rt.block_on(game::host_loop(port)).unwrap();
    });
}

#[repr(C)]
pub struct ClientHandle {
    recv: Receiver<ServerMessage>,
    send: Sender<ClientRequest>,
}

include!("lib_gen.rs");

///
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_client_loop(addr: *const std::ffi::c_char) -> *mut ClientHandle {
    async fn client<A: ToSocketAddrs + std::fmt::Debug>(
        addr: A,
        server_send: Sender<ServerMessage>,
        mut client_req_recv: Receiver<ClientRequest>,
    ) -> anyhow::Result<()> {
        println!("CLIENT - connecting to {addr:?}");
        let mut client = NetworkClient::tcp(addr).await?;
        let mut tick_counter: usize = 0;
        while let Ok(res) = client.poll().await {
            match res {
                ClientPoll::Message(client_message) => server_send.send(client_message).await?,
                ClientPoll::Tick => {
                    tick_counter = tick_counter.wrapping_add(1);
                    if let Ok(msg) = client_req_recv.try_recv() {
                        client.send(msg).await?;
                    }
                }
            }
        }
        Ok(())
    }
    let addr = unsafe { CStr::from_ptr(addr) }
        .to_string_lossy()
        .to_string();
    let (send, recv) = tokio::sync::mpsc::channel(100);
    let (client_send, client_recv) = tokio::sync::mpsc::channel(100);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
        let _enter = rt.enter();
        rt.block_on(client(addr, send, client_recv)).unwrap();
    });
    Box::leak(Box::new(ClientHandle {
        recv,
        send: client_send,
    }))
}
