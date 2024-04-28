use clap::{command, Parser};
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
}

fn main() {
    let args = Args::parse();

    if args.add.is_some() {
        add(
            args.add.unwrap(),
            args.endpoint.clone(),
            args.location.clone(),
        );
    } else {
        process(args.endpoint, args.location);
    }
}
