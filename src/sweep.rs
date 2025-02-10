// Nikki Zivkov 02/07/2025
// Contains sin wave struct and implementation
// Applied to gui.rs to launch sin wave in gui pane 

// Importing crates/modules 
use egui_plot::PlotPoints;
use std::f64::consts::PI;

// Defining SinWave struct 
#[derive(Default)]
pub struct SineWave {
    pub amplitude: f64, // Field for amplitude 
    pub phase: f64, // Field for phase 
}

// Implimenting struct
// Creating a SinWave struct with default amp and phase 
#[allow(dead_code)]
impl SineWave {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            phase: 0.0,
        }
    }
    // Set the amplitude of SinWave (defined in struct) 
    pub fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude;
    }

    // Set the phase of SinWave (defined in struct)
    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    // Making plot plots 
    pub fn generate_points(&self) -> PlotPoints {
        let points: Vec<_> = (0..1000) // Setting to 1000 points, can be adjusted 
            .map(|i| {
                let x = i as f64 * 2.0 * PI / 1000.0; // Solving for x position 
                let y = self.amplitude * (x + self.phase).sin(); // Solving for y position 
                [x, y] // Returning x,y point 
            })
            .collect();
        PlotPoints::from(points)
    }
}