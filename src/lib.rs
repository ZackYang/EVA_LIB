pub mod mat;
pub mod cl;

use cl::CL;
use mat::Mat;
use mat::pixel_description::{PixelDescription, Direction};
use std::time::Instant;

#[macro_use]
extern crate lazy_static;

pub fn stitch_left_right(left: Mat, right: Mat)
    -> Mat
{
    let total_begin = Instant::now();
    let left_gray = left.to_gray();
    let right_gray = right.to_gray();
    // 433ms
    fn gen_masks(src: &Mat, width: usize, height: usize)
    -> Vec<((usize, usize, usize, usize), (usize, usize, usize, usize))>
    {
        let groups = src.rows/height;
        let mut masks = Vec::<((usize, usize, usize, usize), (usize, usize, usize, usize))>::with_capacity(groups as usize);

        for i in 0..groups {
            let left_x = src.cols - width -1;
            let right_x = 0usize;
            let y = i * height;
            let w = width;
            let h = height;
            masks.push(((left_x, y, w, h), (right_x, y, w, h)));
        }
        masks
    }
    
    let mut match_points = Vec::<(PixelDescription, PixelDescription)>::new();
    let mask_pairs = gen_masks(&left_gray, 1000, 200);

    for mask_pair in mask_pairs {
        let left_descriptions = left_gray.fast_search_features(10, &mask_pair.0, Direction::Horizontal);
        let right_descriptions = right_gray.fast_search_features(10, &mask_pair.1, Direction::Horizontal);

        let points = &PixelDescription::match_points(&left_descriptions, &right_descriptions, 900);
        match_points.extend_from_slice(points);
    }

    // 922ms
    let move_vector = Mat::avg_mapping_vector(&match_points);
    let mut dist = Mat::new(left.cols + right.cols - (left.cols - move_vector.0 as usize), left.rows, Some(255u8));

    let shared_section = transition_section(&left, move_vector);
    println!("{:?}", dist.rows);
    // Mat::move_mat(&mut dist, &left, (0., 0.));
    dist.merge(left, 0, 0);
    println!("{:?}", shared_section);
    println!("===================================================================================================");
    let left_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);
    Mat::move_mat(&mut dist, &right, move_vector);
    // Mat::move_mat_by_multi_points(&mut dist, &right, &match_points, move_vector);
    let right_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);

    let shared_mat = fuse(&left_shared_mat, &right_shared_mat, Direction::Horizontal);
    let total_begin = total_begin.elapsed().as_millis();
    // shared_mat.save_as_png("shared_mat_1.png");
    println!("Spend ms on STITCHING:{}", total_begin);
    dist.merge(shared_mat, shared_section.0 as usize, shared_section.1 as usize);
    dist
}

pub fn stitch_top_bottom(top: Mat, bottom: Mat)
    -> Mat
{
    let total_begin = Instant::now();
    let top_gray = top.to_gray();
    let bottom_gray = bottom.to_gray();

    fn gen_masks(src: &Mat, width: usize, height: usize)
    -> Vec<((usize, usize, usize, usize), (usize, usize, usize, usize))>
    {
        let groups = src.cols/width;
        let mut masks = Vec::<((usize, usize, usize, usize), (usize, usize, usize, usize))>::with_capacity(groups as usize);

        for i in 0..groups {
            let x = i*width;
            let top_y = src.rows- height -1;
            let bottom_y = 0;
            let w = width;
            let h = height;
            masks.push(((x, top_y, w, h), (x, bottom_y, w, h)));
        }
        masks
    }

    let mut match_points = Vec::<(PixelDescription, PixelDescription)>::new();
    let mask_pairs = gen_masks(&top_gray, 100, 1100);
    let total_begin = total_begin.elapsed().as_millis();
    for mask_pair in mask_pairs {
        println!("Mask X: {:?}", (mask_pair.0).0);
        let top_descriptions = top_gray.fast_search_features(10, &mask_pair.0, Direction::Vertical);
        let bottom_descriptions = bottom_gray.fast_search_features(10, &mask_pair.1, Direction::Vertical);

        let points = &PixelDescription::match_points(&top_descriptions, &bottom_descriptions, 900);
        println!("Pairs: {:?}", points.len());
        println!("Mask: ====================================================");
        // match_points.extend_from_slice(points);
        if points.len() >= 3 {
            let a_vect = Mat::get_vector(&points[0].0, &points[0].1);
            let b_vect = Mat::get_vector(&points[1].0, &points[1].1);
            let c_vect = Mat::get_vector(&points[2].0, &points[2].1);
            if (a_vect.0 - b_vect.0).abs() < 5.0 && (a_vect.1 - b_vect.1).abs() < 5.0 && (a_vect.1 - c_vect.1).abs() < 5.0 {
                match_points.push(points[0].clone());
            }
        }
    }

    let move_vector = Mat::avg_mapping_vector(&match_points);

    let mut dist = Mat::new(top.cols, top.rows + bottom.rows - (top.rows - move_vector.1 as usize), Some(255u8));
    
    let shared_section = transition_section(&top, move_vector);

    Mat::move_mat(&mut dist, &top, (0., 0.));
    let top_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);
    
    // Mat::move_mat(&mut dist, &bottom, move_vector);
    Mat::move_mat_by_multi_points(&mut dist, &bottom, move_vector, &match_points);
    let bottom_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);
    
    let shared_mat = fuse(&top_shared_mat, &bottom_shared_mat, Direction::Vertical);
    // shared_mat.save_as_png("shared_mat_2.png");

    println!("Spend ms on STITCHING:{}", total_begin);
    dist.merge(shared_mat, shared_section.0 as usize, shared_section.1 as usize);
    // for pair in match_points {
    //     // println!("{:?}, ", pair.0.coordinate.0);
    //     dist.draw_point(pair.0.coordinate, vec!(255u8, 0u8, 0u8));
    // }

    dist
}

fn fuse(a_image: &Mat, b_image: &Mat, direction: Direction) -> Mat {
    let mut new_section = Mat::new(a_image.cols, a_image.rows, Some(255u8));
    for y in 0..(a_image.rows as usize) {
        for x in 0..(a_image.cols as usize) {
            let mut new_vec = Vec::<u8>::new();
            for i in 0..a_image.bytes_per_pixel {
                let factor = match direction {
                    Direction::Horizontal => {
                        1.0 - x as f32/a_image.cols as f32
                    },
                    Direction::Vertical => {
                        1.0 - y as f32/a_image.rows as f32
                    } 
                };
                let mut value = (a_image.get_pixel_by_xy(x, y)[i] as f32 * factor).round() + (b_image.get_pixel_by_xy(x, y)[i] as f32 * (1.0-factor)).round();
                if value > 255.0 {
                    value = 255.0;
                }
                new_vec.push(value as u8);
            }
            new_section.set_pixel_by_xy(x, y, new_vec);
        }
    }
    new_section
}

fn transition_section(a_image: &Mat, move_vector: (f32, f32))
    -> (usize, usize, usize, usize)
{
    let x = match move_vector.0 < 0.0 {
        true => 0usize,
        false => move_vector.0 as usize
    };

    let y = match move_vector.1 < 0.0 {
        true => 0usize,
        false => move_vector.1 as usize
    };

    let width = match a_image.cols as f32 - move_vector.0 < 0.0 {
        true => 0usize,
        false => {
            if move_vector.0 < 0.0f32 {
                a_image.cols as usize
            } else {
                (a_image.cols as f32 - move_vector.0) as usize
            }
        }
    };

    let height = match a_image.rows as f32 - move_vector.1 < 0.0 {
        true => 0usize,
        false => {
            if move_vector.1 < 0.0f32 {
                a_image.rows as usize
            } else {
                (a_image.rows as f32 - move_vector.1) as usize
            }
        }
    };

    (x, y, width, height)
}