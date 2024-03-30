mod headlines;

use eframe::egui::{self, Hyperlink, Label, ScrollArea, Separator, TopBottomPanel};
use eframe::epi::{App, Frame};
use eframe::run_native;
use headlines::{Headlines, PADDING};

impl App for Headlines {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.config_fonts(ctx);
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut Frame) {
        self.render_top_panel(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            render_header(ui);
            ScrollArea::auto_sized().show(ui, |ui| {
                self.render_news_cards(ui);
            });
            render_footer(ctx);
        });
    }

    fn name(&self) -> &str {
        "Headlines"
    }
}

fn render_footer(ctx: &egui::CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            // add the api source website
            ui.add(
                Label::new("Powered by NewsAPI.org")
                    .text_color(egui::Color32::from_rgb(255, 255, 255))
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
    let app = headlines::Headlines::new();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };

    let _ = run_native(Box::new(app), native_options);
}
