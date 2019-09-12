extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use rand::Rng;
use std::mem;

fn main() {
    let left = mat::Mat::load_png("examples/tests/6pics/result.png");
    let right = mat::Mat::load_png("examples/tests/6pics/result1.png");
    let result = eva_lib::stitch_top_bottom(left, right);
    result.save_as_png("examples/tests/6pics/final_result.png");

    // let top = mat::Mat::load_jpeg("examples/tests/top.jpg");
    // let bottom = mat::Mat::load_jpeg("examples/tests/bottom.jpg");
    // let result = eva_lib::stitch_top_bottom(top, bottom);
    // result.save_as_png("lib_example_top_bottom.png");
}


// fn get_img() -> Vec<u8> {
//     let left = mat::Mat::load_jpeg("examples/tests/4k.jpg");
//     std::thread::sleep_ms(20000);
//     println!("==========================================");
//     left.flatten()
// }
