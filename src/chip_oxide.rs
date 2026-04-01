use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex};

use crate::Machine;
use crate::cli::Args;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub fn init(args: Arc<Args>, chip8: Arc<Mutex<Machine>>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ChipOxide::new(cc, args, chip8)))
        }),
    )
    .unwrap_or_else(|e| panic!("Failed to run native: {e}"));
}

struct ChipOxide {
    args: Arc<Args>,
    chip8: Arc<Mutex<Machine>>,
    screen_texture: egui::TextureHandle,
}

impl ChipOxide {
    fn new(cc: &eframe::CreationContext<'_>, args: Arc<Args>, chip8: Arc<Mutex<Machine>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let screen_texture = cc.egui_ctx.load_texture(
            "screen",
            egui::ColorImage::from_rgba_unmultiplied(
                [SCREEN_WIDTH, SCREEN_HEIGHT],
                &[255; SCREEN_HEIGHT * SCREEN_WIDTH * 4],
            ),
            egui::TextureOptions::NEAREST,
        );

        Self {
            args,
            chip8,
            screen_texture,
        }
    }

    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        if self.args.step_mode && ui.button("Step Forward").clicked() {
            self.chip8.lock().unwrap().cycle();
        }
        let chip8 = self.chip8.lock().unwrap();
        ui.heading("CPU");

        ui.label(format!(
            "I: {:03X} DT: {:02X} ST: {:02X}",
            chip8.i, chip8.dt, chip8.st
        ));
        ui.label(format!(
            "PC: {:03X} Opcode: {:04X} Instruction: {}",
            chip8.pc, chip8.opcode, chip8.instruction
        ));

        for i in (0..chip8.v.len()).step_by(2) {
            ui.label(format!(
                "V{:X}: {:02X} V{:X}: {:02X}",
                i,
                chip8.v[i],
                i + 1,
                chip8.v[i + 1]
            ));
        }

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
                            ui.label(format! {"{i:02X}"});
                        });
                    }
                })
                .body(|mut body| {
                    let row_count = chip8.get_memory().len() / memory_column_width;

                    for i in 0..row_count {
                        body.row(10.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{:03X}", i * 16));
                            });
                            for j in 0..memory_column_width {
                                let idx = i * memory_column_width + j;
                                row.col(|ui| {
                                    ui.label(format!("{:02X}", chip8.get_memory()[idx]));
                                });
                            }
                        });
                    }
                });
        });
    }

    fn convert(&self) -> [u8; 2048] {
        let chip8 = self.chip8.lock().unwrap();
        let mut res = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        for (i, pixel) in chip8.get_screen_buffer().iter().enumerate() {
            if *pixel {
                res[i] = 255_u8;
            } else {
                res[i] = 0_u8;
            }
        }
        res
    }
}

impl eframe::App for ChipOxide {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx();
        let screen_rect = ctx.content_rect();
        let width = screen_rect.width();

        // repaint so things update
        ctx.request_repaint();

        let size = [SCREEN_WIDTH, SCREEN_HEIGHT];
        let converted = self.convert();
        let color_image = egui::ColorImage::from_gray(size, &converted);
        self.screen_texture
            .set(color_image, egui::TextureOptions::NEAREST);

        if self.args.debug {
            egui::Panel::right("debug_panel")
                .exact_size(width * 0.3)
                .show_inside(ui, |ui| {
                    self.render_debug_panel(ui);
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let available_size = ui.available_size();

            let scale_x = (available_size.x / SCREEN_WIDTH as f32).floor();
            let scale_y = (available_size.y / SCREEN_HEIGHT as f32).floor();

            let scale = scale_x.min(scale_y).max(1.0);

            ui.centered_and_justified(|ui| {
                ui.add(
                    egui::Image::new(&self.screen_texture).fit_to_exact_size(egui::vec2(
                        SCREEN_WIDTH as f32 * scale,
                        SCREEN_HEIGHT as f32 * scale,
                    )),
                );
            });
        });
    }
}
