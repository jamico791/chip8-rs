use eframe::egui::mutex::{Mutex, RwLock};
use eframe::egui::{self, Key};
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::Machine;
use crate::cli::Args;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::keyboard::Keyboard;

pub fn init(args: Arc<RwLock<Args>>, chip8: Arc<Mutex<Machine>>, keyboard: Arc<Mutex<Keyboard>>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ChipOxide::new(cc, args, chip8, keyboard)))
        }),
    )
    .unwrap_or_else(|e| panic!("Failed to run native: {e}"));
}

struct ChipOxide {
    args: Arc<RwLock<Args>>,
    machine: Arc<Mutex<Machine>>,
    screen_texture: egui::TextureHandle,
    keyboard: Arc<Mutex<Keyboard>>,
}

impl ChipOxide {
    fn new(
        cc: &eframe::CreationContext<'_>,
        args: Arc<RwLock<Args>>,
        chip8: Arc<Mutex<Machine>>,
        keyboard: Arc<Mutex<Keyboard>>,
    ) -> Self {
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
            machine: chip8,
            screen_texture,
            keyboard,
        }
    }

    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        if self.args.read().step_mode && ui.button("Step Forward").clicked() {
            self.machine.lock().cycle();
        }
        let chip8 = self.machine.lock();
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
        let chip8 = self.machine.lock();
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

    fn check_keys(&mut self, ui: &mut egui::Ui) {
        ui.input(|input| {
            // toggle debug mode
            if input.key_pressed(Key::CloseBracket) {
                let is_debug = self.args.read().debug;
                self.args.write().debug = !is_debug;
            }

            //toggle step mode
            if input.key_pressed(Key::Quote) {
                let is_step_mode = self.args.read().step_mode;
                self.args.write().step_mode = !is_step_mode;
            }
        })
    }
}

impl eframe::App for ChipOxide {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let screen_rect = ui.content_rect();
        let width = screen_rect.width();

        self.check_keys(ui);

        // repaint so things update
        ui.request_repaint();

        // set keyboard state
        ui.input(|i| self.keyboard.lock().set_keys(&i.keys_down));

        let size = [SCREEN_WIDTH, SCREEN_HEIGHT];
        let converted = self.convert();
        let color_image = egui::ColorImage::from_gray(size, &converted);
        self.screen_texture
            .set(color_image, egui::TextureOptions::NEAREST);

        if self.args.read().debug {
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
