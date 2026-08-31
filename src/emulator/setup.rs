use crate::emulator::AudioStream;
use crate::nes::cartridge::cartridge_def::{CartMemory, RomConfig};
use crate::nes::cartridge::Mirroring;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::mpsc;

pub fn get_rom_from_file(path: &Path) -> Result<RomConfig, Box<dyn Error>> {
    const INES_HEADER_SIZE: usize = 16;
    const KB: usize = 1024;

    // TODO: Do proper path checks

    let ines_data = fs::read(path)?;

    if ines_data.len() < INES_HEADER_SIZE || !ines_data.starts_with(b"NES\x1A") {
        return Err(format!(
            "{} is not a vaild iNES rom file (header doesn't fit)",
            path.to_str().unwrap()
        )
        .into());
    }

    let prg_rom_end = INES_HEADER_SIZE + 16 * KB * ines_data[4] as usize;
    let chr_rom_end = prg_rom_end + 8 * KB * ines_data[5] as usize;

    if (ines_data.len()) < chr_rom_end {
        return Err(format!(
            "{} is not a vaild iNES rom file (file not long enough)",
            path.to_str().unwrap()
        )
        .into());
    }

    let chr_rom_is_ram = prg_rom_end == chr_rom_end;

    let has_prg_ram = (ines_data[6] & 0b10) > 0;

    let ines_mapper_id = (ines_data[7] & 0xF0) | (ines_data[6] >> 4);

    println!(
        "Mapper: {}\nPRG RAM: {}\nCHR RAM: {}",
        ines_mapper_id, has_prg_ram, chr_rom_is_ram
    );

    Ok(RomConfig {
        ines_mapper_id,
        ines_mirroring: match ines_data[6] & 1 {
            1 => Mirroring::Vertical,
            0 => Mirroring::Horizontal,
            _ => unreachable!(),
        },
        data: CartMemory::new(
            ines_data[INES_HEADER_SIZE..prg_rom_end].to_owned(),
            match chr_rom_is_ram {
                false => Some(ines_data[prg_rom_end..chr_rom_end].to_owned()),
                true => None,
            },
            has_prg_ram,
        ),
    })
}

pub fn create_audio_stream() -> Result<AudioStream, Box<dyn Error>> {
    let (tx, rx) = mpsc::sync_channel::<(f32, f32)>(4096);
    let device = cpal::default_host()
        .default_output_device()
        .ok_or(cpal::BuildStreamError::DeviceNotAvailable)?;
    let config = device.default_output_config()?.config();

    let output = Ok(AudioStream {
        sender: tx,
        sample_rate: config.sample_rate.0 as f32,
    });

    std::thread::spawn(move || {
        let mut prev_sample = (0.0, 0.0);
        let output_stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Uses const generics to magically infer that we want &[f32; 2], wow!
                    for [l_channel, r_channel] in data.array_chunks_mut() {
                        (*l_channel, *r_channel) = match rx.recv() {
                            Ok(sample) => {
                                prev_sample = sample;
                                sample
                            }
                            Err(_) => prev_sample,
                        };
                    }
                },
                |_err| panic!("Audio stream encountered an error: {_err}"),
                None,
            )
            .unwrap();
        output_stream.play().unwrap();
        std::thread::park();
    });
    output
}
