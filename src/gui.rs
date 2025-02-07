// Nikki Zivkov 02/06/2025
// This script generates a template for the gui 
// Will be filled in later with actual gui elements 
// Called to in main 
use eframe::{egui, App, NativeOptions, CreationContext};
use egui_plot::{Line, Plot};
use crate::sweep::SineWave;

#[derive(Default)]
pub struct MyApp {
    current_pane: Pane,
    sine_wave: SineWave,
}

#[derive(PartialEq)]
enum Pane {
    Settings,
    Readout,
    Pump,
    Test,
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Settings
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Menu");

            if ui.button("Settings").clicked() {
                self.current_pane = Pane::Settings;
            }
            if ui.button("Readout").clicked() {
                self.current_pane = Pane::Readout;
            }
            if ui.button("Pump Tone Generation").clicked() {
                self.current_pane = Pane::Pump;
            }
            if ui.button("Test Pane").clicked() {
                self.current_pane = Pane::Test;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_pane {
                Pane::Settings => {
                    ui.heading("Settings");
                    ui.label("More info to come");
                }
                Pane::Readout => {
                    ui.heading("Readout");
                    ui.label("Fill this in later");
                }
                Pane::Pump => {
                    ui.heading("Pump Tone Generation");
                    ui.label("Fridge Stuff ");
                }
                Pane::Test => {
                    ui.heading("Test Pane");
                    ui.label("Sample Pane");

                    ui.horizontal(|ui| {
                        ui.label("Amplitude:");
                        ui.add(egui::Slider::new(&mut self.sine_wave.amplitude, 0.0..=10.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Phase:");
                        ui.add(egui::Slider::new(&mut self.sine_wave.phase, 0.0..=2.0 * std::f64::consts::PI));
                    });

                    let points = self.sine_wave.generate_points();
                    Plot::new("Sine Wave")
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points));
                        });
                }
            }
        });
    }
}

pub fn run_gui() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Reading Rainbow",
        native_options,
        Box::new(|cc: &CreationContext| {
            // Set up a minimal font configuration
            let mut fonts = egui::FontDefinitions::default();
            // fonts.font_data.clear(); // Remove this line to keep the default fonts
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MyApp::default()))
        }),
    ).unwrap_or_else(|e| eprintln!("Failed to run native: {}", e));
}