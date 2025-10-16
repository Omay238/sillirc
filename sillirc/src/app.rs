#![warn(clippy::all)]

use eframe::{App, egui};

use eframe::glow::Context;
use futures::executor::block_on;
use std::sync::Arc;
use tokio::sync::Mutex;

use sillirc_lib::networker::{Networker, SerializableMessage, SerializableMessageType};
use sillirc_lib::user::User;

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
    user: User,
}

impl SillircApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn ez_send(&mut self, message: SerializableMessage) {
        let networker = self.networker.clone();
        let msg = message.clone();
        self.runtime.spawn(async move {
            networker.lock().await.as_mut().unwrap().send(msg).await;
        });
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
            user: User::new(String::new()),
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
            let user = self.user.clone();

            self.runtime.spawn(async move {
                let mut nw = Networker::new("ws://0.0.0.0:80", move |message| {
                    let messages = messages.clone();
                    async move {
                        messages.lock().await.push(message);
                    }
                })
                .await;

                if !user.is_unnamed() {
                    nw.send(SerializableMessage::new(
                        user,
                        SerializableMessageType::Join,
                        String::new(),
                    ))
                    .await;
                }

                *networker.lock().await = Some(nw);
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let _is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("preferences", |ui| {
                    if ui.button("Change Username").clicked() {
                        self.user = User::set_username(&self.user, String::new());
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("sillirc");
            if self.user.is_unnamed() {
                ui.label("what should we call you? (can be changed in preferences)");
                let output = egui::TextEdit::singleline(&mut self.temp_username).show(ui);
                if output.response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.temp_username.is_empty()
                {
                    self.user = User::set_username(&self.user, self.temp_username.clone());
                    self.ez_send(SerializableMessage::new(
                        self.user.clone(),
                        SerializableMessageType::Join,
                        String::new(),
                    ));
                }
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let messages = block_on(self.messages.lock());
                    for message in messages.to_vec() {
                        ui.separator();

                        ui.horizontal(|ui| {
                            let user = message.get_user();
                            let message_type = message.get_message_type();
                            let uuid_bind = user.get_uuid();
                            let mut uuid = uuid_bind.as_bytes().iter();
                            let r = uuid.next().unwrap();
                            let g = uuid.next().unwrap();
                            let b = uuid.next().unwrap();
                            ui.label(
                                egui::RichText::new(user.get_username())
                                    .strong()
                                    .color(egui::Color32::from_rgb(*r, *g, *b)),
                            );
                            match message_type {
                                SerializableMessageType::Join => {
                                    ui.label("has joined the chat.");
                                }
                                SerializableMessageType::Leave => {
                                    ui.label("has left the chat.");
                                }
                                SerializableMessageType::Text => {
                                    ui.label(message.get_content());
                                }
                            }
                        });
                    }
                });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.label("made with ❤ & :3 by OwOmay");

                ui.separator();

                if !self.user.is_unnamed() {
                    let response = ui.text_edit_singleline(&mut self.current_text);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.ez_send(SerializableMessage::new(
                            self.user.clone(),
                            SerializableMessageType::Text,
                            self.current_text.clone(),
                        ));
                        self.current_text = String::new();
                    }
                }
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        self.ez_send(SerializableMessage::new(
            self.user.clone(),
            SerializableMessageType::Leave,
            String::new(),
        ));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
