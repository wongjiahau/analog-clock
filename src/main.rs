mod cli;
mod clock;
use clock::run_clock;
use colors_transform::Rgb;
use std::time::Duration;
use structopt::StructOpt;

use crate::{cli::CliOptions, clock::RunClockOptions};

fn main() {
    let opt = CliOptions::from_args();
    let hex = opt.color.as_str();
    let color = Rgb::from_hex_str(hex).expect("Invalid hex string.");
    match run_clock(RunClockOptions {
        color,
        tick_interval: Duration::from_millis(opt.tick as u64),
        show_second_hand: !opt.hide_second_hand,
        show_hour_labels: !opt.hide_hour_labels,
        show_minute_labels: opt.show_minute_labels,
    }) {
        Ok(_) => (),
        Err(error) => eprintln!("{}", error),
    }
}
