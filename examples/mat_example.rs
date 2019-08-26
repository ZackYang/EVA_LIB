extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use mat::pixel_description::Direction;
use rand::Rng;


fn main() {
    // let mat = mat::Mat::load_jpeg("examples/tests/chips.jpg");
    // mat.save_as_bmp("large.bmp");

    // let mat_png = mat::Mat::load_png("examples/rust.png");
    // mat.save_as_png("large.png");

    // let mat_gray_png = mat.to_gray();
    // mat_gray_png.save_as_png("large_gray.png");

    // mat_gray_png.to_gray().save_as_png("gray_to_gray.png");

    // let new_mat = mat.crop(1, 1, 800, 800);
    // new_mat.save_as_png("croped_large.png");
    // let resized_mat = mat.resize(200, 100);
    // let beauty = mat::Mat::load_jpeg("examples/beauty.jpeg");
    // let mut unfocus = mat::Mat::load_jpeg("examples/tests/chips.jpg");

    // let laplation_result = beauty.convolute(mat::kernels::Kernel::laplation_4());
    // let unfocus_laplation_result = unfocus.to_gray().convolute(mat::kernels::Kernel::laplation_8());
    
    // unfocus_laplation_result.save_as_png("laplation_result.png");
    
    // Stitching two pictures
    
    // END stitching two pictures

    // resized_mat.save_as_png("resized_large.png");
    let mat_ma = mat::Mat::load_jpeg("examples/tests/top.jpg");
    mat_ma.fast_search_features(10, &(0, 0, mat_ma.cols, mat_ma.rows), Direction::Horizontal);
    // let mut mat_mb = mat::Mat::load_jpeg("examples/tests/large.jpg");
    // mat_ma.merge(mat_mb, 100, 100);

    // let new_mat = mat::Mat::load_from_vec(vec![105u8; 40000], 100, 100, 4);

    // mat_ma.save_as_png("new_mat.png");
    
    // let mat_merged_result_channel_0 = mat.get_channel(0);
    // mat_merged_result_channel_0.save_as_png("merge_result_c0.jpg");

    // let mat_merged_result_channel_1 = mat.get_channel(1);
    // mat_merged_result_channel_1.save_as_png("merge_result_c1.jpg");
}
