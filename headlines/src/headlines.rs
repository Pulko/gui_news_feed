use eframe::egui::{self, Button, FontFamily, Label, Layout, TopBottomPanel};
use std::borrow::Cow;

pub const PADDING: f32 = 5.0;

pub struct Headlines {
    articles: Vec<NewsCardData>,
}

pub struct NewsCardData {
    title: String,
    url: String,
    desciption: String,
}

impl Headlines {
    pub fn new() -> Self {
        let mock_iter = (0..20).map(|a| NewsCardData {
            title: format!("Title {}!", a),
            url: "https://example.com".to_string(),
            desciption: format!("This is a description for {}", a),
        });

        Headlines {
            articles: Vec::from_iter(mock_iter),
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

    pub fn render_news_cards(&self, ui: &mut egui::Ui) {
        for article in &self.articles {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(PADDING);
                    let title = format!("> {}", article.title);
                    ui.colored_label(egui::Color32::from_rgb(255, 255, 255), title);

                    ui.add_space(PADDING);

                    let description =
                        Label::new(&article.desciption).text_style(egui::TextStyle::Button);
                    ui.add(description);

                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 255);
                    ui.add_space(PADDING);

                    ui.with_layout(Layout::right_to_left(), |ui| {
                        ui.hyperlink_to("Read more ⤴", article.url.clone());
                    });

                    ui.separator();
                })
            });
        }
    }

    pub fn render_top_panel(&self, ctx: &egui::CtxRef) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📰").text_style(egui::TextStyle::Heading));
                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    let close_btn = ui.add(Button::new("❎").text_style(egui::TextStyle::Heading));
                    let refresh_btn =
                        ui.add(Button::new("🔄").text_style(egui::TextStyle::Heading));
                    let theme_btn = ui.add(Button::new("🌙").text_style(egui::TextStyle::Heading));
                })
            });
            ui.add_space(10.);
        });
    }
}
