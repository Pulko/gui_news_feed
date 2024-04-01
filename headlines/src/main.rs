mod headlines;

use std::thread;

use api::NewsApi;
use eframe::egui::{self, Hyperlink, Label, ScrollArea, Separator, TopBottomPanel};
use eframe::epi::{App, Frame};
use eframe::run_native;
use headlines::{Headlines, Message, NewsCardData, PADDING};
use tracing_subscriber;

impl App for Headlines {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.config_fonts(ctx);

        let api_key = self.config.api_key.to_string();
        let (mut news_tx, news_rx) = std::sync::mpsc::channel::<NewsCardData>();
        let (app_tx, app_rx) = std::sync::mpsc::sync_channel(1);

        self.news_rx = Some(news_rx);
        self.app_tx = Some(app_tx);

        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx);
            } else {
                loop {
                    match app_rx.recv() {
                        Ok(Message::ApiKeySet(api_key)) => {
                            fetch_news(&api_key, &mut news_tx);
                        }
                        Err(error) => {
                            tracing::error!("Error receiving message: {:?}", error);
                        }
                    }
                }
            }
        });
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut Frame) {
        if self.is_api_key_initialized() {
            ctx.request_repaint();
            self.preload_news();
            self.adjust_theme(ctx);
            self.render_top_panel(ctx, frame);
            egui::CentralPanel::default().show(ctx, |ui| {
                render_header(ui);
                ScrollArea::auto_sized().show(ui, |ui| {
                    self.render_news_cards(ui);
                });
                render_footer(ctx);
            });
        } else {
            self.render_config(ctx);
        }
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn fetch_news(api_key: &str, news_tx: &mut std::sync::mpsc::Sender<NewsCardData>) {
    let fetching = NewsApi::new(api_key)
        .endpoint(api::Endpoint::TopHeadlines)
        .country(api::Country::Us)
        .fetch();

    match fetching {
        Ok(response) => {
            for article in response.get_articles() {
                let news_card = headlines::NewsCardData {
                    title: article.title().to_string(),
                    url: article.url().to_string(),
                    desciption: article.desciption().to_string(),
                };

                if let Err(error) = news_tx.send(news_card) {
                    tracing::error!("Error creating news card: {:?}", error)
                };
            }
        }
        Err(e) => {
            tracing::error!("Error fetching news: {}", e);
        }
    }
}

fn render_footer(ctx: &egui::CtxRef) {
    let mut label_color = egui::Color32::from_rgb(255, 255, 255);

    if !ctx.style().visuals.dark_mode {
        label_color = egui::Color32::from_rgb(0, 0, 0);
    }

    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            // add the api source website
            ui.add(
                Label::new("Powered by NewsAPI.org")
                    .text_color(label_color)
                    .monospace(),
            );
            // add link to the source code
            ui.add(
                Hyperlink::new("https://github.com/Pulko")
                    .text("Source code")
                    .text_style(egui::TextStyle::Monospace),
            );
            ui.add_space(10.);
        })
    });
}

fn render_header(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Headlines");
    });
    ui.add_space(PADDING);
    let separator = Separator::default().spacing(20.0);
    ui.add(separator);
}

fn main() {
    tracing_subscriber::fmt::init();
    let app = headlines::Headlines::new();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 900.0)),
        ..Default::default()
    };

    let _ = run_native(Box::new(app), native_options);
}
