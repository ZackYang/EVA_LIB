use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::io::BufWriter;

extern crate bmp;
extern crate png;
use jpeg_decoder::Decoder;
use jpeg_decoder::PixelFormat;

#[derive(Debug)]
pub struct Mat {
    cols: u16,
    rows: u16,
    bytes_per_pixel: usize,
    data: Vec<Vec<Vec<u8>>>
}

impl Mat {
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
        // println!("{:?}", self.data[0]);
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
        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let data = [255, 0, 0, 255, 0, 0, 0, 255]; // An array containing a RGBA sequence. First pixel is red and second pixel is black.
        writer.write_image_data(&data).unwrap(); // Save
    }

    pub fn load_from_vec(raw: Vec<u8>, width: u16, height: u16, bytes_per_pixel: usize)
        -> Mat
    {
        let mut pixels = Vec::new();
        println!("{:?}", bytes_per_pixel);
        for chunk in raw.chunks(bytes_per_pixel) {
            pixels.push(chunk.to_vec());
        }

        let mut data = Vec::new();
        for chunk in pixels.chunks(width as usize) {
            data.push(chunk.to_vec());
        }
        let mat = Mat {cols: width, rows: height, bytes_per_pixel: bytes_per_pixel, data: data};
        mat
    }
    
    pub fn crop(&self, x: usize, y: usize, width: usize, height: usize)
        -> Mat
    {
        let mut new_data = Vec::new();
        for row in (y-1)..(y-1+height) {
            new_data.push((&self.data[row])[x-1..x-1+width].to_vec());
        }
        let mat = Mat {cols: width as u16, rows: height as u16, bytes_per_pixel: self.bytes_per_pixel, data: new_data};
        mat
    }

    pub fn resize(&self, width: usize, height: usize)
        -> Mat
    {
        let scale_x = (self.cols as f32)/(width as f32);
        let scale_y = (self.rows as f32)/(height as f32);
        let mut new_data = vec![vec![vec![0u8; 3]; width]; height];
        let src_data = &self.data;
        for y in 0..height {
            for x in 0..width {
                let src_x = ((x as f32)*scale_x).round() as usize;
                let src_y = ((y as f32)*scale_y).round() as usize;
                new_data[y][x] = src_data[src_y][src_x].to_vec();
            }
        }
        let mat = Mat {cols: width as u16, rows: height as u16, bytes_per_pixel: self.bytes_per_pixel, data: new_data};
        mat
    }

    pub fn clone(&self)
        -> Mat
    {
        self.crop(1, 1, self.cols as usize, self.rows as usize)
    }

    pub fn merge(&self, other: Mat)
        -> Mat
    {
        let mut new_data = vec![vec![vec![0u8; 3]; self.cols as usize]; self.rows as usize];

        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                new_data[y][x] = self.data[y][x].add(other.data[y][x].to_vec());               
            }
        }
        Mat {cols: self.cols as u16, rows: self.rows as u16, bytes_per_pixel: self.bytes_per_pixel, data: new_data}
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

    pub fn each_pixel(&mut self, closure: &Fn(u16, u16, Vec<u8>)) {
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
        channel.change_each_pixel(
            &|x, y, pixel| {
                let mut new_vec = vec![0u8, 0u8, 0u8];
                new_vec[channel_number] = pixel[channel_number];
                new_vec
            }
        );
        channel
    }

    pub fn print(&self) {
        println!("{:?}", self.data);
    }
}

pub trait VecOperators<T> {
    fn add(&self, other: T) -> T; 
}

impl VecOperators<Vec<u8>> for Vec<u8> {
    fn add(&self, other: Vec<u8>) -> Vec<u8> {
        let mut new_vec = Vec::with_capacity(self.len());
        for i in 0..self.len() { 
            new_vec.push(((self[i] as f32 + other[i] as f32)/2.0).round() as u8);
        }
        new_vec
    }
}