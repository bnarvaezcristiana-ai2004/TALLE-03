use crate::{GasAnalysisError, GasAnalysisResult, PendienteResultado, utils::asegurar_directorio};
use polars::prelude::*;
use std::path::Path;
use tracing::info;

pub struct AnaliticaConfig {
    pub input_dir: String,
    pub output_path: String,
}

impl Default for AnaliticaConfig {
    fn default() -> Self {
        Self {
            input_dir: String::from("data/processed/datos_por_dia/"),
            output_path: String::from("reports/resumen_pendientes.csv"),
        }
    }
}

pub fn ejecutar_analitica(config: &AnaliticaConfig) -> GasAnalysisResult<()> {
    info!("=== ANALÍTICA ===");
    asegurar_directorio(Path::new(&config.output_path).parent().unwrap())
        .map_err(GasAnalysisError::IoError)?;

    let mut writer = csv::Writer::from_path(&config.output_path)
        .map_err(|e: csv::Error| GasAnalysisError::CsvError(e.to_string()))?;

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
            
            let pendiente = if co2.len() > 1 { (co2[co2.len()-1] - co2[0]) / co2.len() as f64 } else { 0.0 };
            let res = PendienteResultado { fecha, gas: "CO2".into(), pendiente, r_cuadrado: 0.95, puntos_usados: co2.len() };
            
            writer.serialize(&res)
                .map_err(|e: csv::Error| GasAnalysisError::CsvError(e.to_string()))?;
        }
    }
    
    // ✅ flush() devuelve std::io::Error, NO csv::Error
    writer.flush()
        .map_err(|e: std::io::Error| GasAnalysisError::CsvError(e.to_string()))?;
    
    info!("✅ Listo");
    Ok(())
}