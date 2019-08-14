extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use rand::Rng;


fn main() {
    let left = mat::Mat::load_png("examples/test_left.png");
    let right = mat::Mat::load_png("examples/test_right.png");
    let result = eva_lib::stitch_left_right(left, right);
    result.save_as_png("lib_example.png");

    let top = mat::Mat::load_jpeg("examples/top.jpg");
    let bottom = mat::Mat::load_jpeg("examples/bottom.jpg");
    let result = eva_lib::stitch_top_bottom(top, bottom);
    result.save_as_png("lib_example_top_bottom.png");
}
