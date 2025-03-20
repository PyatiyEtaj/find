use find::envs::Envs;
use find::find_mode::FindMode;
use std::io;

fn main() -> io::Result<()> {
    let words: Vec<String> = std::env::args().collect();

    let program_envs = Envs::new(&words);

    if program_envs.interactive {
        FindMode::interactive(program_envs)?;
    } else {
        FindMode::straight(program_envs)?;
    }

    Ok(())
}
