#![warn(clippy::all)]

use eframe::{App, egui};

use std::sync::Arc;
use tokio::sync::Mutex;

use sillirc_lib::networker::{Networker, SerializableMessage};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SillircApp {
    #[serde(skip)]
    runtime: tokio::runtime::Runtime,
    #[serde(skip)]
    networker: Arc<Mutex<Option<Networker>>>,
    #[serde(skip)]
    messages: Arc<Mutex<Vec<SerializableMessage>>>,
    #[serde(skip)]
    current_text: String,
    #[serde(skip)]
    temp_username: String,
    username: String,
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

#[expect(clippy::derivable_impls)]
impl Default for SillircApp {
    fn default() -> Self {
        Self {
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime"),
            networker: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(Vec::new())),
            current_text: String::new(),
            temp_username: String::new(),
            username: String::new(),
        }
    }
}

impl App for SillircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let networker = self.networker.clone();
        let is_disconnected =
            futures::executor::block_on(async { networker.lock().await.is_none() });
        if is_disconnected {
            let messages = self.messages.clone();

            self.runtime.spawn(async move {
                let nw = Networker::new("ws://0.0.0.0:80", move |message| {
                    let messages = messages.clone();
                    async move {
                        messages.lock().await.push(message);
                    }
                })
                .await;

                *networker.lock().await = Some(nw);
            });
        }

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     egui::MenuBar::new().ui(ui, |ui| {
        //         let is_web = cfg!(target_arch = "wasm32");
        //     });
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("sillirc");

            if self.username.is_empty() {
                ui.label("what should we call you? (can be changed in preferences)");
                let output = egui::TextEdit::singleline(&mut self.temp_username).show(ui);
                if output.response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.temp_username.is_empty()
                {
                    self.username = self.temp_username.clone();
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.label("made with ❤ & :3 by OwOmay");

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
