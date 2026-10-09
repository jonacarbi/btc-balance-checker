#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn AttachConsole(pid: u32) -> i32;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--cli") {
        #[cfg(windows)]
        unsafe {
            AttachConsole(u32::MAX); // ATTACH_PARENT_PROCESS: print into the launching terminal
        }
        if let Err(e) = btc_balance_lib::cli::run(&args[2..]) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    } else {
        btc_balance_lib::run();
    }
}
