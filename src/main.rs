use proyecto_gases_hi::modules::*;
use clap::{Parser, Subcommand};
use tracing_subscriber::FmtSubscriber;
use tracing::info;

#[derive(Parser)]
#[command(name = "gases_hi")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Preparacion,
    Procesamiento,
    Visualizacion,
    Analitica,
    Pipeline,
}

fn main() -> anyhow::Result<()> {
    FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Preparacion => {
            ejecutar_preparacion(&PreparacionConfig::default())?;
        }
        Commands::Procesamiento => {
            ejecutar_procesamiento(&ProcesamientoConfig::default())?;
        }
        Commands::Visualizacion => {
            ejecutar_visualizacion(&VisualizacionConfig::default())?;
        }
        Commands::Analitica => {
            ejecutar_analitica(&AnaliticaConfig::default())?;
        }
        Commands::Pipeline => {
            info!("🚀 Pipeline completo...");
            ejecutar_preparacion(&PreparacionConfig::default())?;
            ejecutar_procesamiento(&ProcesamientoConfig::default())?;
            ejecutar_visualizacion(&VisualizacionConfig::default())?;
            ejecutar_analitica(&AnaliticaConfig::default())?;
            info!("✅ Pipeline OK");
        }
    }
    Ok(())
}
