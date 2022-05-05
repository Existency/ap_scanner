mod scanning;
use clap::Parser;
use scanning::reading::Reading;

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
        let reading = Reading::new(place)?;

        if let Some(save) = args.save {
            reading.serialize(save)?;
        } else {
            println!("{:#?}", reading);
        }
    } else if let Some(load) = args.load {
        let measure = Reading::deserialize(load)?;

        println!("{:#?}", measure);
    } else {
        println!("Run ./ap_scanner --help for more information.");
    }

    Ok(())
}
