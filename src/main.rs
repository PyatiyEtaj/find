mod envs;
mod find_mode;
mod temp_file;

use std::io;

use envs::Envs;
use find_mode::FindMode;

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().map(|e| e).collect();

    //let words: Vec<String> = vec!["1".to_string(), "--interactive".to_string()];

    let program_envs = match Envs::new(&words) {
        Ok(res) => res,
        Err(err) => {
            println!("[ERR] {}", err);
            return Ok(());
        }
    };

    if program_envs.interactive {
        FindMode::interactive(program_envs)?;
    } else {
        FindMode::straight(program_envs)?;
    }

    Ok(())
}
