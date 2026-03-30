pub mod modules;

use anyhow::Result;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum GasAnalysisError {
    #[error("Error IO: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error Polars: {0}")]
    PolarsError(#[from] polars::error::PolarsError),
    #[error("Error CSV: {0}")]
    CsvError(String),
    #[error("Error Plot: {0}")]
    PlotError(String),
}

pub type GasAnalysisResult<T> = Result<T, GasAnalysisError>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PendienteResultado {
    pub fecha: String,
    pub gas: String,
    pub pendiente: f64,
    pub r_cuadrado: f64,
    pub puntos_usados: usize,
}

pub mod utils {
    use super::*;
    pub fn asegurar_directorio(path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)
    }
}