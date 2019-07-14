mod mat;

fn main() {
    let mat = mat::Mat::load("large.jpg");
    mat.save("large.bmp");
    // // println!("Hello, world!");
    let new_mat = mat.crop(1, 1, 200, 800);
    new_mat.save("croped_large.bmp");
    let resized_mat = mat.resize(200, 100);
    resized_mat.save("resized_large.bmp");
}
