#![feature(
    impl_trait_in_bindings,
    never_type,
    vec_into_raw_parts,
    arbitrary_self_types_pointers
)]

use crate::{
    game::{
        Arg, ServerData,
        state::{LobbyState, State},
    },
    network::{
        ClientPoll, Metadata, NetworkClient, NetworkHost,
        messages::{ClientRequest, ServerMessage},
    },
};
use std::ffi::{CStr, CString};
use tokio::{
    net::ToSocketAddrs,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
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
    let Some(mut metadata) = Metadata::grab_host() else {
        panic!("something else has claimed metadata")
    };
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _enter = rt.enter();
        let Err(err): anyhow::Result<()> = rt.block_on(async move {
            let mut host = NetworkHost::tcp(port).await?;
            let mut data = ServerData::default();
            let mut state: Box<dyn State> = LobbyState::new();
            let mut last_tick = Instant::now();
            loop {
                let Ok(poll) = host.poll().await else {
                    break;
                };
                metadata.tick().await?;
                if let Some(new_state) = state
                    .handle_req(
                        poll,
                        Arg {
                            data: &mut data,
                            host: &mut host,
                            last_tick: &mut last_tick,
                            metadata: &mut metadata,
                        },
                    )
                    .await?
                {
                    state = new_state;
                }
            }
            Ok(())
        }) else {
            return;
        };
        unsafe {
            let str = CString::new(format!("{err}")).unwrap().into_raw();
            on_fail(str);
            _ = CString::from_raw(str)
        };
    });
}

include!("lib_gen.rs");

include!("lib_poll.rs");

// ///
// /// # Safety
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn create_client_handle(
//     on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
// ) -> *mut ClientHandle {
// }

pub struct ClientHandle {
    recv: Receiver<ServerMessage>,
    send: Sender<ClientRequest>,
    server_send: Option<Sender<ServerMessage>>,
    client_recv: Option<Receiver<ClientRequest>>,
    on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
}
impl ClientHandle {
    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn create_client(
        on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
    ) -> *mut Self {
        let (server_send, server_recv) = tokio::sync::mpsc::channel(1000);
        let (client_send, client_recv) = tokio::sync::mpsc::channel(1000);
        Metadata::init_tcp();
        Box::leak(Box::new(ClientHandle {
            recv: server_recv,
            send: client_send,
            on_fail,
            server_send: Some(server_send),
            client_recv: Some(client_recv),
        }))
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn client_drop_handle(self: *mut Self) {
        unsafe {
            println!("CLIENT - being dropped!");
            drop(Box::from_raw(self));
        }
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn is_connected(self: *mut Self) -> std::ffi::c_int {
        let client = unsafe { &mut *self } as &mut ClientHandle;
        if client.client_recv.is_some() { 0 } else { 1 }
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn start_client_loop(self: *mut Self, addr: *const std::ffi::c_char) {
        let client = unsafe { &mut *self } as &mut ClientHandle;

        async fn client_func<A: ToSocketAddrs + std::fmt::Debug>(
            addr: A,
            server_send: Sender<ServerMessage>,
            mut client_req_recv: Receiver<ClientRequest>,
        ) -> anyhow::Result<()> {
            println!("CLIENT - connecting to {addr:?}");
            println!(
                "Has metadata been taken already: {}",
                Metadata::grab_client().await.is_none()
            );
            let mut client = NetworkClient::tcp(addr).await?;
            let mut tick_counter: usize = 0;
            loop {
                let poll = tokio::select! {
                    res = client.poll() => {
                        let Ok(res) = res else { break };
                        Some(res)
                    }
                    msg = client_req_recv.recv() => {
                        let Some(msg) = msg else { break };
                        client.send(msg).await?;
                        None
                    }
                };
                let Some(res) = poll else { continue };
                match res {
                    ClientPoll::Message(client_message) => server_send.send(client_message).await?,
                    ClientPoll::Tick => tick_counter = tick_counter.wrapping_add(1),
                }
            }
            Ok(())
        }
        let addr = unsafe { CStr::from_ptr(addr) }
            .to_string_lossy()
            .to_string();
        let on_fail = client.on_fail;
        let (Some(server_send), Some(client_recv)) =
            (client.server_send.take(), client.client_recv.take())
        else {
            return;
        };
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
            let _enter = rt.enter();
            let Err(err) = rt.block_on(client_func(addr, server_send, client_recv)) else {
                return;
            };
            unsafe {
                let str = CString::new(format!("{err}")).unwrap().into_raw();
                on_fail(str);
                _ = CString::from_raw(str)
            };
        });
    }
}
