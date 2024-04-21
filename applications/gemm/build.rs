fn main() {
    println!("cargo:rustc-link-lib=static=myrdma");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-search=./drust");
}