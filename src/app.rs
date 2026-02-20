use std::{collections::HashMap, process::Command, time::{Duration, Instant}};
use file_icon_provider::get_file_icon;
use image::{DynamicImage, RgbaImage};
use egui::{Color32, ColorImage, FontFamily, FontId, Id, Key, RichText, TextStyle, TextureHandle, TextureOptions, ThemePreference, Vec2, Window, load::SizedTexture};
use rfd::FileDialog;
use sysinfo::{Pid, Process, ProcessRefreshKind, RefreshKind, System};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct CatapultApp {

    #[serde(skip)]
    delta_time : Duration,
    #[serde(skip)]
    last_instant : Instant,

    apps : Vec<String>,
    selected_app : String,
    apps_aliases : HashMap<String, String>,
    app_play_time : HashMap<String, usize>,

    #[serde(skip)]
    app_texture_handles : HashMap<String, TextureHandle>,

    #[serde(skip)]
    edit : bool,

    #[serde(skip)]
    is_app_selected : bool,
    #[serde(skip)]
    current_app_name : String,
    #[serde(skip)]
    current_path : String,

    #[serde(skip)]
    sys : System,

    #[serde(skip)]
    running_apps : HashMap<String,usize>,
    #[serde(skip)]
    app_to_remove : String,
}

impl Default for CatapultApp {
    fn default() -> Self {
        Self {
            delta_time : Duration::new(0, 0),
            last_instant : Instant::now(),
            apps : Vec::new(),
            apps_aliases : HashMap::new(),
            app_texture_handles : HashMap::new(),
            app_play_time : HashMap::new(),
            selected_app : "".to_string(),
            edit : false,
            is_app_selected : false,
            current_app_name : "".to_string(),
            current_path : "".to_string(),
            sys : System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything())),
            running_apps : HashMap::new(),
            app_to_remove : "".to_string(),
        }
    }
}

impl CatapultApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for CatapultApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        set_stylings(ctx);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
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
            });
        });

        egui::SidePanel::new(egui::panel::Side::Left, Id::new("Left"))
        .min_width(512.0)
            .show(ctx, |ui| {
            
            if self.is_app_selected{
                Window::new("Confirm App Name").show(ctx, |ui|{
                    
                    let sized_image : SizedTexture;

                    if self.app_texture_handles.get(&self.current_path).is_none(){
                        let color_icon = get_color_icon(self.current_path.clone(), [128,128]);
                        let handle = ctx.load_texture("app_icon", color_icon.clone(), TextureOptions::LINEAR);
                        sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(64.0, 64.0));
                        self.app_texture_handles.insert(self.current_path.clone(), handle.clone());
                    } else {
                        let handle = self.app_texture_handles.get(&self.current_path).unwrap();
                        sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(64.0, 64.0));
                    }
                    
                    
                    
                    ui.add(egui::Image::from_texture(sized_image));

                    if self.apps_aliases.get(&self.current_path).is_some(){
                        self.current_app_name = self.apps_aliases.get(&self.current_path).unwrap().to_string();
                    }
                    if self.current_app_name == get_executable_name(&self.current_path){
                        self.current_app_name = "".to_string()
                    }

                    ui.add(egui::TextEdit::singleline(&mut self.current_app_name).hint_text(get_executable_name(&self.current_path)).min_size(Vec2 { x: 512.0, y: 0.0 }));

                    if self.current_app_name == "".to_string(){
                        self.apps_aliases.insert(self.current_path.clone(), get_executable_name(&self.current_path));
                    } else {
                        self.apps_aliases.insert(self.current_path.clone(), self.current_app_name.clone());
                    }

                    ui.label(RichText::new(format!("Executable Path: {}",&self.current_path)));
                    if ui.button("Add App").clicked() || ui.input(|i| i.key_pressed(Key::Enter)){
                        self.current_app_name = "".to_string();
                        if ! self.apps.contains(&self.current_path){
                            let _ = &mut self.apps.push(self.current_path.clone());
                            self.app_play_time.insert(self.current_path.clone(), 0);
                            self.apps.sort_by(|a, b| {
                                let a_name = self.apps_aliases.get(a).unwrap().to_lowercase();
                                let b_name = self.apps_aliases.get(b).unwrap().to_lowercase();
                                a_name.cmp(&b_name)
                            });
                        } else {
                            if self.app_play_time.get(&self.current_path.clone()).is_none(){
                                self.app_play_time.insert(self.current_path.clone(), 0);
                            }
                            self.apps.sort_by(|a, b| {
                                let a_name = self.apps_aliases.get(a).unwrap().to_lowercase();
                                let b_name = self.apps_aliases.get(b).unwrap().to_lowercase();
                                a_name.cmp(&b_name)
                            });
                        }
                        self.is_app_selected = false
                    };
                    if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)){
                        self.is_app_selected = false
                    };
                }); 
            }

            
            ui.heading("Applications");

            ui.add_space(32.0);

            ui.label(format!("Count: {}", self.apps.len()));

            ui.add_space(32.0);
            
            if ui.button("Add App [+]").clicked() {

                let files = FileDialog::new()
                //.add_filter("Executable", &["exe"])
                .set_directory("C:/")
                .pick_file();
                
                if files.is_some(){
                    let picked_path = files.expect("Holy moly this was supposed to be a file!");
                    let path = picked_path.as_path();
                    let exe_path = path.to_str().unwrap();
                    self.current_path = exe_path.to_string();
                    self.is_app_selected = true;
                }
            };
            ui.add_space(16.0);
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in &mut self.apps.iter(){

                    let sized_image : SizedTexture;

                    if self.app_texture_handles.get(&i.to_string()).is_none(){
                        let color_icon = get_color_icon(i.clone(), [128,128]);
                        let handle = ctx.load_texture("app_icon", color_icon.clone(), TextureOptions::LINEAR);
                        sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(48.0, 48.0));
                        self.app_texture_handles.insert(i.clone(), handle.clone());
                    } else {
                        let handle = self.app_texture_handles.get(&i.to_string()).unwrap();
                        sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(48.0, 48.0));
                    }
                    let icon = egui::Image::from_texture(sized_image);

                    let app_name = self.apps_aliases.get(i);
                    let text : RichText;
                    if app_name.is_some(){
                        text = RichText::new(app_name.unwrap().to_string()).size(24.0);
                    } else {
                        text = RichText::new(i).size(24.0);
                    }

                    if ui.add(egui::Button::image_and_text(icon.clone(), text.clone()).min_size(Vec2 { x: 32.0, y: 32.0 })).clicked(){
                        self.selected_app = i.to_string();
                    }
                    ui.add_space(8.0);
                }
            });
        });

        egui::CentralPanel::default()
            .show(ctx, |ui|{
                if self.selected_app != "".to_string() && self.apps.contains(&self.selected_app){
                    let color_icon = get_color_icon(self.selected_app.clone(), [128,128]);
                    let handle = ctx.load_texture("app_icon", color_icon.clone(), TextureOptions::LINEAR);
                    let sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(512.0, 512.0));
                    ui.add(egui::Image::from_texture(sized_image));
                    let app_name = RichText::new(self.apps_aliases.get(&self.selected_app).unwrap()).size(64.0);
                    ui.add(egui::Label::new(app_name));
                    let button_text = RichText::new("LAUNCH >").size(64.0);
                    if ui.add(egui::Button::new(button_text)).clicked(){
                        let pid = open_app(&self.selected_app);
                        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                        track_app(pid, &self);
                        self.running_apps.insert(self.selected_app.clone(), pid);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    };
                    ui.add_space(8.0);
                    if ui.add(egui::Button::new("Edit App")).clicked(){
                        self.edit = true;
                    }
                    let readable_time = time_from_millis(*self.app_play_time.get(&self.selected_app).unwrap_or(&0));
                    ui.label(format!("Time played: {}", readable_time));
                } else {
                    ui.label("Select an App");
                };
                if self.edit{
                    Window::new("Edit App").show(ctx, |ui|{

                        let sized_image : SizedTexture;

                        if self.app_texture_handles.get(&self.selected_app).is_none(){
                            let color_icon = get_color_icon(self.selected_app.clone(), [128,128]);
                            let handle = ctx.load_texture("app_icon", color_icon.clone(), TextureOptions::LINEAR);
                            sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(64.0, 64.0));
                            self.app_texture_handles.insert(self.selected_app.clone(), handle.clone());
                        } else {
                            let handle = self.app_texture_handles.get(&self.selected_app).unwrap();
                            sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(64.0, 64.0));
                        }
                        
                        ui.add(egui::Image::from_texture(sized_image));

                        if self.apps_aliases.get(&self.selected_app).is_some(){
                            self.current_app_name = self.apps_aliases.get(&self.selected_app).unwrap().to_string();
                        }
                        if self.current_app_name == get_executable_name(&self.selected_app){
                            self.current_app_name = "".to_string()
                        }

                        ui.add(egui::TextEdit::singleline(&mut self.current_app_name).hint_text(get_executable_name(&self.selected_app)).min_size(Vec2 { x: 512.0, y: 0.0 }));

                        if self.current_app_name == "".to_string(){
                            self.apps_aliases.insert(self.selected_app.clone(), get_executable_name(&self.selected_app));
                        } else {
                            self.apps_aliases.insert(self.selected_app.clone(), self.current_app_name.clone());
                        }

                        ui.label(RichText::new(format!("Executable Path: {}",&self.selected_app)));
                        
                        if ui.button("Remove").clicked(){
                            self.apps.retain(|path| path != &self.selected_app);
                            if self.apps.len() > 0 {
                                self.selected_app = self.apps.get(0).unwrap().to_string();
                            }
                        }

                        if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(Key::Escape)){
                            self.edit = false
                    };
                }); 
            }

            ctx.request_repaint();        
        });

        self.delta_time = Instant::now().checked_duration_since(self.last_instant).unwrap();
        self.last_instant = Instant::now();

        for app in self.running_apps.keys(){
            let pid = self.running_apps.get(app).unwrap();
            self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            let process = track_app(*pid, &self);
            if process.is_none(){
                self.app_to_remove = app.clone();
            } else {
                let current_play_time = *self.app_play_time.get(app).unwrap_or(&0);
                self.app_play_time.insert(app.clone(), current_play_time + self.delta_time.as_millis() as usize);
            }
        }

        if self.app_to_remove != "".to_string(){
            self.running_apps.remove(&self.app_to_remove);
            self.app_to_remove = "".to_string();
        }

    }
}


fn get_executable_name(path : &String) -> String{
    let mut new_path = path.clone();
    let split_path : Vec<&str> = new_path.split("\\").collect();
    new_path = (*split_path.get(split_path.iter().count() - 1).unwrap()).to_string().replace(".exe", "");
    new_path
}

fn set_stylings(ctx: &egui::Context){
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Monospace, FontId::new(18.0, FontFamily::Monospace)),
        ].into();
    
    style.visuals.panel_fill = Color32::from_hex("#0D0D0E").unwrap();
    style.visuals.window_fill = Color32::from_hex("#0D0D0E").unwrap();
    
    style.visuals.weak_text_color = Some(Color32::from_hex("#323749").unwrap());
    
    
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_hex("#111112").unwrap();
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_hex("#141519").unwrap();
    style.visuals.widgets.active.weak_bg_fill = Color32::from_hex("#202231").unwrap();
    
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0,Color32::from_hex("#323749").unwrap());
    
    style.visuals.widgets.inactive.corner_radius = 4.into();
    style.visuals.widgets.hovered.corner_radius = 4.into();
    style.visuals.widgets.active.corner_radius = 4.into();

    
    ctx.set_style(style);
    ctx.set_theme(ThemePreference::Dark);
}

fn open_app(name : &String) -> usize{
    let mut command = Command::new(name);
    
    if let Ok(child) = command.spawn() {
        child.id() as usize
    } else {
        println!("open command didn't start");
        0
    }
}

fn track_app(pid : usize, app : &CatapultApp) -> Option<&Process>{
    let process = app.sys.process(Pid::from(pid));
    process
}

fn get_color_icon(exe_path : String, size : [usize; 2]) -> ColorImage{
    let app_icon = get_file_icon(exe_path.clone(), 128).expect("Failed to get icon");
    let app_icon_image = RgbaImage::from_raw(app_icon.width, app_icon.height, app_icon.pixels)
        .map(DynamicImage::ImageRgba8)
        .expect("Failed to convert image");
                    
    let color_icon = egui::ColorImage::from_rgba_premultiplied(size, app_icon_image.as_bytes());
    color_icon
}

fn time_from_millis(millis : usize) -> String{
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    format!("{} hours, {} minutes, {} seconds", hours % 24, minutes % 60, seconds % 60)
}