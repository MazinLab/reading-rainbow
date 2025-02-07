// Nikki Zivkov 02/06/2025
// This script generates a template for the gui 
// Will be filled in later with actual gui elements 
// Called to in main 


// Importing crates/modules
use eframe::{egui, App, NativeOptions, CreationContext};
use egui_plot::{Line, Plot}; // Gives functionality to see x,y when cursor hovers over plot 
use crate::sweep::SineWave; // Importing sin wave struct from sweep.rs 

// Defining structs 
#[derive(Default)]
pub struct MyApp {
    current_pane: Pane, // Keeps track of current pane
    sine_wave: SineWave, // Test sin wave structure 
    // Fill this in later with more structs for other panes 
}

// Defining different panes in the gui
#[derive(PartialEq)]
enum Pane {
    Settings,
    Readout,
    Pump,
    Test,
    // Change these later with actual panes 
}

impl Default for Pane {
    fn default() -> Self {
        Pane::Settings
    }
}

// Defining each gui pane/clickable functionality
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

        // Showing the central pane selected
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

                    // Adding sliders to the test pane where we have sin wave 
                    // egui_plot crate has built in functionality to print out x,y when cursor hovers 

                    
                    // Specify that we are adjusting amplitude 
                    ui.horizontal(|ui| {
                        ui.label("Amplitude:");
                        ui.add(egui::Slider::new(&mut self.sine_wave.amplitude, 0.0..=10.0));
                    });

                    // Specify that we are adjusting phase 
                    ui.horizontal(|ui| {
                        ui.label("Phase:");
                        ui.add(egui::Slider::new(&mut self.sine_wave.phase, 0.0..=2.0 * std::f64::consts::PI));
                    });

                    // Plot the sin wave for each adjustment 
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


// Outputting the gui 
pub fn run_gui() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Reading Rainbow",
        native_options,
        Box::new(|cc: &CreationContext| {
            let mut fonts = egui::FontDefinitions::default(); // Mininal fonts (don't delete )
            // fonts.font_data.clear(); // Uncomment to remove default fonts (you need to upload a customf font file)
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MyApp::default())) // Don't remove 
        }),
    ).unwrap_or_else(|e| eprintln!("Failed to run native: {}", e));
}