use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Subdivision {
    Quarter,
    Eighth,
    Triplet,
    Sixteenth,
}

impl Subdivision {
    pub fn ticks_per_beat(self) -> u8 {
        match self {
            Subdivision::Quarter => 1,
            Subdivision::Eighth => 2,
            Subdivision::Triplet => 3,
            Subdivision::Sixteenth => 4,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SoundType {
    Click,
    Wood,
    Cowbell,
    Sidestick,
    Beep,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Estimate BPM by tapping a key at a steady tempo")]
    Tap {
        #[arg(
            short = 'a',
            long = "amount",
            default_value_t = 8,
            value_parser = clap::value_parser!(u8).range(2..=64),
            help = "Number of taps to collect before accepting the BPM"
        )]
        amount: u8,
    },
    Ramp {
        pattern: String,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "metronome",
    version,
    about = "A precise CLI metronome",
    disable_help_subcommand = false
)]
pub struct Cli {
    #[arg(value_parser = clap::value_parser!(u16).range(20..=400))]
    pub bpm_positional: Option<u16>,
    #[arg(short = 'b', long = "bpm", default_value_t = 120, value_parser = clap::value_parser!(u16).range(20..=400))]
    pub bpm: u16,
    #[arg(
        short = 's',
        long = "signature",
        default_value = "4/4",
        global = true,
        help = "Time signature to use (for example 3/4, 6/8, or 7/8)"
    )]
    pub signature: String,
    #[arg(long = "subdivision", value_enum, default_value_t = Subdivision::Quarter)]
    pub subdivision: Subdivision,
    #[arg(long = "mute", action = ArgAction::SetTrue)]
    pub mute: bool,
    #[arg(long = "sound", value_enum, default_value_t = SoundType::Click)]
    pub sound: SoundType,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tap_amount_and_trailing_signature() {
        let cli = Cli::try_parse_from(["metronome", "tap", "--amount", "10", "--signature", "7/8"])
            .unwrap();

        assert_eq!(cli.signature, "7/8");
        assert!(matches!(cli.command, Some(Commands::Tap { amount: 10 })));
    }

    #[test]
    fn rejects_tap_amount_below_two() {
        assert!(Cli::try_parse_from(["metronome", "tap", "-a", "1"]).is_err());
    }
}
