use clap::Parser;

use self::pack::PackOpts;
use self::unpack::UnpackOpts;

mod pack;
mod types;
mod unpack;

#[derive(Parser)]
#[clap(about)]
pub enum Command {
    /// Unpack the resources into individual files
    #[clap(about)]
    Unpack(UnpackOpts),
    /// Pack a directory of resources into an arf archive
    #[clap(about)]
    Pack(PackOpts),
}

impl Command {
    pub fn process(self) -> anyhow::Result<()> {
        match self {
            Command::Unpack(opts) => unpack::unpack_arf(opts),
            Command::Pack(opts) => pack::pack_arf(opts),
        }
    }
}
