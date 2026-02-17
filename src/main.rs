mod app;

fn main() -> eframe::Result {

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(false)
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Chauncer",
        native_options,
        Box::new(|cc| Ok(Box::new(app::ChauncerApp::new(cc)))),
    )
}