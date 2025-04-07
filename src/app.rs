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
    x_mult: f64,
    y_mult: f64,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            points: vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)],
            x_mult: 1.0,
            y_mult: 1.0,
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
            ui.horizontal(|ui| {
                egui::DragValue::new(&mut self.x_mult)
                    .clamp_to_range(true)
                    .range(1.0..=f64::MAX)
                    .prefix("x_mult: ")
                    .speed(0.01)
                    .ui(ui);
                egui::DragValue::new(&mut self.y_mult)
                    .clamp_to_range(true)
                    .range(1.0..=f64::MAX)
                    .prefix("y_mult: ")
                    .speed(0.01)
                    .ui(ui);
            });

            let mut i = 0;
            let mut last = Point::new(0.0, 0.0);
            let mut move_x = 0.0;

            let can_remove = self.points.len() > 2;

            let mut add_point = None;

            self.points.retain_mut(|point| {
                let mut retain = true;
                ui.horizontal(|ui| {
                    point.x += move_x;
                    let o_x = point.x;
                    let mut tmp_x = point.x * self.x_mult;

                    let r = egui::DragValue::new(&mut tmp_x)
                        .clamp_to_range(true)
                        .range(last.x..=self.x_mult)
                        .prefix("x: ")
                        .speed(0.01)
                        .ui(ui);
                    if r.changed() {
                        point.x = tmp_x / self.x_mult;
                        move_x = point.x - o_x;
                    }

                    let mut tmp_y = point.y * self.y_mult;

                    let r = egui::DragValue::new(&mut tmp_y)
                        .clamp_to_range(true)
                        .range(0.0..=self.y_mult)
                        .prefix("y: ")
                        .speed(0.01)
                        .ui(ui);

                    if r.changed() {
                        point.y = tmp_y / self.y_mult;
                    }

                    ui.label("Linear");
                    if ui.add_enabled(can_remove, egui::Button::new("-")).clicked() {
                        retain = false;
                    }
                    if ui.button("+").clicked() {
                        add_point = Some(i);
                    }
                });
                i += 1;
                last = *point;
                retain
            });

            if let Some(i) = add_point {
                let n = self.points[i];
                self.points.insert(i + 1, n);
            }
            let mut txt = ron::ser::to_string(&self.points).unwrap();

            ui.horizontal(|ui| {
                let r = ui.text_edit_singleline(&mut txt);
                if r.changed() {
                    if let Ok(p) = ron::de::from_str::<Vec<Point>>(&txt) {
                        if p.len() >= 2 {
                            self.points = p;
                        }
                        use total_cmp_float_wrapper::TotalCmpF64;
                        let max_x = self
                            .points
                            .iter()
                            .map(|p| TotalCmpF64(p.x))
                            .max()
                            .unwrap()
                            .0;
                        let max_y = self
                            .points
                            .iter()
                            .map(|p| TotalCmpF64(p.y))
                            .max()
                            .unwrap()
                            .0;
                        for p in self.points.iter_mut() {
                            p.x /= max_x;
                            p.y /= max_y;
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("integral:");
                let mut integral = ron::ser::to_string(&intergral_of_points(&self.points)).unwrap();
                ui.text_edit_singleline(&mut integral);
            });

            egui_plot::Plot::new("plot")
                .allow_drag(Vec2b::new(true, false))
                .allow_zoom(Vec2b::new(true, true))
                .allow_scroll(false)
                .allow_double_click_reset(false)
                .allow_boxed_zoom(false)
                .data_aspect(1.0)
                .auto_bounds(Vec2b::new(false, false))
                .show(ui, |plot_ui| {
                    let mut last = *self.points.first().unwrap();
                    last.x *= self.x_mult;
                    last.y *= self.y_mult;
                    plot_ui.points(
                        Points::new(last)
                            .radius(5.0)
                            .filled(true)
                            .shape(egui_plot::MarkerShape::Circle),
                    );
                    for mut p in self.points[1..].iter().copied() {
                        p.x *= self.x_mult;
                        p.y *= self.y_mult;
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

fn intergral_of_points(points: &[Point]) -> f64 {
    let mut iter = points.into_iter();

    let mut integral = 0.0;

    let Some(mut last) = iter.next() else {
        return 0.0;
    };

    for next in iter {
        let d_x = next.x - last.x;
        let d_y = next.y - last.y;

        let m = d_y / d_x;

        match (last.y >= 0.0, next.y >= 0.0) {
            (true, true) | (false, false) => {
                // triangle at top
                integral += d_x * d_y.abs() / 2.0 * next.y.signum();
                // square at bottom
                integral += d_x * f64::min(last.y.abs(), next.y.abs()) * next.y.signum();
            }
            (true, false) | (false, true) => {
                // 2 triangles, crossing zero

                let t1 = last.y / m;
                let t2 = d_x - t1;

                assert!(t1 >= 0.0);
                assert!(t2 >= 0.0);

                integral += t1 * last.y / 2.0;
                integral += t2 * next.y / 2.0;
            }
        }

        last = next;
    }

    integral
}
