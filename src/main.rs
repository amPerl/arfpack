use clap::Parser;

mod actions;

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "amPerl")]
struct Opts {
    #[clap(subcommand)]
    file_type: actions::Command,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    opts.file_type.process()
}
