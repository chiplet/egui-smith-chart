use std::{borrow::Borrow, error::Error, rc::Rc};

use egui::epaint::PathShape;
use egui::{
    plot::{self, PlotPoints},
    pos2, vec2, Align2, Color32, FontId, Id, Painter, Pos2, Rect, Sense, Stroke, Vec2,
};
use num::traits::Pow;
use num::Complex;

// TODO: add theme support
// TODO: don't normalized to clipping plane, it's not necessarily a square if the window is resized.

// signature pink debug colour
const DEBUG_PINK: Color32 = Color32::from_rgb(255, 0, 255);

#[derive(PartialEq, Eq)]
pub enum Plane {
    Impedance,
    Admittance,
    Both,
}
impl ToString for Plane {
    fn to_string(&self) -> String {
        match self {
            Self::Impedance => "impedance",
            Self::Admittance => "admittance",
            Self::Both => "impedance and admittance",
        }
        .to_string()
    }
}

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct SmithChart {
    id_source: Id,

    /// Characteristic impedance
    Z0: Complex<f32>,

    /// Impedance, Admittance or Both
    plane: Plane,

    size: f32,

    /// Draw debug shapes
    debug: bool,

    /// Enable drawing of VSWR circle under mouse position
    mouse_vswr: bool,
}
impl SmithChart {
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id_source: Id::new(id_source),
            Z0: Complex { re: 50.0, im: 0.0 },
            plane: Plane::Impedance,
            size: 64.0,
            debug: false,
            mouse_vswr: false,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:
        let desired_size = Vec2::splat(self.size);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let mut painter = ui.painter().with_clip_rect(rect);

        let mut local_pos = None;
        if let Some(pos) = response.hover_pos() {
            local_pos = Some(self.abs_to_local(&rect, &pos.to_vec2()));
        }

        // 4. Paint!
        // Make sure we need to paint:
        if ui.is_rect_visible(rect) {
            // let (response, painter) =
            //     ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

            // We will follow the current style by asking
            // "how should something that is being interacted with be painted?".
            // This will, for instance, give us different colors when the widget is hovered or clicked.
            let visuals = ui.style().interact(&response);
            let normal_line = Stroke::new(1.0, visuals.fg_stroke.color);
            let strong_line = Stroke::new(3.0, visuals.fg_stroke.color);
            // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
            let rect = rect.expand(visuals.expansion);

            // draw reactance circles
            let coarse_reactances = vec![0.4, 1.0, 3.0];
            for x in coarse_reactances {
                self.reactance_arc(ui, &mut painter, x, &normal_line);
                self.reactance_arc(ui, &mut painter, -x, &normal_line);
            }

            // draw resistance circles
            let coarse_resistances = [0.0, 1.0 / 3.0, 1.0, 3.0];
            for r in coarse_resistances {
                self.resistance_circle(ui, &mut painter, r, &normal_line);
            }
            // emphasize r=0 and r=1
            for r in [0.0, 1.0] {
                self.resistance_circle(ui, &mut painter, r, &strong_line);
            }

            // zero reactance/susceptance curve (x-axis)
            let xaxis_start_abs = self.local_to_abs(&rect, &vec2(-1.0, 0.0));
            let xaxis_end_abs = self.local_to_abs(&rect, &vec2(1.0, 0.0));
            painter.line_segment(
                [xaxis_start_abs.to_pos2(), xaxis_end_abs.to_pos2()],
                normal_line,
            );

            // plot points/curves to Smith chart
            // match plot_points {
            //     PlotPoints::Points(points) => {
            //         for p in points {
            //             let gamma = self.z_to_gamma(p);
            //             let local = self.gamma_to_local(&gamma);
            //             let center_pos = self.local_to_abs(&rect, &local).to_pos2();
            //             painter.circle_filled(center_pos, 8.0, Color32::YELLOW);
            //         }
            //     },
            //     PlotPoints::Range(_) => todo!(),
            // }

            if let Some(local_pos) = local_pos {
                let mouse_impedance = self.gamma_to_z(&Complex {
                    re: local_pos.x,
                    im: local_pos.y,
                });
                if self.debug {
                    println!(
                        "Mouse Local (Gamma) = ({}, {}), z = {:?}",
                        local_pos.x, local_pos.y, mouse_impedance
                    );
                }

                // check if mouse is inside the Smith chart
                if local_pos.length() < 1.0 {
                    // draw resistance and reactance circles under mouse
                    self.resistance_circle(
                        ui,
                        &mut painter,
                        mouse_impedance.re,
                        &Stroke::new(1.0, Color32::GREEN),
                    );
                    self.reactance_arc(
                        ui,
                        &mut painter,
                        mouse_impedance.im,
                        &Stroke::new(1.0, Color32::RED),
                    );

                    const font_size: f32 = 14.0;
                    painter.text(
                        rect.left_bottom() + vec2(0.0, -3.0 * font_size),
                        Align2::LEFT_CENTER,
                        format!("Z0 = {:.3}", self.Z0),
                        FontId::monospace(font_size),
                        Color32::WHITE,
                    );
                    painter.text(
                        rect.left_bottom() + vec2(0.0, -2.0 * font_size),
                        Align2::LEFT_CENTER,
                        format!(
                            "r = {:+.3}, R = {:+2.3}",
                            mouse_impedance.re,
                            (mouse_impedance * self.Z0).re
                        ),
                        FontId::monospace(font_size),
                        Color32::GREEN,
                    );
                    painter.text(
                        rect.left_bottom() + vec2(0.0, -font_size),
                        Align2::LEFT_CENTER,
                        format!(
                            "x = {:+.3}, X = {:+2.3}",
                            mouse_impedance.im,
                            (mouse_impedance * self.Z0).im
                        ),
                        FontId::monospace(font_size),
                        Color32::RED,
                    );

                    // draw VSWR circle
                    if self.mouse_vswr {
                        let rel_center = egui::vec2(0.0, 0.0);
                        let rel_radius = local_pos.length();
                        let center = self.local_to_abs(&painter.clip_rect(), &rel_center);
                        let radius = self.scale(&painter.clip_rect(), rel_radius);
                        painter.circle(
                            center.to_pos2(),
                            radius,
                            Color32::TRANSPARENT,
                            Stroke::new(1.0, Color32::GOLD),
                        );
                    }
                }
            }

            // draw debug features
            if self.debug {
                let center = self.local_to_abs(&rect, &vec2(0.0, 0.0)).to_pos2();
                painter.circle(
                    center,
                    1.0,
                    Color32::TRANSPARENT,
                    Stroke::new(5.0, DEBUG_PINK),
                );

                if let Some(pos) = response.hover_pos() {
                    painter.line_segment([center, pos], Stroke::new(1.0, Color32::DARK_RED));
                }

                // bounding box
                painter.rect(
                    rect,
                    egui::Rounding::none(),
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, DEBUG_PINK),
                );
            }
        }

        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        response
    }

    /// Impedance, Admittance, or Both
    pub fn plane(mut self, plane: Plane) -> Self {
        self.plane = plane;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn mouse_vswr(mut self, show: bool) -> Self {
        self.mouse_vswr = show;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// return
    fn abs_to_local(&self, rect: &Rect, abs: &Vec2) -> Vec2 {
        let widget_origin = rect.left_top();
        vec2(
            (abs.x - widget_origin.x) / rect.width() * 2.0 - 1.0,
            -(abs.y - widget_origin.y) / rect.height() * 2.0 + 1.0,
        )
    }

    fn local_to_abs(&self, rect: &Rect, local: &Vec2) -> Vec2 {
        let x_normalized = (local.x + 1.0) / 2.0;
        let y_normalized = (local.y + 1.0) / 2.0;
        let abs_origin = rect.left_top();
        vec2(
            abs_origin.x + x_normalized * rect.width(),
            abs_origin.y + (1.0 - y_normalized) * rect.height(),
        )
    }

    fn scale(&self, rect: &Rect, x: f32) -> f32 {
        x * rect.width() / 2.0
    }

    fn resistance_circle(&self, ui: &mut egui::Ui, painter: &mut Painter, r: f32, stroke: &Stroke) {
        let rel_center = egui::vec2(r / (1.0 + r), 0.0);
        let rel_radius = 1.0 / (1.0 + r);
        let center = self.local_to_abs(&painter.clip_rect(), &rel_center);
        let radius = self.scale(&painter.clip_rect(), rel_radius);
        //let center = egui::pos2(radius, rect.center().y);
        painter.circle(center.to_pos2(), radius, Color32::TRANSPARENT, *stroke);
    }

    fn reactance_arc(
        &self,
        ui: &mut egui::Ui,
        painter: &mut Painter,
        x: f32, // normalized reactance
        stroke: &Stroke,
    ) {
        let arc_points: Vec<Pos2> = if x.abs() >= 1.0 {
            let yend: f32 = (2.0 * x) / (1.0 + x.powf(2.0));
            let n = 128; // TODO: adaptive step count based on arc size

            fn x_gt_one_arc(x: f32, gi: f32) -> f32 {
                1.0 - f32::sqrt((gi * (2.0 - x * gi)) / x)
            }

            (0..=n)
                .map(|i| {
                    let gi = egui::remap(i as f32, 0.0..=(n as f32), 0.0..=yend);
                    self.local_to_abs(&painter.clip_rect(), &vec2(x_gt_one_arc(x, gi), gi))
                        .to_pos2()
                })
                .collect()
        } else {
            let xstart = (x.powf(2.0) - 1.0) / (x.powf(2.0) + 1.0);
            let n = 128; // TODO: adaptive step count based on arc size

            fn x_lt_one_arc(x: f32, gr: f32) -> f32 {
                if x > 0.0 {
                    1.0 / x - f32::sqrt(x.powf(-2.0) - (gr - 1.0).pow(2.0))
                } else {
                    1.0 / x + f32::sqrt(x.powf(-2.0) - (gr - 1.0).pow(2.0))
                }
            }

            (0..=n)
                .map(|i| {
                    let gr = egui::remap(i as f32, 0.0..=(n as f32), xstart..=1.0);
                    self.local_to_abs(&painter.clip_rect(), &vec2(gr, x_lt_one_arc(x, gr)))
                        .to_pos2()
                })
                .collect()
        };
        painter.add(PathShape::line(arc_points, *stroke));
    }

    fn local_to_gamma(&self, local: &Vec2) -> Complex<f32> {
        Complex {
            re: local.x,
            im: -local.y,
        }
    }

    fn gamma_to_local(&self, gamma: &Complex<f32>) -> Vec2 {
        vec2(gamma.re, -gamma.im)
    }

    fn gamma_to_z(&self, gamma: &Complex<f32>) -> Complex<f32> {
        (Complex::from(1.0) + gamma) / (Complex::from(1.0) - gamma)
    }

    fn z_to_gamma(&self, z: &Complex<f32>) -> Complex<f32> {
        (z - Complex::from(1.0)) / (z + Complex::from(1.0))
    }
}
