fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let artifacts = flua_src::Build::new().build();
    artifacts.print_cargo_metadata();
}
