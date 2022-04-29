mod scanning;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    /// load a measure from a json file
    load: Option<String>,

    #[clap(short, long)]
    /// save a measure to a json file
    save: Option<String>,

    #[clap(short, long)]
    /// local where the measure was taken
    place: Option<String>,

    #[clap(short, long)]
    daemon: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.daemon {
        unimplemented!("Daemon not yet implemented. Give the dev a coffee.")
    }

    if let Some(place) = args.place {
        let measure = scanning::measure::Measure::new(place)?;

        if let Some(save) = args.save {
            measure.to_json(save)?;
        } else {
            println!("{:#?}", measure);
        }
    } else if let Some(load) = args.load {
        let measure = scanning::measure::Measure::from_json(load)?;

        println!("{:#?}", measure);
    } else {
        println!("Run ./ap_scanner --help for more information.");
    }

    Ok(())
}
