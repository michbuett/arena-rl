use std::io::Result;
use std::process::Command;

fn main() {
    let result = Command::new("sh").arg("assets/images/compile-sprites").output();

    match result {
        Result::Ok(output) => {
            if output.status.success() {
                for dbg_str in String::from_utf8(output.stdout).unwrap().lines() {
                    println!("[DEBUG] {}", dbg_str);
                }

            } else {
                for err_str in String::from_utf8(output.stderr).unwrap().lines() {
                    println!("cargo:warning={}", err_str);
                }
            }
        }
        
        Result::Err(e) => {
            println!("cargo:warning={:?}", e);
        }
    }
}
