use eframe::egui::{self, Rect};
use egui::{Key, ScrollArea, RichText};
use egui_extras::{TableBuilder, Column};

use crate::Chip8;
use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};

pub fn init(chip8: Chip8) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::dark()); 
        Ok(Box::new(MyEguiApp::new(cc, chip8)))
    })).unwrap_or_else(|e| panic!("Failed to run native: {e}"));
}

struct MyEguiApp {
    chip8: Chip8,
    screen_texture: egui::TextureHandle,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, chip8: Chip8) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let screen_texture = cc.egui_ctx.load_texture(
            "screen",
            egui::ColorImage::example(),
            egui::TextureOptions::NEAREST,
        );

        Self { 
            chip8,
            screen_texture,
        }
    }

    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("CPU");

        ui.label(format!("I: {:03X} DT: {:02X} ST: {:02X}", self.chip8.i, self.chip8.dt, self.chip8.st));

        ui.collapsing("Memory", |ui| {
            let memory_column_width = 16;

            TableBuilder::new(ui)
                .column(Column::auto())
                .columns(Column::auto(), memory_column_width)
                .header(10.0, |mut header| {
                    header.col(|ui| {
                        ui.label(" ");
                    });
                    for i in 0..memory_column_width {
                        header.col(|ui| {
                            ui.label(format!{"{i:02X}"});
                        });
                    }
                })
                .body(|mut body| {
                    let row_count = self.chip8.get_memory().len() / memory_column_width;

                    for i in 0..row_count {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{:03X}", i * 16));
                            });
                            for j in 0..memory_column_width {
                                let idx = i * memory_column_width + j;
                                row.col(|ui| {
                                    ui.label(format!("{:02X}", self.chip8.get_memory()[idx]));
                                });
                            }
                        }); 
                    }
                });

            });
    }

    fn render_display(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
    }

}


impl eframe::App for MyEguiApp {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        let screen_rect = ctx.content_rect();
        let width = screen_rect.width();
        let height = screen_rect.height();

        self.chip8.cycle();

        egui::Panel::right("debug_panel")
            .exact_size(width * 0.3)
            .show_inside(ui, |ui| {
            ui.label("Hello");
            self.render_debug_panel(ui);
        });


        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(egui::Image::new(&self.screen_texture).fit_to_exact_size(egui::vec2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32)));
            // let available = ui.available_size();
            // let pixel_size = (available.x / 64.0).min(available.y / 32.0);
            // let display_size = egui::vec2(pixel_size * 64.0, pixel_size * 32.0);

            // let offset = (available - display_size) / 2.0;
            // let top_left = ui.min_rect().min + offset;
            // let rect = egui::Rect::from_min_size(top_left, display_size);

            // self.render_display(ui, rect);
        });
   }
}