use eframe::egui;
use egui::Widget;
use egui_smith_chart::{Plane, SmithChart};

fn main() {
    let options = eframe::NativeOptions {
        default_theme: eframe::Theme::Dark,
        initial_window_size: Some(egui::vec2(800.0, 840.0)),
        ..Default::default()
    };

    eframe::run_native(
        "egui-smith-chart demo",
        options,
        Box::new(|_cc| {
            Box::new(SmithChartDemo {
                ..Default::default()
            })
        }),
    );
}

struct SmithChartDemo {
    chart_size: f32,
    chart_plane: Plane,
    mouse_vswr: bool,
    chart_debug: bool,
}

impl Default for SmithChartDemo {
    fn default() -> Self {
        Self {
            chart_size: 400.0,
            chart_plane: Plane::Impedance,
            mouse_vswr: false,
            chart_debug: false,
        }
    }
}

impl eframe::App for SmithChartDemo {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Options");
            ui.collapsing("preview options", |ui| {
                egui::ComboBox::from_label("Plane")
                    .selected_text(self.chart_plane.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.chart_plane,
                            Plane::Impedance,
                            Plane::Impedance.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.chart_plane,
                            Plane::Admittance,
                            Plane::Admittance.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.chart_plane,
                            Plane::Both,
                            Plane::Both.to_string(),
                        );
                    });
                egui::Slider::new(&mut self.chart_size, 64.0..=2048.0)
                    .text("Chart size")
                    .ui(ui);
                ui.checkbox(&mut self.mouse_vswr, "Mouse VSWR");
                ui.checkbox(&mut self.chart_debug, "Debug");
            });

            ui.separator(); //---------------------------------------------------------------------------

            // ui.heading("Plot Point");
            // egui::Slider::new(&mut self.point.re, 0.0..=5.0)
            //     .step_by(0.001)
            //     .text("normalized resistance")
            //     .ui(ui);
            // egui::Slider::new(&mut self.point.im, -10.0..=10.0)
            //     .step_by(0.001)
            //     .text("normalized reactance")
            //     .ui(ui);

            ui.separator(); //---------------------------------------------------------------------------

            ui.horizontal(|ui| {
                SmithChart::new("smith-chart-demo")
                    .size(self.chart_size)
                    .plane(Plane::Impedance)
                    .mouse_vswr(self.mouse_vswr)
                    .debug(self.chart_debug)
                    .show(ui);
            });
        });
    }
}
