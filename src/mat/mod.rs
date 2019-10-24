use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::io::BufWriter;

extern crate bmp;
extern crate png;
use jpeg_decoder::Decoder;
use jpeg_decoder::PixelFormat;

use std::time::{Instant};
use super::CL;

pub mod kernels;
pub mod pixel_description;
pub mod transform;

use pixel_description::PixelDescription;
use pixel_description::Direction;


lazy_static! {
    static ref CL_INSTANCE: CL = CL::new();
}

#[derive(Debug, Clone)]
pub struct Mat {
    pub cols: usize,
    pub rows: usize,
    pub bytes_per_pixel: usize,
    // pub data: Vec<Vec<Vec<u8>>>,
    pub pixels: Vec<u8>,
    pub size: usize,
}

impl Mat {
    pub fn new(w: usize, h: usize, color: Option<u8>)
        -> Mat
    {
        let color = color.unwrap_or_else(
            || 0u8
        );
        let mut data = Vec::<u8>::with_capacity(w*h*3);
        data.resize((w as usize) * (h as usize) * 3, color);
        Mat::load_from_vec(data, w, h, 3)
    }

    pub fn create(w: usize, h: usize, color: Vec<u8>)
        -> Mat
    {

        let mut data = Vec::<u8>::with_capacity(w*h*3);
        for i in 0..(w*h) {
            for chn in &color {
                data.push(*chn);
            }
        }
        Mat::load_from_vec(data, w, h, 3)
    }

    pub fn load_jpeg(path: &str)
        -> Mat
    {
        let file = File::open(path).expect("failed to open file");
        let mut decoder = Decoder::new(BufReader::new(file));
        let raw_pixels = decoder.decode().expect("failed to decode image");
        let metadata = decoder.info().unwrap();
        let bytes_per_pixel = match metadata.pixel_format {
            PixelFormat::L8     => 2,
            PixelFormat::RGB24  => 3,
            PixelFormat::CMYK32 => 4
        };
        
        Mat::load_from_vec(raw_pixels, metadata.width as usize, metadata.height as usize, bytes_per_pixel as usize)
    }

    pub fn save_as_bmp(&self, path: &str)
    {
        let mut bmp_image = bmp::Image::new(self.cols as u32, self.rows as u32);
        for y in 0..(self.rows) {
            for x in 0..(self.cols) {
                let pixel = self.get_pixel_by_xy(x, y);
                if self.bytes_per_pixel == 1 {
                    bmp_image.set_pixel(x as u32, y as u32, bmp::Pixel::new(pixel[0], pixel[0], pixel[0]));
                } else if self.bytes_per_pixel == 3 {
                    bmp_image.set_pixel(x as u32, y as u32, bmp::Pixel::new(pixel[0], pixel[1], pixel[2]));
                } else {
                    panic!("Image channels should be 1 or 3");
                }
                
            }
        }

        let _ = bmp_image.save(path).unwrap_or_else(|e| {
            panic!("Failed to save: {}", e)
        });
    }

    pub fn load_png(path: &str)
        -> Mat
    {
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let (output_info, mut reader) = decoder.read_info().unwrap();

        println!("{:?}", output_info.color_type);

        let bytes = reader.info().bytes_per_pixel();
        let mut buf = vec![0; output_info.buffer_size()];
        let (width, height) = reader.info().size();

        reader.next_frame(&mut buf).unwrap();
        Mat::load_from_vec(buf, width as usize, height as usize, bytes)
    }

    pub fn save_as_png(&self, path: &str)
    {
        use png::HasParameters;

        let path = Path::new(path);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.cols as u32, self.rows as u32); // Width is 2 pixels and height is 1.
        
        if self.bytes_per_pixel == 1 {
            // Save as grayscale picture
            encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
        } else if self.bytes_per_pixel == 2 {
            // Save as grayscale picture
            encoder.set(png::ColorType::GrayscaleAlpha).set(png::BitDepth::Eight);
        } else if self.bytes_per_pixel == 3 {
            // Save as RGB picture
            encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
        } else {
            // Save as RGBA picture
            encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        }
        let mut writer = encoder.write_header().unwrap();

        // An array containing a sequence.
        writer.write_image_data(&self.pixels).unwrap(); // Save
    }

    pub fn load_from_vec(raw: Vec<u8>, width: usize, height: usize, bytes_per_pixel: usize)
        -> Mat
    {
        let mut new_bytes_per_pixel = bytes_per_pixel;
        let mut new_data = raw;
        
        // println!("{:?}", new_bytes_per_pixel);

        if bytes_per_pixel == 4usize || bytes_per_pixel == 2usize {
            let mut vec = Vec::<u8>::with_capacity(width*height*(bytes_per_pixel-1));
            for i in 0..width*height {
                for p in 0..(bytes_per_pixel-1) {
                    vec.push(new_data[i*bytes_per_pixel+p]);
                }
            }
            new_data = vec;
            new_bytes_per_pixel = bytes_per_pixel - 1;
        }

        Mat {cols: width, rows: height, bytes_per_pixel: new_bytes_per_pixel, pixels: new_data, size: width*height}
    }

    pub fn crop(&self, x: usize, y: usize, width: usize, height: usize) -> Mat {
        let new_data = CL_INSTANCE.cl_crop(&self.pixels, self.cols as i32, x as i32, y as i32, width as i32, height as i32, self.bytes_per_pixel as i32).unwrap();
        Mat::load_from_vec(new_data, width, height, self.bytes_per_pixel)
    }


    // pub fn resize(&self, width: usize, height: usize)
    //     -> Mat
    // {
    //     let scale_x = (self.cols as f32)/(width as f32);
    //     let scale_y = (self.rows as f32)/(height as f32);
    //     let mut new_data = vec![vec![vec![0u8; 4]; width]; height];
    //     let src_data = &self.data;
    //     for y in 0..height {
    //         for x in 0..width {
    //             let src_x = ((x as f32)*scale_x).round() as usize;
    //             let src_y = ((y as f32)*scale_y).round() as usize;
    //             if src_x < self.cols as usize && src_y < self.rows as usize {
    //                 new_data[y][x] = src_data[src_y][src_x].to_vec();
    //             }
    //         }
    //     }
    //     let mat = Mat {cols: width as usize, rows: height as usize, bytes_per_pixel: self.bytes_per_pixel, data: new_data, size: height*width};
    //     mat
    // }

    pub fn merge(&mut self, other: &Mat, x: usize, y: usize) {
        for row in 0..other.rows {
            for col in 0..other.cols {
                let pixel = other.get_pixel_by_xy(col, row);
                // println!("{:?}", pixel);
                self.set_pixel_by_xy(col+x, row+y, pixel);
            }
        }
    }

    pub fn rectangle(&self) {

    }

    // TODO
    pub fn change_each_pixel(&mut self, closure: &Fn(usize, usize, Vec<u8>) -> Vec<u8>) {
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                let index = self.find_index(x, y);
                let pixel = self.get_pixel(index);
                let new_pixel = closure(x, y, pixel);
                self.set_pixel(index, new_pixel);
            }
        }
    }

    pub fn find_index(&self, x: usize, y: usize) -> usize {
        let result = y*self.cols + x;
        // if result > 33177600 {
        //     println!("错的时候 Y: {}, Cols: {}", y, self.cols);
        // }
        result
    }

    pub fn get_pixel(&self, index: usize) -> Vec<u8> {
        if index >= self.size {
            return vec![0u8; self.bytes_per_pixel];
        }
        self.pixels.get(index*self.bytes_per_pixel..(index*self.bytes_per_pixel+self.bytes_per_pixel)).unwrap().to_vec()
    }

    pub fn set_pixel(&mut self, index: usize, pixel: Vec<u8>) {
        if self.bytes_per_pixel !=  pixel.len() {
            panic!(format!("The pixel should contain {} U8, but there are only {}", self.bytes_per_pixel, pixel.len()))
        }

        for i in 0..pixel.len() {
            self.pixels[index*self.bytes_per_pixel+i] = pixel[i];
        }
    }

    pub fn each_pixel(&self, closure: &Fn(usize, usize, Vec<u8>)) {
       for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                let index = self.find_index(x, y);
                let pixel = self.get_pixel(index);
                closure(x, y, pixel);
            }
        }
    }

    pub fn get_channel(&self, channel_number: usize)
        -> Mat
    {
        let mut channel = self.clone();
        channel.bytes_per_pixel = 1;
        channel.change_each_pixel(
            &|_, _, pixel| {
                let mut new_vec = vec![0u8];
                new_vec[0] = pixel[channel_number];
                new_vec
            }
        );
        channel
    }

    pub fn to_gray(&self)
        -> Mat
    {
        let new_data = CL_INSTANCE.cl_to_gray(&self.pixels, self.bytes_per_pixel).unwrap();
        Mat::load_from_vec(new_data, self.cols, self.rows, 1)
    }

    pub fn convolute(&self, kernel: kernels::Kernel)
        // -> Vec<u8>
        -> Mat
    {
        let new_cols = self.cols as usize - kernel.size() + 1;
        let new_rows = self.rows as usize - kernel.size() + 1;
        let result_size = new_cols * new_rows;

        let mut result_pixels = Vec::<u8>::with_capacity(result_size);
        let all_pixels = self.to_gray().get_channel(0);
        let kernel_values = kernel.flatten();

        let pixels = &all_pixels.pixels;
        let mut unified_pixels = Vec::<f32>::with_capacity(pixels.len());
        
        for pixel in pixels {
            unified_pixels.push(*pixel as f32/255.0);
        }

        for i in 0..unified_pixels.len() {
            let indexes_result = kernel.indexes(i, self.cols as usize, unified_pixels.len());
            if indexes_result.0 {
                let mut point_result = 0f32;
                
                for i in 0..(kernel.elements()) {
                    let pixel_index = indexes_result.1[i];
                    point_result = unified_pixels[pixel_index] * kernel_values[i] + point_result;
                }
                
                let pixel = ((point_result/4.0)*255.0).abs();
                result_pixels.push(pixel as u8);
            }
        }
        Mat::load_from_vec(result_pixels, new_cols as usize, new_rows as usize, 1)
    }

    pub fn fast_search_features(&self, threshold: usize, mask: &(usize, usize, usize, usize), direction: Direction)
        -> Vec<PixelDescription>
    {   
        let mut descriptions = Vec::<PixelDescription>::new();
        let now = Instant::now();
        
        for y in (mask.1)..(mask.1+mask.3) {
            for x in (mask.0)..(mask.0+mask.2) {
                let (result, mut description) = PixelDescription::load_as_fast((x, y), self, threshold, &direction);
                if result {
                    descriptions.push(description);
                }
            }   
        }
        println!("");
        println!("Spend ms on generate descriptions: {:?}", now.elapsed().as_millis());

        let before_nms = Instant::now();
        descriptions = self.nms(&mut descriptions);
        println!("Spend ms on NMS:{}", before_nms.elapsed().as_millis());
        let len = descriptions.len();
        
        let before_calculate_pair = Instant::now();
        for i in 0..descriptions.len() {
            descriptions[i].calculate_pair(self, &direction);
        }
        println!("Spend ms on calculate pairs:{}", before_calculate_pair.elapsed().as_millis());

        println!("Feature points:{:?}", len);
        descriptions
    }

    // non maximum suppression(NMS)
    fn nms(&self, descriptions: &mut Vec<PixelDescription>)
        -> Vec<PixelDescription>
    {
        let window_size = 5;
        let r = window_size/2;
        println!("R = {:?}", r);

        let mut current_descriptions = Vec::<PixelDescription>::new();
        let len = descriptions.len();
        println!("Total descriptions {:?}", len);
        for desc_i in 0..len {
            for other_i in 0..len {
                if descriptions[other_i].coordinate != descriptions[desc_i].coordinate {
                    let xr = ((descriptions[desc_i].coordinate.0 as i32 - descriptions[other_i].coordinate.0 as i32)).abs();
                    let yr = ((descriptions[desc_i].coordinate.1 as i32 - descriptions[other_i].coordinate.1 as i32)).abs();
                    if (xr <= r) && (yr <= r) {
                        if descriptions[desc_i].maximum_value() >= descriptions[other_i].maximum_value() {
                            descriptions[other_i].remove();
                        }
                    }
                }
            }
        }

        for desc in descriptions {
            // println!("{:?}", desc.removed);
            if !(desc.removed) {
                current_descriptions.push(desc.clone());
            }
        }
        println!("Effective descriptions {:?}", current_descriptions.len());

        current_descriptions
    }

    pub fn draw_point(&mut self, coordinate: (usize, usize), color: Vec<u8>) {
        let mark = vec![(-3, 0),(-2, 0),(-1, 0),(3, 0),(2, 0),(1, 0),(0, -3),(0, -2),(0, -1),(0, 3),(0, 2),(0, 1)];
        for xy in mark {
            let y = coordinate.1 as i32 + xy.1;
            let x = coordinate.0 as i32 + xy.0;
            if x >= 0 || y >= 0 || x < self.cols as i32 || y < self.rows as i32 {
                self.set_pixel_by_xy(x as usize, y as usize, color.to_vec());
            }
        }
    }

    pub fn draw_line(&mut self, end1: (usize, usize), end2: (usize, usize), color:&mut Vec<u8>) {
        let distance = ((end1.0 as f32 - end2.0 as f32).powi(2) + (end1.1 as f32 - end2.1 as f32).powi(2)).sqrt().round() as f32;
        let sin = (end2.1 as f32-end1.1 as f32)/distance;
        let cos = (end2.0 as f32-end1.0 as f32)/distance;
        for d in 0..(distance as usize) {
            let x = (d as f32*cos).round() as usize;
            let y = (d as f32*sin).round() as usize;
            self.set_pixel_by_xy(x, y, color.to_vec());
        }
    }

    pub fn set_pixel_by_xy(&mut self, x: usize, y: usize, pixel: Vec<u8>) {
        if x < self.cols && y < self.rows {
            let index = self.find_index(x, y);
            self.set_pixel(index, pixel);   
        }
    }

    pub fn polarize(&self) -> Mat {
        let mut new_image = self.clone();
        new_image.change_each_pixel(&|_, _, vec| {
            if vec[0] > 10u8 {
                let mut new_value = vec[0] as u32 * 8;
                if new_value > 255 { new_value = 255; }
                return vec![new_value as u8, 255u8];
            } else {
                return vec![0u8, 255u8];
            }
        });
        new_image
    }

    // pub fn rotate(&self, degree: f32)
    //  -> Mat
    // {
        
    // }

    pub fn avg_mapping_vector(pairs: &Vec<(PixelDescription, PixelDescription)>) -> (f32, f32) {
        let mut x_move_total = 0.0;
        let mut y_move_total = 0.0;
        for pair in pairs {
            let x_move = pair.0.coordinate.0 as f32 - pair.1.coordinate.0 as f32;
            let y_move = pair.0.coordinate.1 as f32 - pair.1.coordinate.1 as f32;
            x_move_total += x_move;
            y_move_total += y_move;
        }
        (x_move_total/pairs.len() as f32, y_move_total/pairs.len() as f32)
    }

    pub fn move_mat(dist: &mut Mat, src: &Mat, vec: (f32, f32)) {
        for y in 0..src.rows {
            for x in 0..src.cols {
                let dist_x = (x as f32 + vec.0).round() as usize;
                let dist_y = (y as f32 + vec.1).round() as usize;

                if dist_x < dist.cols as usize && dist_y < dist.rows as usize {
                    // dist.data[dist_y][dist_x] = src.data[y as usize][x as usize].to_vec();
                    dist.set_pixel_by_xy(dist_x, dist_y, src.get_pixel_by_xy(x, y));
                }
            }
        }
    }

    pub fn get_vector(desc_a: &PixelDescription, desc_b: &PixelDescription) -> (f32, f32) {
        (desc_b.coordinate.0 as f32 - desc_a.coordinate.0 as f32, desc_b.coordinate.1 as f32 - desc_a.coordinate.1 as f32)
    }

    pub fn add_padding(&self, width: usize) -> Mat {
        let mut colors = Vec::<Vec<u8>>::new();
        for x in (0..self.cols) {
            colors.push(self.get_pixel_by_xy(x, 0));
            colors.push(self.get_pixel_by_xy(x, self.rows));
        }

        let mut avg_color = vec![0usize; self.bytes_per_pixel];
        for color in &colors {
            for (i, chn) in color.iter().enumerate() {
                avg_color[i] += *chn as usize;
            }
        }

        let mut new_color = vec![0u8; self.bytes_per_pixel];

        for (i, sum_chn) in avg_color.iter().enumerate() {
            new_color[i] = (avg_color[i]/colors.len()) as u8;
        }

        let mut new_image = Mat::create(self.cols + width*2, self.rows + width*2, new_color);
        new_image.merge(self, width, width);
        new_image
    }

    pub fn region_vector(x: usize, y: usize, pairs: &Vec<(PixelDescription, PixelDescription)>, direction: Direction) -> (f32, f32) {
        let mut left_pair: Option<&(PixelDescription, PixelDescription)> = None;

        for pair in pairs {
            if (left_pair.is_none() || (pair.0.coordinate.0 > left_pair.unwrap().0.coordinate.0)) && pair.0.coordinate.0 <= x {
                left_pair = Some(pair);
            }
        }
        
        let mut right_pair: Option<&(PixelDescription, PixelDescription)> = None;

        for pair in pairs {
            if (right_pair.is_none() || (pair.0.coordinate.0 < right_pair.unwrap().0.coordinate.0)) && pair.0.coordinate.0 > x {
                right_pair = Some(pair);
            }
        }

        if left_pair.is_none() {
            let pair = right_pair.unwrap();
            return Mat::get_vector(&pair.0, &pair.1);
        }

        if right_pair.is_none() {
            let pair = left_pair.unwrap();
            return Mat::get_vector(&pair.0, &pair.1);
        }

        let left_pair = left_pair.unwrap();
        let right_pair = right_pair.unwrap();

        let left_right_distance = right_pair.0.coordinate.0 - left_pair.0.coordinate.0;
        let left_distance = x - left_pair.0.coordinate.0;
        let right_distance = right_pair.0.coordinate.0 - x;
        let right_weight = left_distance as f32/left_right_distance  as f32;
        let left_weight = right_distance as f32/left_right_distance  as f32;
        
        let left_vector = Mat::get_vector(&left_pair.0, &left_pair.1);
        let right_vector = Mat::get_vector(&right_pair.0, &right_pair.1);

        return (left_vector.0*left_weight + right_vector.0*right_weight, left_vector.1*left_weight + right_vector.1*right_weight);

    }

    pub fn move_mat_by_multi_points(dist: &mut Mat, src: &Mat, avg_vector: (f32, f32), points: &Vec<(PixelDescription, PixelDescription)>) {
        for y in 0..src.rows {
            // println!("{:?}", y);
            for x in 0..src.cols {
                let dist_x = x as f32 + avg_vector.0;
                let dist_y = y as f32 + avg_vector.1;

                if dist_x >= 0.0f32 && dist_x < dist.cols as f32 && dist_y >= 0.0f32 && dist_y < dist.rows as f32 {
                    let dist_x = dist_x.round() as usize;
                    let dist_y = dist_y.round() as usize;

                    let vec = Mat::region_vector(dist_x, dist_y, points, Direction::Vertical);
                    // println!("{:?}", vec);
                    let src_x = (dist_x as f32 + vec.0).round() as i32;
                    let src_y = (dist_y as f32 + vec.1).round() as i32;

                    if src_x >= 0 && src_y >= 0 {
                        dist.set_pixel_by_xy(dist_x, dist_y, src.get_pixel_by_xy(src_x as usize, src_y as usize));
                    }
                }
            }
        }
    }

    

    pub fn get_pixel_by_xy(&self, x: usize, y: usize) -> Vec<u8> {
        let index = self.find_index(x, y);
        self.get_pixel(index)
    }

    pub fn elements(&self) -> u32 {
        self.rows as u32 * self.cols as u32
    }

}

pub trait VecOperators<T> {
    fn add(&self, other: T) -> T;
    fn times(&self, factor: f32) -> T;
}

impl VecOperators<Vec<u8>> for Vec<u8> {
    fn add(&self, other: Vec<u8>) -> Vec<u8> {
        let mut new_vec = Vec::with_capacity(self.len());
        for i in 0..self.len() { 
            new_vec.push(((self[i] as f32 + other[i] as f32)/2.0).round() as u8);
        }
        new_vec
    }

    fn times(&self, factor: f32) -> Vec<u8> {
        let mut new_vec = Vec::with_capacity(self.len());
        for i in 0..self.len() { 
            new_vec.push((self[i] as f32 * factor).round() as u8);
        }
        new_vec
    }
}