use crate::gui::main_menu::MainMenu;

mod connected;
mod main_menu;

pub fn start() {
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
}

trait GuiState {
    fn draw(&mut self, ctx: &egui::Context, new_state: &mut Option<Box<dyn GuiState>>);
}

struct Game(Box<dyn GuiState>);

impl Default for Game {
    fn default() -> Self {
        Self(Box::new(MainMenu {
            socket_addr: "localhost:4000".to_string(),
            port: 4000,
        }))
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_state = None;
        self.0.draw(ctx, &mut new_state);
        if let Some(new) = new_state {
            self.0 = new;
        }
    }
}
