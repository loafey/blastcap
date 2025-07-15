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
        ClientPoll, NetworkClient, NetworkHost,
        messages::{ClientRequest, ServerMessage},
        metadata, metadata_block,
    },
};
use std::ffi::{CStr, CString};
use tokio::{
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
pub extern "C" fn start_host_loop(on_fail: unsafe extern "C" fn(*const std::ffi::c_char)) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _enter = rt.enter();
        let Err(err): anyhow::Result<()> = rt.block_on(async move {
            let mut host = NetworkHost::create().await?;
            let mut data = ServerData::default();
            let mut state: Box<dyn State> = LobbyState::new();
            let mut last_tick = Instant::now();
            loop {
                let Ok(poll) = host.poll().await else { break };
                if let Some(new_state) = state
                    .handle_req(
                        poll,
                        Arg {
                            data: &mut data,
                            host: &mut host,
                            last_tick: &mut last_tick,
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

///
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_panic_callback(
    callback: unsafe extern "C" fn(*const std::ffi::c_char),
) {
    std::panic::set_hook(Box::new(move |e| {
        let thread = std::thread::current()
            .name()
            .map(|a| a.to_string())
            .unwrap_or("<unnamed>".to_string());
        let thread = format!("'{thread}'(id: {:?})", std::thread::current().id());
        let location = e
            .location()
            .map(|e| format!("{}:{}:{}", e.file(), e.line(), e.column()))
            .unwrap_or("unknown location".to_string());
        let payload = if let Some(s) = e.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = e.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown error".to_string()
        };
        let payload = payload
            .lines()
            .map(|a| format!("    {a}"))
            .collect::<Vec<_>>()
            .join("\n");
        let top = format!("thread {thread} panicked");
        let location = format!("crashed at {location}:");
        let end = if payload.contains("Stack backtrace:") {
            ""
        } else {
            "\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
        };
        let final_string = format!(
            "{0}\n{top}\n{location}\n{payload}{end}",
            "=".repeat(payload.lines().map(|l| l.len()).max().unwrap_or(4).min(60))
        );
        unsafe {
            let Ok(cstr) = CString::new(final_string.clone()) else {
                eprintln!("= Error string contained multiple null!");
                eprintln!("{final_string}");
                let err_str =
                    c"Error contains multiple null! Please look at the terminal output!".as_ptr();
                callback(err_str);
                return;
            };
            let raw = cstr.as_ptr();
            callback(raw);
        }
    }));
}
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

        async fn client_func(
            addr: String,
            server_send: Sender<ServerMessage>,
            mut client_req_recv: Receiver<ClientRequest>,
        ) -> anyhow::Result<()> {
            println!("CLIENT - connecting to {addr:?}");
            let mut client = NetworkClient::create(addr).await?;
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
                    ClientPoll::Tick => {
                        tick_counter = tick_counter.wrapping_add(1);
                    }
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

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn metadata_get_id(self: *mut Self) -> u64 {
        metadata_block(|m| m.get_my_id())
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn metadata_get_name(self: *mut Self, id: u64) -> *mut i8 {
        let name =
            metadata_block(move |m| m.get_name(id)).unwrap_or_else(|_| "unknown name".to_string());
        let Ok(str) = CString::new(name) else {
            return std::ptr::null_mut();
        };
        str.into_raw()
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn metadata_get_avatar(
        self: *mut Self,
        id: u64,
        callback: extern "C" fn(*const u8, u32, u16, u16),
    ) {
        let Some((data, width, height)) = metadata_block(move |m| m.get_avatar(id)) else {
            return;
        };

        let (ptr, length, _capacity) = data.into_raw_parts();
        callback(ptr, length as u32, width, height);
        // Vec::from_raw_parts(ptr, length, capacity);
    }
}
