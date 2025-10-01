fn main() {
    println!("cargo:rustc-env=ESP_IDF_PATH_ISSUES=warn");
    embuild::espidf::sysenv::output();
}
