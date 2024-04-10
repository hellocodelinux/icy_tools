use eframe::egui::{self, Ui};
use i18n_embed_fl::fl;
use icy_engine::SaveOptions;

pub fn create_settings_page(ui: &mut Ui, options: &mut SaveOptions) {
    ui.vertical(|ui| {
        ui.checkbox(&mut options.compress, fl!(crate::LANGUAGE_LOADER, "export-compression-label"));
        ui.add(egui::Checkbox::new(
            &mut options.save_sauce,
            fl!(crate::LANGUAGE_LOADER, "export-save-sauce-label"),
        ));
    });
}
