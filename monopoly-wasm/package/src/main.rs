use std::{error::Error, fs::File, io::Write, os::unix::ffi::OsStrExt, path::PathBuf};

use clap::Parser;
use memmap2::Mmap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Start file
    #[arg()]
    start: PathBuf,

    /// Output directory
    #[arg()]
    outdir: PathBuf,

    /// Single page
    #[arg(short, long)]
    single: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line args
    let args = Cli::parse();

    let cb = if args.single {
        single_process
    } else {
        multi_process
    };

    let config = Config {
        outroot: args.outdir.clone(),
        callback: cb,
    };

    process_input(&config, args.start)?;

    Ok(())
}

type ProcessCallback =
    fn(config: &Config, infile: PathBuf, mmap: &Mmap) -> Result<(), Box<dyn Error>>;

struct Config {
    outroot: PathBuf,
    callback: ProcessCallback,
}

fn process_input(config: &Config, infile: PathBuf) -> Result<(), Box<dyn Error>> {
    println!("Processing {}", infile.display());

    let mmap = {
        let file = File::open(&infile)?;

        unsafe { Mmap::map(&file)? }
    };

    (config.callback)(config, infile, &mmap)?;

    Ok(())
}

// Single file processing

struct SingleState<'a> {
    file: File,
    infile: &'a PathBuf,
    config: &'a Config,
}

fn single_process(config: &Config, infile: PathBuf, mmap: &Mmap) -> Result<(), Box<dyn Error>> {
    let outfile = config.outroot.join(infile.file_name().unwrap());

    println!("  {} -> {}", infile.display(), outfile.display());

    let mut state = SingleState {
        file: openout(&outfile)?,
        infile: &infile,
        config,
    };

    if !parse_file(mmap, &mut state, single_text, single_link)? {
        println!("  {} is binary", infile.display());
        state.file.write_all(mmap)?;
    }

    Ok(())
}

fn single_text(text: &str, state: &mut SingleState) -> Result<(), Box<dyn Error>> {
    state.file.write_all(text.as_bytes())?;

    Ok(())
}

fn single_link(link: &str, state: &mut SingleState) -> Result<(), Box<dyn Error>> {
    let linkfile = state.infile.parent().unwrap().join(link);

    // TODO
    println!("{:?}", linkfile);
    todo!();

    Ok(())
}

// Multi file processing

struct MultiState<'a> {
    file: File,
    infile: &'a PathBuf,
    config: &'a Config,
}

fn multi_process(config: &Config, infile: PathBuf, mmap: &Mmap) -> Result<(), Box<dyn Error>> {
    let outfile = config.outroot.join(infile.file_name().unwrap());

    println!("  {} -> {}", infile.display(), outfile.display());

    let mut state = MultiState {
        file: openout(&outfile)?,
        infile: &infile,
        config,
    };

    if !parse_file(mmap, &mut state, multi_text, multi_link)? {
        println!("  {} is binary", infile.display());
        state.file.write_all(mmap)?;
    }

    Ok(())
}

fn multi_text(text: &str, state: &mut MultiState) -> Result<(), Box<dyn Error>> {
    state.file.write_all(text.as_bytes())?;

    Ok(())
}

fn multi_link(link: &str, state: &mut MultiState) -> Result<(), Box<dyn Error>> {
    let linkfile = state.infile.parent().unwrap().join(link);

    state
        .file
        .write_all(linkfile.file_name().unwrap().as_bytes())?;

    process_input(state.config, linkfile)?;

    Ok(())
}

fn openout(outfile: &PathBuf) -> Result<File, Box<dyn Error>> {
    if let Some(dir) = outfile.parent() {
        std::fs::create_dir_all(dir)?;
    };

    let f = File::create(outfile)?;

    Ok(f)
}

type ParseCallback<S> = fn(text: &str, state: &mut S) -> Result<(), Box<dyn Error>>;

fn parse_file<S>(
    mmap: &Mmap,
    state: &mut S,
    textcb: ParseCallback<S>,
    linkcb: ParseCallback<S>,
) -> Result<bool, Box<dyn Error>> {
    // Try and convert to UTF-8
    let ok = if let Ok(mapstr) = std::str::from_utf8(mmap) {
        let mut pos = 0;

        loop {
            match &mapstr[pos..].find("$link(") {
                Some(idx) => {
                    let idx = idx + pos;
                    let end = mapstr[idx..].find(')').unwrap() + idx;

                    textcb(&mapstr[pos..idx], state)?;

                    let link = &mapstr[idx + 6..end];

                    linkcb(link, state)?;

                    pos = end + 1;
                }
                None => {
                    textcb(&mapstr[pos..], state)?;

                    break;
                }
            }
        }

        true
    } else {
        false
    };

    Ok(ok)
}
