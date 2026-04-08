use eframe::egui::{self, Key};
use egui_extras::{Column, TableBuilder};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::Machine;
use crate::cli::Args;
use crate::constants::{FRAME_TIME, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::instruction::Instruction;
use crate::keyboard::Keyboard;

pub fn init(args: Rc<RefCell<Args>>, machine: Machine, keyboard: Rc<RefCell<Keyboard>>) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ChipOxide::new(cc, args, machine, keyboard)))
        }),
    )
    .unwrap_or_else(|e| panic!("Failed to run native: {e}"));
}

struct ChipOxide {
    args: Rc<RefCell<Args>>,
    machine: Machine,
    screen_texture: egui::TextureHandle,
    keyboard: Rc<RefCell<Keyboard>>,
}

impl ChipOxide {
    fn new(
        cc: &eframe::CreationContext<'_>,
        args: Rc<RefCell<Args>>,
        machine: Machine,
        keyboard: Rc<RefCell<Keyboard>>,
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
            machine,
            screen_texture,
            keyboard,
        }
    }

    fn convert(&self) -> [u8; 2048] {
        let mut res = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        for (i, pixel) in self.machine.get_display_buffer().iter().enumerate() {
            if *pixel {
                res[i] = 255_u8;
            } else {
                res[i] = 0_u8;
            }
        }
        res
    }

    fn set_screen_texture(&mut self) {
        let size = [SCREEN_WIDTH, SCREEN_HEIGHT];
        let converted = self.convert();
        let color_image = egui::ColorImage::from_gray(size, &converted);
        self.screen_texture
            .set(color_image, egui::TextureOptions::NEAREST);
    }
}

// Implement debug related functions
impl ChipOxide {
    fn check_keys(&mut self, ui: &mut egui::Ui) {
        ui.input(|input| {
            // toggle debug mode
            if input.key_pressed(Key::CloseBracket) {
                let is_debug = self.args.borrow().debug;
                self.args.borrow_mut().debug = !is_debug;
            }

            //toggle step mode
            if input.key_pressed(Key::Quote) {
                let is_step_mode = self.args.borrow().step_mode;
                self.args.borrow_mut().step_mode = !is_step_mode;
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
                    let rows = (self.machine.v.len() + 4).div_ceil(2);
                    body.rows(20.0, rows, |mut row| {
                        let i = row.index() * 2;
                        match i {
                            0..16 => {
                                for j in 0..2 {
                                    row.col(|ui| {
                                        ui.monospace(format!("v{:X}", i + j));
                                    });
                                    row.col(|ui| {
                                        ui.monospace(format!("{:#04X}", self.machine.v[i + j]));
                                    });
                                }
                            }
                            16 => {
                                row.col(|ui| {
                                    ui.monospace("delay");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#04X}", self.machine.dt));
                                });
                                row.col(|ui| {
                                    ui.monospace("buzzer");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#04X}", self.machine.st));
                                });
                            }
                            18 => {
                                row.col(|ui| {
                                    ui.monospace("i");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#05X}", self.machine.i));
                                });
                                row.col(|ui| {
                                    ui.monospace("pc");
                                });
                                row.col(|ui| {
                                    ui.monospace(format!("{:#05X}", self.machine.pc));
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
            self.machine.pc.div_ceil(2) as usize,
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
                let rows = self.machine.get_memory().len().div_ceil(2);

                body.rows(10.0, rows, |mut row| {
                    let i = row.index() * 2;

                    let left_byte = *self.machine.memory.get(i).unwrap_or(&0) as u16;
                    let right_byte = *self.machine.memory.get(i + 1).unwrap_or(&0) as u16;
                    let opcode = (left_byte << 8) | right_byte;

                    if self.machine.pc as usize == i {
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
                            Instruction::new(opcode, self.args.borrow().jump)
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
                    let stack = &self.machine.stack;
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
        egui::Panel::left("debug_panel")
            .exact_size(300.0)
            .show_inside(ui, |ui| {
                if self.args.borrow().step_mode && ui.button("Step Forward").clicked() {
                    self.machine.cycle();
                }
                self.render_memory(ui);
            });

        egui::Panel::bottom("bottom_panel")
            .exact_size(300.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui(
                        egui::vec2(ui.available_width() / 2.0, ui.available_height()),
                        |ui| {
                            self.render_registers(ui);
                        },
                    );
                    ui.allocate_ui(
                        egui::vec2(ui.available_width() / 2.0, ui.available_height()),
                        |ui| {
                            self.render_stack(ui);
                        },
                    );
                });
            });
    }
}

impl eframe::App for ChipOxide {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let frame_start = Instant::now();
        self.check_keys(ui);

        // set keyboard state
        ui.input(|i| self.keyboard.borrow_mut().set_keys(&i.keys_down));

        self.set_screen_texture();

        if self.args.borrow().debug {
            self.render_debug_panel(ui);
        }

        if !self.args.borrow().step_mode {
            for _ in 0..self.args.borrow().instructions_per_frame {
                let return_code = self.machine.cycle();

                // break if draw is waiting for end of frame
                if return_code == 1 {
                    break;
                }
                ui.request_repaint();
            }
        }

        self.machine.decrement_timers();

        // copy back buffer to front buffer to produce a frame
        self.machine.swap_buffers();

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

        // repaint so things update
        ui.request_repaint();

        // sleep until end of frame time
        let cycle_duration = frame_start - Instant::now();
        spin_sleep::sleep(Duration::from_secs_f64(FRAME_TIME) - cycle_duration);
    }
}
