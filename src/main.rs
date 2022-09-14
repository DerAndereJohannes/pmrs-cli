use std::path::Path;
use clap::{Parser, Subcommand, Args};
use pmrs::objects::ocel::validator::{validate_ocel, validate_ocel_verbose};
use pmrs::objects::ocel::importer::import_ocel;
use pmrs::objects::ocdg::{generate_ocdg, Relations};
use pmrs::objects::ocdg::exporter::export_ocdg;
use strum::IntoEnumIterator;

use log::{debug, error, LevelFilter};
use env_logger::{Builder, Target};


#[derive(Parser, Debug)]
#[clap(name = "pmrs-cli", author, version, about, long_about = None)]
struct Cli {
    /// Generate debug text in stdout
    #[clap(short, long, global = true)]
    debug: bool,

    #[clap(subcommand)]
    commands: BaseCommands
}

#[derive(Subcommand, Debug)]
enum BaseCommands {
    Ocel(OcelBase),
    Ocdg(OcdgBase)
}

#[derive(Parser, Debug)]
struct OcelBase {
    #[clap(subcommand)]
    commands: OcelCommands
}

#[derive(Subcommand, Debug)]
enum OcelCommands {
    Validate(Validate)
}

#[derive(Parser, Debug)]
struct OcdgBase {
    #[clap(subcommand)]
    commands: OcdgCommands
}

#[derive(Subcommand, Debug)]
enum OcdgCommands {
    Generate(OcdgGeneration)
}

#[derive(Args, Debug)]
struct OcdgGeneration {
    /// Path to OCEL file
    path: String,

    /// Output file name and location. Default: output.gexf
    #[clap(short, long)]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct Validate {
    path: String,
    #[clap(short, long)]
    verbose: bool
}

fn main() {

    let cli = Cli::parse();

    if cli.debug {
        Builder::new().target(Target::Stdout).filter_level(LevelFilter::Debug).init();
    } else {
        env_logger::init();
    } 

    match &cli.commands {
        BaseCommands::Ocel(ocel_sub) => {
            match &ocel_sub.commands {
                OcelCommands::Validate(validate) => {
                    if validate.path.ends_with(".jsonocel") {
                        if validate.verbose {
                            match validate_ocel_verbose(&validate.path) {
                                Ok(v) => {
                                    for (i, error) in v.iter().enumerate() {
                                        println!("Error {}: {} at {}", i+1, error.0, error.1);
                                    }
                                    
                                    println!("{}: {}", validate.path, v.is_empty());
                                }
                                Err(e) => println!("There was an Error: {}", e),
                            }
                        } else {
                            match validate_ocel(&validate.path) {
                                Ok(v) => {
                                    println!("{}: {}", validate.path, v);
                                }
                                Err(e) => println!("There was an Error: {}", e),
                            }
                        }
                    } else {
                        error!("Error: {} file format is not supported.", validate.path);
                    }
                }
            }
        },
        BaseCommands::Ocdg(ocdg_sub) => {
            match &ocdg_sub.commands {
                OcdgCommands::Generate(generation) => {
                    let mut output_path = Path::new("output.gexf");

                    if let Some(custom_name) = &generation.output {
                        debug!("Setting custom output path to {:?}", custom_name);
                        output_path = Path::new(custom_name);
                    }

                    // import ocel
                    let relations: Vec<Relations> = Relations::iter().collect(); 
                    debug!("Importing log: {:?}", &generation.path);
                    match import_ocel(&generation.path) {
                        Ok(log) => {
                            debug!("Generating OCDG on relations: {:?}", relations);
                            let ocdg = generate_ocdg(&log, &relations);
                            debug!("Exporting the generated OCDG.");
                            match export_ocdg(&ocdg, &output_path.to_string_lossy()) {
                                Ok(_) => {debug!("Successfully exported the OCDG to: {:?}", output_path);},
                                Err(e) => {error!("Generating the OCDG had the following error: {:?}", e);}
                            }
                        }
                        Err(e) => {error!("Importing the log had the following error: {:?}", e);}
                    }
                }
            }
        }
    }
}
