mod headers;
mod mapper;
mod mbc1;
mod rom_only;

use crate::cartridge::mapper::{Mapper, MapperTrait};
use crate::cartridge::mbc1::Mbc1;
use crate::cartridge::rom_only::RomOnly;
use headers::Headers;
use log::debug;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;

pub struct Cartridge {
    title: String,
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    mapper: Mapper,
}

pub const ROM_BANK_SIZE: usize = 0x4000;
pub const RAM_BANK_SIZE: usize = 0x2000;

impl Cartridge {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Cartridge, Error> {
        let mut file = File::open(&path)?;
        let ext = path.as_ref().extension().and_then(OsStr::to_str);

        let (rom, _) = match ext {
            Some("gb") => Self::read_file(&mut file)?,
            Some("zip") => Self::read_zip(file)?,
            _ => panic!("unsupported file type"),
        };

        let title = &rom[Headers::ROM_TITLE];
        let title = String::from_utf8_lossy(title).trim_end_matches('\0').to_string();
        let (ram_banks, ram_size): (usize, usize) = match rom[Headers::RAM_SIZE] {
            0x00 => (0, 0),           //    No RAM
            0x02 => (1, 8 * 1024),    //  1 x 8KiB = 8KiB
            0x03 => (4, 32 * 1024),   //  4 x 8KiB = 32KiB
            0x05 => (8, 64 * 1024),   //  8 x 8Kib = 64KiB
            0x04 => (16, 128 * 1024), // 16 x 8KiB = 128KiB
            t => return Err(Error::other(format!("unsupported ram size ${:02x}", t))),
        };
        let (rom_banks, rom_size): (usize, usize) = match rom[Headers::ROM_SIZE] {
            0x00 => (2, 32 * 1024),         //  32 KiB = 2 banks (no banking)
            0x01 => (4, 64 * 1024),         //  64 KiB = 4 banks
            0x02 => (8, 128 * 1024),        // 128 KiB = 8 banks
            0x03 => (16, 256 * 1024),       // 256 KiB = 16 banks
            0x04 => (32, 512 * 1024),       // 512 KiB = 32 banks
            0x05 => (64, 1024 * 1024),      //   1 MiB = 64 banks
            0x06 => (128, 2 * 1024 * 1024), //   2 MiB = 128 banks
            0x07 => (256, 4 * 1024 * 1024), //   4 MiB = 256 banks
            0x08 => (512, 8 * 1024 * 1024), //   8 MiB = 512 banks
            t => return Err(Error::other(format!("unsupported rom size ${:02x}", t))),
        };

        let mapper = match rom[Headers::TYPE] {
            0x00 => Mapper::RomOnly(RomOnly),
            0x01..=0x03 => Mapper::Mbc1(Mbc1::new(rom_banks, ram_banks)), // MBC1
            t => return Err(Error::other(format!("unsupported cartridge type ${:02x}", t))),
        };

        let ram = if ram_size > 0 { Some(vec![0u8; ram_size]) } else { None };
        let rom_raw = rom;
        let mut rom = vec![0u8; rom_size];
        let copy_len = rom_raw.len().min(rom.len());
        rom[..copy_len].copy_from_slice(&rom_raw[..copy_len]);

        Ok(Cartridge {
            title,
            rom,
            ram,
            mapper,
        })
    }

    pub fn empty() -> Cartridge {
        Cartridge {
            title: "EMPTY".to_string(),
            rom: vec![0xFF; 0x4000],
            mapper: Mapper::RomOnly(RomOnly {}),
            ram: None,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    fn read_file(file: &mut File) -> Result<(Vec<u8>, usize), Error> {
        let mut rom = vec![];
        let rom_size = file.read_to_end(&mut rom)?;

        Ok((rom, rom_size))
    }

    fn read_zip(file: File) -> Result<(Vec<u8>, usize), Error> {
        debug!("Unzipping rom...");
        let mut archive = zip::ZipArchive::new(file)?;

        let filename = archive
            .file_names()
            .find(|name| name.to_lowercase().ends_with(".gb"))
            .expect("any roms found in the archive!")
            .to_string();
        debug!(" > file extract: {}", filename);

        let mut file = archive.by_name(&filename)?;
        let mut rom = vec![];
        let rom_size = file.read_to_end(&mut rom)?;
        Ok((rom, rom_size))
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mapper.read(&self.rom, self.ram.as_deref(), address)
    }

    pub(crate) fn write_byte(&mut self, address: u16, byte: u8) {
        self.mapper.write(&self.rom, self.ram.as_deref_mut(), address, byte);
    }
}

#[cfg(test)]
#[cfg(feature = "use-test-roms")]
mod tests {
    use super::*;

    #[test]
    fn test_read_gb() -> Result<(), Error> {
        let cartridge = Cartridge::load_from_path("../doctor/roms/demos/cncd-at.zip")?;
        assert_eq!(cartridge.title(), "CNCD ALT'02    �");
        Ok(())
    }

    #[test]
    fn test_read_zip() -> Result<(), Error> {
        let cartridge = Cartridge::load_from_path("../doctor/roms/demos/alttoo.gb")?;
        assert_eq!(cartridge.title(), "CNCD ALT'02    �");
        Ok(())
    }
}
