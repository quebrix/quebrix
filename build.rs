use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let src = "./config/config.json"; 
    let bat = "src/installer/russel_install.bat";
    let nssm="src/nssm/nssm.exe";

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut config_dest = PathBuf::from(out_dir.clone());
    config_dest.push(".."); 
    config_dest.push(".."); 
    config_dest.push(".."); 
    config_dest.push("config"); 

    let mut bat_dest = PathBuf::from(out_dir.clone());
    bat_dest.push(".."); 
    bat_dest.push(".."); 
    bat_dest.push(".."); 

    let mut nssm_dest = PathBuf::from(out_dir.clone());
    nssm_dest.push(".."); 
    nssm_dest.push(".."); 
    nssm_dest.push(".."); 
    nssm_dest.push("nssm");
    
    
    fs::create_dir_all(&config_dest).unwrap();
    fs::create_dir_all(&nssm_dest).unwrap();
    
    config_dest.push("config.json"); 
    bat_dest.push("russel_install.bat"); 
    nssm_dest.push("nssm.exe");

    fs::copy(src, &config_dest).unwrap();
    fs::copy(bat, &bat_dest).unwrap();
    fs::copy(nssm, &nssm_dest).unwrap();

    println!("cargo:rerun-if-changed={}", src);
    println!("cargo:rerun-if-changed={}", bat);
    println!("cargo:rerun-if-changed={}", nssm);
}

