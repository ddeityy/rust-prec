use keyvalues_parser::Vdf;
use std::io::Write;
use std::{
    fs::OpenOptions,
    io::{Read, Seek},
    path::PathBuf,
};
use steamid_ng;
use steamlocate::SteamDir;

pub fn insert_launch_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut steamdir = SteamDir::locate().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to locate Steam directory",
        )
    })?;
    let app_info_option = steamdir.app(&440);
    let user_id_64: u64;
    match app_info_option {
        Some(app_info) => {
            user_id_64 = app_info.last_user.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "App not found or no last user",
                )
            })?;
        }
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Failed to find the specified app",
            )))
        }
    }
    let user_id_3 = steamid_ng::SteamID::from(user_id_64).account_id();

    let config_path: PathBuf = steamdir
        .path
        .join("userdata")
        .join(user_id_3.to_string())
        .join("config")
        .join("localconfig.vdf");

    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .open(config_path)?;

    let mut buf = String::new();
    let _ = config_file.read_to_string(&mut buf)?;
    let vdf = Vdf::parse(&buf)?;

    let binding = vdf.value.unwrap_obj();
    let launch_opts: &Vec<keyvalues_parser::Value> = binding.get("Software").unwrap()[0]
        .get_obj()
        .unwrap()
        .get("Valve")
        .unwrap()[0]
        .get_obj()
        .unwrap()
        .get("Steam")
        .unwrap()[0]
        .get_obj()
        .unwrap()
        .get("apps")
        .unwrap()[0]
        .get_obj()
        .unwrap()
        .get("440")
        .unwrap()[0]
        .get_obj()
        .unwrap()
        .get("LaunchOptions")
        .unwrap();

    for opt in launch_opts {
        match opt {
            keyvalues_parser::Value::Str(s) => {
                if s.contains("-condebug -conclearlog -usercon") {
                } else {
                    let string = s.to_string();
                    let opt_string = format!("{} -condebug -conclearlog -usercon", string);
                    let mut lines = String::new();
                    config_file.read_to_string(&mut lines)?;
                    let updated_lines = lines.replace(&string, &opt_string);
                    config_file.seek(std::io::SeekFrom::Start(0))?;

                    writeln!(config_file, "{}", updated_lines)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
