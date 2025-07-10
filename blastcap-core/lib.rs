#![feature(impl_trait_in_bindings, never_type, vec_into_raw_parts)]

use crate::network::{
    ClientPoll, NetworkClient,
    messages::{ClientRequest, ServerMessage},
};
use std::ffi::{CStr, CString};
use tokio::{
    net::ToSocketAddrs,
    sync::mpsc::{Receiver, Sender},
};

mod game;
mod network;

sharpify::constants!(
    mod constants {
        pub const TILES_PER_SECOND: usize = 4;
    }
);

#[unsafe(no_mangle)]
pub extern "C" fn start_host_loop(
    port: u16,
    on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _enter = rt.enter();
        let Err(err) = rt.block_on(game::host_loop(port)) else {
            return;
        };
        unsafe {
            let str = CString::new(format!("{err}")).unwrap().into_raw();
            on_fail(str);
            _ = CString::from_raw(str)
        };
    });
}

pub struct ClientHandle {
    recv: Receiver<ServerMessage>,
    send: Sender<ClientRequest>,
    on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
}

include!("lib_gen.rs");

include!("lib_poll.rs");
///
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn client_drop_handle(ch: *mut ClientHandle) {
    unsafe {
        println!("CLIENT - being dropped!");
        drop(Box::from_raw(ch));
    }
}

///
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_client_loop(
    addr: *const std::ffi::c_char,
    on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
) -> *mut ClientHandle {
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
                    while let Ok(msg) = client_req_recv.try_recv() {
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
    let (send, recv) = tokio::sync::mpsc::channel(1000);
    let (client_send, client_recv) = tokio::sync::mpsc::channel(1000);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
        let _enter = rt.enter();
        let Err(err) = rt.block_on(client(addr, send, client_recv)) else {
            return;
        };
        unsafe {
            let str = CString::new(format!("{err}")).unwrap().into_raw();
            on_fail(str);
            _ = CString::from_raw(str)
        };
    });
    Box::leak(Box::new(ClientHandle {
        recv,
        send: client_send,
        on_fail,
    }))
}
