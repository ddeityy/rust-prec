use keyvalues_parser::Vdf;
use log::info;
use std::io::Write;
use std::{
    fs::OpenOptions,
    io::{Read, Seek},
    path::PathBuf,
};
use steamid_ng;
use steamlocate::SteamDir;

pub fn insert_launch_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut steamdir = SteamDir::locate().unwrap();

    let user_id_64 = steamdir.app(&440).unwrap().last_user.unwrap();
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

    let mut text_buf = String::new();
    config_file.read_to_string(&mut text_buf)?;

    let vdf = Vdf::parse(&text_buf)?;

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
                if !s.contains("-condebug -conclearlog -usercon") {
                    let string = s.to_string();
                    let opt_string = format!("{} -condebug -conclearlog -usercon", string);
                    let mut lines = String::new();
                    config_file.read_to_string(&mut lines)?;
                    let updated_lines = lines.replace(&string, &opt_string);
                    config_file.seek(std::io::SeekFrom::Start(0))?;

                    writeln!(config_file, "{}", updated_lines)?;
                    info!("Launch options added")
                }
            }
            _ => (),
        }
    }
    Ok(())
}
