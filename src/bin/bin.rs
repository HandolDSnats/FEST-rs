use std::{env, fs::File, io::Read, path::Path};

use fest::FESData;

fn process_file(file_name: &str) -> Result<(), String> {
    let mut file_name = file_name.to_string();
    let mut file = File::open(&file_name).map_err(|e| e.to_string())?;
    let mut raw = vec![];

    file.read_to_end(&mut raw).map_err(|e| e.to_string())?;

    let mut file_data = FESData::process_data(&raw).map_err(|e| e.to_string())?;

    if file_data.is_compressed {
        file_name = format!("{}_dec", file_name);

        file_data = file_data.decompress().map_err(|e| e.to_string())?;
    } else {
        file_name = if file_name.ends_with("_dec") {
            file_name.trim_end_matches("_dec").to_string()
        } else {
            format!("{}_com", file_name)
        };

        file_data = file_data.compress().map_err(|e| e.to_string())?;
    }

    file_data.write_to(&file_name).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    let args = env::args()
        .filter(|e| Path::new(e).exists())
        .collect::<Vec<String>>();

    for arg in args {
        if let Err(error) = process_file(&arg) {
            eprintln!("Error ocurred when reading '{0}': '{1}'", arg, error)
        } else {
            println!("Finished processing of '{0}'", arg)
        }
    }
}
