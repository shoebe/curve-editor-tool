use std::time::Duration;

use egui::Vec2b;
use egui_plot::{Line, PlotPoint, PlotPoints, Points};

use crate::math::{EguiPlotPointExt, closest_point_on_line};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct Point {
    time: f64,
    val: f64,
}

impl Point {
    pub fn new(time: f64, val: f64) -> Self {
        Self { time, val }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct AppPoints {
    x: Vec<Point>,
    y: Vec<Point>,
}

impl Default for AppPoints {
    fn default() -> Self {
        Self {
            x: vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)],
            y: vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq)]
enum PlotId {
    X,
    Y,
}

#[derive(Default)]
pub struct TemplateApp {
    points: AppPoints,
    selected_point_i: Option<(PlotId, usize)>,
    toasts: egui_notify::Toasts,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        //} else {
        //    Default::default()
        //}
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        self.toasts.show(ui.ctx());

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut txt = ron::ser::to_string(&self.points).unwrap();

            ui.horizontal(|ui| {
                let r = ui.text_edit_singleline(&mut txt);
                if !r.changed() {
                    return;
                }

                match ron::de::from_str::<AppPoints>(&txt) {
                    Ok(p) => {
                        if p.x.len() >= 2 && p.y.len() >= 2 {
                            self.points = p;
                        } else {
                            self.toasts
                                .error("x or y list did not have at least 2 elements")
                                .duration(Duration::from_secs(5));
                        }
                        use total_cmp_float_wrapper::TotalCmpF64;
                        for ps in [&mut self.points.x, &mut self.points.y] {
                            let max = ps.iter().map(|p| TotalCmpF64(p.val)).max().unwrap().0;
                            for p in ps {
                                p.val /= max;
                            }
                        }
                    }
                    Err(e) => {
                        self.toasts
                            .error(format!("err: {e:?}"))
                            .duration(Duration::from_secs(5));
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("integrals:");
                let int_x = intergral_of_points(self.points.x.iter().map(|p| (p.time, p.val)));
                let int_y = intergral_of_points(self.points.y.iter().map(|p| (p.time, p.val)));
                let mut integral = ron::ser::to_string(&(int_x, int_y)).unwrap();
                ui.text_edit_singleline(&mut integral);
            });

            let s = ui.available_size();

            const POINTER_DISTANCE_SNAP: f64 = 0.02;

            if ui.input(|input| input.key_down(egui::Key::Escape)) {
                self.selected_point_i = None;
            }

            ui.vertical_centered(|ui| {
                ui.set_max_height(s.y);
                dbg!(s);
                let points_x: Vec<_> = self
                    .points
                    .x
                    .iter()
                    .map(|p| PlotPoint::new(p.time, p.val))
                    .collect();
                let points_y: Vec<_> = self
                    .points
                    .y
                    .iter()
                    .map(|p| PlotPoint::new(p.time, p.val))
                    .collect();
                let dim = f32::min(s.x, s.y / 2.0);
                for (points, name, plot_id) in [
                    (points_x, "x plot", PlotId::X),
                    (points_y, "y plot", PlotId::Y),
                ] {
                    egui_plot::Plot::new(name)
                        .allow_drag(Vec2b::new(false, false))
                        .allow_zoom(Vec2b::new(false, false))
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .allow_boxed_zoom(false)
                        .data_aspect(1.0)
                        .view_aspect(1.0)
                        .width(dim)
                        .height(dim)
                        .auto_bounds(Vec2b::new(false, false))
                        .show(ui, |plot_ui| {
                            plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                                [0.0, 0.0],
                                [1.0, 1.0],
                            ));

                            if let Some((sel_plot_id, point_i)) = self.selected_point_i {
                                if sel_plot_id == plot_id {
                                    let point = match plot_id {
                                        PlotId::X => self.points.x[point_i],
                                        PlotId::Y => self.points.y[point_i],
                                    };

                                    let plot_point =
                                        egui_plot::PlotPoint::new(point.time, point.val);
                                    if plot_ui.response().clicked() {
                                        let pointer_pos =
                                            plot_ui.response().interact_pointer_pos().unwrap();
                                        let pointer_plot = plot_ui.plot_from_screen(pointer_pos);

                                        if plot_point.distance(pointer_plot) > POINTER_DISTANCE_SNAP
                                        {
                                            self.selected_point_i = None;
                                        }
                                    }
                                } else {
                                    if plot_ui.response().clicked() {
                                        self.selected_point_i = None;
                                    }
                                }
                            }

                            if let Some(hover_pos) = plot_ui.response().hover_pos() {
                                let hover_plot_pos = plot_ui.plot_from_screen(hover_pos);
                                let (point_i, dist) = points
                                    .iter()
                                    .copied()
                                    .enumerate()
                                    .map(|(point_i, point)| {
                                        let dist_sq = (hover_plot_pos.x - point.x).powi(2)
                                            + (hover_plot_pos.y - point.y).powi(2);
                                        (point_i, dist_sq.sqrt())
                                    })
                                    .min_by_key(|(_point_i, dist)| {
                                        total_cmp_float_wrapper::TotalCmpF64(*dist)
                                    })
                                    .unwrap();

                                if dist < POINTER_DISTANCE_SNAP
                                    && plot_ui.response().is_pointer_button_down_on()
                                {
                                    self.selected_point_i = Some((plot_id, point_i));
                                }
                            }

                            if let Some((sel_plot_id, point_i)) = self.selected_point_i
                                && sel_plot_id == plot_id
                                && plot_ui.response().dragged()
                                && let Some(drag_pos) = plot_ui.response().interact_pointer_pos()
                            {
                                let drag_plot_pos = plot_ui.plot_from_screen(drag_pos);

                                let points = match plot_id {
                                    PlotId::X => &mut self.points.x,
                                    PlotId::Y => &mut self.points.y,
                                };

                                let time_frozen = point_i == 0 || point_i == (points.len() - 1);

                                if !time_frozen {
                                    points[point_i].time = drag_plot_pos.x;
                                }

                                points[point_i].val = drag_plot_pos.y;
                            }

                            for (p_i, p) in points.iter().copied().enumerate() {
                                let is_sel = Some((plot_id, p_i)) == self.selected_point_i;
                                plot_ui.points(
                                    Points::new("point", PlotPoints::Owned(vec![p]))
                                        .id(egui::Id::new("point").with(p_i))
                                        .radius(5.0)
                                        .filled(true)
                                        .shape(egui_plot::MarkerShape::Circle)
                                        .highlight(is_sel),
                                );
                            }

                            let mut last = *points.first().unwrap();
                            let mut it = points.iter().copied().enumerate();
                            it.next(); // skip first elem

                            for (p_i, p) in it {
                                plot_ui.line(
                                    Line::new("line", PlotPoints::Owned(vec![last, p]))
                                        .id(egui::Id::new("line").with(p_i)),
                                );
                                if self.selected_point_i.is_none()
                                    && let Some(pos) = plot_ui.response().hover_pos()
                                {
                                    let plot_pos = plot_ui.plot_from_screen(pos);
                                    let (distance, pos) = closest_point_on_line(
                                        plot_pos.to_pos2(),
                                        last.to_pos2(),
                                        p.to_pos2(),
                                    );
                                    if distance < 0.02 {
                                        plot_ui.points(
                                            Points::new(
                                                "potential_new_point",
                                                PlotPoints::Owned(vec![egui_plot::PlotPoint::new(
                                                    pos.x, pos.y,
                                                )]),
                                            )
                                            .radius(5.0)
                                            .filled(true)
                                            .shape(egui_plot::MarkerShape::Circle),
                                        );
                                    }
                                }
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
