use std::fs::File;

use zip::ZipArchive;

pub mod frontend;

fn main() {
    let mut archive = ZipArchive::new(File::open("./jagexappletviewer.jar").unwrap()).unwrap();

    for file in 0..archive.len(){
        let file = archive.by_index(file).unwrap();
        if file.name().ends_with(".class"){
            println!("{}", file.name())
        }
    }
    println!("Hello, world!");
}
