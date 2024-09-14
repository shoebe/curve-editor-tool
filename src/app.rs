use egui::{Vec2b, Widget};
use egui_plot::{Line, PlotPoint, PlotPoints, Points};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<Point> for PlotPoint {
    fn from(val: Point) -> Self {
        PlotPoint::new(val.x, val.y)
    }
}

impl From<Point> for PlotPoints {
    fn from(val: Point) -> Self {
        [val].into_points()
    }
}

trait IntoPlotPoints {
    fn into_points(self) -> PlotPoints;
}

impl<const N: usize> IntoPlotPoints for [Point; N] {
    fn into_points(self) -> PlotPoints {
        PlotPoints::Owned(self.iter().copied().map(|p| p.into()).collect())
    }
}

pub struct TemplateApp {
    points: Vec<Point>,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            points: vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)],
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut i = 0;
            let mut last = Point::new(0.0, 0.0);
            self.points.retain_mut(|point| {
                let mut retain = true;
                ui.horizontal(|ui| {
                    egui::DragValue::new(&mut point.x)
                        .clamp_to_range(true)
                        .range(last.x..=f64::MAX)
                        .prefix("x: ")
                        .speed(0.01)
                        .ui(ui);
                    egui::DragValue::new(&mut point.y)
                        .prefix("y: ")
                        .speed(0.01)
                        .ui(ui);
                    ui.label("Linear");
                    if i >= 2 && ui.button("remove").clicked() {
                        retain = false;
                    }
                });
                i += 1;
                last = *point;
                retain
            });

            if ui.button("add point").clicked() {
                let mut n = *self.points.last().unwrap();
                n.x += 0.1;
                self.points.push(n);
            }
            let mut txt = ron::ser::to_string(&self.points).unwrap();
            let r = ui.text_edit_singleline(&mut txt);
            if r.changed() {
                if let Ok(p) = ron::de::from_str::<Vec<Point>>(&txt) {
                    if p.len() >= 2 {
                        self.points = p;
                    }
                }
            }
            egui_plot::Plot::new("plot")
                .allow_drag(Vec2b::new(true, false))
                .allow_zoom(Vec2b::new(true, false))
                .allow_scroll(false)
                .allow_double_click_reset(false)
                .allow_boxed_zoom(false)
                .show(ui, |plot_ui| {
                    let mut last = *self.points.first().unwrap();
                    plot_ui.points(
                        Points::new(last)
                            .radius(5.0)
                            .filled(true)
                            .shape(egui_plot::MarkerShape::Circle),
                    );
                    for p in self.points[1..].iter().copied() {
                        plot_ui.points(
                            Points::new(p)
                                .radius(5.0)
                                .filled(true)
                                .shape(egui_plot::MarkerShape::Circle),
                        );
                        plot_ui.line(Line::new([last, p].into_points()));
                        last = p;
                    }
                });
        });
    }
}
