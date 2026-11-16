fn main() {
    println!("cargo:rerun-if-changed=assets/wiu.rc");
    println!("cargo:rerun-if-changed=assets/wiu.ico");
    let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("wiu.o");
    let status=std::process::Command::new("windres").args(["-I","assets","assets/wiu.rc","-O","coff","-o"]).arg(&out).status().expect("windres requis (MinGW)");
    assert!(status.success(),"Échec des ressources Windows");
    println!("cargo:rustc-link-arg={}",out.display());
}
