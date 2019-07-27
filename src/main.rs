mod mat;

fn main() {
    let mat = mat::Mat::load_jpeg("large.jpg");
    mat.save_as_bmp("large.bmp");

    let mat_png = mat::Mat::load_png("rust.png");
    mat_png.save_as_png("rust_1.png");

    let new_mat = mat.crop(1, 1, 200, 200);
    new_mat.save_as_png("croped_large.png");
    let resized_mat = mat.resize(200, 100);
    resized_mat.save_as_png("resized_large.png");
    let mat_ma = mat::Mat::load_png("rust.png");
    let mat_mb = mat::Mat::load_jpeg("large.jpg");
    let mat_merged_result = mat_ma.merge(mat_mb.resize(mat_ma.cols as usize, mat_ma.rows as usize));
    mat_merged_result.save_as_png("merge_result.png");
    
    // let mat_merged_result_channel_0 = mat.get_channel(0);
    // mat_merged_result_channel_0.save_as_png("merge_result_c0.jpg");

    // let mat_merged_result_channel_1 = mat.get_channel(1);
    // mat_merged_result_channel_1.save_as_png("merge_result_c1.jpg");
}
