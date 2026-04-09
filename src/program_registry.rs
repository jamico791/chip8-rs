use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Embed)]
#[folder = "database/"]
struct DatabaseFolder;

#[derive(Embed)]
#[folder = "roms/"]
struct RomsFolder;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    id: String,
    name: String,
    description: String,
    release: Option<String>,
    authors: Option<Vec<String>>,
    urls: Option<Vec<String>>,
    copyright: Option<String>,
    license: Option<String>,
    display_resolutions: Vec<String>,
    default_tickrate: u32,
    quirks: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Rom {
    pub file: String,
    pub embedded_title: Option<String>,
    pub description: Option<String>,
    pub release: Option<String>,
    pub platforms: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    title: String,
    description: Option<String>,
    release: Option<String>,
    authors: Option<Vec<String>>,
    roms: HashMap<String, Rom>,
}

pub struct ProgramRegistry {
    pub platforms: HashMap<String, Platform>,
    pub programs: Vec<Program>,
    pub hashes: HashMap<String, usize>,
}

impl ProgramRegistry {
    pub fn new() -> Self {
        let programs_file = DatabaseFolder::get("programs.json").unwrap();
        let programs_list: Vec<Program> =
            serde_json::from_str(std::str::from_utf8(programs_file.data.as_ref()).unwrap())
                .unwrap();

        let hashes_file = DatabaseFolder::get("sha1-hashes.json").unwrap();
        let hashes_map =
            serde_json::from_str(std::str::from_utf8(hashes_file.data.as_ref()).unwrap()).unwrap();

        Self {
            platforms: Self::build_platform_map("platforms.json"),
            programs: programs_list,
            hashes: hashes_map,
        }
    }

    fn build_platform_map(file_path: &str) -> HashMap<String, Platform> {
        let platforms_file = DatabaseFolder::get(file_path).unwrap();
        let platform_list: Vec<Platform> =
            serde_json::from_str(std::str::from_utf8(platforms_file.data.as_ref()).unwrap())
                .unwrap();

        let mut platform_map = HashMap::new();
        for platform in platform_list {
            platform_map.insert(platform.id.clone(), platform);
        }

        platform_map
    }

    fn get_program_from_hash(&self, hash: &str) -> Option<&Program> {
        let idx = self.hashes.get(hash).unwrap();
        match self.programs.get(*idx) {
            Some(program) => Some(program),
            None => None,
        }
    }

    pub fn get_rom_from_hash(&self, hash: &str) -> Option<&Rom> {
        let program = self.get_program_from_hash(hash);
        match program {
            Some(program) => {
                let mut selected_rom: Option<&Rom> = None;
                for (key, rom) in &program.roms {
                    if key == hash {
                        selected_rom = Some(rom);
                    }
                }
                selected_rom
            }
            None => None,
        }
    }
}

// let platforms_file = DatabaseFolder::get("platforms.json").unwrap();
// println!("{:?}", std::str::from_utf8(platforms_file.data.as_ref()));
//
// for file in DatabaseFolder::iter() {
//     println!("{}", file.as_ref());
// }
//
// for file in RomsFolder::iter() {
//     let rom = RomsFolder::get(&file).unwrap();
//     println!("{}", file.as_ref());
//     for byte in rom.data.as_ref() {
//         print!("0x{byte:02X} ");
//     }
// }
