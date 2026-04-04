use eframe::egui::mutex::{Mutex, RwLock};
use eframe::egui::{self, Key};
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

use crate::Machine;
use crate::cli::Args;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::instruction::Instruction;
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
}

// Implement debug related functions
impl ChipOxide {
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

    fn render_registers(&mut self, ui: &mut egui::Ui) {
        let height = ui.content_rect().height();
        ui.vertical(|ui| {
            ui.heading("Registers");
            TableBuilder::new(ui)
                .id_salt("registers")
                .striped(true)
                .resizable(false)
                .min_scrolled_height(height)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::auto())
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    for _ in 0..2 {
                        header.col(|ui| {
                            ui.strong("Register");
                        });
                        header.col(|ui| {
                            ui.strong("Value");
                        });
                    }
                })
                .body(|body| {
                    let m = self.machine.lock();
                    let rows = (m.v.len() + 4).div_ceil(2);
                    body.rows(20.0, rows, |mut row| {
                        let i = row.index() * 2;
                        match i {
                            0..16 => {
                                for j in 0..2 {
                                    row.col(|ui| {
                                        ui.monospace(format!("v{:X}", i + j));
                                    });
                                    row.col(|ui| {
                                        ui.monospace(format!("{:#04X}", m.v[i + j]));
                                    });
                                }
                            }
                            16 => {
                                row.col(|ui| {
                                    ui.monospace("delay");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#04X}", m.dt));
                                });
                                row.col(|ui| {
                                    ui.monospace("buzzer");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#04X}", m.st));
                                });
                            }
                            18 => {
                                row.col(|ui| {
                                    ui.monospace("i");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#05X}", m.i));
                                });
                                row.col(|ui| {
                                    ui.monospace("pc");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#05X}", m.pc));
                                });
                            }
                            _ => {
                                panic!("Invalid debug register");
                            }
                        }
                    });
                });
        });
    }

    fn render_memory(&mut self, ui: &mut egui::Ui) {
        ui.heading("Memory");
        let table = TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .resizable(false)
            .striped(true)
            .animate_scrolling(false);

        let table = table.scroll_to_row(
            self.machine.lock().pc.div_ceil(2) as usize,
            Some(egui::Align::Center),
        );
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Address");
                });
                header.col(|ui| {
                    ui.strong("Hex");
                });
                header.col(|ui| {
                    ui.strong("Instruction");
                });
            })
            .body(|body| {
                let m = self.machine.lock();
                let rows = m.get_memory().len().div_ceil(2);

                body.rows(10.0, rows, |mut row| {
                    let i = row.index() * 2;

                    let left_byte = *m.memory.get(i).unwrap_or(&0) as u16;
                    let right_byte = *m.memory.get(i + 1).unwrap_or(&0) as u16;
                    let opcode = (left_byte << 8) | right_byte;

                    if m.pc as usize == i {
                        row.set_selected(true);
                    }

                    row.col(|ui| {
                        ui.monospace(format!("{:#05X}", i));
                    });
                    row.col(|ui| {
                        ui.monospace(format!("{:#06X}", opcode));
                    });
                    row.col(|ui| {
                        ui.monospace(format!(
                            "{}",
                            Instruction::new(opcode, self.args.read().jump)
                        ));
                    });
                });
            });
    }

    fn render_stack(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Stack");
            TableBuilder::new(ui)
                .id_salt("stack")
                .column(Column::auto())
                .column(Column::remainder())
                .resizable(false)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Level");
                    });
                    header.col(|ui| {
                        ui.strong("Address");
                    });
                })
                .body(|body| {
                    let stack = &self.machine.lock().stack;
                    body.rows(20.0, stack.len(), |mut row| {
                        let i = row.index();
                        row.col(|ui| {
                            ui.monospace(format!("{}", i + 1));
                        });
                        row.col(|ui| {
                            ui.monospace(format!("{:#05X}", stack.get(i).unwrap()));
                        });
                    });
                });
        });
    }

    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        let screen_rect = ui.content_rect();
        let width = screen_rect.width();
        let height = screen_rect.height();

        egui::Panel::left("debug_panel")
            .exact_size(300.0)
            .show_inside(ui, |ui| {
                if self.args.read().step_mode && ui.button("Step Forward").clicked() {
                    self.machine.lock().cycle();
                }
                self.render_memory(ui);
            });

        egui::Panel::bottom("bottom_panel")
            .exact_size(300.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(ui.available_width() / 2.0, ui.available_height()), |ui| {
                        self.render_registers(ui);
                    });
                    ui.allocate_ui(egui::vec2(ui.available_width() / 2.0, ui.available_height()), |ui| {
                        self.render_stack(ui);
                    });
                });
            });
    }
}

impl eframe::App for ChipOxide {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
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
            self.render_debug_panel(ui);
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
