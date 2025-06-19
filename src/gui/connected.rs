use std::time::Duration;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    gui::GuiState,
    network::messages::{ClientRequest, ServerMessage},
};

pub struct ConnectedScreen {
    pub recv: Receiver<ServerMessage>,
    pub send: Sender<ClientRequest>,
    pub msgs: Vec<String>,
    pub curr_message: String,
}
impl GuiState for ConnectedScreen {
    fn draw(&mut self, ctx: &egui::Context, _new_state: &mut Option<Box<dyn GuiState>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(msg) = self.recv.try_recv() {
                self.msgs.push(format!("{msg:?}"));
            }
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.curr_message);
                if ui.button("Send").clicked() {
                    let mut msg = String::new();
                    std::mem::swap(&mut msg, &mut self.curr_message);
                    self.send
                        .blocking_send(ClientRequest::ChatMessage(msg))
                        .unwrap();
                }
            });
            for msg in &self.msgs {
                ui.label(msg);
            }
        });
        ctx.request_repaint_after(Duration::from_secs_f64(1.0 / 60.0));
    }
}
