fn main() {
    println!("cargo:rerun-if-changed=src/app/drivers");
    println!("cargo:rerun-if-changed=templates");
}
