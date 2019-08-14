extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use rand::Rng;

use std::time::{Duration, Instant};
use std::thread::sleep;


fn main() {
    let tree_left = mat::Mat::load_png("examples/test_left.png");
    let tree_right = mat::Mat::load_png("examples/test_right.png");
    
    let tree_left_gray = tree_left.to_gray();
    let tree_right_gray = tree_right.to_gray();

    let mut match_points = Vec::<(PixelDescription, PixelDescription)>::new();

    let groups = tree_left_gray.rows/200;
    let mut masks = Vec::<((u16, u16, u16, u16), (u16, u16, u16, u16))>::with_capacity(groups as usize);

    for i in 0..groups {
        let left_x = tree_left_gray.cols - 200 -1;
        let right_x = 0u16;
        let y = i * 200;
        let w = 200;
        let h = 200;
        masks.push(((left_x, y, w, h), (right_x, y, w, h)));
    }
 

    let total_begin = Instant::now();

    for i in 0..groups {
        let now = Instant::now();

        let tree_left_descriptions = tree_left_gray.fast_search_features(10, &masks[i as usize].0, mat::pixel_description::Direction::Horizontal);
        let tree_right_descriptions = tree_right_gray.fast_search_features(10, &masks[i as usize].1, mat::pixel_description::Direction::Horizontal);
        
        println!("Spend ms on SEARCH:{}", now.elapsed().as_millis());
        println!("{:?}", "111111111111111111111111111111111111111111111111111111111111111111111111111111111111");

        let now = Instant::now();
        let points = &PixelDescription::match_points(&tree_left_descriptions, &tree_right_descriptions, 900);
        match_points.extend_from_slice(points);
        
        println!("Spend ms on MATCH:{}", now.elapsed().as_millis());
        println!("{:?}", "222222222222222222222222222222222222222222222222222222222222222222222222222222222222");
    }

    println!("Spend ms on TOTAL:{}", total_begin.elapsed().as_millis());
    println!("{:?}", "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");

    let mut combined_image = Mat::new(tree_left.cols + tree_right.cols, 3500, None);
    let tree_left_cols = tree_left.cols;
    let left_tree_cols = tree_right.cols;
    combined_image = combined_image.merge(tree_left, 0, 0);
    combined_image = combined_image.merge(tree_right, left_tree_cols - 1, 0);
    
    println!("Pairs: {:?}", match_points.len());

    for point_pair in &match_points {
        let point_a_xy = point_pair.0.coordinate;
        let point_b_xy = point_pair.1.coordinate;
        let mut rng = rand::thread_rng();

        let r: u8 = rng.gen();
        let g: u8 = rng.gen();
        let b: u8 = rng.gen();

        let mut color = vec![r, g, b, 255u8];
        combined_image.draw_line((point_a_xy.0 as usize, point_a_xy.1 as usize), ((point_b_xy.0 + tree_left_cols) as usize, point_b_xy.1 as usize), &mut color);
    }

    combined_image.save_as_png("combined_image.png");

    let tree_left = mat::Mat::load_png("examples/test_left.png");
    let tree_right = mat::Mat::load_png("examples/test_right.png");

    let move_vector = Mat::avg_mapping_vector(&match_points);
    let mut dist = Mat::new(tree_left.cols + tree_right.cols, tree_left.rows, Some(255u8));
    Mat::move_mat(&mut dist, &tree_left, (0., 0.));
    Mat::move_mat(&mut dist, &tree_right, move_vector);
    dist.save_as_png("merged_image.png");
}