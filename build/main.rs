use std::{
    env, io,
    path::{Path, PathBuf},
    process, str,
};

use zip_extensions::zip_extract;

// Path to PROS release zip (relative to project root)
const PROS_ZIP_STR: &str = "build/kernel@3.4.0.zip";

// Path to PROS wrapper.h (relative to project root)
const PROS_WRAPPER_STR: &str = "build/wrapper.h";

// Types to be included by bindgen
const WHITELISTED_TYPES: &[&str] = &[];

// Enums to be treated as bitfields/bitflags by bindgen
const BITFIELD_ENUMS: &[&str] = &[];

// Functions to be included by bindgen
const WHITELISTED_FUNCS: &[&str] = &[
    "controller_get_analog",
    "controller_get_digital",
    "ext_adi_encoder_get",
    "ext_adi_encoder_init",
    "ext_adi_encoder_reset",
    "ext_adi_encoder_shutdown",
    "micros",
    "motor_get_actual_velocity",
    "motor_get_brake_mode",
    "motor_get_current_draw",
    "motor_get_current_limit",
    "motor_get_direction",
    "motor_get_efficiency",
    "motor_get_encoder_units",
    "motor_get_gearing",
    "motor_get_position",
    "motor_get_power",
    "motor_get_target_position",
    "motor_get_target_velocity",
    "motor_get_temperature",
    "motor_get_torque",
    "motor_get_velocity",
    "motor_get_voltage",
    "motor_get_voltage_limit",
    "motor_is_over_current",
    "motor_is_over_temp",
    "motor_is_reversed",
    "motor_modify_profiled_velocity",
    "motor_move",
    "motor_move_absolute",
    "motor_move_relative",
    "motor_move_velocity",
    "motor_move_voltage",
    "motor_set_brake_mode",
    "motor_set_current_limit",
    "motor_set_encoder_units",
    "motor_set_gearing",
    "motor_set_reversed",
    "motor_set_voltage_limit",
    "motor_set_zero_position",
    "motor_tare_position",
    "mutex_delete",
    "mutex_recursive_create",
    "mutex_recursive_give",
    "mutex_recursive_take",
    "sem_create",
    "sem_delete",
    "sem_get_count",
    "sem_post",
    "sem_wait",
    "serial_enable",
    "serial_flush",
    "serial_get_read_avail",
    "serial_get_write_free",
    "serial_peek_byte",
    "serial_read",
    "serial_read_byte",
    "serial_set_baudrate",
    "serial_write",
    "serial_write_byte",
    "task_create",
    "task_delay",
    "task_delete",
    "task_get_by_name",
    "task_get_current",
    "task_get_name",
    "task_get_priority",
    "task_get_state",
    "task_notify_ext",
    "task_notify_take",
];

// Variables to be included by bindgen
const WHITELISTED_VARS: &[&str] = &[
    "INTERNAL_ADI_PORT",
    "PROS_ERR_",
    "PROS_ERR_F_",
    "TASK_PRIORITY_DEFAULT",
    "TASK_STACK_DEPTH_DEFAULT",
];

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
fn get_includes(pros_extract_path: &Path) -> Vec<String> {
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
    includes: &[String],
    wrapper_h_path: &Path,
    bindings_gen_path: &Path,
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
