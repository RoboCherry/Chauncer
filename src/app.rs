use egui::{Context, FontFamily, FontId, Key, RichText, TextStyle};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ChauncerApp {
    apps : Vec<String>
}

impl Default for ChauncerApp {
    fn default() -> Self {
        Self {
            apps : Vec::new()
        }
    }
}

impl ChauncerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for ChauncerApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //set_styles(ctx);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked(){
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Fullscreen").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    }
                    if ui.button("Hover Window").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    }
                });
                    ui.add_space(16.0);


                //egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Applications");
            
            if ui.input(|i| i.key_pressed(Key::Escape)){
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if ui.button("Delete Apps [-]").clicked(){
                self.apps = Vec::new()
            }
            if ui.button("Add App [+]").clicked() {
                let _ = &mut self.apps.push("heyo".to_string());
            }
            
            for i in &mut self.apps.iter(){
                ui.label(RichText::new(i));
            }
        });
    }
}


fn set_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [(TextStyle::Heading,FontId::new(30.0, FontFamily::Monospace))].into();
    ctx.set_style(style);
    
}