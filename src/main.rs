use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// Handy helper function for user input
fn ask_user(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Input read error");
    input.trim().to_string()
}

// Check and install sshfs if missing
fn ensure_sshfs() {
    let check = Command::new("which")
        .arg("sshfs")
        .output()
        .expect("Failed to check for sshfs");

    if !check.status.success() {
        println!("[-] sshfs not found. Starting installation via pacman...");
        let install = Command::new("sudo")
            .args(["pacman", "-S", "sshfs", "--noconfirm"])
            .status()
            .expect("Failed to execute pacman");

        if !install.success() {
            println!("[!] Failed to install sshfs. Check your permissions or internet connection.");
            std::process::exit(1);
        }
    } else {
        println!("[+] sshfs is already installed.");
    }
}

fn main() {
    println!("--- Uber SSH Manager ---");

    // 1. Gather core data into our vector
    let mut config = Vec::new();
    config.push(ask_user("Server IP: "));       // config[0]
    config.push(ask_user("Port: "));            // config[1]
    config.push(ask_user("Username: "));        // config[2]

    let ip = &config[0];
    let port = &config[1];
    let user = &config[2];

    // The terminal will handle the password prompt natively when ssh/sshfs runs.

    // 2. Mount Branch
    let need_mount = ask_user("Do you need to mount a folder? (y/n): ");

    if need_mount.to_lowercase() == "y" {
        ensure_sshfs(); // Check and install if necessary

        let folder_name = ask_user("Enter the name of the folder to mount: ");

        // Grab the current user's home directory from the OS
        let home_dir = env::var("HOME").expect("Failed to find $HOME");
        let mut mount_path = PathBuf::from(home_dir);
        mount_path.push(&folder_name);

        // Create the directory in /home/user/
        if !mount_path.exists() {
            fs::create_dir_all(&mount_path).expect("Failed to create directory");
            println!("[+] Directory {:?} created.", mount_path);
        }

        println!("[*] Connecting and mounting via sshfs...");
        // Command execution: sshfs -p port user@ip:/remote/path /local/path
        let mount_status = Command::new("sshfs")
            .args([
                format!("{}@{}:/", user, ip), // Mounting the server's root (can be changed)
                mount_path.to_string_lossy().into_owned(),
                "-p".to_string(),
                port.to_string(),
            ])
            .status()
            .expect("Error launching sshfs");

        if mount_status.success() {
            println!("[+] Successfully mounted to {:?}", mount_path);
        } else {
            println!("[-] Mount failed.");
        }
    }

    // 3. Terminal Branch
    let need_terminal = ask_user("Continue working in the server's terminal? (y/n): ");

    if need_terminal.to_lowercase() == "y" {
        println!("[*] Connecting via SSH...");
        // Connection: ssh login@ip -p port
        let mut ssh_proc = Command::new("ssh")
            .args([
                format!("{}@{}", user, ip),
                "-p".to_string(),
                port.to_string(),
            ])
            .spawn() // spawn() lets us "enter" the interactive process
            .expect("Failed to launch ssh");

        // Wait until you manually close the session on the server
        ssh_proc.wait().expect("Error waiting for ssh process");
    }

    // 4. Finale
    println!("[*] Shutting down. Catch you later, man!");
}
