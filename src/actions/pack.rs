use std::{fs::File, io::BufWriter};

use anyhow::Context;
use binrw::BinWriterExt;
use clap::Parser;
use walkdir::WalkDir;

use super::types::{ArfFileDirectoryEntry, ArfHeader};

#[derive(Parser, Debug)]
pub struct PackOpts {
    /// Input directory
    #[clap(short = 'i', long)]
    input_dir: String,
    /// Path to ARF
    #[clap(short = 'o', long)]
    output_file: String,
}

pub fn pack_arf(opts: PackOpts) -> anyhow::Result<()> {
    let mut out_file =
        BufWriter::new(File::create(opts.output_file).context("Failed to create output file")?);

    let mut entries = Vec::new();

    for entry in WalkDir::new(&opts.input_dir) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }

        let os_path = entry.into_path();
        let entry_path = os_path.strip_prefix(&opts.input_dir)?;
        let entry_path = entry_path.to_string_lossy().to_string();
        let file_metadata = File::open(&os_path)?.metadata()?;
        entries.push((
            os_path,
            entry_path,
            ArfFileDirectoryEntry {
                id: entries.len() as _,
                unknown: 0,
                offset: 0,
                length: file_metadata.len() as _,
                file_xor_key: 0x69,
                padding: [0, 0, 0],
            },
        ));
    }

    let directory_offset = 16
        + entries
            .iter()
            .map(|(_os_path, entry_path, _entry)| entry_path.len() as u32 + 1)
            .sum::<u32>();

    let mut data_offset = directory_offset + 16 * entries.len() as u32;

    for (_, _, entry) in entries.iter_mut() {
        entry.offset = data_offset;
        data_offset += entry.length;
    }

    let xor_key = [0x69, 0x13, 0x37, 0x42];
    // let xor_key = [0x71, 0x7E, 0xB0, 0x0C];
    // let xor_key = [0, 0, 0, 0];

    // write header
    out_file.write_le(&ArfHeader {
        file_count: entries.len() as u32,
        xor_key,
        directory_offset,
    })?;

    // write filename directory
    for (_, path, _) in entries.iter() {
        let path_len = path.len();
        let mut path_buf = path.as_bytes().to_owned();
        for (i, val) in path_buf.iter_mut().take((path_len / 4) * 4).enumerate() {
            *val ^= xor_key[i % 4];
        }
        out_file.write_le(&(path_len as u8))?;
        out_file.write_le(&path_buf)?;
    }

    // write directory
    for (_, _, entry) in entries.iter() {
        out_file.write_le(entry)?;
    }

    // write file data
    for (path, _, entry) in entries.into_iter() {
        let mut in_file_buf = std::fs::read(path).context("Failed to read from input file")?;
        for val in in_file_buf.iter_mut().take(20) {
            *val ^= entry.file_xor_key;
        }
        out_file.write_le(&in_file_buf)?;
    }

    Ok(())
}
