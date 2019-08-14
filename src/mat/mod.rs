use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::io::BufWriter;
use std::num::Wrapping;

extern crate bmp;
extern crate png;
use jpeg_decoder::Decoder;
use jpeg_decoder::PixelFormat;

use std::time::{Instant};

pub mod kernels;
pub mod pixel_description;
pub mod transform;

use pixel_description::PixelDescription;
use pixel_description::Direction;

#[derive(Debug, Clone)]
pub struct Mat {
    pub cols: u16,
    pub rows: u16,
    pub bytes_per_pixel: usize,
    pub data: Vec<Vec<Vec<u8>>>
}

impl Mat {
    pub fn new(w: u16, h: u16, color: Option<u8>)
        -> Mat
    {
        let color = color.unwrap_or_else(
            || 0u8
        );
        let mut data = Vec::<u8>::new();
        data.resize((w as usize) * (h as usize) * 4, color);
        Mat::load_from_vec(data, w, h, 4)
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
        
        Mat::load_from_vec(raw_pixels, metadata.width as u16, metadata.height as u16, bytes_per_pixel as usize)
    }

    pub fn save_as_bmp(&self, path: &str)
    {
        let mut bmp_image = bmp::Image::new(self.cols as u32, self.rows as u32);
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                bmp_image.set_pixel(x as u32, y as u32, bmp::Pixel::new(self.data[y][x][0], self.data[y][x][1], self.data[y][x][2]))
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
        let bytes = reader.info().bytes_per_pixel();
        let mut buf = vec![0; output_info.buffer_size()];
        let (width, height) = reader.info().size();

        reader.next_frame(&mut buf).unwrap();
        Mat::load_from_vec(buf, width as u16, height as u16, bytes)
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
        let data: &[u8] = &self.flatten();
        // An array containing a sequence.
        writer.write_image_data(data).unwrap(); // Save
    }

    pub fn load_from_vec(raw: Vec<u8>, width: u16, height: u16, bytes_per_pixel: usize)
        -> Mat
    {
        let mut data = Vec::<Vec<Vec<u8>>>::new();
        
        let mut new_bytes_per_pixel = 2;

        if bytes_per_pixel == 3 {
            new_bytes_per_pixel = 4;
            data = vec![vec![vec![0u8, 0u8, 0u8, 255u8]; width as usize]; height as usize];
        }

        if bytes_per_pixel == 1 {
            new_bytes_per_pixel = 2;
            data = vec![vec![vec![0u8, 255u8]; width as usize]; height as usize];
        }
        for i in 0..raw.len() {
            let offset = i%(width as usize * height as usize)%bytes_per_pixel;
            let x = i/bytes_per_pixel%width as usize;
            let y = i/bytes_per_pixel/width as usize;
            data[y][x][offset] = raw[i];
        }
        Mat {cols: width, rows: height, bytes_per_pixel: new_bytes_per_pixel, data: data}
    }
    
    pub fn crop(&self, x: usize, y: usize, width: usize, height: usize)
        -> Mat
    {
        let mut new_data = Vec::new();
        for row in (y)..(y+height) {
            new_data.push((&self.data[row])[x..x+width].to_vec());
        }
        let mat = Mat {cols: width as u16, rows: height as u16, bytes_per_pixel: self.bytes_per_pixel, data: new_data};
        mat
    }

    pub fn resize(&self, width: usize, height: usize)
        -> Mat
    {
        let scale_x = (self.cols as f32)/(width as f32);
        let scale_y = (self.rows as f32)/(height as f32);
        let mut new_data = vec![vec![vec![0u8; 4]; width]; height];
        let src_data = &self.data;
        for y in 0..height {
            for x in 0..width {
                let src_x = ((x as f32)*scale_x).round() as usize;
                let src_y = ((y as f32)*scale_y).round() as usize;
                if src_x < self.cols as usize && src_y < self.rows as usize {
                    new_data[y][x] = src_data[src_y][src_x].to_vec();
                }
            }
        }
        let mat = Mat {cols: width as u16, rows: height as u16, bytes_per_pixel: self.bytes_per_pixel, data: new_data};
        mat
    }

    pub fn merge(&self, other: Mat, x: u16, y: u16)
        -> Mat
    {
        let mut new_image = self.clone();
        for row in 0..other.rows {
            for col in 0..other.cols {
                new_image.data[(row+y) as usize][(col+x) as usize] = other.data[row as usize][col as usize].to_vec();
            }
        }
        new_image
    }

    pub fn rectangle(&self) {

    }

    pub fn change_each_pixel(&mut self, closure: &Fn(u16, u16, Vec<u8>) -> Vec<u8>) {
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                let new_pixel = closure(x as u16, y as u16, self.data[y][x].to_vec());
                self.data[y][x] = new_pixel;
            }
        }
    }

    pub fn each_pixel(&self, closure: &Fn(u16, u16, Vec<u8>)) {
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                closure(x as u16, y as u16, self.data[y][x].to_vec());
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
        let mut gray = self.clone();
        gray.change_each_pixel(&|_, _, vec| {
            if self.bytes_per_pixel > 2 {
                return vec![((vec[0] as f32 * 299f32 + vec[1] as f32 * 587f32 + vec[2] as f32) / 1000f32) as u8, 255u8];
            } else {
                return vec
            }
        });
        gray.bytes_per_pixel = 2;
        gray
    }

    pub fn flatten(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(self.rows as usize * self.cols as usize);
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                for value in self.data[y][x].to_vec() {
                    pixels.push(value);
                }
            }
        }
        pixels
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

        let pixels = all_pixels.flatten();
        let mut unified_pixels = Vec::<f32>::with_capacity(pixels.len());
        
        for pixel in pixels {
            unified_pixels.push(pixel as f32/255.0);
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
        Mat::load_from_vec(result_pixels, new_cols as u16, new_rows as u16, 1)
    }

    pub fn fast_search_features(&self, threshold: usize, mask: &(u16, u16, u16, u16), direction: Direction)
        -> Vec<PixelDescription>
    {   
        let mut descriptions = Vec::<PixelDescription>::new();
        let now = Instant::now();
        
        for y in (mask.1)..(mask.1+mask.3) {
            for x in (mask.0)..(mask.0+mask.2) {
                let (result, description) = PixelDescription::load_as_fast((x, y), &self.data, threshold, &direction);
                if result {
                    descriptions.push(description);
                }
            }   
        }

        println!("Spend ms on generate descriptions: {:?}", now.elapsed().as_millis());

        let before_nms = Instant::now();
        descriptions = self.nms(&mut descriptions);
        println!("Spend ms on NMS:{}", before_nms.elapsed().as_millis());
        let len = descriptions.len();
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
        println!("{:?}", len);
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
        current_descriptions
    }

    pub fn draw_point(&mut self, coordinate: (u16, u16), color:&mut Vec<u8>) {
        let pixel = &self.data[coordinate.1 as usize][coordinate.0 as usize];
        color.resize(pixel.len(), 255u8);
        let mark = vec![(-3, 0),(-2, 0),(-1, 0),(3, 0),(2, 0),(1, 0),(0, -3),(0, -2),(0, -1),(0, 3),(0, 2),(0, 1)];
        for xy in mark {
            let y = coordinate.1 as i32 + xy.1;
            let x = coordinate.0 as i32 + xy.0;
            if x >= 0 || y >= 0 || x < self.cols as i32 || y < self.rows as i32 {
                self.data[y as usize][x as usize] = color.to_vec();
            }
        }
    }

    pub fn draw_line(&mut self, end1: (usize, usize), end2: (usize, usize), color:&mut Vec<u8>) {
        let distance = ((end1.0 as f32 - end2.0 as f32).powi(2) + (end1.1 as f32 - end2.1 as f32).powi(2)).sqrt().round() as f32;
        let sin = (end2.1 as f32-end1.1 as f32)/distance;
        let cos = (end2.0 as f32-end1.0 as f32)/distance;
        for d in 0..(distance as usize) {
            let x = (d as f32*cos).round() as u16;
            let y = (d as f32*sin).round() as u16;
            self.data[(y+(end1.1 as u16)) as usize][(x+(end1.0 as u16)) as usize] = color.to_vec();
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

    pub fn print(&self) {
        println!("{:?}", self.data);
    }

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
                    dist.data[dist_y][dist_x] = src.data[y as usize][x as usize].to_vec();
                }
            }
        }
    }

    pub fn standard_deviation(&self) -> f32 {
        let avg = self.avg();
        let mut total = 0.0;
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                let value = (self.data[y][x][0] as f32 - avg).powi(2);
                total += value;
            }
        }
        (total/self.elements() as f32).sqrt()
    }

    pub fn elements(&self) -> u32 {
        self.rows as u32 * self.cols as u32
    }

    pub fn avg(&self) -> f32 {
        let mut total = 0.0;
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                let value = self.data[y][x][0] as f32;
                total += value;
            }
        }
        total/(self.rows as f32*self.cols as f32)
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