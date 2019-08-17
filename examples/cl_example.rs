extern crate eva_lib;
use eva_lib::cl::CL;
use eva_lib::mat::Mat;
use eva_lib::mat::kernels::Kernel;
use std::time::Instant;


fn main() {
    to_gray();
    crop();
    normalize();
    convolute();
    laplation();
}

fn to_gray() {
    let image = Mat::load_png("examples/tests/test_left.png");
    let cl = CL::new();
    let raw_data = image.flatten();
    let now = Instant::now();
    let gray_data = cl.cl_to_gray(&raw_data, 4).unwrap();
    pt(&now, Some("To Gray:"));
    let gray = Mat::load_from_vec(gray_data, image.cols, image.rows, 1);
    gray.save_as_png("examples/results/cl_to_gray.png");
}

fn crop() {
    let image = Mat::load_png("examples/tests/test_left.png");
    let cl = CL::new();
    let raw_data = image.flatten();
    let now = Instant::now();
    let data = cl.cl_crop(&raw_data, image.cols as i32, (image.cols as i32)/2-100, 200, 200, 200, 4).unwrap();
    pt(&now, Some("Crop:"));
    let croped = Mat::load_from_vec(data, 200, 200, 4);
    croped.save_as_png("examples/results/cl_crop.png");
}

fn normalize() {
    let data = vec![128u8; 900];
    let cl = CL::new();
    println!("First normalized element: {:?}", cl.cl_normalize(&data, 255.0).unwrap()[0]);
}

fn convolute() {
    let image = Mat::load_png("examples/tests/test_left.png");
    let cl = CL::new();
    let raw_data = image.flatten();
    let gray_data = cl.cl_to_gray(&raw_data, 4).unwrap();
    let normalized = cl.cl_normalize(&gray_data, 255.0).unwrap();
    let now = Instant::now();
    let convoluted = cl.cl_convolute(&normalized, image.cols as usize, image.rows as usize, &Kernel::laplation_8()).unwrap();
    pt(&now, Some("Convolution:"));
    // println!("{:?}", convoluted);
    let laplation = cl.cl_recover(&convoluted, 255.0).unwrap();

    let convoluted_image = Mat::load_from_vec(laplation, image.cols - 2, image.rows - 2, 1);
    convoluted_image.save_as_png("examples/results/cl_convolute.png");
}

fn laplation() {
    let image = Mat::load_jpeg("examples/tests/black.jpg");
    let cl = CL::new();
    let data = image.flatten();
    let now = Instant::now();
    let (width, height, sd, laplation_data) = cl.cl_laplation(&data, image.cols as usize, image.rows as usize, &Kernel::laplation_8(), 4).unwrap();
    pt(&now, Some("Laplation:"));
    println!("Standard Deviation: {:?}", sd);
    let laplation_image = Mat::load_from_vec(laplation_data, width as u16, height as u16, 1);
    laplation_image.save_as_png("examples/results/cl_laplation.png");
}

fn pt(now: &Instant, text: Option<&str>) {
    match text {
        Some(s) => {
            println!("{:?} {:?}", s, now.elapsed().as_millis());
        },
        None => {
            println!("{:?}", now.elapsed().as_millis());
        }
    }
}
