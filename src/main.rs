pub mod io_utils;
pub mod label;
pub mod pipeline;
pub mod types;

use anyhow::Result;
use clap::Parser;
use io_utils::{buf_read, buf_write, open_csv_reader, open_csv_writer};
use pipeline::{Config, run_pipeline};

/// CLI для стриминговой обработки LOB CSV -> фичи + лейбл
#[derive(Parser, Debug)]
#[command(version, about = "LOB streaming featurizer + labeler")]
struct Args {
    /// Путь к входному файлу (.csv или .csv.gz)
    #[arg(short = 'i', long)]
    input: String,

    /// Путь к выходному файлу (.csv или .csv.gz). Рекомендуется .gz
    #[arg(short = 'o', long)]
    output: String,

    /// Горизонт прогноза (сек)
    #[arg(long, default_value_t = 1.0)]
    horizon_sec: f64,

    /// eps_min (например, 3e-4 = 3 б.п.)
    #[arg(long, default_value_t = 3e-4)]
    eps_min: f64,

    /// alpha (множитель на rel_spread)
    #[arg(long, default_value_t = 0.5)]
    alpha: f64,

    /// cost_bp — «неизбежные» издержки в долях (например, 2e-4 = 2 б.п.)
    #[arg(long, default_value_t = 2e-4)]
    cost_bp: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rdr = open_csv_reader(&args.input)?;
    let rbuf = buf_read(rdr);

    let w = open_csv_writer(&args.output)?;
    let wbuf = buf_write(w);

    let cfg = Config {
        horizon_sec: args.horizon_sec,
        eps_min: args.eps_min,
        alpha: args.alpha,
        cost_bp: args.cost_bp,
    };

    run_pipeline(rbuf, wbuf, cfg)
}
