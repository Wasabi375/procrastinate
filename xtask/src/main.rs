mod args;

use clap::{Command, CommandFactory, Parser, ValueEnum};
use core::convert::From;
use std::{
    io::Result,
    path::{absolute, Path, PathBuf},
};

use args::{Arguments, ShellType};

fn main() -> Result<()> {
    let args = Arguments::parse();

    let manifest_path = std::env::var_os("CARGO_MANIFEST_DIR").map(|p| PathBuf::from(p));
    let default_out_path = manifest_path.map(|p| p.parent().unwrap().join("target/assets"));

    let out_dir: PathBuf = args
        .out_dir
        .or(default_out_path)
        .ok_or(std::io::ErrorKind::NotFound)?;
    let out_dir = absolute(out_dir)?;

    match args.cmd {
        args::Cmd::Man => generate_man(&out_dir),
        args::Cmd::Completion { shell } => generate_completions(&out_dir, shell),
        args::Cmd::Clear => std::fs::remove_dir_all(out_dir),
    }
}

fn proc_cmd() -> Command {
    procrastiante_bin::args::Arguments::command()
}

fn daemon_cmd() -> Command {
    procrastinate_daemon::args::Args::command()
}

fn work_cmd() -> Command {
    procrastinate_work::args::Args::command()
}

fn commands() -> [(String, Command); 3] {
    [
        ("procrastinate".to_owned(), proc_cmd()),
        ("procrastinate-daemon".to_owned(), daemon_cmd()),
        ("procrastinate-work".to_owned(), work_cmd()),
    ]
}

fn generate_man(out_dir: &Path) -> Result<()> {
    let out_dir = out_dir.join("man");
    std::fs::create_dir_all(&out_dir)?;

    for (_name, cmd) in commands() {
        clap_mangen::generate_to(cmd, &out_dir)?;
    }

    Ok(())
}

fn generate_completions(out_dir: &Path, shell: Option<ShellType>) -> Result<()> {
    let out_dir = out_dir.join("completions");
    std::fs::create_dir_all(&out_dir)?;

    let shells = if let Some(shell) = shell {
        &[shell]
    } else {
        ShellType::value_variants()
    };

    for shell in shells {
        generate_completion(&out_dir, *shell)?;
    }

    Ok(())
}

fn generate_completion(out_dir: &Path, shell: ShellType) -> Result<()> {
    for (name, mut cmd) in commands() {
        clap_complete::generate_to(shell, &mut cmd, name, out_dir)?;
    }

    Ok(())
}
