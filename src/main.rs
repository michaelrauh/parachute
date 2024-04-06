use clap::{command, Parser, ValueEnum};
use parachute::add;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long,)]
    local: bool,

    #[arg(short, long, value_name = "FILENAME")]
    add: Option<String>,

    #[arg(short, long, conflicts_with = "add")]
    mode: Option<AgentMode>,
}

#[derive(Parser, Debug, Clone, ValueEnum)]
enum AgentMode {
    Single, Merge
}

fn main() {
    let args = Args::parse();

    dbg!(args.local);
    // add(args.add, args.local)
}