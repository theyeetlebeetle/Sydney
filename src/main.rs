use std::io::{Read, Write};
use std::net::TcpStream;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use sysinfo::{System, SystemExt};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8081").unwrap();
    loop {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        if n == 0 { break; }
        let msg = String::from_utf8_lossy(&buffer[..n]);
        if msg.starts_with("UPLOAD: ") {
            handle_upload(&msg[8..], &mut stream);
        } else if msg.starts_with("DOWNLOAD: ") {
            handle_download(&msg[10..], &mut stream);
        } else if msg.starts_with("EXIT: ") {
            break;
        } else if msg.starts_with("STATUS") {
            handle_status(&mut stream);
        } else {
            println!("Unknown message: {}", msg);
        }
    }
}

fn handle_upload(path: &str, stream: &mut TcpStream) {
    let mut file = match OpenOptions::new().write(true).create(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            stream.write(format!("Failed to open file {}: {}\n", path, e).as_bytes()).unwrap();
            return;
        }
    };
    let n = stream.read(&mut [0; 1024]).unwrap();
    if n == 0 { return; }
    match file.write_all(&[0; 1024][..n]) {
        Ok(_) => {
            stream.write(b"File saved successfully\n").unwrap();
            let hash = calculate_hash(path);
            stream.write(format!("File hash: {}\n", hash).as_bytes()).unwrap();
        },
        Err(e) => {
            stream.write(format!("Failed to write file {}: {}\n", path, e).as_bytes()).unwrap();
        }
    };
}

fn handle_download(path: &str, stream: &mut TcpStream) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            stream.write(format!("Failed to open file {}: {}\n", path, e).as_bytes()).unwrap();
            return;
        }
    };
    loop {
       let mut buffer = [0; 1024];
       let n = file.read(&mut buffer).unwrap();
       if n == 0 { break; }
       stream.write_all(&buffer[..n]).unwrap(); 
    }
    let hash = calculate_hash(path);
    stream.write(format!("\nFile hash: {}\n", hash).as_bytes()).unwrap();
}

fn calculate_hash(path:&str)->u64{
let mut hasher=DefaultHasher::new(); // create a new hasher
let mut file=File::open(path).expect("Unable to open"); // open the file
let mut buffer=[0;1024]; // create a buffer
loop{
let n=file.read(&mut buffer).expect("Unable to read"); // read from the file
if n==0{break;} // break if end of file
buffer[..n].hash(&mut hasher); // update the hasher with the buffer slice
}
hasher.finish() // return the final hash value
}

fn handle_status(stream:&mut TcpStream){
    let mut system=System::new_all(); // create a new system object with all information
    system.refresh_all(); // refresh all system information
    let os_name=system.name().unwrap_or_else(||String::from("Unknown")); // get the OS name or "Unknown"
    let os_version=system.os_version().unwrap_or_else(||String::from("Unknown")); // get the OS version or "Unknown"
    let total_memory=system.total_memory(); // get the total memory in kB
    let used_memory=system.used_memory(); // get the used memory in kB
    let total_swap=system.total_swap(); // get the total swap in kB
    let used_swap=system.used_swap(); // get the used swap in kB

    stream.write(format!("Operating System Name: {}\n",os_name).as_bytes()).expect("Unable to write"); // write OS name to stream
    stream.write(format!("Operating System Version: {}\n",os_version).as_bytes()).expect("Unable to write"); // write OS version to stream
    stream.write(format!("Total Memory: {} kB\n",total_memory).as_bytes()).expect("Unable to write"); // write total memory to stream
    stream.write(format!("Used Memory: {} kB\n",used_memory).as_bytes()).expect("Unable to write"); // write used memory to stream
    stream.write(format!("Total Swap: {} kB\n",total_swap).as_bytes()).expect("Unable to write"); // write total swap to stream
    stream.write(format!("Used Swap: {} kB\n",used_swap).as_bytes()).expect("Unable to write"); // write used swap to stream

}
