use confy;
use eframe::egui::{
    self, Align2, Button, CtxRef, FontFamily, Label, Layout, TopBottomPanel, Window,
};
use eframe::epi::Frame;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub const PADDING: f32 = 5.0;

pub enum Message {
    ApiKeySet(String),
}

pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<std::sync::mpsc::Receiver<NewsCardData>>,
    pub app_tx: Option<std::sync::mpsc::SyncSender<Message>>,
}

pub struct NewsCardData {
    pub title: String,
    pub url: String,
    pub desciption: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HeadlinesConfig {
    dark_mode: bool,
    pub api_key: String,
}

impl HeadlinesConfig {
    pub fn new() -> Self {
        Self {
            dark_mode: true,
            api_key: "".to_string(),
        }
    }

    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Headlines {
    pub fn adjust_theme(&self, ctx: &egui::Context) -> () {
        if self.config.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    pub fn is_api_key_initialized(&self) -> bool {
        self.api_key_initialized
    }

    pub fn set_api_key_initialized(&mut self, value: bool) {
        self.api_key_initialized = value;
    }

    pub fn new() -> Self {
        let config: HeadlinesConfig = confy::load("gui_news", "headlines").unwrap_or_default();

        tracing::info!("Loaded config: {:?}", config);

        Headlines {
            articles: vec![],
            api_key_initialized: !config.api_key.is_empty(),
            config,
            news_rx: None,
            app_tx: None,
        }
    }

    pub fn config_fonts(&self, ctx: &egui::Context) -> () {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "MesloLGS".to_string(),
            Cow::Borrowed(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );

        fonts
            .family_and_size
            .insert(egui::TextStyle::Heading, (FontFamily::Proportional, 24.0));

        fonts
            .family_and_size
            .insert(egui::TextStyle::Body, (FontFamily::Proportional, 20.0));

        fonts
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());

        ctx.set_fonts(fonts);
    }

    pub fn preload_news(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_card) => {
                    self.articles.push(news_card);
                }
                Err(_) => {}
            }
        }
    }

    pub fn render_news_cards(&self, ui: &mut egui::Ui) {
        let mut title_color = egui::Color32::from_rgb(255, 255, 255);
        let mut hyperlink_color = egui::Color32::from_rgb(0, 255, 255);

        if !self.config.dark_mode {
            title_color = egui::Color32::from_rgb(0, 0, 0);
            hyperlink_color = egui::Color32::from_rgb(0, 0, 255);
        }

        for article in &self.articles {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(PADDING);
                    let title = format!("> {}", article.title);
                    ui.colored_label(title_color, title);

                    ui.add_space(PADDING);

                    let description =
                        Label::new(&article.desciption).text_style(egui::TextStyle::Button);
                    ui.add(description);

                    ui.style_mut().visuals.hyperlink_color = hyperlink_color;
                    ui.add_space(PADDING);

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ui.hyperlink_to("Read more ⤴", article.url.clone());
                    });

                    ui.separator();
                })
            });
        }
    }

    pub fn render_top_panel(&mut self, ctx: &egui::CtxRef, frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📰 Feed").text_style(egui::TextStyle::Heading));
                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("❎").text_style(egui::TextStyle::Button));
                    let refresh_btn = ui.add(Button::new("🔄").text_style(egui::TextStyle::Button));
                    let theme_btn = ui.add(
                        Button::new({
                            if self.config.dark_mode {
                                "🌘"
                            } else {
                                "🌖"
                            }
                        })
                        .text_style(egui::TextStyle::Button),
                    );

                    if theme_btn.clicked() {
                        self.config.toggle_dark_mode();
                    }

                    if close_btn.clicked() {
                        frame.quit();
                    }

                    if refresh_btn.clicked() {
                        // self.preload_news();
                        tracing::info!("Refreshing news");
                    }
                })
            });
            ui.add_space(10.);
        });
    }

    pub fn render_config(&mut self, ctx: &CtxRef) {
        Window::new("Configuration")
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("API Key");
                        let text_input = ui.text_edit_singleline(&mut self.config.api_key);

                        if text_input.lost_focus() || ui.input().key_released(egui::Key::Enter) {
                            if let Err(err) = confy::store("gui_news", "headlines", &self.config) {
                                tracing::error!("Failed to store config: {:?}", err);
                            } else {
                                tracing::info!("Stored config");
                                self.set_api_key_initialized(true);

                                if let Some(tx) = &self.app_tx {
                                    let _ = tx
                                        .send(Message::ApiKeySet(self.config.api_key.to_string()));
                                }
                            }
                        }
                    });
                    ui.label("You may find your API key at newsapi.org");
                })
            });
    }
}
