extern crate proc_macro;
use proc_macro::TokenStream;
use std::fs;

fn read_sloth_image_directory(folder: &str, syntax: &mut Vec<String>) {
    syntax.push(format!("\nconst SLOTH_IMAGES_{}: &[&[u8]] = &[", folder));

    for file in fs::read_dir(format!("./resources/{}/", folder)).unwrap() {
        syntax.push(format!("include_bytes!(\"./../resources/{}/{}\"),", folder, file.unwrap().file_name().to_str().unwrap()));
    }

    syntax.push(String::from("];"));
}

#[proc_macro]
pub fn make_sloth_images_array(_item: TokenStream) -> TokenStream {
    let mut syntax: Vec<String> = Vec::new();

    read_sloth_image_directory("1_BY_1", &mut syntax);
    read_sloth_image_directory("4_BY_3", &mut syntax);
    read_sloth_image_directory("3_BY_4", &mut syntax);

    syntax.join("").parse().unwrap()
}
