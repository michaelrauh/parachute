use clap::{command, Parser, ValueEnum};
use parachute::{add, process};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "S3 PATH")]
    location: String,

    #[arg(short, long)]
    endpoint: String,

    #[arg(short, long, value_name = "FILENAME")]
    add: Option<String>,

    #[arg(short, long, conflicts_with = "add")]
    mode: Option<AgentMode>, // todo remove agent mode. Do single if there are chunks and multiple if not
}

#[derive(Parser, Debug, Clone, ValueEnum)]
enum AgentMode {
    Single,
    Merge,
}

fn main() {
    let args = Args::parse();

    if args.add.is_some() {
        add(args.add.unwrap(), args.endpoint, args.location)
    } else {
        process(args.endpoint, args.location);
    }
}
