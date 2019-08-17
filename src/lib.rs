pub mod mat;
pub mod cl;

use mat::Mat;
use mat::pixel_description::{PixelDescription, Direction};
use std::time::Instant;

pub fn stitch_left_right(left: Mat, right: Mat)
    -> Mat
{
    let total_begin = Instant::now();
    let left_gray = left.to_gray();
    let right_gray = right.to_gray();

    fn gen_masks(src: &Mat, width: u16, height: u16)
    -> Vec<((u16, u16, u16, u16), (u16, u16, u16, u16))>
    {
        let groups = src.rows/height;
        let mut masks = Vec::<((u16, u16, u16, u16), (u16, u16, u16, u16))>::with_capacity(groups as usize);

        for i in 0..groups {
            let left_x = src.cols - width -1;
            let right_x = 0u16;
            let y = i * height;
            let w = width;
            let h = height;
            masks.push(((left_x, y, w, h), (right_x, y, w, h)));
        }
        masks
    }

    let mut match_points = Vec::<(PixelDescription, PixelDescription)>::new();
    let mask_pairs = gen_masks(&left_gray, 200, 200);

    for mask_pair in mask_pairs {
        let left_descriptions = left_gray.fast_search_features(10, &mask_pair.0, Direction::Horizontal);
        let right_descriptions = right_gray.fast_search_features(10, &mask_pair.1, Direction::Horizontal);

        let points = &PixelDescription::match_points(&left_descriptions, &right_descriptions, 900);
        match_points.extend_from_slice(points);
    }

    let move_vector = Mat::avg_mapping_vector(&match_points);

    let mut dist = Mat::new(left.cols + right.cols - (left.cols - move_vector.0 as u16), left.rows, Some(255u8));
    
    let shared_section = transition_section(&left, move_vector);

    Mat::move_mat(&mut dist, &left, (0., 0.));
    let left_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);

    Mat::move_mat(&mut dist, &right, move_vector);
    let right_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);

    let shared_mat = fuse(&left_shared_mat, &right_shared_mat, Direction::Horizontal);
    shared_mat.save_as_png("shared_mat_1.png");

    println!("Spend ms on STITCHING:{}", total_begin.elapsed().as_millis());
    dist.merge(shared_mat, shared_section.0 as u16, shared_section.1 as u16)
}

pub fn stitch_top_bottom(top: Mat, bottom: Mat)
    -> Mat
{
    let total_begin = Instant::now();
    let top_gray = top.to_gray();
    let bottom_gray = bottom.to_gray();

    fn gen_masks(src: &Mat, width: u16, height: u16)
    -> Vec<((u16, u16, u16, u16), (u16, u16, u16, u16))>
    {
        let groups = src.rows/width;
        let mut masks = Vec::<((u16, u16, u16, u16), (u16, u16, u16, u16))>::with_capacity(groups as usize);

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
    let mask_pairs = gen_masks(&top_gray, 200, 200);

    for mask_pair in mask_pairs {
        let top_descriptions = top_gray.fast_search_features(10, &mask_pair.0, Direction::Vertical);
        let bottom_descriptions = bottom_gray.fast_search_features(10, &mask_pair.1, Direction::Vertical);

        let points = &PixelDescription::match_points(&top_descriptions, &bottom_descriptions, 900);
        match_points.extend_from_slice(points);
    }

    let move_vector = Mat::avg_mapping_vector(&match_points);

    let mut dist = Mat::new(top.cols, top.rows + bottom.rows - (top.rows - move_vector.1 as u16), Some(255u8));
    
    let shared_section = transition_section(&top, move_vector);

    Mat::move_mat(&mut dist, &top, (0., 0.));
    let top_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);

    Mat::move_mat(&mut dist, &bottom, move_vector);
    let bottom_shared_mat = dist.crop(shared_section.0, shared_section.1, shared_section.2, shared_section.3);
    
    let shared_mat = fuse(&top_shared_mat, &bottom_shared_mat, Direction::Vertical);
    shared_mat.save_as_png("shared_mat_2.png");

    println!("Spend ms on STITCHING:{}", total_begin.elapsed().as_millis());
    dist.merge(shared_mat, shared_section.0 as u16, shared_section.1 as u16)
}

fn fuse(a_image: &Mat, b_image: &Mat, direction: Direction) -> Mat {
    let mut new_section = Mat::new(a_image.cols, a_image.rows, Some(255u8));
    for y in 0..(a_image.rows as usize) {
        for x in 0..(a_image.cols as usize) {
            let mut new_vec = Vec::<u8>::new();
            for i in 0..a_image.data[y][x].len() {
                let factor = match direction {
                    Direction::Horizontal => {
                        1.0 - x as f32/a_image.cols as f32
                    },
                    Direction::Vertical => {
                        1.0 - y as f32/a_image.rows as f32
                    } 
                };
                let mut value = (a_image.data[y][x][i] as f32 * factor).round() + (b_image.data[y][x][i] as f32 * (1.0-factor)).round();
                if value > 255.0 {
                    value = 255.0;
                }
                new_vec.push(value as u8);
            }
            new_section.data[y][x] = new_vec;
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
        false => (a_image.cols as f32 - move_vector.0) as usize
    };

    let height = match a_image.rows as f32 - move_vector.1 < 0.0 {
        true => 0usize,
        false => (a_image.rows as f32 - move_vector.1) as usize
    };

    (x, y, width, height)
}