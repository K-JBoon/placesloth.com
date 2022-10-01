#[macro_use] extern crate rocket;
use rocket::{State, Request, http::{Status, ContentType}};
use std::{io::Cursor, path::PathBuf};
use std::sync::Mutex;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::distributions::{Distribution, Uniform};
use image::{ImageFormat, DynamicImage, imageops::FilterType};
use include_images_proc_macro::*;


type ImageResponse = (ContentType, Vec<u8>);

// As the caches are shared between worked threads we need to mutex them to guarantee thread safety
type ImageCache = Mutex<HashMap<usize, DynamicImage>>;
type ImageResponseCache = Mutex<HashMap<String, ImageResponse>>;

// Dealing with the file system is effort, avoid effort and include everything in the binary
const INDEX_HTML: &str = include_str!("./../resources/index.html");
make_sloth_images_array!();

/// Pick a random byte array from SLOTH_IMAGES and convert to a DynamicImage
#[allow(clippy::map_entry)]
fn pick_random_sloth_image(image_cache: &State<ImageCache>) -> DynamicImage {
    let mut rng = rand::thread_rng();

    let range = Uniform::from(0..SLOTH_IMAGES.len());
    let index = range.sample(&mut rng);

    // Decoding is effort, effort is not very slothlike
    // Avoid effort 
    if image_cache.lock().unwrap().contains_key(&index) {
        image_cache.lock().unwrap().get(&index).unwrap().clone()
    } else {
        let image = image::load_from_memory_with_format(SLOTH_IMAGES.choose(&mut rng).unwrap(), ImageFormat::Jpeg).expect("Failed to create JPEG image from bytes");
        image_cache.lock().unwrap().insert(index, image.clone());
        image
    }

}

/// Resizes the image so both width and height are at least as large as the largest requested
/// dimensions, preserving aspect ratio
fn resize_image(image: DynamicImage, minimum_width: u32, minimum_height: u32) -> DynamicImage {
    let larger_request_size = if minimum_height > minimum_width { minimum_height } else { minimum_width };
    image.resize_to_fill(larger_request_size, larger_request_size, FilterType::Triangle)
}

/// Create a cutout centered on the image with the requested dimensions
fn crop_image(image: DynamicImage, width: u32, height: u32) -> DynamicImage {
    let mut crop_start_x: u32 = 0;
    let mut crop_start_y: u32 = 0;

    let original_width = image.width();
    let original_height = image.height();

    if width < original_width {
        crop_start_x = (original_width - width ) / 2;
    }
    if height < original_height {
        crop_start_y = (original_height - height) / 2;
    }

    image.crop_imm(crop_start_x, crop_start_y, width, height)
}

/// Handle any placesloth image requests
#[get("/<requested_width>/<height_or_height_and_ext..>")]
#[allow(clippy::map_entry)]
fn placesloth(requested_width: u32, height_or_height_and_ext: PathBuf, image_response_cache: &State<ImageResponseCache>, image_cache: &State<ImageCache>) -> Result<ImageResponse, Status> {
    // Get the requested extension if present, otherwise default to jpg
    let extension = if let Some(f) = height_or_height_and_ext.extension() {
        f.to_str().unwrap()
    } else {
        "jpg"
    };

    // Get the requested height, if we fail to parse stop and return a 400 response
    let requested_height = match height_or_height_and_ext.file_stem().expect("Incorrect path requested").to_str().unwrap().parse::<u32>() {
        Ok(v) => v,
        Err(_) => {
            return Err(Status::BadRequest)
        }
    };

    if requested_height > 2000 || requested_width > 2000 {
        return Err(Status::BadRequest);
    }

    let (output_format, content_type) = match extension {
        "png" => (ImageFormat::Png, ContentType::PNG),
        "gif" => (ImageFormat::Gif, ContentType::GIF),
        "ico" => (ImageFormat::Ico, ContentType::Icon),
        "bmp" => (ImageFormat::Bmp, ContentType::BMP),
        "jpg" => (ImageFormat::Jpeg, ContentType::JPEG),
        "jpeg" => (ImageFormat::Jpeg, ContentType::JPEG),
        _ => {
            return Err(Status::BadRequest)
        } 
    };

    // Check if we have a stored image for this resolution + extension combination
    // so we don't have to do effort
    let cache_key = format!("{}x{}.{}", requested_width, requested_height, extension);

    if image_response_cache.lock().unwrap().contains_key(&cache_key) {
        Ok(image_response_cache.lock().unwrap().get(&cache_key).unwrap().clone())
    } else {
        // Pick a random image from the set, resize it to be as large as needed to create a cropout
        // and then crop to the desired size
        let image = pick_random_sloth_image(image_cache); 
        let resized_image = resize_image(image, requested_width, requested_height); 
        let cropped_image = crop_image(resized_image, requested_width, requested_height);

        let mut buffer = vec![];

        let mut stream = Cursor::new(&mut buffer);
        cropped_image.write_to(&mut stream, output_format).expect("Failed to write resized JPEG to buffer");

        let res = (content_type, buffer);
        image_response_cache.lock().unwrap().insert(cache_key, res.clone());

        Ok(res)
    }
}

#[catch(400)]
fn bad_request(req: &Request) -> String {
    format!("400 Bad Request\n\n{} is not a valid request path. Please make sure neither width nor height exceed 2000px and that if you specify an extension it's one of: jpg, png, ico, gif, bmp", req.uri())
}

#[catch(500)]
fn internal_server_error(_: &Request) -> &'static str {
    "500 Internal Server Error\n\nWorking is effort, we don't like that here" 
}

/// Generate a favicon icon via the standard placesloth handler
#[get("/favicon.ico")]
fn favicon(image_response_cache: &State<ImageResponseCache>, image_cache: &State<ImageCache>) -> Result<ImageResponse, Status> {
    placesloth(64_u32, PathBuf::from("64.ico"), image_response_cache, image_cache)
}

#[get("/")]
fn index() -> (ContentType, &'static str) {
    (ContentType::HTML, INDEX_HTML)
}

#[launch]
fn rocket() -> _ {
    println!("Happy slothing!");
    let image_response_cache: ImageResponseCache = Mutex::new(HashMap::<String, ImageResponse>::new()); 
    let image_cache: ImageCache = Mutex::new(HashMap::<usize, DynamicImage>::new());

    rocket::build()
        .manage(image_response_cache)
        .manage(image_cache)
        .mount("/", routes![index, placesloth, favicon])
        .register("/", catchers![bad_request, internal_server_error])
}
