mod mapper;
mod rom_only;

use crate::cartridge::mapper::{Mapper, MapperTrait};
use crate::cartridge::rom_only::RomOnly;
use log::debug;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;

pub struct Cartridge {
    title: String,
    rom: Vec<u8>,
    mapper: Mapper,
}

impl Cartridge {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Cartridge, Error> {
        let mut file = File::open(&path)?;
        let ext = path.as_ref().extension().and_then(OsStr::to_str);

        let (rom, _) = match ext {
            Some("gb") => Self::read_file(&mut file)?,
            Some("zip") => Self::read_zip(file)?,
            _ => panic!("unsupported file type"),
        };

        let title = &rom[0x0134..=0x0143];
        let title = String::from_utf8_lossy(title).trim_end_matches('\0').to_string();

        let mapper = match rom[0x0147] {
            0x00 => Mapper::RomOnly(RomOnly),

            t => return Err(Error::other(format!("unsupported cartridge type ${:02x}", t))),
        };

        Ok(Cartridge { title, rom, mapper })
    }

    pub fn empty() -> Cartridge {
        Cartridge {
            title: "EMPTY".to_string(),
            rom: vec![0xFF; 0x4000],
            mapper: Mapper::RomOnly(RomOnly {}),
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
        self.mapper.read_rom(&self.rom, address)
    }

    pub(crate) fn write_byte(&mut self, address: u16, byte: u8) {
        self.mapper.write_rom(&mut self.rom, address, byte);
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
