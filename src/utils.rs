use std::fs;

// FFI to call malloc_trim from glibc
extern "C" {
    fn malloc_trim(pad: usize) -> i32;
}

pub fn trim_memory() {
    unsafe {
        malloc_trim(0);
    }
}

pub fn get_current_rss_kb() -> u64 {
    trim_memory();
    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() > 1 {
            return parts[1].parse::<u64>().unwrap_or(0) * 4;
        }
    }
    0
}

pub fn print_box_top(title: &str) {
    println!("╭──── {} ────", title);
}

pub fn print_box_line(text: &str) {
    for line in text.lines() {
        println!("  {}", line);
    }
}

pub fn print_box_bottom() {
    let rss = get_current_rss_kb();
    println!("╰──── [RSS: {:.2} MB] ────", rss as f32 / 1024.0);
}
