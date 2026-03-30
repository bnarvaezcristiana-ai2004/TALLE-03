use crate::{GasAnalysisResult, utils::asegurar_directorio};
use polars::prelude::*;
use std::path::Path;
use tracing::info;

pub struct PreparacionConfig {
    pub input_path: String,
    pub output_path: String,
}

impl Default for PreparacionConfig {
    fn default() -> Self {
        Self {
            input_path: String::from("data/raw/datosbase.csv"),
            output_path: String::from("data/interim/datos_filtrados.parquet"),
        }
    }
}

pub fn ejecutar_preparacion(config: &PreparacionConfig) -> GasAnalysisResult<()> {
    info!("=== INICIANDO PREPARACIÓN ===");
    asegurar_directorio(Path::new(&config.output_path).parent().unwrap())?;

    // Leer CSV con LazyCsvReader (API correcta para 0.38)
    let df = LazyCsvReader::new(&config.input_path)
        .with_has_header(true)
        .finish()?
        .select([
            col("entry_id"),
            col("field2").cast(DataType::Float64).alias("co2_ppm"),
            col("field4").cast(DataType::Float64).alias("ch4_ppm"),
            col("created_at"),
        ])
        .with_column(
            col("created_at")
                .cast(DataType::String)
                .str()
                .to_datetime(
                    None,
                    None,
                    StrptimeOptions {
                        format: Some("%Y-%m-%d %H:%M:%S".into()),
                        ..Default::default()
                    },
                    lit("raise"),
                )
                .alias("fecha_parsed")
        )
        .with_column(
            col("fecha_parsed").dt().date().alias("fecha_dia")
        )
        .collect()
        .map_err(GasAnalysisError::PolarsError)?;

    // Guardar en Parquet con API correcta
    let file = std::fs::File::create(&config.output_path)?;
    ParquetWriter::new(file)
        .with_compression(ParquetCompression::ZSTD(None))
        .finish(&mut df.clone())
        .map_err(GasAnalysisError::PolarsError)?;

    info!("✅ Preparación completada. Filas: {}", df.height());
    Ok(())
}