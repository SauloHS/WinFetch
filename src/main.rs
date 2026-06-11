use windows::core::s;
use windows::core::PWSTR;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::SystemInformation::OSVERSIONINFOEXW;
use windows::Win32::System::WindowsProgramming::{GetComputerNameW, GetUserNameW};

use sysinfo::System;

use colored::Colorize;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

fn get_resolution() -> String {
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    format!("{}x{}", w, h)
}

struct HardwareInfo {
    cpu_name: String,
    cpu_cores: usize,
    ram_used: u64,
    ram_total: u64,
    disks: Vec<(String, u64, u64)>,
}

fn get_hardware_info() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.cpus().first().unwrap();
    let cpu_name = cpu.brand().to_string();
    let cpu_cores = sys.cpus().len();

    let ram_used = sys.used_memory() / 1024 / 1024;
    let ram_total = sys.total_memory() / 1024 / 1024;

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disks = disks
        .list()
        .iter()
        .map(|d| {
            let used = (d.total_space() - d.available_space()) / 1024 / 1024 / 1024;
            let total = d.total_space() / 1024 / 1024 / 1024;
            (d.mount_point().display().to_string(), used, total)
        })
        .collect();

    HardwareInfo {
        cpu_name,
        cpu_cores,
        ram_used,
        ram_total,
        disks,
    }
}

fn get_hostname() -> String {
    let mut buf = [0u16; 256];
    let mut size = buf.len() as u32;
    unsafe { GetComputerNameW(PWSTR(buf.as_mut_ptr()), &mut size) };
    String::from_utf16_lossy(&buf[..size as usize])
}

fn get_username() -> String {
    let mut buf = [0u16; 256];
    let mut size = buf.len() as u32;
    unsafe { GetUserNameW(PWSTR(buf.as_mut_ptr()), &mut size).unwrap() };
    String::from_utf16_lossy(&buf[..size as usize - 1])
}

fn get_uptime() -> String {
    let ms = unsafe { GetTickCount64() };
    let secs = ms / 1000;
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    format!("{hours}h {mins}m")
}

fn get_version() -> (u32, u32, u32) {
    unsafe {
        let ntdll = LoadLibraryA(s!("ntdll.dll")).unwrap();
        let proc = GetProcAddress(ntdll, s!("RtlGetVersion")).unwrap();

        let rtl_get_version: unsafe extern "system" fn(*mut OSVERSIONINFOEXW) -> i32 =
            std::mem::transmute(proc);

        let mut info = OSVERSIONINFOEXW::default();
        info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;
        rtl_get_version(&mut info);

        (info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber)
    }
}

fn get_shell() -> String {
    std::env::var("COMSPEC")
        .unwrap_or_default()
        .split('\\')
        .last()
        .unwrap_or("unknown")
        .to_string()
}

fn get_terminal() -> String {
    if std::env::var("WT_SESSION").is_ok() {
        return "Windows Terminal".to_string();
    }
    if let Ok(term) = std::env::var("TERM_PROGRAM") {
        return term;
    }
    "Console Host".to_string()
}

fn main() {
    let (major, minor, build) = get_version();
    let version_name = if build >= 22000 {
        "Windows 11"
    } else {
        "Windows 10"
    };
    let kernel = format!("{major}.{minor}.{build}");
    let uptime = get_uptime();
    let hostname = get_hostname();
    let username = get_username();
    let resolution = get_resolution();
    let shell = get_shell();
    let terminal = get_terminal();
    let hw = get_hardware_info();

    let logo: Vec<&str> = vec![
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "                                    ",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
        "/////////////////  /////////////////",
    ];

    let disk_lines: Vec<String> = hw
        .disks
        .iter()
        .map(|(mount, used, total)| {
            format!(
                "{}: {} {} GB / {} GB",
                "Disk".truecolor(43, 251, 225),
                mount,
                used,
                total
            )
        })
        .collect();

    let colors_dark = [
        (0, 0, 0),
        (197, 15, 31),
        (19, 161, 14),
        (193, 156, 0),
        (0, 0, 255),
        (128, 0, 128),
        (58, 150, 221),
        (192, 192, 192),
    ];
    let colors_light = [
        (118, 118, 118),
        (231, 72, 86),
        (92, 250, 86),
        (249, 224, 115),
        (100, 100, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    let mut row1 = String::new();
    let mut row2 = String::new();

    for (r, g, b) in colors_dark {
        row1.push_str(&"   ".on_truecolor(r, g, b).to_string());
    }
    for (r, g, b) in colors_light {
        row2.push_str(&"   ".on_truecolor(r, g, b).to_string());
    }

    let mut info: Vec<String> = vec![
        format!(
            "{}@{}",
            username.truecolor(43, 251, 225),
            hostname.truecolor(43, 251, 225)
        ),
        format!("------------------------------------------"),
        format!(
            "{}: {} (build {})",
            "OS".truecolor(43, 251, 225),
            version_name,
            build
        ),
        format!("{}: {}", "Kernel".truecolor(43, 251, 225), kernel),
        format!("{}: {}", "Uptime".truecolor(43, 251, 225), uptime),
        format!("{}: {}", "Shell".truecolor(43, 251, 225), shell),
        format!("{}: {}", "Terminal".truecolor(43, 251, 225), terminal),
        format!("{}: {}", "Resolution".truecolor(43, 251, 225), resolution),
        format!(
            "{}: {} ({} cores)",
            "CPU".truecolor(43, 251, 225),
            hw.cpu_name,
            hw.cpu_cores
        ),
        format!(
            "{}: {} MB / {} MB",
            "Memory".truecolor(43, 251, 225),
            hw.ram_used,
            hw.ram_total
        ),
    ]
    .into_iter()
    .chain(disk_lines)
    .collect();

    info.push(String::new());
    info.push(String::new());
    info.push(format!("{}", row1));
    info.push(format!("{}", row2));

    let empty = String::new();
    for (logo_line, info_line) in logo
        .iter()
        .zip(info.iter().chain(std::iter::repeat(&empty)))
    {
        println!("{}    {}", logo_line.truecolor(0, 210, 245), info_line);
    }
}
