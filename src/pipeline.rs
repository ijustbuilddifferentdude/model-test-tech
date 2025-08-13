use crate::label::{Label, classify};
use crate::types::{Feat, Snapshot, compute_features};
use anyhow::{Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Config {
    pub horizon_sec: f64, // H
    pub eps_min: f64,     // 3e-4
    pub alpha: f64,       //  0.5
    pub cost_bp: f64,
}

struct Pending {
    feat: Feat,
    target_ts: f64, // ts_ms + H*1000
}

struct OutRow<'a> {
    f: &'a Feat,
    label: Label,
}

impl<'a> OutRow<'a> {
    fn write_csv<W: Write>(&self, wtr: &mut csv::Writer<W>) -> csv::Result<()> {
        wtr.write_record(&[
            self.f.ts_ms.to_string(),
            self.f.mid.to_string(),
            self.f.spread.to_string(),
            self.f.rel_spread.to_string(),
            self.f.microprice.to_string(),
            self.f.imb1.to_string(),
            self.f.imb3.to_string(),
            self.f.imb5.to_string(),
            self.f.dt_ms.to_string(),
            self.label.as_str().to_string(),
        ])
    }
}

pub fn run_pipeline<R: Read, W: Write>(reader: R, mut writer: W, cfg: Config) -> Result<()> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(false)
        .double_quote(true)
        .from_reader(reader);

    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(&mut writer);

    wtr.write_record(Feat::header_slice())?;

    let mut pending: VecDeque<Pending> = VecDeque::new();
    let mut prev_ts: Option<f64> = None;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} processed: {pos} rows")?.tick_chars("/|\\- "),
    );

    let mut cnt = 0u64;
    let mut down = 0u64;
    let mut flat = 0u64;
    let mut up = 0u64;

    for rec in rdr.deserialize::<Snapshot>() {
        let row: Snapshot = rec.context("deserialize Snapshot")?;
        let feat = compute_features(&row, prev_ts);
        prev_ts = Some(feat.ts_ms);

        let current_ts = feat.ts_ms;
        let mut to_flush = Vec::<(Feat, Label)>::new();

        while let Some(front) = pending.front() {
            if front.target_ts <= current_ts {
                let f0 = &front.feat;
                let lbl = classify(
                    f0.mid,
                    feat.mid,
                    f0.rel_spread,
                    cfg.eps_min,
                    cfg.alpha,
                    cfg.cost_bp,
                );
                to_flush.push((f0.clone(), lbl));
                pending.pop_front();
            } else {
                break;
            }
        }

        for (f, lbl) in to_flush.into_iter() {
            let out = OutRow { f: &f, label: lbl };
            out.write_csv(&mut wtr)?;
            cnt += 1;
            match lbl {
                Label::Down => down += 1,
                Label::Flat => flat += 1,
                Label::Up => up += 1,
            }
        }

        let target_ts = feat.ts_ms + cfg.horizon_sec * 1000.0;
        pending.push_back(Pending { feat, target_ts });

        pb.inc(1);
    }

    wtr.flush()?;
    pb.finish_and_clear();

    eprintln!("Done. Written rows: {cnt}. Label distribution: down={down}, flat={flat}, up={up}");
    Ok(())
}
