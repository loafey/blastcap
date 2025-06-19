#![feature(impl_trait_in_bindings, never_type)]
mod args;
mod network;

use tokio::runtime::Runtime;

use crate::network::{
    HostPoll, NetworkClient, NetworkHost, TICK_RATE,
    messages::{ClientRequest, ServerMessage},
};

#[allow(unused)]
async fn old_main() -> anyhow::Result<()> {
    for i in 0..10 {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(0.5)).await;
            let mut client = NetworkClient::tcp("localhost:4000").await.unwrap();
            let mut tick_counter: usize = 0;
            while let Ok(res) = client.poll().await {
                match res {
                    network::ClientPoll::Message(client_message) => {
                        // println!("CLIENT {i} - Got message {client_message:?}")
                    }
                    network::ClientPoll::Tick => {
                        tick_counter = tick_counter.wrapping_add(1);

                        if tick_counter % (TICK_RATE * rand::random_range(2..=7)) == 0 {
                            // println!("CLIENT {i} - sending message!");
                            client
                                .send(ClientRequest::ChatMessage(format!(
                                    "I am stinky {i}: {tick_counter}!"
                                )))
                                .await
                                .unwrap();
                            if rand::random_range(0..5) == 0 {
                                // println!("CLIENT {i} - I am leaving");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    Ok(())
}

async fn host(port: u16) -> anyhow::Result<()> {
    let mut host = NetworkHost::tcp(port).await.unwrap();

    let mut tick_counter: usize = 0;
    let mut tick_time = std::time::Instant::now();
    while let Ok(res) = host.poll().await {
        match res {
            HostPoll::ClientConnected(socket_addr) => {
                println!("SERVER - A user at {socket_addr} connected")
            }
            HostPoll::ClientRequest { addr, req } => match req {
                ClientRequest::Ping => {
                    let clients = host.get_clients();
                    host.send(addr, ServerMessage::Pong(clients)).await?
                }
                ClientRequest::ChatMessage(msg) => {
                    host.broadcast(ServerMessage::ChatMessage(addr, msg))
                        .await?;
                }
            },
            HostPoll::Tick => {
                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter % (TICK_RATE - 1) == 0 {
                    let metrics = tokio::runtime::Handle::current().metrics();
                    println!(
                        "SERVER - time since last check: {:0.03?}s, {} client(s) connected\n\t queue depth: {}, alive tasks: {}, workers: {}",
                        tick_time.elapsed().as_secs_f64(),
                        host.get_client_count(),
                        metrics.global_queue_depth(),
                        metrics.num_alive_tasks(),
                        metrics.num_workers(),
                    );
                    tick_time = std::time::Instant::now();
                }
            }
            HostPoll::RemoveClient(socket_addr) => host.remove_client(socket_addr),
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_| Ok(Box::<Game>::default())),
    )
    .unwrap();

    Ok(())
}

struct Game {
    socket_addr: String,
    port: u16,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            socket_addr: "localhost:4000".to_string(),
            port: 4000,
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let name_label = ui.label("SocketAddr: ");
                ui.text_edit_singleline(&mut self.socket_addr)
                    .labelled_by(name_label.id);
                if ui.button("Connect").clicked() {
                    println!("connecting to server!")
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.port, 1000..=u16::MAX).text("age"));
                if ui.button("Host").clicked() {
                    let port = self.port;
                    std::thread::spawn(move || {
                        let rt = Runtime::new().expect("Unable to create Runtime");
                        let _enter = rt.enter();
                        rt.block_on(host(port)).unwrap();
                    });
                }
            });
        });
    }
}
