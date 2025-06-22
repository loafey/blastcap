use crate::gui::GuiState;
// use blastcap::network::messages::{ClientRequest, ServerMessage};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ConnectedScreen {
    // pub recv: Receiver<ServerMessage>,
    // pub send: Sender<ClientRequest>,
    pub msgs: Vec<String>,
    pub curr_message: String,
    pub server_stats: Option<(usize, f32)>,
}
impl GuiState for ConnectedScreen {
    fn draw(&mut self, ctx: &egui::Context, _new_state: &mut Option<Box<dyn GuiState>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // if let Ok(msg) = self.recv.try_recv() {
            //     match msg {
            //         ServerMessage::Pong(socket_addrs) => println!("{socket_addrs:?}"),
            //         ServerMessage::ChatMessage(addr, msg) => {
            //             self.msgs.push(format!("{addr}: {msg:?}"));
            //         }
            //         ServerMessage::NewUser(addr) => {
            //             self.msgs.push(format!("{addr} joined"));
            //         }
            //         ServerMessage::UserLeft(socket_addr) => {
            //             self.msgs.push(format!("{socket_addr} joined"));
            //         }
            //         ServerMessage::Status {
            //             user_count,
            //             tick_diff,
            //         } => self.server_stats = Some((user_count, tick_diff)),
            //     }
            // }

            if let Some((user_count, tick_diff)) = self.server_stats {
                ui.label(format!("Users: {user_count}, tick_diff: {tick_diff}"));
            }

            ui.horizontal(|ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut self.curr_message));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut msg = String::new();
                    std::mem::swap(&mut msg, &mut self.curr_message);
                    // self.send
                    //     .blocking_send(ClientRequest::ChatMessage(msg))
                    //     .unwrap();
                    response.request_focus();
                }
            });
            for i in (0..self.msgs.len()).rev() {
                ui.label(&self.msgs[i]);
            }
        });
        ctx.request_repaint_after(Duration::from_secs_f64(1.0 / 60.0));
    }
}
