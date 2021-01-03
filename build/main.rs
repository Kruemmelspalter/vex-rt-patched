use std::env;
use std::io;
use std::path::PathBuf;
use std::process;
use std::str;

use bindgen;
use zip_extensions::zip_extract;

// Path to PROS release zip (relative to project root)
const PROS_ZIP_STR: &str = "build/kernel@3.3.1.zip";

// Path to PROS wrapper.h (relative to project root)
const PROS_WRAPPER_STR: &str = "build/wrapper.h";

// Types to be included by bindgen
const WHITELISTED_TYPES: &[&str] = &["motor_.*", "task_.*", "mutex_.*"];

// Enums to be automatically "rustified" by bindgen
const RUSTIFIED_ENUMS: &[&str] = &["motor_.*", "task_.*"];

// Enums to be treated as bitfields/bitflags by bindgen
const BITFIELD_ENUMS: &[&str] = &["motor_flag_e"];

// Functions to be included by bindgen
const WHITELISTED_FUNCS: &[&str] = &["motor_.*", "task_.*", "mutex_.*", "millis"];

// Variables to be included by bindgen
const WHITELISTED_VARS: &[&str] = &["VEX_RT_.*", ".*_DEFAULT"];

fn main() -> Result<(), io::Error> {
    // tell cargo to rerun this script if it's dependent files change
    println!("cargo:rerun-if-changed=build/main.rs");
    println!("cargo:rerun-if-changed={}", PROS_ZIP_STR);
    println!("cargo:rerun-if-changed={}", PROS_WRAPPER_STR);

    // define input paths
    let pros_zip_path = PathBuf::from(PROS_ZIP_STR);
    let wrapper_h_path = PathBuf::from(PROS_WRAPPER_STR);

    // define output paths
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let pros_extract_path = out_dir_path.join("pros");
    let bindings_gen_path = out_dir_path.join("bindings.rs");

    // extract pros firmware
    zip_extract(&pros_zip_path, &pros_extract_path)?;

    // tell cargo where to find pros link scripts and libraries
    println!(
        "cargo:rustc-link-search={}",
        pros_extract_path.join("firmware").display()
    );

    let includes = get_includes(&pros_extract_path);
    generate_bindings(&includes, &wrapper_h_path, &bindings_gen_path)?;

    Ok(())
}

/// detects system include paths for `arm-none-eabi` and pros.
fn get_includes(pros_extract_path: &PathBuf) -> Vec<String> {
    // https://stackoverflow.com/questions/17939930/finding-out-what-the-gcc-include-path-is
    let output = process::Command::new("arm-none-eabi-gcc")
        .args(&["-E", "-Wp,-v", "-xc", "/dev/null"])
        .output()
        .expect("failed to execute arm-none-eabi-gcc. is the arm-none-eabi toolchain installed?");

    #[rustfmt::skip]
    // output we want is in stderr
    //
    // On my system it looks like this:
    //
    // #include <...> search starts here:
    // /usr/local/Cellar/arm-none-eabi-gcc/10.1.0/lib/gcc/arm-none-eabi/gcc/arm-none-eabi/10.1.0/include
    // /usr/local/Cellar/arm-none-eabi-gcc/10.1.0/lib/gcc/arm-none-eabi/gcc/arm-none-eabi/10.1.0/include-fixed
    // /usr/local/Cellar/arm-none-eabi-gcc/10.1.0/lib/gcc/arm-none-eabi/gcc/arm-none-eabi/10.1.0/../../../../../../arm-none-eabi/include
    // End of search list.

    let mut in_include_section = false;
    let mut include_paths: Vec<String> = vec![format!(
        "-I{}",
        pros_extract_path.join("include").to_str().unwrap()
    )];

    let stderr = str::from_utf8(&output.stderr).unwrap();

    for line in stderr.lines() {
        if line == "#include <...> search starts here:" {
            in_include_section = true;
        } else if line == "End of search list." {
            in_include_section = false;
        } else if in_include_section {
            include_paths.push(format!("-I{}", line.trim()));
        }
    }

    include_paths
}

/// Generates bindings using bindgen.
fn generate_bindings(
    includes: &Vec<String>,
    wrapper_h_path: &PathBuf,
    bindings_gen_path: &PathBuf,
) -> Result<(), io::Error> {
    let mut bindings = bindgen::Builder::default()
        .header(wrapper_h_path.to_str().unwrap())
        .clang_arg("-target")
        .clang_arg("arm-none-eabi")
        .clang_args(includes)
        .ctypes_prefix("libc")
        .use_core()
        .layout_tests(false);

    for t in WHITELISTED_TYPES {
        bindings = bindings.whitelist_type(t);
    }

    for t in RUSTIFIED_ENUMS {
        bindings = bindings.rustified_enum(t);
    }

    for t in BITFIELD_ENUMS {
        bindings = bindings.bitfield_enum(t);
    }

    for f in WHITELISTED_FUNCS {
        bindings = bindings.whitelist_function(f);
    }

    for v in WHITELISTED_VARS {
        bindings = bindings.whitelist_var(v);
    }

    bindings
        .generate()
        .expect("Could not generate bindings.")
        .write_to_file(&bindings_gen_path)?;

    Ok(())
}
