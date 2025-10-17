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
    temp_color: [u8; 3],
    #[serde(skip)]
    renaming: bool,
    #[serde(skip)]
    coloring: bool,
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

    fn ez_send(&self, message: SerializableMessage) {
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
        drop(message);
    }

    fn connect(&mut self) {
        let networker = self.networker.clone();
        if !self.is_connected {
            let messages = self.messages.clone();
            let user = self.user.clone();

            self.runtime.spawn(async move {
                let mut nw = Networker::new("ws://sillirc.owomay.hackclub.app", move |message| {
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
    }

    fn render_message(message: &SerializableMessage, ui: &mut egui::Ui) {
        ui.separator();

        ui.horizontal(|ui| {
            let user = message.get_user();
            let message_type = message.get_message_type();
            let (r, g, b) = user.get_color();
            let col = egui::Color32::from_rgb(r, g, b);
            ui.label(egui::RichText::new(user.get_username()).strong().color(col));
            match message_type {
                SerializableMessageType::Join => {
                    ui.label("has joined the chat.");
                }
                SerializableMessageType::Leave => {
                    ui.label("has left the chat.");
                }
                SerializableMessageType::Rename => {
                    ui.label("changed their name to ");
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
}

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
            temp_color: [0, 0, 0],
            renaming: false,
            coloring: false,
            user: User::new(String::new()),
        }
    }
}

impl App for SillircApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.connect();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let _is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("preferences", |ui| {
                    if ui.button("change username").clicked() && !self.user.is_unnamed() {
                        self.renaming = true;
                    }
                    if ui.button("change username color").clicked() {
                        self.temp_color = <[u8; 3]>::from(self.user.get_color());
                        self.coloring = true;
                    }

                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("sillirc");
            if self.user.is_unnamed() || self.renaming {
                if self.renaming {
                    ui.label("what's your new name?");
                } else {
                    ui.label("what should we call you? (can be changed in preferences)");
                }
                let output = egui::TextEdit::singleline(&mut self.temp_username).show(ui);
                if output.response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.temp_username.is_empty()
                {
                    self.ez_send(SerializableMessage::new(
                        self.user.clone(),
                        if self.renaming {
                            SerializableMessageType::Rename
                        } else {
                            SerializableMessageType::Join
                        },
                        self.temp_username.clone(),
                    ));
                    self.user = self.user.clone().set_username(self.temp_username.clone());
                    self.renaming = false;
                }
            }

            if self.coloring {
                ui.color_edit_button_srgb(&mut self.temp_color);
                if ui.button("accept").clicked() {
                    self.user = self
                        .user
                        .clone()
                        .set_color(<(u8, u8, u8)>::from(self.temp_color));
                    self.coloring = false;
                }
            }

            if self.networker.blocking_lock().is_none() {
                ui.heading("YOU ARE NOT CONNECTED TO A SERVER!");
            }

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(ui.available_height() - 56.0)
                .show(ui, |ui| {
                    let messages: Vec<_> = {
                        let guard = self.messages.blocking_lock();
                        guard.clone()
                    };
                    for message in messages {
                        Self::render_message(&message, ui);
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
