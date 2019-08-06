extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use rand::Rng;


fn main() {
    let mut mat = mat::Mat::load_jpeg("examples/unfocus.jpg");
    let mut mask = Mat::new(mat.cols, mat.rows, None);
    mask = mask.merge(Mat::new(300, mat.rows, Some(255u8)), mat.cols - 300 - 1, 0);
    let descriptions = mat.fast_search_features(30, Some(mask));
    
    for desc in descriptions {
        mat.draw_point(desc.coordinate, &mut vec![0u8, 255u8, 0u8, 255u8]);
    }
    mat.save_as_png("feature_points.png");
}
