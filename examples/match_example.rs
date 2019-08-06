extern crate eva_lib;
use eva_lib::mat;
use eva_lib::mat::Mat;
use mat::pixel_description::PixelDescription;
use rand::Rng;


fn main() {
    let mut tree_left = mat::Mat::load_png("examples/test_left.png").resize(1680, 1920);
    let mut tree_right = mat::Mat::load_png("examples/test_right.png").resize(1680, 1920);

    let groups = tree_left.cols/200;

    let mut match_points = Vec::<(PixelDescription, PixelDescription)>::new();

    for i in 0..groups {
        let mut left_mask = Mat::new(tree_left.cols, tree_left.rows, None);
        left_mask = left_mask.merge(Mat::new(200, 200, Some(255u8)), tree_left.cols - 200 - 1, i*200);
        let tree_left_descriptions = tree_left.fast_search_features(10, Some(left_mask));

        let mut right_mask = Mat::new(tree_right.cols, tree_right.rows, None);
        right_mask = right_mask.merge(Mat::new(200, 200, Some(255u8)), 0, i*200);
        let tree_right_descriptions = tree_right.fast_search_features(10, Some(right_mask));

        let points = &PixelDescription::match_points(&tree_left_descriptions, &tree_right_descriptions, 880, 2000, 50);
        match_points.extend_from_slice(points);

        for desc in tree_left_descriptions {
            tree_left.draw_point(desc.coordinate, &mut vec![0u8, 255u8, 0u8, 255u8]);
        }
        for desc in tree_right_descriptions {
            tree_right.draw_point(desc.coordinate, &mut vec![0u8, 255u8, 0u8, 255u8]);
        }
    }

    // match_points = PixelDescription::filter_pair(&match_points);
    let mut combined_image = Mat::new(tree_left.cols + tree_right.cols, 1920, None);

    let left_tree_cols = tree_right.cols;
    combined_image = combined_image.merge(tree_left, 0, 0);
    combined_image = combined_image.merge(tree_right, left_tree_cols - 1, 0);
    
    println!("Pairs: {:?}", match_points.len());

    for point_pair in match_points {
        let point_a_xy = point_pair.0.coordinate;
        let point_b_xy = point_pair.1.coordinate;
        let mut rng = rand::thread_rng();

        let r: u8 = rng.gen();
        let g: u8 = rng.gen();
        let b: u8 = rng.gen();

        let mut color = vec![r, g, b, 255u8];
        combined_image.draw_line((point_a_xy.0 as usize, point_a_xy.1 as usize), ((point_b_xy.0 + 1680) as usize, point_b_xy.1 as usize), &mut color);
    }

    combined_image.save_as_png("combined_image.png");
}