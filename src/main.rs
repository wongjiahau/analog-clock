mod cli;
mod clock;
mod theme;
use clock::run_clock;
use std::{process::exit, time::Duration};
use structopt::StructOpt;
use theme::THEMES;

use crate::{cli::CliOptions, clock::RunClockOptions};

fn main() {
    let opt = CliOptions::from_args();
    let theme_index = IntoIterator::into_iter(THEMES)
        .enumerate()
        .find_map(|(index, theme)| {
            if theme.name == opt.theme {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            eprintln!("\n
  No theme has the name of '{}'.

  Feel free to contribute more theme at https://github.com/wongjiahau/analog-clock/blob/master/src/theme.rs
", opt.theme);
            exit(1)
        });
    match run_clock(RunClockOptions {
        theme_index,
        tick_interval: Duration::from_millis(opt.tick as u64),
        show_second_hand: !opt.hide_second_hand,
        show_hour_labels: !opt.hide_hour_labels,
        show_minute_labels: opt.show_minute_labels,
    }) {
        Ok(_) => (),
        Err(error) => eprintln!("{}", error),
    }
}
