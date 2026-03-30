use crate::{GasAnalysisError, GasAnalysisResult, utils::asegurar_directorio};
use polars::prelude::*;
use std::path::Path;
use tracing::info;

pub struct ProcesamientoConfig {
    pub input_path: String,
    pub output_dir: String,
}

impl Default for ProcesamientoConfig {
    fn default() -> Self {
        Self {
            input_path: String::from("data/interim/datos_filtrados.parquet"),
            output_dir: String::from("data/processed/datos_por_dia/"),
        }
    }
}

pub fn ejecutar_procesamiento(config: &ProcesamientoConfig) -> GasAnalysisResult<()> {
    info!("=== PROCESAMIENTO ===");
    asegurar_directorio(Path::new(&config.output_dir)).map_err(GasAnalysisError::IoError)?;

    let file = std::fs::File::open(&config.input_path).map_err(GasAnalysisError::IoError)?;
    let df = ParquetReader::new(file).finish().map_err(GasAnalysisError::PolarsError)?;
    
    let df_filtrado = df.clone().lazy()
        .with_column(col("co2_ppm").shift(lit(1)).alias("co2_prev"))
        .with_column(col("ch4_ppm").shift(lit(1)).alias("ch4_prev"))
        .filter(col("co2_ppm").neq(col("co2_prev")).or(col("ch4_ppm").neq(col("ch4_prev"))))
        .drop(["co2_prev", "ch4_prev"])
        .collect()
        .map_err(GasAnalysisError::PolarsError)?;

    info!("Post-filtro: {} filas", df_filtrado.height());

    let fechas: Vec<String> = df_filtrado.column("fecha_dia")
        .map_err(GasAnalysisError::PolarsError)?
        .unique().map_err(GasAnalysisError::PolarsError)?
        .cast(&DataType::String).map_err(GasAnalysisError::PolarsError)?
        .str().map_err(GasAnalysisError::PolarsError)?
        .into_iter().flatten().map(|s: &str| s.to_string()).collect();

    for fecha in fechas {
        let df_dia = df_filtrado.clone().lazy()
            .filter(col("fecha_dia").eq(lit(fecha.as_str()).cast(DataType::Date)))
            .sort("fecha_parsed", SortOptions::default())  // ✅ String, no array
            .collect()
            .map_err(GasAnalysisError::PolarsError)?;
            
        if df_dia.height() > 0 {
            let nombre = fecha.replace("-", "_");
            let path_out = format!("{}/datos_{}.parquet", config.output_dir, nombre);
            let file = std::fs::File::create(&path_out).map_err(GasAnalysisError::IoError)?;
            ParquetWriter::new(file).finish(&mut df_dia.clone())
                .map_err(GasAnalysisError::PolarsError)?;
            info!("✅ {}", path_out);
        }
    }
    Ok(())
}