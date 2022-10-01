extern crate proc_macro;
use proc_macro::TokenStream;
use std::fs;

#[proc_macro]
pub fn make_sloth_images_array(_item: TokenStream) -> TokenStream {
    let mut syntax: Vec<String> = vec![String::from("const SLOTH_IMAGES: &[&[u8]] = &[")];

    for file in fs::read_dir("./resources/processed/").unwrap() {
        syntax.push(format!("include_bytes!(\"./../resources/processed/{}\"),", file.unwrap().file_name().to_str().unwrap()));
    }

    syntax.push(String::from("];"));

    syntax.join("").parse().unwrap()
}
