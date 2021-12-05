use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Analog Clock",
    about = "\nSee https://github.com/wongjiahau/analog-clock"
)]
pub struct CliOptions {
    /// Theme of the clock.
    /// See https://github.com/wongjiahau/analog-clock/blob/master/src/theme.rs
    #[structopt(long, default_value = "nord-frost")]
    pub theme: String,

    /// How often should the clock be redrawn in millisecond.
    #[structopt(long, default_value = "1000")]
    pub tick: usize,

    /// Hide second hand.
    #[structopt(long)]
    pub hide_second_hand: bool,

    /// Hide hour labels.
    #[structopt(long)]
    pub hide_hour_labels: bool,

    /// Show minute labels.
    #[structopt(long)]
    pub show_minute_labels: bool,
}
