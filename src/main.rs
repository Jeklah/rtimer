use libloading::{Library, Symbol};
use std::env;
use std::ffi::CString;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

type ExampleFunction = unsafe extern "C" fn();

fn compile_cpp_to_shared_lib(cpp_file: &str) -> Result<String, String> {
    let output_lib = if cfg!(target_os = "windows") {
        "exmaple.dll"
    } else if cfg!(target_os = "macos") {
        "libexample.dylib"
    } else {
        "libexample.so"
    };

    let status = Command::new("g++")
        .args(&[
            "-fPIC",
            "-shared",
            "-o",
            output_lib,
            cpp_file,
            "-I",
            "/usr/include/qt5",
            "-I",
            "/usr/invlude/qt5/QtCore",
            "-lQt5Core",
        ])
        .status()
        .map_err(|e| format!("failed to compile cpp file: {}", e))?;

    if !status.success() {
        return Err("failed to compile cpp file".into());
    }

    Ok(output_lib.to_string())
}

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <cpp_file> <function_name>", args[0]);
        return;
    }

    let cpp_file = &args[1];
    let function_name = &args[2];
    let c_func_name = match CString::new(function_name.as_str()) {
        Ok(c_string) => c_string,
        Err(e) => {
            eprintln!("failed to convert function name to CString: {}", e);
            return;
        }
    };

    // Compile the C++ file to a shared library
    let lib_path = match compile_cpp_to_shared_lib(cpp_file) {
        Ok(lib_path) => lib_path,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // Load the shared library
    let lib = match Library::new(lib_path) {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("failed to load shared library: {}", e);
            return;
        }
    };

    // Find the function in the shared library
    unsafe {
        let func: Symbol<ExampleFunction> = match lib.get(c_func_name.as_bytes_with_nul()) {
            Ok(symbol) => symbol,
            Err(e) => {
                eprintln!("failed to find function in shared library: {}", e);
                return;
            }
        };

        // Time the function call
        let start = Instant::now();
        func();
        let duration = start.elapsed();

        println!("Function '{}' executed in {:?}", function_name, duration);
    }
}
