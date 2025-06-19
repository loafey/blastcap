use tokio::{runtime::Runtime, sync::mpsc::channel};

use crate::{
    game,
    gui::{GuiState, connected::ConnectedScreen},
};

pub struct MainMenuScreen {
    pub socket_addr: String,
    pub port: u16,
}
impl GuiState for MainMenuScreen {
    fn draw(&mut self, ctx: &egui::Context, new_state: &mut Option<Box<dyn GuiState>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let name_label = ui.label("SocketAddr: ");
                ui.text_edit_singleline(&mut self.socket_addr)
                    .labelled_by(name_label.id);
                if ui.button("Connect").clicked() {
                    let addr = self.socket_addr.clone();
                    let (send, recv) = channel(100);
                    let (client_send, client_recv) = channel(100);
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        let rt = Runtime::new().expect("Unable to create Runtime");
                        let _enter = rt.enter();
                        rt.block_on(game::client(addr, send, client_recv)).unwrap();
                    });
                    *new_state = Some(Box::new(ConnectedScreen {
                        recv,
                        send: client_send,
                        msgs: Default::default(),
                        curr_message: Default::default(),
                    }));
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.port, 1000..=u16::MAX).text("age"));
                if ui.button("Host").clicked() {
                    let port = self.port;
                    let (send, recv) = channel(100);
                    let (client_send, client_recv) = channel(100);

                    std::thread::spawn(move || {
                        let rt = Runtime::new().expect("Unable to create Runtime");
                        let _enter = rt.enter();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            let rt = Runtime::new().expect("Unable to create Runtime");
                            let _enter = rt.enter();
                            rt.block_on(game::client(
                                format!("localhost:{port}"),
                                send,
                                client_recv,
                            ))
                            .unwrap();
                        });
                        rt.block_on(game::host(port)).unwrap();
                    });
                    *new_state = Some(Box::new(ConnectedScreen {
                        send: client_send,
                        recv,
                        msgs: Default::default(),
                        curr_message: Default::default(),
                    }));
                }
            });
        });
    }
}
