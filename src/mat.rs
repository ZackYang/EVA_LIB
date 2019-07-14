use std::fs::File;
use std::io::BufReader;

extern crate bmp;
use jpeg_decoder::Decoder;
use jpeg_decoder::PixelFormat;

pub struct Mat {
    cols: u16,
    rows: u16,
    format: PixelFormat,
    data: Vec<Vec<Vec<u8>>>
}

impl Mat {
    pub fn load(path: &str)
        -> Mat
    {
        let file = File::open(path).expect("failed to open file");
        let mut decoder = Decoder::new(BufReader::new(file));
        let raw_pixels = decoder.decode().expect("failed to decode image");
        let metadata = decoder.info().unwrap();

        let mut pixels = Vec::new();

        for chunk in raw_pixels.chunks(3) {
            pixels.push(chunk.to_vec());
        }

        let mut data = Vec::new();
        for chunk in pixels.chunks(metadata.width as usize) {
            data.push(chunk.to_vec());
        }

        let mat = Mat {cols: metadata.width, rows: metadata.height, format: metadata.pixel_format, data: data};
        mat
    }

    pub fn save(&self, path: &str)
    {

        let mut bmp_image = bmp::Image::new(self.cols as u32, self.rows as u32);
        for y in 0..(self.rows as usize) {
            for x in 0..(self.cols as usize) {
                bmp_image.set_pixel(x as u32, y as u32, bmp::Pixel::new(self.data[y][x][0], self.data[y][x][1], self.data[y][x][2]))
            }
        }

        // println!("{:?}", flatten_array);
        let _ = bmp_image.save(path).unwrap_or_else(|e| {
            panic!("Failed to save: {}", e)
        });
    }

    pub fn crop(&self, x: usize, y: usize, width: usize, height: usize)
        -> Mat
    {
        let mut new_data = Vec::new();
        for row in (y-1)..(y-1+height) {
            new_data.push((&self.data[row])[x-1..x-1+width].to_vec());
        }
        let mat = Mat {cols: width as u16, rows: height as u16, format: self.format, data: new_data};
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
        let mat = Mat {cols: width as u16, rows: height as u16, format: self.format, data: new_data};
        mat
    }

    pub fn print(&self) {
        println!("{:?}", self.data);
    }
}