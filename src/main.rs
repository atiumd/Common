use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let icon_data = match eframe::icon_data::from_png_bytes(include_bytes!("buh.png")) {
        Ok(data) => Some(Arc::new(data)),
        Err(e) => {
            eprintln!("Failed to load icon: {}", e);
            None
        }
    };

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0])
        .with_title("Common");
    
    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Common",
        options,
        Box::new(|_cc| Ok(Box::new(ScaleApp::new()))),
    )
}

#[derive(Default)]
struct ScaleApp {
    diag_a: f32,
    res_a_x: u32,
    res_a_y: u32,
    diag_b: f32,
    res_b_x: u32,
    res_b_y: u32,
    monitor_a: Monitor,
    monitor_b: Monitor,
    scale_factor: f32,
    dragging_monitor: Option<usize>,
    last_mouse_pos: Pos2,
}

#[derive(Clone)]
struct Monitor {
    position: Vec2,
    rotation: f32,
    color: Color32,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            color: Color32::from_rgb(100, 150, 200),
        }
    }
}

impl ScaleApp {
    fn new() -> Self {
        let mut app = Self::default();
        app.diag_a = 27.0;
        app.res_a_x = 2560;
        app.res_a_y = 1440;
        app.diag_b = 24.0;
        app.res_b_x = 1920;
        app.res_b_y = 1080;
        app.scale_factor = 30.0;
        
        app.monitor_a.position = Vec2::new(-150.0, 0.0);
        app.monitor_a.color = Color32::from_rgb(100, 150, 200);
        app.monitor_b.position = Vec2::new(150.0, 0.0);
        app.monitor_b.color = Color32::from_rgb(150, 100, 200);
        
        app
    }
    
    fn get_monitor_physical_size(&self, res_x: u32, res_y: u32, diag: f32) -> (f32, f32) {
        let aspect_ratio = res_x as f32 / res_y as f32;
        let height = diag / (aspect_ratio * aspect_ratio + 1.0).sqrt();
        let width = height * aspect_ratio;
        (width, height)
    }
    
    fn draw_monitor(&self, ui: &mut egui::Ui, monitor: &Monitor, width: f32, height: f32, name: &str, center: Pos2) {
        let painter = ui.painter();
        
        let vis_width = width * self.scale_factor;
        let vis_height = height * self.scale_factor;
        
        let pos = center + monitor.position;
        
        let half_w = vis_width / 2.0;
        let half_h = vis_height / 2.0;
        let corners = [
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ];
        
        let cos_r = monitor.rotation.cos();
        let sin_r = monitor.rotation.sin();
        let rotated_corners: Vec<Pos2> = corners
            .iter()
            .map(|corner| {
                let rotated_x = corner.x * cos_r - corner.y * sin_r;
                let rotated_y = corner.x * sin_r + corner.y * cos_r;
                pos + Vec2::new(rotated_x, rotated_y)
            })
            .collect();
        
        painter.add(egui::Shape::closed_line(
            rotated_corners.clone(),
            Stroke::new(3.0, monitor.color),
        ));
        
        let mut fill_color = monitor.color;
        fill_color[3] = 50;
        painter.add(egui::Shape::convex_polygon(
            rotated_corners.clone(),
            fill_color,
            Stroke::NONE,
        ));
        
        let label_pos = pos + Vec2::new(0.0, -vis_height / 2.0 - 20.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId::default(),
            Color32::WHITE,
        );
        
        let info = format!("{:.1}\" × {:.1}\"", width, height);
        let info_pos = pos + Vec2::new(0.0, vis_height / 2.0 + 10.0);
        painter.text(
            info_pos,
            egui::Align2::CENTER_CENTER,
            info,
            egui::FontId::default(),
            Color32::LIGHT_GRAY,
        );
    }
    
    fn is_point_in_monitor(&self, point: Pos2, monitor: &Monitor, width: f32, height: f32, center: Pos2) -> bool {
        let vis_width = width * self.scale_factor;
        let vis_height = height * self.scale_factor;
        let monitor_pos = center + monitor.position;
        
        let relative = point - monitor_pos;
        let cos_r = monitor.rotation.cos();
        let sin_r = monitor.rotation.sin();
        let local_x = relative.x * cos_r + relative.y * sin_r;
        let local_y = -relative.x * sin_r + relative.y * cos_r;
        
        local_x.abs() <= vis_width / 2.0 && local_y.abs() <= vis_height / 2.0
    }
}

impl eframe::App for ScaleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Monitor Settings");
            
            ui.group(|ui| {
                ui.label("Monitor A");
                ui.horizontal(|ui| {
                    ui.label("Diagonal (inches):");
                    ui.add(egui::DragValue::new(&mut self.diag_a).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Resolution:");
                    ui.add(egui::DragValue::new(&mut self.res_a_x));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut self.res_a_y));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    let mut deg = self.monitor_a.rotation * 180.0 / PI;
                    ui.add(egui::DragValue::new(&mut deg).speed(1.0).suffix("°"));
                    self.monitor_a.rotation = deg * PI / 180.0;
                });
                ui.color_edit_button_srgba(&mut self.monitor_a.color);
            });
            
            ui.separator();
            
            ui.group(|ui| {
                ui.label("Monitor B");
                ui.horizontal(|ui| {
                    ui.label("Diagonal (inches):");
                    ui.add(egui::DragValue::new(&mut self.diag_b).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Resolution:");
                    ui.add(egui::DragValue::new(&mut self.res_b_x));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut self.res_b_y));
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    let mut deg = self.monitor_b.rotation * 180.0 / PI;
                    ui.add(egui::DragValue::new(&mut deg).speed(1.0).suffix("°"));
                    self.monitor_b.rotation = deg * PI / 180.0;
                });
                ui.color_edit_button_srgba(&mut self.monitor_b.color);
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Visualization Scale:");
                ui.add(egui::DragValue::new(&mut self.scale_factor).speed(1.0));
            });
            
            if ui.button("Reset Positions").clicked() {
                self.monitor_a.position = Vec2::new(-150.0, 0.0);
                self.monitor_b.position = Vec2::new(150.0, 0.0);
                self.monitor_a.rotation = 0.0;
                self.monitor_b.rotation = 0.0;
            }
            
            ui.separator();
            
            if self.diag_a > 0.0 && self.diag_b > 0.0 {
                let ppi_a = calc_ppi(self.res_a_x, self.res_a_y, self.diag_a);
                let ppi_b = calc_ppi(self.res_b_x, self.res_b_y, self.diag_b);
                
                ui.label(format!("Monitor A PPI: {:.2}", ppi_a));
                ui.label(format!("Monitor B PPI: {:.2}", ppi_b));
                
                let scale = ppi_b / ppi_a;
                ui.label(format!(
                    "100px on A = {:.1}px on B",
                    100.0 * scale
                ));
                
                let (width_a, height_a) = self.get_monitor_physical_size(self.res_a_x, self.res_a_y, self.diag_a);
                let (width_b, height_b) = self.get_monitor_physical_size(self.res_b_x, self.res_b_y, self.diag_b);
                
                ui.label(format!("A: {:.1}\" × {:.1}\"", width_a, height_a));
                ui.label(format!("B: {:.1}\" × {:.1}\"", width_b, height_b));
            }
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Visualizer");
            ui.label("Drag monitors to move them. Use the rotation controls in the sidebar.");
            
            let available_rect = ui.available_rect_before_wrap();
            let center = available_rect.center();
            
            let mouse_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
            let mouse_pressed = ctx.input(|i| i.pointer.primary_pressed());
            let mouse_released = ctx.input(|i| i.pointer.primary_released());
            
            if self.diag_a > 0.0 && self.diag_b > 0.0 {
                let (width_a, height_a) = self.get_monitor_physical_size(self.res_a_x, self.res_a_y, self.diag_a);
                let (width_b, height_b) = self.get_monitor_physical_size(self.res_b_x, self.res_b_y, self.diag_b);
                
                if mouse_pressed && self.dragging_monitor.is_none() {
                    if self.is_point_in_monitor(mouse_pos, &self.monitor_a, width_a, height_a, center) {
                        self.dragging_monitor = Some(0);
                    } else if self.is_point_in_monitor(mouse_pos, &self.monitor_b, width_b, height_b, center) {
                        self.dragging_monitor = Some(1);
                    }
                    self.last_mouse_pos = mouse_pos;
                }
                
                if mouse_released {
                    self.dragging_monitor = None;
                }
                
                if let Some(monitor_idx) = self.dragging_monitor {
                    let delta = mouse_pos - self.last_mouse_pos;
                    match monitor_idx {
                        0 => self.monitor_a.position += delta,
                        1 => self.monitor_b.position += delta,
                        _ => {}
                    }
                    self.last_mouse_pos = mouse_pos;
                }
                
                self.draw_monitor(ui, &self.monitor_a, width_a, height_a, "Monitor A", center);
                self.draw_monitor(ui, &self.monitor_b, width_b, height_b, "Monitor B", center);
                
                let painter = ui.painter();
                painter.circle_filled(center, 3.0, Color32::RED);
                painter.text(
                    center + Vec2::new(10.0, -10.0),
                    egui::Align2::LEFT_BOTTOM,
                    "Center",
                    egui::FontId::default(),
                    Color32::RED,
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Enter monitor specifications to see visualization");
                });
            }
        });
    }
}

fn calc_ppi(x: u32, y: u32, diag_inch: f32) -> f32 {
    let diag_px = ((x * x + y * y) as f32).sqrt();
    diag_px / diag_inch
}