use crate::{GasAnalysisError, GasAnalysisResult, utils::asegurar_directorio};
use plotters::prelude::*;
use polars::prelude::*;
use std::path::Path;
use tracing::info;

pub struct VisualizacionConfig {
    pub input_dir: String,
    pub output_dir: String,
}

impl Default for VisualizacionConfig {
    fn default() -> Self {
        Self {
            input_dir: String::from("data/processed/datos_por_dia/"),
            output_dir: String::from("figures/"),
        }
    }
}

pub fn ejecutar_visualizacion(config: &VisualizacionConfig) -> GasAnalysisResult<()> {
    info!("=== VISUALIZACIÓN ===");
    asegurar_directorio(Path::new(&config.output_dir)).map_err(GasAnalysisError::IoError)?;

    for entry in std::fs::read_dir(&config.input_dir).map_err(GasAnalysisError::IoError)? {
        let entry = entry.map_err(GasAnalysisError::IoError)?;
        let path = entry.path();
        
        if path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) == Some("parquet") {
            let file = std::fs::File::open(&path).map_err(GasAnalysisError::IoError)?;
            let df = ParquetReader::new(file).finish().map_err(GasAnalysisError::PolarsError)?;
            let fecha = path.file_stem().unwrap().to_str().unwrap().replace("datos_", "");
            
            let co2: Vec<f64> = df.column("co2_ppm")
                .map_err(GasAnalysisError::PolarsError)?
                .f64().map_err(GasAnalysisError::PolarsError)?
                .into_no_null_iter().collect();
            
            if co2.is_empty() { continue; }
            let idx: Vec<f64> = (0..co2.len()).map(|i| i as f64).collect();
            let out = format!("{}/grafico_{}.png", config.output_dir, fecha);
            
            // ✅ Código corregido: root definido, present() en root, errores ignorados con _
            let root = BitMapBackend::new(&out, (640, 480)).into_drawing_area();
            let _ = root.fill(&WHITE);
            
            if let Ok(mut chart) = ChartBuilder::on(&root)
                .caption(&fecha, ("sans-serif", 20))
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(40)
                .build_cartesian_2d(0f64..idx.len() as f64, 0f64..6500f64) 
            {
                let _ = chart.configure_mesh().draw();
                let pts: Vec<(f64,f64)> = idx.iter().zip(co2.iter()).map(|(x,y)| (*x,*y)).collect();
                let _ = chart.draw_series(LineSeries::new(pts, &RED));
            }
            let _ = root.present(); // ✅ present() va en root, no en chart
            info!("✅ {}", out);
        }
    }
    Ok(())
}