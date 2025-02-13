// Nikki Zivkov 02/06/2025
// This script generates a template for the gui
// Will be filled in later with actual gui elements
// Called to in main

// Importing crates/modules
use crate::logger::Logger;
use crate::status::Status;
use crate::sweep::SineWave; // Importing sin wave struct from sweep.rs
use crate::worker::{RPCCommand, RPCResponse};
use eframe::{egui, App, CreationContext, NativeOptions};
use egui_plot::{Line, Plot}; // Gives functionality to see x,y when cursor hovers over plot
use std::process::Command; // Importing Command for running shell commands
use std::sync::mpsc::{Receiver, Sender};

// Defining structs
pub struct MyApp {
    current_pane: Pane,     // Keeps track of current pane
    sine_wave: SineWave,    // Test sin wave structure
    command_input: String,  // Command line input
    command_output: String, // Command line output
    logger: Option<Logger>, // Data logger
    status: Status,         // Device status
    settings: Settings,
    command: Sender<RPCCommand>,
    response: Receiver<RPCResponse>,
}

// Defining different panes in the gui
#[derive(PartialEq, Default)]
enum Pane {
    #[default]
    Settings,
    Readout,
    Pump,
    Test,
    Command, // New pane for command line
    DataLogging,
    Status,
}

#[derive(Default)]
struct Settings {
    fft_scale: Option<u16>,
}

// Defining each gui pane/clickable functionality
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(c) = self.response.try_recv() {
            match c {
                RPCResponse::FFTScale(i) => {
                    self.settings.fft_scale = i;
                }
                RPCResponse::Connected => {
                    self.status.update("Connected");
                }
            }
        }
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
            if ui.button("Command Line").clicked() {
                self.current_pane = Pane::Command;
            }
            if ui.button("Data Logging").clicked() {
                self.current_pane = Pane::DataLogging;
            }
            if ui.button("Status").clicked() {
                self.current_pane = Pane::Status;
            }
        });

        // Showing the central pane selected
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_pane {
                Pane::Settings => {
                    ui.heading("Settings");
                    match self.settings.fft_scale {
                        Some(mut scale) => {
                            let widget = egui::widgets::DragValue::new(&mut scale)
                                .range(0..=0xfff)
                                .hexadecimal(3, false, true)
                                .clamp_existing_to_range(true);
                            if ui.add(widget).changed() {
                                self.command.send(RPCCommand::SetFFTScale(scale)).unwrap()
                            }
                        }
                        None => self.command.send(RPCCommand::GetFFTScale).unwrap(),
                    }
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
                        ui.add(egui::Slider::new(
                            &mut self.sine_wave.phase,
                            0.0..=2.0 * std::f64::consts::PI,
                        ));
                    });

                    // Plot the sin wave for each adjustment
                    let points = self.sine_wave.generate_points();
                    Plot::new("Sine Wave").show(ui, |plot_ui| {
                        plot_ui.line(Line::new(points));
                    });
                }

                // Command line pane
                Pane::Command => {
                    ui.heading("Command Line");

                    // Text input for command
                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        ui.text_edit_singleline(&mut self.command_input);
                        if ui.button("Run").clicked() {
                            self.command_output = run_command(&self.command_input);
                        }
                    });

                    // Display command output
                    ui.label("Output:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.command_output)
                            .desired_width(f32::INFINITY) // Make the output box fill the pane width
                            .desired_rows(10), // Number of output rows (can adjust)
                    );
                }

                // Data logging pane
                // Will create a file log.txt to save logged data
                Pane::DataLogging => {
                    ui.heading("Data Logging");

                    if ui.button("Start Logging").clicked() {
                        self.logger =
                            Some(Logger::new("log.txt").expect("Failed to create log file"));
                        self.status.update("Logging started");
                    }
                    if ui.button("Stop Logging").clicked() {
                        self.logger = None;
                        self.status.update("Logging stopped");
                    }

                    if let Some(logger) = &mut self.logger {
                        logger.log("Sample data").expect("Failed to log data");
                        ui.label("Logging data...");
                    } else {
                        ui.label("Logging stopped.");
                    }
                }

                // Status pane
                Pane::Status => {
                    ui.heading("Status");
                    ui.label(&self.status.status_message);
                }
            }
        });
    }
}

// Function to run a command and return the output
// This will be implimented into the command line pane
fn run_command(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}

// Outputting the gui
pub fn run_gui(command: Sender<RPCCommand>, response: Receiver<RPCResponse>) {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Reading Rainbow",
        native_options,
        Box::new(|cc: &CreationContext| {
            let fonts = egui::FontDefinitions::default(); // Mininal fonts (don't delete )
                                                          // fonts.font_data.clear(); // Uncomment to remove default fonts (you need to upload a customf font file)
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MyApp {
                current_pane: Pane::Settings,
                sine_wave: SineWave::default(),
                command_input: String::new(),
                command_output: String::new(),
                logger: None,
                status: Status::new(),
                settings: Settings::default(),
                command,
                response,
            })) // Don't remove
        }),
    )
    .unwrap_or_else(|e| eprintln!("Failed to run native: {}", e));
}
