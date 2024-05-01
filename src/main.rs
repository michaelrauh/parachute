use clap::{command, Parser};
use parachute::{add, delete, get, process};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "S3 PATH")]
    location: String,

    #[arg(short, long)]
    endpoint: String,

    #[arg(short, long, value_name = "FILENAME")]
    add: Option<String>,

    #[arg(short, long)]
    get: bool,

    #[arg(short, long)]
    delete: bool,
}

fn main() {
    let args = Args::parse();

    if args.get {
        get(args.endpoint, args.location)
    } else if args.add.is_some() {
        add(
            args.add.unwrap(),
            args.endpoint.clone(),
            args.location.clone(),
        );
    } else if args.delete {
        delete(args.endpoint, args.location);
    }
        else {
        process(args.endpoint, args.location);
    }
}
