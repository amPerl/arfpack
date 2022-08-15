use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

use anyhow::Context;
use binrw::BinReaderExt;
use clap::Parser;

use super::types::{ArfFileDirectoryEntry, ArfHeader};

#[derive(Parser, Debug)]
pub struct UnpackOpts {
    /// Path to ARF
    #[clap(short = 'i', long)]
    input_path: String,
    /// Output directory
    #[clap(short = 'o', long)]
    output_dir: String,
}

pub fn unpack_arf(opts: UnpackOpts) -> anyhow::Result<()> {
    let mut file = BufReader::new(File::open(&opts.input_path)?);

    let header: ArfHeader = file.read_le()?;

    let mut filenames = Vec::new();

    // Filename directory
    for _ in 0..header.file_count {
        let filename_length: u8 = file.read_le()?;
        let mut filename_buf = vec![0u8; filename_length as _];
        file.read_exact(&mut filename_buf)?;
        for (i, val) in filename_buf
            .iter_mut()
            .take((filename_length as usize / 4) * 4)
            .enumerate()
        {
            *val ^= header.xor_key[i % 4];
        }
        let filename_str = String::from_utf8_lossy(&filename_buf);
        filenames.push(filename_str.to_string());
    }

    let file_directory_entries: Vec<ArfFileDirectoryEntry> = file.read_le_args(binrw::VecArgs {
        count: header.file_count as _,
        inner: (),
    })?;

    for (entry, filename) in file_directory_entries
        .into_iter()
        .zip(filenames.into_iter())
    {
        let mut file_buf = vec![0u8; entry.length as _];
        file.read_exact(&mut file_buf)?;

        for val in file_buf.iter_mut().take(20) {
            *val ^= entry.file_xor_key;
        }

        let output_path = Path::new(&opts.output_dir).join(&filename);
        std::fs::create_dir_all(output_path.parent().unwrap())
            .context("failed to create output directory")?;

        let mut output_file = File::create(output_path).context("failed to create output file")?;

        output_file.write_all(&file_buf)?;
    }

    Ok(())
}
