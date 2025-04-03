// Nikki Zivkov 02/06/2025
// This script generates a template for the gui
// Will be filled in later with actual gui elements
// Called to in main

// Importing crates/modules
use crate::logger::Logger;
use crate::status::Status;
use crate::worker::{RPCCommand, RPCResponse};
use eframe::{egui, App, CreationContext, NativeOptions};
use num::Complex;
use std::process::Command; // Importing Command for running shell commands
use std::sync::mpsc::{Receiver, Sender};
use gen3_rpc::{Hertz, Attens}; // Importing Hertz and Attens for IF board
use gen3_rpc::utils::client::{PowerSetting, SweepConfig}; // Corrected imports for PowerSetting and SweepConfig
use std::time::{SystemTime, UNIX_EPOCH, Duration};

// Defining structs
pub struct MyApp {
    current_pane: Pane,     // Keeps track of current pane
    command_input: String,  // Command line input
    command_output: String, // Command line output
    logger: Option<Logger>, // Data logger
    status: Status,         // Device status
    settings: Settings,
    command: Sender<RPCCommand>,
    response: Receiver<RPCResponse>,
    error_message: Option<String>, // Add a field for the error message
    dac_table: Option<Box<[Complex<i16>; 524288]>>, // Add a field for the DAC table
    if_freq: Option<Hertz>, // Add a field for the IF frequency
    if_attens: Option<Attens>, // Add a field for the IF attenuations
    connection_time: Option<SystemTime>, // Add a field for the connection timestamp
    sweep_freqs: String, // Input for sweep frequencies (comma-separated)
    sweep_settings: String, // Input for power settings (comma-separated)
    sweep_average: String, // Input for the average value
    sweep_result: Option<String>, // Field to display the sweep result
}

// Defining different panes in the gui
#[derive(PartialEq, Default)]
enum Pane {
    #[default]
    Settings,
    Command, // New pane for command line
    DataLogging,
    Status,
    DSPScale, // New pane for DSP scale adjustment
    DACTable, // New pane for DAC table operations
    IFBoard, // New pane for IF board operations
    Sweep, // New pane for SweepConfig
}

#[derive(Default)]
struct Settings {
    fft_scale: String, // Use String to handle text input
    if_freq: String, // Use String to handle IF frequency input
    if_input_atten: String, // Use String to handle IF input attenuation
    if_output_atten: String, // Use String to handle IF output attenuation
}

// Defining each gui pane/clickable functionality
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(c) = self.response.try_recv() {
            match c {
                // Update the FFT scale in the settings
                RPCResponse::FFTScale(i) => {
                    self.settings.fft_scale = i.map_or_else(|| "0".to_string(), |v| v.to_string());
                }
                // Update the DAC table
                RPCResponse::DACTable(d) => {
                    self.dac_table = d;
                }
                // Update the IF frequency
                RPCResponse::IFFreq(f) => {
                    self.if_freq = f;
                }
                // Update the IF attenuations
                RPCResponse::IFAttens(a) => {
                    self.if_attens = a;
                }
                // Update the connection status
                RPCResponse::Connected(time) => {
                    self.status.update("Connected");
                    self.connection_time = Some(time);
                }
                RPCResponse::Sweep(sweep) => {
                    self.sweep_result = Some(format!("{:?}", sweep));
                }
            }
        }
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Menu");

            if ui.button("Settings").clicked() {
                self.current_pane = Pane::Settings;
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
            if ui.button("DSP Scale Adjustment").clicked() {
                self.current_pane = Pane::DSPScale;
            }
            if ui.button("DAC Table").clicked() {
                self.current_pane = Pane::DACTable;
            }
            if ui.button("IF Board").clicked() {
                self.current_pane = Pane::IFBoard;
            }
            if ui.button("Sweep").clicked() {
                self.current_pane = Pane::Sweep;
            }
        });

        // Showing the central pane selected
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_pane {
                Pane::Settings => {
                    ui.heading("Settings");

                    // Display connection status and timestamp
                    if let Some(connection_time) = self.connection_time {
                        let duration = connection_time.elapsed().unwrap_or(Duration::new(0, 0));
                        ui.label(format!("Successfully connected to server"));
                        ui.label(format!("Connection duration: {:.2?}", duration));
                    } else {
                        ui.label("Not connected to server");
                    }
                }
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
                Pane::Status => {
                    ui.heading("Status");
                    ui.label(&self.status.status_message);
                }
                Pane::DSPScale => {
                    ui.heading("DSP Scale Adjustment");

                    // Button to request the current DSP scale
                    if ui.button("Get DSP Scale").clicked() {
                        self.command.send(RPCCommand::GetFFTScale).unwrap();
                    }

                    // Display the current DSP scale if available
                    ui.label(format!("Current DSP Scale: {}", self.settings.fft_scale));

                    // Text input for adjusting scale
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.text_edit_singleline(&mut self.settings.fft_scale);
                    });

                    // Display valid range and example values
                    ui.label("Valid values: 4095, 3967, 1919, 1911, 1879, 1877, 1365, 1301, 277, 273, 257, 1, 0");

                    // Button to apply scale
                    if ui.button("Apply Scale").clicked() {
                        let valid_values = vec![4095, 3967, 1919, 1911, 1879, 1877, 1365, 1301, 277, 273, 257, 1, 0];
                        if let Ok(scale_value) = self.settings.fft_scale.parse::<u16>() {
                            // Only pass the value to the worker.rs if it is within the accepted values
                            if valid_values.contains(&scale_value) {
                                if let Err(e) = set_scale(&self.command, scale_value) {
                                    self.error_message = Some(format!("Failed to set scale: {}", e));
                                } else {
                                    self.error_message = None; // Clear the error message on success
                                }
                            } else {
                                // Display an error message if the value is not valid
                                self.error_message = Some("Invalid scale value. Please enter one of the valid values.".to_string());
                            }
                        } else {
                            // Display an error message if the value is not a valid number
                            self.error_message = Some("Invalid scale value. Please enter one of the valid values.".to_string());
                        }
                    }

                    // Display the error message if it exists
                    if let Some(ref error_message) = self.error_message {
                        ui.label(error_message);
                    }
                }
                Pane::DACTable => {
                    ui.heading("DAC Table");

                    // Button to request the current DAC table
                    if ui.button("Get DAC Table").clicked() {
                        self.command.send(RPCCommand::GetDACTable).unwrap();
                    }

                    // Display the current DAC table if available
                    if let Some(ref dac_table) = self.dac_table {
                        ui.label(format!("DAC Table: {:?}", &dac_table[..16]));
                    }

                    // Text input for setting DAC table
                    ui.horizontal(|ui| {
                        ui.label("DAC Table Data:");
                        ui.text_edit_singleline(&mut self.settings.fft_scale); // Use a different field for DAC table input
                    });

                    // Button to set the DAC table
                    if ui.button("Set DAC Table").clicked() {
                        // Parse the user input and set the DAC table
                        let data: Box<[Complex<i16>; 524288]> = Box::new([Complex::new(0, 0); 524288]); // Replace with actual user input parsing
                        if let Err(e) = set_dac_table(&self.command, data) {
                            self.error_message = Some(format!("Failed to set DAC table: {}", e));
                        } else {
                            self.error_message = None; // Clear the error message on success
                        }
                    }

                    // Display the error message if it exists
                    if let Some(ref error_message) = self.error_message {
                        ui.label(error_message);
                    }
                }
                Pane::IFBoard => {
                    ui.heading("IF Board");

                    // Button to request the current IF frequency
                    if ui.button("Get IF Frequency").clicked() {
                        self.command.send(RPCCommand::GetIFFreq).unwrap();
                    }

                    // Display the current IF frequency if available
                    if let Some(ref if_freq) = self.if_freq {
                        ui.label(format!("Current IF Frequency: {}/{}", if_freq.numer(), if_freq.denom()));
                    }

                    // Text input for setting IF frequency
                    ui.horizontal(|ui| {
                        ui.label("IF Frequency:");
                        ui.text_edit_singleline(&mut self.settings.if_freq);
                    });

                    // Button to set the IF frequency
                    if ui.button("Set IF Frequency").clicked() {
                        // Parse the user input and set the IF frequency
                        let parts: Vec<&str> = self.settings.if_freq.split('/').collect();
                        if parts.len() == 2 {
                            if let (Ok(numer), Ok(denom)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                let freq = Hertz::new(numer, denom);
                                if let Err(e) = set_if_freq(&self.command, freq) {
                                    self.error_message = Some(format!("Failed to set IF frequency: {}", e));
                                } else {
                                    self.error_message = None; // Clear the error message on success
                                }
                            } else {
                                self.error_message = Some("Invalid IF frequency. Please enter valid numerator and denominator.".to_string());
                            }
                        } else {
                            self.error_message = Some("Invalid IF frequency format. Please use the format numerator/denominator.".to_string());
                        }
                    }

                    // Button to request the current IF attenuations
                    if ui.button("Get IF Attenuations").clicked() {
                        self.command.send(RPCCommand::GetIFAttens).unwrap();
                    }

                    // Display the current IF attenuations if available
                    if let Some(ref if_attens) = self.if_attens {
                        ui.label(format!("Current IF Attenuations - Input: {}, Output: {}", if_attens.input, if_attens.output));
                    }

                    // Text input for setting IF attenuations
                    ui.horizontal(|ui| {
                        ui.label("IF Input Attenuation:");
                        ui.text_edit_singleline(&mut self.settings.if_input_atten);
                    });
                    ui.horizontal(|ui| {
                        ui.label("IF Output Attenuation:");
                        ui.text_edit_singleline(&mut self.settings.if_output_atten);
                    });

                    // Button to set the IF attenuations
                    if ui.button("Set IF Attenuations").clicked() {
                        // Parse the user input and set the IF attenuations
                        if let (Ok(input), Ok(output)) = (self.settings.if_input_atten.parse::<i32>(), self.settings.if_output_atten.parse::<i32>()) {
                            let attens = Attens {
                                input: input as f32,
                                output: output as f32,
                            };
                            if let Err(e) = set_if_attens(&self.command, attens) {
                                self.error_message = Some(format!("Failed to set IF attenuations: {}", e));
                            } else {
                                self.error_message = None; // Clear the error message on success
                            }
                        } else {
                            self.error_message = Some("Invalid IF attenuations. Please enter valid input and output attenuations.".to_string());
                        }
                    }

                    // Display the error message if it exists
                    if let Some(ref error_message) = self.error_message {
                        ui.label(error_message);
                    }
                }
                Pane::Sweep => {
                    ui.heading("Sweep Configuration");

                    // Input for sweep frequencies
                    ui.horizontal(|ui| {
                        ui.label("Frequencies (comma-separated, e.g., 6000000000/1,6020000000/1):");
                        ui.text_edit_singleline(&mut self.sweep_freqs);
                    });

                    // Input for power settings
                    ui.horizontal(|ui| {
                        ui.label("Power Settings (comma-separated, e.g., 60/60/4095):");
                        ui.text_edit_singleline(&mut self.sweep_settings);
                    });

                    // Input for average value
                    ui.horizontal(|ui| {
                        ui.label("Average:");
                        ui.text_edit_singleline(&mut self.sweep_average);
                    });

                    // Button to trigger the sweep
                    if ui.button("Perform Sweep").clicked() {
                        // Parse the frequencies
                        let freqs = self
                            .sweep_freqs
                            .split(',')
                            .filter_map(|f| {
                                let parts: Vec<&str> = f.split('/').collect();
                                if parts.len() == 2 {
                                    if let (Ok(numer), Ok(denom)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                        Some(Hertz::new(numer, denom))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        // Parse the power settings
                        let settings = self
                            .sweep_settings
                            .split(',')
                            .filter_map(|s| {
                                let parts: Vec<&str> = s.split('/').collect();
                                if parts.len() == 3 {
                                    if let (Ok(input), Ok(output), Ok(fft_scale)) =
                                        (parts[0].parse::<f32>(), parts[1].parse::<f32>(), parts[2].parse::<u16>())
                                    {
                                        Some(PowerSetting {
                                            attens: Attens { input, output },
                                            fft_scale,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        // Parse the average value
                        if let Ok(average) = self.sweep_average.parse::<u64>() {
                            // Send the SweepConfig command
                            let config = SweepConfig {
                                freqs,
                                settings,
                                average,
                            };
                            self.command.send(RPCCommand::SweepConfig(config)).unwrap();
                        } else {
                            self.error_message = Some("Invalid average value.".to_string());
                        }
                    }

                    // Display the sweep result if available
                    if let Some(ref result) = self.sweep_result {
                        ui.label(format!("Sweep Result: {}", result));
                    }

                    // Display the error message if it exists
                    if let Some(ref error_message) = self.error_message {
                        ui.label(error_message);
                    }
                }
            }
        });
    }
}

// Function to run a command and return the output
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

// Function to set the scale value
fn set_scale(tx: &Sender<RPCCommand>, scale: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting scale to: {}", scale);
    tx.send(RPCCommand::SetFFTScale(scale))?;
    Ok(())
}

// Function to set the DAC table
fn set_dac_table(tx: &Sender<RPCCommand>, data: Box<[Complex<i16>; 524288]>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting DAC table");
    tx.send(RPCCommand::SetDACTable(data))?;
    Ok(())
}

// Function to set the IF frequency
fn set_if_freq(tx: &Sender<RPCCommand>, freq: Hertz) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting IF frequency to: {}/{}", freq.numer(), freq.denom());
    tx.send(RPCCommand::SetIFFreq(freq))?;
    Ok(())
}

// Function to set the IF attenuations
fn set_if_attens(tx: &Sender<RPCCommand>, attens: Attens) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting IF attenuations - Input: {}, Output: {}", attens.input, attens.output);
    tx.send(RPCCommand::SetIFAttens(attens))?;
    Ok(())
}


// Outputting the gui
pub fn run_gui(command: Sender<RPCCommand>, response: Receiver<RPCResponse>) {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Reading Rainbow",
        native_options,
        Box::new(|cc: &CreationContext| {
            let fonts = egui::FontDefinitions::default(); // Minimal fonts (don't delete)
            // fonts.font_data.clear(); // Uncomment to remove default fonts (you need to upload a custom font file)
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MyApp {
                current_pane: Pane::Settings,
                command_input: String::new(),
                command_output: String::new(),
                logger: None,
                status: Status::new(),
                settings: Settings::default(),
                command,
                response,
                error_message: None,
                dac_table: None,
                if_freq: None,
                if_attens: None,
                connection_time: None,
                sweep_freqs: String::new(),
                sweep_settings: String::new(),
                sweep_average: String::new(),
                sweep_result: None,
            }))
        }),
    )
    .unwrap_or_else(|e| eprintln!("Failed to run native: {}", e));
}
