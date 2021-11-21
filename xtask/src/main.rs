use std::env;

use xshell::cmd;

type DynError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, DynError>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<()> {
    let task = env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("ci") => ci()?,
        Some("check_fmt") => check_fmt()?,
        Some("build") => build()?,
        Some("clippy") => clippy()?,
        Some("upload") => upload()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
ci              Runs CI locally.
check_fmt       Checks formatting.
build           Builds library and examples.
clippy          Lints library and examples.
upload          Uploads an example to the robot.
"
    )
}

fn ci() -> Result<()> {
    check_fmt()?;
    build()?;
    clippy()?;

    eprintln!("Done!");

    Ok(())
}

fn check_fmt() -> Result<()> {
    eprintln!("Checking formatting...");
    cmd!("cargo fmt -- --check").run()?;
    Ok(())
}

fn build() -> Result<()> {
    eprintln!("Building library...");
    cmd!("cargo build --target=armv7a-vex-eabi.json -Z build-std=core,alloc")
        .env("RUSTFLAGS", "-D warnings")
        .run()?;

    eprintln!("Building examples...");
    cmd!("cargo build --examples --target=armv7a-vex-eabi.json -Z build-std=core,alloc")
        .env("RUSTFLAGS", "-D warnings")
        .run()?;

    Ok(())
}

fn clippy() -> Result<()> {
    eprintln!("Linting library...");
    cmd!("cargo clippy --target=armv7a-vex-eabi.json -Z build-std=core,alloc -- -D warnings")
        .run()?;

    eprintln!("Linting examples...");
    cmd!("cargo clippy --examples --target=armv7a-vex-eabi.json -Z build-std=core,alloc -- -D warnings")
        .run()?;

    Ok(())
}

fn upload() -> Result<()> {
    let _example = env::args().nth(1).expect(
        "Usage:
cargo xtask upload <example>",
    );

    cmd!("echo TODO").run()?;

    Ok(())
}
