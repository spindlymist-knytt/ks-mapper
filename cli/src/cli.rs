use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

use ksmap::partition::{
    PartitionStrategy,
    GridStrategy,
    IslandsStrategy,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The partitioning strategy to use
    #[command(subcommand)]
    pub strategy: Strategy,
    /// The maximum width (in pixels) of an output image
    #[arg(short = 'x', long, default_value = "48000")]
    pub max_width: u64,
    /// The maximum height (in pixels) of an output image
    #[arg(short = 'y', long, default_value = "48000")]
    pub max_height: u64,
    /// Draw objects that are only visible in the editor
    #[arg(short, long)]
    pub editor_only: bool,
    /// Path to the KS data directory. If unspecified, it will be located relative to the level directory
    #[arg(long = "data")]
    pub data_dir: Option<PathBuf>,
    /// Path to the directory containing object templates
    #[arg(long = "templates", default_value = "Mapper Templates")]
    pub templates_dir: PathBuf,
    /// Path to the directory to save images to. If unspecified, it will be `Level Author - Level Name`
    #[arg(short, long = "output")]
    pub output_dir: Option<PathBuf>,
    /// Path to the level's directory or Map.bin
    pub level: PathBuf,
}

#[derive(Subcommand)]
pub enum Strategy {
   Grid(GridArgs),
   Islands(IslandsArgs),
}

type DynamicStrategy = Box<dyn PartitionStrategy>;

impl Strategy {
    pub fn into_strategy(self, max_size: (u64, u64)) -> DynamicStrategy {
        match self {
            Strategy::Grid(args) => args.into_strategy(max_size),
            Strategy::Islands(args) => args.into_strategy(max_size),
        }
    }
}

#[derive(Args)]
pub struct GridArgs {
    /// The number of rows to divide the level into. Leave blank to calculate from max height
    #[arg(short, long)]
    rows: Option<u64>,
    /// The number of columns to divide the level into. Leave blank to calculate from max width
    #[arg(short, long)]
    cols: Option<u64>,
}

impl GridArgs {
    fn into_strategy(self, max_size: (u64, u64)) -> DynamicStrategy {
        let strategy = GridStrategy {
            max_size,
            rows: self.rows,
            cols: self.cols,
        };
        Box::new(strategy)
    }
}

#[derive(Args)]
pub struct IslandsArgs {
    /// How many screens apart two islands can be before they are split into separate images
    #[arg(short = 'g', long, default_value = "20")]
    max_gap: u64,
    #[arg(short = 'g', long, default_value = "1")]
    min_gap: u64,
}

impl IslandsArgs {
    fn into_strategy(self, max_size: (u64, u64)) -> DynamicStrategy {
        let strategy = IslandsStrategy {
            max_size,
            min_gap: self.min_gap,
            max_gap: self.max_gap,
        };
        Box::new(strategy)
    }
}
