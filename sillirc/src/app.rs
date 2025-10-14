use eframe::{App, egui};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SillircApp {
    #[serde(skip)]
    current_text: String,
}

#[allow(clippy::derivable_impls)]
impl Default for SillircApp {
    fn default() -> Self {
        Self {
            current_text: String::new(),
        }
    }
}

impl SillircApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl App for SillircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     egui::MenuBar::new().ui(ui, |ui| {
        //         let is_web = cfg!(target_arch = "wasm32");
        //     });
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("sillirc");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.label("Made with ❤ & :3 by OwOmay");

                ui.separator();

                let response = ui.text_edit_singleline(&mut self.current_text);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.current_text = String::new();
                }
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
