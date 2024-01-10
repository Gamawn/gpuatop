use std::{io, str};
use std::process::{Command, ExitStatus, Output};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
enum GpuType {
    Nvidia,
    Amd,
    Intel,
}

#[derive(Debug)]
enum PackageManager {
    Apt,
    Pacman,
    Yum,
}


const PACKAGE_MANAGERS: [&str; 3] = ["apt", "pacman", "yum"];

fn identify_package_manager() -> PackageManager {
    for package_manager in PACKAGE_MANAGERS {
        let output = Command::new("which")
            .arg(package_manager)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            match package_manager {
                "apt" => return PackageManager::Apt,
                "pacman" => return PackageManager::Pacman,
                "yum" => return PackageManager::Yum,
                _ => panic!("Package manager not found"),
            }
        }
    }

    panic!("Package manager not found");
}

fn identify_gpu_card() -> GpuType {
    let output = Command::new("lspci")
        .arg("-v")
        .output()
        .expect("Failed to execute command");

    let output = str::from_utf8(&output.stdout).expect("Not UTF-8");

    if output.contains("NVIDIA") {
        GpuType::Nvidia
    } else if output.contains("AMD") {
        GpuType::Amd
    } else if output.contains("Intel") {
        GpuType::Intel
    } else {
        panic!("GPU not found");
    }
}

fn check_top_exists_local(gpu_type: GpuType) -> io::Result<ExitStatus>  {
    let cmd = match gpu_type {
        GpuType::Nvidia => "nvidia-smi",
        GpuType::Amd => "radeontop",
        GpuType::Intel => "intel_gpu_top",
    };

    Command::new("which").arg(cmd).spawn()?.wait()
}

fn install_package_for_gpu(package_manager: PackageManager, package_name: &str) -> io::Result<Output>{
    let package_manager_command = match package_manager {
        PackageManager::Apt => "apt",
        PackageManager::Pacman => "pacman",
        PackageManager::Yum => "yum",
    };

    let package_manager_install_command = match package_manager {
        PackageManager::Apt => "install",
        PackageManager::Pacman => "-S",
        PackageManager::Yum => "install",
    };

    let package_manager_install_without_confirm_command = match package_manager {
        PackageManager::Apt => "-y",
        PackageManager::Pacman => "--noconfirm",
        PackageManager::Yum => "-y",
    };

    Command::new(package_manager_command)
        .args([package_manager_install_command, package_manager_install_without_confirm_command, package_name])
        .output()
}

fn install_top_for_gpu_to(gpu_type: GpuType, package_manager: PackageManager) -> io::Result<Output>{
    match gpu_type {
        GpuType::Nvidia => install_package_for_gpu( package_manager, "nvidia-smi"),
        GpuType::Amd => install_package_for_gpu( package_manager, "radeontop"),
        GpuType::Intel => install_package_for_gpu( package_manager, "intel_gpu_top"),
    }
}

fn main() {
    println!("Identifying GPU type...");
    let gpu_type = identify_gpu_card();
    println!("GPU type: {:?}", gpu_type);

    println!("Checking if top exists locally...");
    let top_exists = match check_top_exists_local(gpu_type) {
        Ok(ok) => ok.success(),
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    println!("Top exists locally: {}", top_exists);

    if !top_exists {
        println!("Identifying package manager...");
        let package_manager = identify_package_manager();
        println!("Package manager: {:?}", package_manager);

        println!("Installing top for GPU type...");
        let is_ok = match install_top_for_gpu_to(gpu_type, package_manager) {
            Ok(e) => e.status.success(),
            Err(er) => {  println!("Error: {}", er); return; }
        };

        if !is_ok {
            println!("Error: Failed to install top for GPU type");
            return;
        }
    }

    loop {
        let output = match gpu_type {
            GpuType::Nvidia => Command::new("nvidia-smi")
                .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
                .output()
                .expect("Failed to execute command"),
            GpuType::Amd => Command::new("radeontop")
                .arg("-d -")
                .output()
                .expect("Failed to execute command"),
            GpuType::Intel => Command::new("intel_gpu_top")
                .args(["-s", "1","-o", "-"])
                .output()
                .expect("Failed to execute command"),
        };

        let output = str::from_utf8(&output.stdout).expect("Not UTF-8");

        println!("GPU Utilization: {}%", output.trim());

        thread::sleep(Duration::from_secs(1));
    }
}
