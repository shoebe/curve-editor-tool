use egui::{Vec2b, Widget};
use egui_plot::{Line, PlotPoint, PlotPoints, Points};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct Value {
    x: f64,
    y: f64,
}

impl Value {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct Point {
    time: f64,
    val: Value,
}

impl Point {
    pub fn new(time: f64, val: Value) -> Self {
        Self { time, val }
    }
}

pub struct TemplateApp {
    points: Vec<Point>,
    time_mult: f64,
    x_mult: f64,
    y_mult: f64,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            points: vec![
                Point::new(0.0, Value::new(0.0, 0.0)),
                Point::new(1.0, Value::new(1.0, 1.0)),
            ],
            time_mult: 1.0,
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
                for (v, name) in [
                    (&mut self.time_mult, "time_mult: "),
                    (&mut self.x_mult, "x_mult: "),
                    (&mut self.y_mult, "y_mult: "),
                ] {
                    egui::DragValue::new(v)
                        .clamp_to_range(true)
                        .range(1.0..=f64::MAX)
                        .prefix(name)
                        .speed(0.01)
                        .ui(ui);
                }
            });

            let mut i = 0;
            let mut last = Point::new(0.0, Value::new(0.0, 0.0));
            let mut move_time = 0.0;

            let can_remove = self.points.len() > 2;

            let mut add_point = None;

            self.points.retain_mut(|point| {
                let mut retain = true;
                ui.horizontal(|ui| {
                    point.time += move_time;
                    let o_time = point.time;
                    let mut tmp_time = point.time * self.time_mult;

                    let r = egui::DragValue::new(&mut tmp_time)
                        .clamp_to_range(true)
                        .range(last.time..=self.time_mult)
                        .prefix("time: ")
                        .speed(0.01)
                        .ui(ui);
                    if r.changed() {
                        point.time = tmp_time / self.time_mult;
                        move_time = point.time - o_time;
                    }

                    for (val, name, mult) in [
                        (&mut point.val.x, "x: ", self.x_mult),
                        (&mut point.val.y, "y: ", self.y_mult),
                    ] {
                        let mut tmp = *val * mult;

                        let r = egui::DragValue::new(&mut tmp)
                            .clamp_to_range(true)
                            .range(0.0..=mult)
                            .prefix(name)
                            .speed(0.01)
                            .ui(ui);

                        if r.changed() {
                            *val = tmp / mult;
                        }
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
                            .map(|p| TotalCmpF64(p.val.x))
                            .max()
                            .unwrap()
                            .0;
                        let max_y = self
                            .points
                            .iter()
                            .map(|p| TotalCmpF64(p.val.y))
                            .max()
                            .unwrap()
                            .0;
                        for p in self.points.iter_mut() {
                            p.val.x /= max_x;
                            p.val.y /= max_y;
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("integrals:");
                let int_x = intergral_of_points(self.points.iter().map(|p| (p.time, p.val.x)));
                let int_y = intergral_of_points(self.points.iter().map(|p| (p.time, p.val.y)));
                let mut integral = ron::ser::to_string(&(int_x, int_y)).unwrap();
                ui.text_edit_singleline(&mut integral);
            });

            let s = ui.available_size();

            ui.horizontal(|ui| {
                ui.set_max_height(s.y);
                dbg!(s);
                let points_x: Vec<_> = self
                    .points
                    .iter()
                    .map(|p| PlotPoint::new(p.time * self.time_mult, p.val.x * self.x_mult))
                    .collect();
                let points_y: Vec<_> = self
                    .points
                    .iter()
                    .map(|p| PlotPoint::new(p.time * self.time_mult, p.val.y * self.y_mult))
                    .collect();
                for (points, name) in [(points_x, "x plot"), (points_y, "y plot")] {
                    egui_plot::Plot::new(name)
                        .allow_drag(Vec2b::new(true, true))
                        .allow_zoom(Vec2b::new(true, true))
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .allow_boxed_zoom(false)
                        .data_aspect(1.0)
                        .width(s.x / 2.0)
                        .height(s.x / 2.0)
                        .auto_bounds(Vec2b::new(false, false))
                        .show(ui, |plot_ui| {
                            let mut last = *points.first().unwrap();
                            plot_ui.points(
                                Points::new(PlotPoints::Owned(vec![last]))
                                    .radius(5.0)
                                    .filled(true)
                                    .shape(egui_plot::MarkerShape::Circle),
                            );
                            for p in points[1..].iter().copied() {
                                plot_ui.points(
                                    Points::new(PlotPoints::Owned(vec![p]))
                                        .radius(5.0)
                                        .filled(true)
                                        .shape(egui_plot::MarkerShape::Circle),
                                );
                                plot_ui.line(Line::new(PlotPoints::Owned(vec![last, p])));
                                last = p;
                            }
                        });
                }
            });
        });
    }
}

fn intergral_of_points(mut iter: impl Iterator<Item = (f64, f64)>) -> f64 {
    let mut integral = 0.0;

    let Some(mut last) = iter.next() else {
        return 0.0;
    };

    for next in iter {
        let d_t = next.0 - last.0;
        let d_y = next.1 - last.1;

        let m = d_y / d_t;

        match (last.1 >= 0.0, next.1 >= 0.0) {
            (true, true) | (false, false) => {
                // triangle at top
                integral += d_t * d_y.abs() / 2.0 * next.1.signum();
                // square at bottom
                integral += d_t * f64::min(last.1.abs(), next.1.abs()) * next.1.signum();
            }
            (true, false) | (false, true) => {
                // 2 triangles, crossing zero

                let t1 = last.1 / m;
                let t2 = d_t - t1;

                assert!(t1 >= 0.0);
                assert!(t2 >= 0.0);

                integral += t1 * last.1 / 2.0;
                integral += t2 * next.1 / 2.0;
            }
        }

        last = next;
    }

    integral
}
