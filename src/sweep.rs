// Nikki Zivkov 02/07/2025
// Contains sin wave struct and implementation
// Applied to gui.rs to launch sin wave in gui pane 

use egui_plot::PlotPoints;
use std::f64::consts::PI;

#[derive(Default)]
pub struct SineWave {
    pub amplitude: f64,
    pub phase: f64,
}

impl SineWave {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    pub fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude;
    }

    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    pub fn generate_points(&self) -> PlotPoints {
        let points: Vec<_> = (0..1000)
            .map(|i| {
                let x = i as f64 * 2.0 * PI / 1000.0;
                let y = self.amplitude * (x + self.phase).sin();
                [x, y]
            })
            .collect();
        PlotPoints::from(points)
    }
}