#![warn(clippy::all)]

use eframe::{App, egui};

use eframe::glow::Context;
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
    is_connected: bool,
    #[serde(skip)]
    messages: Arc<Mutex<Vec<SerializableMessage>>>,
    #[serde(skip)]
    current_text: String,
    #[serde(skip)]
    temp_username: String,
    #[serde(skip)]
    renaming: bool,
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
            match networker.lock().await.as_mut() {
                Some(nw) => nw,
                None => return,
            }
            .send(msg)
            .await;
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
            is_connected: false,
            messages: Arc::new(Mutex::new(Vec::new())),
            current_text: String::new(),
            temp_username: String::new(),
            renaming: false,
            user: User::new(String::new()),
        }
    }
}

impl App for SillircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let networker = self.networker.clone();
        let is_disconnected = networker.blocking_lock().is_none();
        if !self.is_connected {
            let messages = self.messages.clone();
            let user = self.user.clone();

            self.runtime.spawn(async move {
                let mut nw = Networker::new("ws://owomay.hackclub.app:9001", move |message| {
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

            self.is_connected = true;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let _is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("preferences", |ui| {
                    if ui.button("change username").clicked() {
                        self.user = User::set_username(&self.user, String::new());
                        self.renaming = true;
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
                    let old_username = self.user.get_username().clone();
                    self.user = User::set_username(&self.user, self.temp_username.clone());
                    self.ez_send(SerializableMessage::new(
                        self.user.clone(),
                        if self.renaming {
                            SerializableMessageType::Rename
                        } else {
                            SerializableMessageType::Join
                        },
                        old_username,
                    ));
                }
            }

            if is_disconnected {
                ui.heading("YOU ARE NOT CONNECTED TO A SERVER!");
            }

            egui::ScrollArea::vertical()
                .auto_shrink([true, true])
                .stick_to_bottom(true)
                .max_height(ui.available_height() - 56.0)
                .show(ui, |ui| {
                    let messages = self.messages.blocking_lock();
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
                            let col = egui::Color32::from_rgb(*r, *g, *b);
                            ui.label(egui::RichText::new(user.get_username()).strong().color(col));
                            match message_type {
                                SerializableMessageType::Join => {
                                    ui.label("has joined the chat.");
                                }
                                SerializableMessageType::Leave => {
                                    ui.label("has left the chat.");
                                }
                                SerializableMessageType::Rename => {
                                    ui.label("changed their name from ");
                                    ui.label(
                                        egui::RichText::new(message.get_content().as_str())
                                            .strong()
                                            .color(col),
                                    );
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
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.current_text.is_empty()
                    {
                        self.ez_send(SerializableMessage::new(
                            self.user.clone(),
                            SerializableMessageType::Text,
                            self.current_text.clone(),
                        ));
                        self.current_text = String::new();
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        response.request_focus();
                    }
                }
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        self.ez_send(SerializableMessage::new(
            self.user.clone(),
            SerializableMessageType::Leave,
            String::new(),
        ));
    }
}
