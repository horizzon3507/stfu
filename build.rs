use std::env;
use std::fs;
use std::path::Path;

#[path = "src/cli.rs"]
mod cli;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

fn main() {
    let out_dir = match env::var_os("STFU_OUT_DIR") {
        Some(out) => std::path::PathBuf::from(out),
        None => {
            let out = env::var_os("OUT_DIR").unwrap();
            Path::new(&out).join("assets")
        }
    };

    let comp_dir = out_dir.join("completions");
    let man_dir = out_dir.join("man");

    fs::create_dir_all(&comp_dir).unwrap();
    fs::create_dir_all(&man_dir).unwrap();

    let mut cmd = cli::Cli::command();

    // Completions
    let _ = generate_to(Shell::Bash, &mut cmd, "stfu", &comp_dir);
    let _ = generate_to(Shell::Zsh, &mut cmd, "stfu", &comp_dir);
    let _ = generate_to(Shell::Fish, &mut cmd, "stfu", &comp_dir);

    // Manpage
    let man = clap_mangen::Man::new(cmd);
    let mut man_file = fs::File::create(man_dir.join("stfu.1")).unwrap();
    man.render(&mut man_file).unwrap();
}
