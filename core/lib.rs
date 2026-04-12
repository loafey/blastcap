#![feature(
    impl_trait_in_bindings,
    never_type,
    arbitrary_self_types_pointers,
    macro_metavar_expr,
    try_blocks
)]
#![warn(clippy::print_stdout, clippy::print_stderr)]

#[macro_use]
extern crate tracing;

pub use random::bindings as __random_bindings;
use smol::channel;
use smol_concurrency_tools::select;

use crate::{
    game::{
        Arg, ServerData,
        state::{LobbyState, State},
    },
    game_data::DATA,
    network::{
        ClientPoll,
        messages::{ClientRequest, ServerMessage},
        metadata, metadata_block,
    },
};
use std::{
    ffi::{CStr, CString},
    time::{Duration, Instant},
};

mod game;
mod game_data;
mod network;

sharpify::constants!(
    mod constants {
        pub const TILES_PER_SECOND: usize = 4;
    }
);

#[unsafe(no_mangle)]
pub extern "C" fn start_host_loop(on_fail: unsafe extern "C" fn(*const std::ffi::c_char)) {
    std::thread::spawn(move || {
        let Err(err): anyhow::Result<()> = smol::block_on(async move {
            let mut host = metadata(async |a| a.create_lobby().await).await?;
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
pub unsafe extern "C" fn register_logging(
    print: unsafe extern "C" fn(*const std::ffi::c_char),
    print_error: unsafe extern "C" fn(*const std::ffi::c_char),
) {
    struct CustomWriter {
        print: unsafe extern "C" fn(*const std::ffi::c_char),
        print_error: unsafe extern "C" fn(*const std::ffi::c_char),
    }
    impl std::io::Write for CustomWriter {
        #[allow(clippy::print_stderr)]
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let text = String::from_utf8_lossy(buf).to_string();
            let stripped = format!("🦀 {}", strip_ansi_escapes::strip_str(&text).trim());
            let is_err = stripped
                .split_whitespace()
                .nth(2)
                .map(|s| matches!(s, "WARN" | "ERROR" | "DEBUG"))
                .unwrap_or_default();
            let Ok(cstr) = CString::new(stripped) else {
                eprintln!("trace contained invalid chars: {text}");
                return Ok(0);
            };
            unsafe {
                if is_err {
                    (self.print_error)(cstr.as_ptr());
                } else {
                    (self.print)(cstr.as_ptr());
                }
            }
            Ok(text.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || CustomWriter { print, print_error })
        .with_env_filter("none,blastcap=trace")
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    info!("logging running");

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
        #[allow(clippy::print_stderr)]
        unsafe {
            let Ok(cstr) = CString::new(final_string.clone()) else {
                eprintln!("= Error string contained multiple null!");
                eprintln!("{final_string}");
                let err_str =
                    c"Error contains multiple null! Please look at the terminal output!".as_ptr();
                print_error(err_str);
                return;
            };
            let raw = cstr.as_ptr();
            print_error(raw);
        }
    }));
}
pub struct ClientHandle {
    recv: channel::Receiver<ServerMessage>,
    send: channel::Sender<ClientRequest>,
    server_send: Option<channel::Sender<ServerMessage>>,
    client_recv: Option<channel::Receiver<ClientRequest>>,
    on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
}
impl ClientHandle {
    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn create_client(
        on_fail: unsafe extern "C" fn(*const std::ffi::c_char),
    ) -> *mut Self {
        let (server_send, server_recv) = channel::unbounded();
        let (client_send, client_recv) = channel::unbounded();
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
            warn!("CLIENT - being dropped!");
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
            server_send: channel::Sender<ServerMessage>,
            client_req_recv: channel::Receiver<ClientRequest>,
        ) -> anyhow::Result<()> {
            trace!("CLIENT - connecting to {addr:?}");
            let mut client = metadata(async |m| m.create_client(0).await).await?;
            let mut tick_counter: usize = 0;
            loop {
                let poll = select!(
                    (client.poll(), |res| { Ok(res?) }),
                    (client_req_recv.recv(), |msg| { Err(msg?) })
                );
                match poll {
                    Ok(res) => match res {
                        ClientPoll::Message(client_message) => {
                            server_send.send(client_message).await?
                        }
                        ClientPoll::Tick => {
                            tick_counter = tick_counter.wrapping_add(1);
                        }
                    },
                    Err(msg) => client.send(msg).await?,
                }
            }
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
            let Err(err) = smol::block_on(client_func(addr, server_send, client_recv)) else {
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
        metadata_block(async |m| m.get_my_id())
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn metadata_get_name(self: *mut Self, id: u64) -> *mut i8 {
        let name = metadata_block(async move |m| m.get_name(id))
            .unwrap_or_else(|_| "unknown name".to_string());
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
        let Some((data, width, height)) = metadata_block(async move |m| m.get_avatar(id)) else {
            return;
        };

        let (ptr, length, _capacity) = data.into_raw_parts();
        callback(ptr, length as u32, width, height);
        // Vec::from_raw_parts(ptr, length, capacity);
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn load_game_content(self: *mut Self) {
        let client = unsafe { &mut *self } as &mut ClientHandle;
        let Some(channel) = client.server_send.as_mut() else {
            error!("failed to start loading game content!");
            return;
        };
        let channel = channel.clone();
        let send = async move |msg| {
            channel
                .send(msg)
                .await
                .map_err(|_| anyhow::Error::msg("failed sending to client"))
        };
        smol::spawn(async move {
            smol::Timer::after(Duration::from_secs(1)).await;
            let error: anyhow::Result<()> = try {
                let total = DATA.cards.len();
                send(ServerMessage::GameLoadingTotal(total)).await?;

                for (id, card) in DATA.cards.iter() {
                    send(ServerMessage::GameLoadingCard(*id, card.clone())).await?;
                }
            };
            if let Err(e) = error {
                error!("load failure: {e}!");
            }
        })
        .detach();
    }
}
