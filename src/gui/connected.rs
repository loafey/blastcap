use crate::gui::GuiState;

#[derive(Default)]
pub struct Connected {}
impl GuiState for Connected {
    fn draw(&mut self, ctx: &egui::Context, _new_state: &mut Option<Box<dyn GuiState>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("yoo!");
        });
    }
}
