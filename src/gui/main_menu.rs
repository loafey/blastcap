use tokio::runtime::Runtime;

use crate::{
    client,
    gui::{GuiState, connected::Connected},
    host,
};

pub struct MainMenu {
    pub socket_addr: String,
    pub port: u16,
}
impl GuiState for MainMenu {
    fn draw(&mut self, ctx: &egui::Context, new_state: &mut Option<Box<dyn GuiState>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let name_label = ui.label("SocketAddr: ");
                ui.text_edit_singleline(&mut self.socket_addr)
                    .labelled_by(name_label.id);
                if ui.button("Connect").clicked() {
                    let addr = self.socket_addr.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                        let rt = Runtime::new().expect("Unable to create Runtime");
                        let _enter = rt.enter();
                        rt.block_on(client(addr)).unwrap();
                    });
                    *new_state = Some(Box::new(Connected::default()));
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut self.port, 1000..=u16::MAX).text("age"));
                if ui.button("Host").clicked() {
                    let port = self.port;
                    std::thread::spawn(move || {
                        let rt = Runtime::new().expect("Unable to create Runtime");
                        let _enter = rt.enter();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
                            let rt = Runtime::new().expect("Unable to create Runtime");
                            let _enter = rt.enter();
                            rt.block_on(client(format!("localhost:{port}"))).unwrap();
                        });
                        rt.block_on(host(port)).unwrap();
                    });
                    *new_state = Some(Box::new(Connected::default()));
                }
            });
        });
    }
}
