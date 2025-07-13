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
        ClientPoll, Metadata, MetadataTask, NetworkClient, NetworkHost,
        messages::{ClientRequest, ServerMessage},
    },
};
use std::ffi::{CStr, CString};
use tokio::{
    net::ToSocketAddrs,
    sync::{
        mpsc::{Receiver, Sender},
        oneshot::channel as oneshot,
    },
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
    let (mut metadata, mut recv) = Metadata::grab_host();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _enter = rt.enter();
        let Err(err): anyhow::Result<()> = rt.block_on(async move {
            let mut host = NetworkHost::create().await?;
            let mut data = ServerData::default();
            let mut state: Box<dyn State> = LobbyState::new();
            let mut last_tick = Instant::now();
            loop {
                let poll = tokio::select! {
                    poll = host.poll() => poll,
                    task = recv.recv() => {
                        let Some(task) = task else { continue };
                        task(&metadata)?;
                        continue;
                    }
                };
                let Ok(poll) = poll else { break };
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
    metadata_send: Sender<MetadataTask>,
    metadata_recv: Option<Receiver<MetadataTask>>,
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
        let (metadata_send, metadata_recv) = tokio::sync::mpsc::channel(10);
        Metadata::init();
        Box::leak(Box::new(ClientHandle {
            recv: server_recv,
            send: client_send,
            on_fail,
            server_send: Some(server_send),
            client_recv: Some(client_recv),
            metadata_send,
            metadata_recv: Some(metadata_recv),
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
            mut metadata_req_recv: Receiver<MetadataTask>,
        ) -> anyhow::Result<()> {
            println!("CLIENT - connecting to {addr:?}");
            let m_holder = Metadata::grab_client().await;
            println!("Metadata status: {m_holder:?}",);
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
                    task = metadata_req_recv.recv() => {
                        let Some(task) = task else { break };
                        m_holder.act(task).await?;
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
        let Some(metadata_task_recv) = client.metadata_recv.take() else {
            return;
        };
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
            let _enter = rt.enter();
            let Err(err) = rt.block_on(client_func(
                addr,
                server_send,
                client_recv,
                metadata_task_recv,
            )) else {
                return;
            };
            unsafe {
                let str = CString::new(format!("{err}")).unwrap().into_raw();
                on_fail(str);
                _ = CString::from_raw(str)
            };
        });
    }

    pub fn metadata<T: Send + 'static, F: FnOnce(&Metadata) -> T + Send + 'static>(
        &self,
        f: F,
    ) -> anyhow::Result<T> {
        let (send, recv) = oneshot();
        if let Some(m) = unsafe { Metadata::peek() } {
            _ = send.send(f(m));
        } else {
            _ = self.metadata_send.blocking_send(Box::new(move |m| {
                _ = send.send(f(m));
                Ok(())
            }));
        }
        Ok(recv.blocking_recv()?)
    }

    ///
    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn get_my_name(self: *mut Self) -> *mut i8 {
        let client = unsafe { &mut *self } as &mut ClientHandle;

        let Ok(Ok(name)) = client.metadata(|m| m.get_my_name()) else {
            return std::ptr::null_mut();
        };
        let Ok(str) = CString::new(name) else {
            return std::ptr::null_mut();
        };
        str.into_raw()
    }
}
