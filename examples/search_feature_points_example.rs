extern crate eva_lib;

use eva_lib::mat::Mat;
use eva_lib::mat::pixel_description::Direction;


fn main() {
    let mut mat = Mat::load_jpeg("examples/unfocus.jpg");
    let mask = (0, 0, mat.cols, mat.rows);
    let descriptions = mat.fast_search_features(30, &mask, Direction::Horizontal);
    
    for desc in descriptions {
        mat.draw_point(desc.coordinate, &mut vec![0u8, 255u8, 0u8, 255u8]);
    }
    mat.save_as_png("feature_points.png");
}
