use std::time::Instant;

use eframe::{
    egui::{self, menu, ImageButton, TopBottomPanel, Ui},
    epaint::Vec2,
};
use egui::Image;
use i18n_embed_fl::fl;
use icy_engine::{
    util::{pop_data, BUFFER_DATA},
    FontMode, IceMode, PaletteMode,
};

use crate::{button_with_shortcut, MainWindow, Message, Settings, LATEST_VERSION, MRU_FILES, PLUGINS, SETTINGS, VERSION};

lazy_static::lazy_static! {
    pub static ref DOCK_LEFT_SVG: Image<'static> = Image::new(egui::include_image!("../../data/icons/dock_left.svg"));
    pub static ref DOCK_RIGHT_SVG: Image<'static> = Image::new(egui::include_image!("../../data/icons/dock_right.svg"));
}

pub struct TopBar {}

impl TopBar {
    pub fn new(_ctx: &egui::Context) -> Self {
        Self {}
    }
}

impl<'a> MainWindow<'a> {
    pub fn show_top_bar(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Option<Message> {
        let mut result = None;

        TopBottomPanel::top("top_panel").exact_height(24.0).show(ctx, |ui| {
            result = self.main_menu(ui, frame);
        });
        result
    }

    fn main_menu(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> Option<Message> {
        let mut result = None;

        menu::bar(ui, |ui| {
            let mut has_buffer = false;
            let mut has_reference_image = false;
            let mut cur_raster = Some(Vec2::new(f32::NAN, f32::NAN));
            let mut cur_guide = Some(Vec2::new(f32::NAN, f32::NAN));

            if self.last_command_update.elapsed().as_millis() > 250 {
                let mut c = self.commands.pop().unwrap();
                if let Some((_, pane)) = self.get_active_pane() {
                    c.update_states(Some(pane));
                } else {
                    c.update_states(None);
                }
                self.commands.push(c);
                self.last_command_update = Instant::now();
            }

            if let Some(pane) = self.get_active_pane_mut() {
                if let Some(editor) = pane.doc.lock().get_ansi_editor() {
                    has_buffer = true;
                    cur_raster = editor.raster;
                    cur_guide = editor.guide;
                    has_reference_image = editor.buffer_view.lock().has_reference_image();
                } else {
                    has_buffer = false;
                }
            }

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-file"), |ui| {
                ui.set_min_width(300.0);

                self.commands[0].new_file.ui(ui, &mut result);
                self.commands[0].open_file.ui(ui, &mut result);
                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-open_recent"), |ui| unsafe {
                    ui.style_mut().wrap = Some(false);

                    let get_recent_files = MRU_FILES.get_recent_files();
                    if !get_recent_files.is_empty() {
                        for file in get_recent_files.iter().rev() {
                            let button = ui.button(file.file_name().unwrap().to_string_lossy());
                            if button.clicked() {
                                result = Some(Message::TryLoadFile(file.clone()));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                    }
                    self.commands[0].clear_recent_open.ui(ui, &mut result);
                });
                ui.separator();
                self.commands[0].save.ui(ui, &mut result);
                self.commands[0].save_as.ui(ui, &mut result);
                self.commands[0].export.ui(ui, &mut result);
                ui.separator();
                self.commands[0].show_settings.ui(ui, &mut result);
                ui.separator();
                self.commands[0].close_window.ui(ui, &mut result);
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-edit"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(200.0);
                if let Some(doc) = self.get_active_document() {
                    if doc.lock().can_undo() {
                        let enabled = doc.lock().can_undo();

                        let button = button_with_shortcut(
                            ui,
                            enabled,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "menu-undo-op",
                                op = doc.lock().undo_description().unwrap_or("No undo description".to_string())
                            ),
                            "Ctrl+Z",
                        );
                        if button.clicked() {
                            result = Some(Message::Undo);
                            ui.close_menu();
                        }
                    } else {
                        self.commands[0].undo.ui(ui, &mut result);
                    }

                    if doc.lock().can_redo() {
                        let button = button_with_shortcut(
                            ui,
                            true,
                            fl!(
                                crate::LANGUAGE_LOADER,
                                "menu-redo-op",
                                op = doc.lock().redo_description().unwrap_or("No redo description".to_string())
                            ),
                            "Ctrl+Shift+Z",
                        );
                        if button.clicked() {
                            result = Some(Message::Redo);
                            ui.close_menu();
                        }
                    } else {
                        self.commands[0].redo.ui(ui, &mut result);
                    }
                } else {
                    self.commands[0].undo.ui(ui, &mut result);
                    self.commands[0].redo.ui(ui, &mut result);
                }
                ui.separator();
                if self.get_active_document().is_some() {
                    self.commands[0].cut.ui(ui, &mut result);
                    self.commands[0].copy.ui(ui, &mut result);
                    self.commands[0].paste.ui(ui, &mut result);
                }

                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-paste-as"), |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(200.0);

                    let button = button_with_shortcut(ui, pop_data(BUFFER_DATA).is_some(), fl!(crate::LANGUAGE_LOADER, "menu-paste-as-new-image"), "");
                    if button.clicked() {
                        result = Some(Message::PasteAsNewImage);
                        ui.close_menu();
                    }

                    let button = button_with_shortcut(ui, pop_data(BUFFER_DATA).is_some(), fl!(crate::LANGUAGE_LOADER, "menu-paste-as-brush"), "");
                    if button.clicked() {
                        result = Some(Message::PasteAsBrush);
                        ui.close_menu();
                    }
                });
                ui.separator();
                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-area_operations"), |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(300.0);

                    self.commands[0].justify_line_left.ui(ui, &mut result);
                    self.commands[0].justify_line_right.ui(ui, &mut result);
                    self.commands[0].justify_line_center.ui(ui, &mut result);
                    ui.separator();
                    self.commands[0].insert_row.ui(ui, &mut result);
                    self.commands[0].delete_row.ui(ui, &mut result);
                    ui.separator();
                    self.commands[0].insert_column.ui(ui, &mut result);
                    self.commands[0].delete_column.ui(ui, &mut result);
                    ui.separator();
                    self.commands[0].erase_row.ui(ui, &mut result);
                    self.commands[0].erase_row_to_start.ui(ui, &mut result);
                    self.commands[0].erase_row_to_end.ui(ui, &mut result);
                    ui.separator();
                    self.commands[0].erase_column.ui(ui, &mut result);
                    self.commands[0].erase_column_to_end.ui(ui, &mut result);
                    self.commands[0].erase_column_to_start.ui(ui, &mut result);
                    ui.separator();
                    self.commands[0].scroll_area_up.ui(ui, &mut result);
                    self.commands[0].scroll_area_down.ui(ui, &mut result);
                    self.commands[0].scroll_area_left.ui(ui, &mut result);
                    self.commands[0].scroll_area_right.ui(ui, &mut result);
                });
                self.commands[0].mirror_mode.ui(ui, &mut result);

                ui.separator();
                if ui
                    .add_enabled(has_buffer, egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-edit-sauce")).wrap(false))
                    .clicked()
                {
                    result = Some(Message::EditSauce);
                    ui.close_menu();
                }
                self.commands[0].lga_font.ui(ui, &mut result);
                self.commands[0].aspect_ratio.ui(ui, &mut result);
                ui.separator();

                if ui
                    .add_enabled(has_buffer, egui::Button::new(fl!(crate::LANGUAGE_LOADER, "menu-set-canvas-size")).wrap(false))
                    .clicked()
                {
                    result = Some(Message::SetCanvasSize);
                    ui.close_menu();
                }
            });

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-selection"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(200.0);
                self.commands[0].select_all.ui(ui, &mut result);
                self.commands[0].deselect.ui(ui, &mut result);
                self.commands[0].inverse_selection.ui(ui, &mut result);
                ui.separator();
                self.commands[0].erase_selection.ui(ui, &mut result);
                self.commands[0].flip_x.ui(ui, &mut result);
                self.commands[0].flip_y.ui(ui, &mut result);
                self.commands[0].justifycenter.ui(ui, &mut result);
                self.commands[0].justifyleft.ui(ui, &mut result);
                self.commands[0].justifyright.ui(ui, &mut result);
                self.commands[0].crop.ui(ui, &mut result);
            });
            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-colors"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(300.0);
                if has_buffer {
                    if let Some(pane) = self.get_active_pane_mut() {
                        let lock = &mut pane.doc.lock();
                        if let Some(editor) = lock.get_ansi_editor_mut() {
                            let lock = &mut editor.buffer_view.lock();
                            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-ice-mode"), |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(240.0);

                                if ui
                                    .selectable_label(
                                        lock.get_buffer().ice_mode == IceMode::Unlimited,
                                        fl!(crate::LANGUAGE_LOADER, "menu-ice-mode-unrestricted"),
                                    )
                                    .clicked()
                                {
                                    result = Some(Message::SwitchIceMode(IceMode::Unlimited));
                                    ui.close_menu();
                                }

                                if ui
                                    .selectable_label(lock.get_buffer().ice_mode == IceMode::Blink, fl!(crate::LANGUAGE_LOADER, "menu-ice-mode-blink"))
                                    .clicked()
                                {
                                    result = Some(Message::SwitchIceMode(IceMode::Blink));
                                    ui.close_menu();
                                }

                                if ui
                                    .selectable_label(lock.get_buffer().ice_mode == IceMode::Ice, fl!(crate::LANGUAGE_LOADER, "menu-ice-mode-ice"))
                                    .clicked()
                                {
                                    result = Some(Message::SwitchIceMode(IceMode::Ice));
                                    ui.close_menu();
                                }
                            });

                            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-palette-mode"), |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(240.0);

                                if ui
                                    .selectable_label(
                                        lock.get_buffer().palette_mode == PaletteMode::RGB,
                                        fl!(crate::LANGUAGE_LOADER, "menu-palette-mode-unrestricted"),
                                    )
                                    .clicked()
                                {
                                    result = Some(Message::SwitchPaletteMode(PaletteMode::RGB));
                                    ui.close_menu();
                                }

                                if ui
                                    .selectable_label(
                                        lock.get_buffer().palette_mode == PaletteMode::Fixed16,
                                        fl!(crate::LANGUAGE_LOADER, "menu-palette-mode-dos"),
                                    )
                                    .clicked()
                                {
                                    result = Some(Message::SwitchPaletteMode(PaletteMode::Fixed16));
                                    ui.close_menu();
                                }

                                if ui
                                    .selectable_label(
                                        lock.get_buffer().palette_mode == PaletteMode::Free16,
                                        fl!(crate::LANGUAGE_LOADER, "menu-palette-mode-free"),
                                    )
                                    .clicked()
                                {
                                    result = Some(Message::SwitchPaletteMode(PaletteMode::Free16));
                                    ui.close_menu();
                                }

                                if ui
                                    .selectable_label(
                                        lock.get_buffer().palette_mode == PaletteMode::Free8,
                                        fl!(crate::LANGUAGE_LOADER, "menu-palette-mode-free8"),
                                    )
                                    .clicked()
                                {
                                    result = Some(Message::SwitchPaletteMode(PaletteMode::Free8));
                                    ui.close_menu();
                                }
                            });
                        }
                        ui.separator();
                    }
                }
                self.commands[0].select_palette.ui(ui, &mut result);
                self.commands[0].open_palettes_directory.ui(ui, &mut result);
                ui.separator();

                self.commands[0].next_fg_color.ui(ui, &mut result);
                self.commands[0].prev_fg_color.ui(ui, &mut result);

                ui.separator();

                self.commands[0].next_bg_color.ui(ui, &mut result);
                self.commands[0].prev_bg_color.ui(ui, &mut result);

                self.commands[0].pick_attribute_under_caret.ui(ui, &mut result);
                self.commands[0].toggle_color.ui(ui, &mut result);
                self.commands[0].switch_to_default_color.ui(ui, &mut result);
            });
            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-fonts"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(220.0);
                let mut font_mode = FontMode::Single;
                if let Some(pane) = self.get_active_pane_mut() {
                    if let Some(editor) = pane.doc.lock().get_ansi_editor_mut() {
                        font_mode = editor.buffer_view.lock().get_buffer().font_mode;

                        ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-font-mode"), |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(240.0);

                            let lock = &mut editor.buffer_view.lock();

                            if ui
                                .selectable_label(
                                    lock.get_buffer().font_mode == FontMode::Unlimited,
                                    fl!(crate::LANGUAGE_LOADER, "menu-font-mode-unrestricted"),
                                )
                                .clicked()
                            {
                                lock.get_buffer_mut().font_mode = FontMode::Unlimited;
                                ui.close_menu();
                            }

                            if ui
                                .selectable_label(
                                    lock.get_buffer().font_mode == FontMode::Single,
                                    fl!(crate::LANGUAGE_LOADER, "menu-font-mode-single"),
                                )
                                .clicked()
                            {
                                lock.get_buffer_mut().font_mode = FontMode::Single;
                                ui.close_menu();
                            }

                            if ui
                                .selectable_label(
                                    lock.get_buffer().font_mode == FontMode::Sauce,
                                    fl!(crate::LANGUAGE_LOADER, "menu-font-mode-sauce"),
                                )
                                .clicked()
                            {
                                lock.get_buffer_mut().font_mode = FontMode::Sauce;
                                ui.close_menu();
                            }

                            if ui
                                .selectable_label(
                                    lock.get_buffer().font_mode == FontMode::FixedSize,
                                    fl!(crate::LANGUAGE_LOADER, "menu-font-mode-dual"),
                                )
                                .clicked()
                            {
                                lock.get_buffer_mut().font_mode = FontMode::FixedSize;
                                ui.close_menu();
                            }
                        });
                    }
                }
                self.commands[0].open_font_selector.ui(ui, &mut result);
                if matches!(font_mode, FontMode::Unlimited) {
                    self.commands[0].add_fonts.ui(ui, &mut result);
                }
                self.commands[0].open_font_manager.ui(ui, &mut result);

                ui.separator();
                self.commands[0].open_font_directory.ui(ui, &mut result);
                self.commands[0].open_tdf_directory.ui(ui, &mut result);
            });
            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-view"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(320.0);
                ui.menu_button(
                    fl!(
                        crate::LANGUAGE_LOADER,
                        "menu-zoom",
                        zoom = format!("{}%", (100. * unsafe { SETTINGS.get_scale().x }) as i32)
                    ),
                    |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(270.0);

                        self.commands[0].zoom_reset.ui(ui, &mut result);
                        self.commands[0].zoom_in.ui(ui, &mut result);

                        self.commands[0].zoom_out.ui(ui, &mut result);
                        ui.separator();

                        if ui.button("4:1 400%").clicked() {
                            unsafe { SETTINGS.set_scale(Vec2::new(4.0, 4.0)) };
                            ui.close_menu();
                        }
                        if ui.button("2:1 200%").clicked() {
                            unsafe { SETTINGS.set_scale(Vec2::new(2.0, 2.0)) };
                            ui.close_menu();
                        }
                        if ui.button("1:1 100%").clicked() {
                            unsafe { SETTINGS.set_scale(Vec2::new(1.0, 1.0)) };
                            ui.close_menu();
                        }
                        if ui.button("1:2 50%").clicked() {
                            unsafe { SETTINGS.set_scale(Vec2::new(0.5, 0.5)) };
                            ui.close_menu();
                        }
                        if ui.button("1:4 25%").clicked() {
                            unsafe { SETTINGS.set_scale(Vec2::new(0.25, 0.25)) };
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui
                            .checkbox(
                                &mut self.document_behavior.document_options.fit_width,
                                fl!(crate::LANGUAGE_LOADER, "menu-zoom-fit_size"),
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    },
                );

                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-guides"), |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(200.0);
                    if ui.selectable_label(cur_guide == Some(Vec2::new(80.0, 25.0)), "Smallscale 80x25").clicked() {
                        result = Some(Message::SetGuide(80, 25));
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_guide == Some(Vec2::new(80.0, 40.0)), "Square 80x40").clicked() {
                        result = Some(Message::SetGuide(80, 40));
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_guide == Some(Vec2::new(80.0, 50.0)), "Instagram 80x50").clicked() {
                        result = Some(Message::SetGuide(80, 50));
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_guide == Some(Vec2::new(44.0, 22.0)), "File_ID.DIZ 44x22").clicked() {
                        result = Some(Message::SetGuide(44, 22));
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .selectable_label(cur_guide.is_none(), fl!(crate::LANGUAGE_LOADER, "menu-guides-off"))
                        .clicked()
                    {
                        result = Some(Message::SetGuide(0, 0));
                        ui.close_menu();
                    }
                });

                ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-raster"), |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(100.0);

                    let raster = [(1, 1), (2, 2), (4, 2), (4, 4), (8, 2), (8, 4), (8, 8), (16, 4), (16, 8), (16, 16)];
                    for (x, y) in raster {
                        if ui
                            .selectable_label(cur_raster == Some(Vec2::new(x as f32, y as f32)), format!("{x}x{y}"))
                            .clicked()
                        {
                            result = Some(Message::SetRaster(x, y));
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui
                        .selectable_label(cur_raster.is_none(), fl!(crate::LANGUAGE_LOADER, "menu-guides-off"))
                        .clicked()
                    {
                        result = Some(Message::SetRaster(0, 0));
                        ui.close_menu();
                    }
                });
                self.commands[0].toggle_grid_guides.ui(ui, &mut result);

                self.commands[0].show_layer_borders.ui(ui, &mut result);
                self.commands[0].show_line_numbers.ui(ui, &mut result);

                self.commands[0].fullscreen.ui(ui, &mut result);

                ui.separator();
                self.commands[0].set_reference_image.ui(ui, &mut result);

                self.commands[0].toggle_reference_image.is_enabled = has_reference_image;
                self.commands[0].toggle_reference_image.ui(ui, &mut result);
                self.commands[0].clear_reference_image.is_enabled = has_reference_image;
                self.commands[0].clear_reference_image.ui(ui, &mut result);
            });

            unsafe {
                if !PLUGINS.is_empty() {
                    ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-plugins"), |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(250.0);
                        for (i, p) in PLUGINS.iter().enumerate() {
                            if ui.add_enabled(has_buffer, egui::Button::new(p.title.clone()).wrap(false)).clicked() {
                                result = Some(Message::RunPlugin(i));
                                ui.close_menu();
                            }
                        }

                        ui.separator();
                        self.commands[0].open_plugin_directory.ui(ui, &mut result);
                    });
                }
            }

            ui.menu_button(fl!(crate::LANGUAGE_LOADER, "menu-help"), |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(170.0);
                let r = ui.hyperlink_to(fl!(crate::LANGUAGE_LOADER, "menu-discuss"), "https://github.com/mkrueger/icy_draw/discussions");
                if r.clicked() {
                    ui.close_menu();
                }
                let r = ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-report-bug"),
                    "https://github.com/mkrueger/icy_draw/issues/new",
                );
                if r.clicked() {
                    ui.close_menu();
                }
                let r = ui.button(fl!(crate::LANGUAGE_LOADER, "menu-open_log_file"));
                if r.clicked() {
                    if let Ok(log_file) = Settings::get_log_file() {
                        let _ = open::that(log_file);
                    }
                    ui.close_menu();
                }
                ui.separator();
                self.commands[0].about.ui(ui, &mut result);
            });
            self.top_bar_ui(ui, frame);
        });

        result
    }

    fn top_bar_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let right = medium_toggle_button(ui, &DOCK_RIGHT_SVG, self.right_panel);
            if right.clicked() {
                self.right_panel = !self.right_panel;
            }

            let left = medium_toggle_button(ui, &DOCK_LEFT_SVG, self.left_panel);
            if left.clicked() {
                self.left_panel = !self.left_panel;
            }

            if *VERSION < *LATEST_VERSION {
                ui.hyperlink_to(
                    fl!(crate::LANGUAGE_LOADER, "menu-upgrade_version", version = LATEST_VERSION.to_string()),
                    "https://github.com/mkrueger/icy_draw/releases/latest",
                );
            }
        });
    }
}

pub fn medium_toggle_button(ui: &mut egui::Ui, icon: &Image<'_>, selected: bool) -> egui::Response {
    let size_points = egui::Vec2::splat(20.0);

    let tint = if selected {
        ui.visuals().widgets.active.fg_stroke.color
    } else {
        ui.visuals().widgets.inactive.fg_stroke.color
    };

    ui.add(ImageButton::new(icon.clone().fit_to_exact_size(size_points).tint(tint)))
}
