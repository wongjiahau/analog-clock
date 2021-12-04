use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Analog Clock",
    about = "\nSee https://github.com/wongjiahau/analog-clock"
)]
pub struct CliOptions {
    /// Color of the clock in hex.
    /// Default is "Green Gecko", which is suitable for both black and white screen.
    /// See https://eyeondesign.aiga.org/its-not-just-you-the-neon-glow-of-terminal-green-really-is-ubiquitous/
    #[structopt(short, long, default_value = "#39ff14")]
    pub color: String,

    /// How often should the clock be redrawn in millisecond.
    #[structopt(short, long, default_value = "1000")]
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
