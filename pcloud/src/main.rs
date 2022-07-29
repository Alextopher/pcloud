mod redirect;

use clap::{Parser, Subcommand};

/// Interface with configured pcloud server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Redirect {
        #[clap(value_parser)]
        url: url::Url,

        /// How many times the redirect can be used
        #[clap(short, long, value_parser)]
        usages: Option<u32>,

        #[clap(
            short,
            long,
            value_parser,
            help = "How long the redirect should be valid for\nSuffixes: s, m, h, d, w, y",
            default_value = "1w"
        )]
        duration: String,
    },
}

fn main() {
    let args = Args::parse();

    let target = "https://localhost:8080";

    match args.subcommand {
        Some(Commands::Redirect {
            url,
            usages,
            duration,
        }) => {
            redirect::create_redirect(target.to_string(), url, usages, duration);
        }
        None => {
            println!("No subcommand given");
        }
    }
}
