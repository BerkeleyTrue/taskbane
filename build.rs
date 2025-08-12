use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/style.css");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=src/app/drivers");
    println!("cargo:rerun-if-changed=templates");

    let output = Command::new("tailwindcss")
        .args(&["-i", "src/style.css", "-o", "public/css/style.css", "--content", "templates/**/*.html"])
        .output()
        .expect("Failed to execute tailwindcss command");

    if !output.status.success() {
        eprintln!("Tailwind CSS compilation failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    println!("Tailwind CSS compiled successfully to public/css/style.css");
}
