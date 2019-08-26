use std::time::{Duration, Instant};
use std::thread::sleep;
use super::Mat;

pub enum Direction {
    Horizontal,
    Vertical
}

pub struct PixelDescription {
    pub coordinate: (usize, usize),
    pub description: Vec<i16>,
    pub value: u8,
    pub removed: bool,
    pub feature_pairs: Vec<u8>
}

impl PixelDescription {
    pub fn new() -> PixelDescription {
        PixelDescription { coordinate: (0, 0), description: Vec::new(), value: 0, removed: true, feature_pairs: Vec::<u8>::new() }
    }

    pub fn load_as_fast(coordinate: (usize, usize), src: &Mat, threshold: usize, direction: &Direction)
        -> (bool, PixelDescription)
    {
        let c = coordinate;
        let x = c.0 as i16;
        let y = c.1 as i16;
        let value = src.get_pixel_by_xy(x as usize, y as usize)[0];
        // src[c.1 as usize][c.0 as usize][0];

        let feature_points = vec![
            (x, y-3),
            (x+1, y-3),
            (x+2, y-2),
            (x+3, y-1),
            (x+3, y),
            (x+3, y+1),
            (x+2, y+2),
            (x+1, y+3),
            (x, y+3),
            (x-1, y+3),
            (x-2, y+2),
            (x-3, y+1),
            (x-3, y+0),
            (x-3, y-1),
            (x-2, y-2),
            (x-1, y-3),
            // repeat
            (x, y-3),
            (x+1, y-3),
            (x+2, y-2),
            (x+3, y-1),
            (x+3, y),
            (x+3, y+1),
            (x+2, y+2),
            (x+1, y+3),
            (x, y+3)
        ];

        let fast_feature_points = vec![
            (x, y-3),
            (x+3, y),
            (x, y+3),
            (x-3, y)
        ];


        let mut total = 0;
        for coor in fast_feature_points {
            let x = coor.0;
            let y = coor.1;
            if x < 0 || y < 0 || y as usize >= src.rows || x as usize >= src.cols {
                return (false, PixelDescription::new());
            }
            let coor_value = src.get_pixel_by_xy(x as usize, y as usize)[0];
            let value = (coor_value as i16 - value as i16).abs() as usize;
            if value > threshold {
                total += 1;
            }
        }

        if total < 4 {
            return (false, PixelDescription::new());
        }

        let mut max_hits = 0usize;
        let mut current_hits = 0usize;

        let mut description_values = Vec::<i16>::with_capacity(16);

        for coor in feature_points {
            let x = coor.0;
            let y = coor.1;
            if x < 0 || y < 0 || y as usize >= src.rows || x as usize >= src.cols {
                return (false, PixelDescription::new());
            }
            let coor_value = src.get_pixel_by_xy(x as usize, y as usize)[0];

            let description_value = coor_value as i16 - value as i16;
            if description_values.len() < 16 {
                description_values.push(description_value);
            }

            if description_value.abs() as usize > threshold {
                current_hits += 1;
            } else {
                current_hits = 0;
            }
            if current_hits > max_hits {
                max_hits = current_hits;
            }
        }

        if max_hits >= 10 {
            // let feature_pairs = PixelDescription::calculate_pair((x as usize, y as usize), src, &direction);
            return (true, PixelDescription {
                coordinate: coordinate,
                description: description_values,
                feature_pairs: vec![0u8],
                value: value,
                removed: false
            });
        }
        return (false, PixelDescription::new())
    }

    pub fn calculate_pair(&mut self, src: &Mat, direction: &Direction) {
        let coordinate = self.coordinate;
        let a_x = [
            -11,-10,-8,-8,12,-12,-15,0,-13,-4,-1,6,8,1,-12,0,-2,-12,10,-15,-4,13,1,-15,-12,-10,1,-9,-15,12,0,-1,-7,7,13,-11,-6,6,14,-2,-14,-2,-3,-8,10,10,12,7,6,-5,13,-2,-1,-12,8,-5,14,-10,-14,11,2,8,-13,-7,-7,-13,-12,9,-6,-15,3,7,9,7,8,8,-4,15,-8,-9,-4,0,10,1,-12,0,15,13,-9,0,-5,-11,9,3,0,2,-12,-15,-7,7,12,-11,-12,14,-8,-14,11,9,-1,7,7,-7,15,-8,14,5,5,4,1,1,11,1,-4,-4,-11,-13,6,-9,-14,-5,12,4,-8,-3,-11,12,-6,-3,15,14,5,-15,15,3,3,-10,-15,13,11,-11,10,15,3,4,11,4,7,-3,13,-15,7,9,2,3,2,-2,-4,-14,3,-10,-11,0,-1,-10,6,-9,5,2,12,7,9,-2,1,-13,5,-9,-9,4,-4,-1,-11,-4,8,2,-15,-3,2,-5,15,-4,-3,-9,-8,-6,-7,6,-10,11,6,5,-8,10,6,13,6,11,-14,-4,2,-9,-11,13,-7,8,-12,-14,5,6,10,-2,2,1,-5,2,-2,1,-12,-13,4,-3,-13,-10,12,7,10,14,-6,12,9,7,-9,8,5,6,-4,7,-8,-8,-8,-6,-15,7,-7,0,12,-1,-15,4,10,5,-6,-11,8,-13,-7,-11,-5,-14,12,6,-14,4,-2,-13,0,8,0,-9,5,-4,-4,-13,2,-14,10,10,-13,-15,1,-8,11,8,8,-3,-7,2,-6,-7,-12,13,-1,-11,-13,-4,14,6,2,-3,-15,1,7,-5,5,7,-9,2,13,11,-7,-11,-12,6,-1,5,5,-6,-15,-13,-3,-8,-7,7,-8,-13,-8,1,-5,13,1,3,0,4,-4,-15,-8,10,-1,14,-6,13,-15,-11,15,6,-6,-11,-9,6,-4,-6,-3,-14,7,-13,-13,8,11,-3,14,-2,11,-15,3,0,9,5,-3,-4,-15,6,6,11,-14,-11,0,12,13,-10,12,-3,-2,4,-2,7,-4,-3,-9,9,-6,0,4,-12,-6,8,-7,9,-3,7,-8,8,4,4,12,-10,8,-1,-3,7,14,-2,-10,2,10,1,5,15,-12,13,-13,14,-15,-9,0,-4,5,-3,12,12,6,4,10,12,-9,12,-11,-4,-10,-5,-13,12,-3,-5,9,5,10,8,-4,-6,-14,-12,-10,-5,0,1,11,0,0,3,-10,2,11,4,-8,14,5,5,6,8,-3,12,4,-7,-6,-8,-3,-9,8,-6,-11,-15,14,3,7,-6,0,-15,8,6,11,-2,11,-10,11,-9,0,14,14,-10,9,4,2,5,10,-11,-8,-1,12,9,-15,14,5,-4,-15,-14,-15,5,13,15,-9,14,-14,-15,0,-14,6,-7,-6,4,11,-11,-12,-3,15,-12,14,6,13,1,14,11,3,2,-12,3,-3,-3,-14,-9,-12,11,14,-14,5,-14,-2,-1,-9,-15,13,6,-2,-10,0,6,4,-13,11,4,12,-5,-10,10,-12,-3,-4,-6,10,-3,3,-1,-14,-4,8,15,-15,-8,11,5,0,4,-13,13,8,15,8,-10,-12,9,4,-14,-15,-2,-3,4,-4,14,-3,-8,15,-12,1,-11,-1,13,1,6,-12,-12,0,6,-8,-15,7,-6,-14,-7,-4,-1,8,15,14,-8,1,-6,2,-11,-3,-5,-1,8,-5,-15,-4,-3,-2,-14,-1,-14,13,12,-8,-10,-5,-13,-4,10,8,8,-7,1,-13,-5,15,-8,1,-12,5,-8,-11,-15,8,-4,0,-9,-14,-5,-11,6,12,3,4,10,6,5,4,8,2,11,-12,8,12,3,-13,-10,-15,15,-3,-9,10,-1,-6,0,-15,9,-6,-3,2,6,2,14,-2,6,10,0,-10,-2,-4,5,-15,6,-7,-7,13,-12,-12,5,13,9,-2,-8,-12,-14,6,15,3,15,-10,10,15,7,-13,10,-9,13,-6,-6,4,-8,5,-6,6,13,9,0,4,3,-7,-2,-4,-3,13,-1,-10,13,13,-7,7,4,-10,-13,-14,-12,15,15,3,0,-4,-11,14,-3,-9,-14,6,-7,-15,1,7,6,-12,5,2,5,13,-8,-7,-7,1,-15,-2,11,0,9,-13,2,2,-15,4,-5,-12,9,-11,9,-9,10,-1,-3,-11,-14,7,-7,6,11,-11,5,-5,6,9,5,-15,10,-2,4,12,-8,2,7,-7,-2,4,-6,-4,14,8,10,-9,-14,4,15,5,4,5,-12,9,-4,-10,-2,-11,6,-3,-10,-15,6,13,5,-10,5,6,-12,-8,-4,0,11,2,0,10,14,-8,13,8,-1,11,-11,-10,13,-1,-13,-12,8,0,-8,1,10,5,3,-3,-9,8,-1,15,10,-8,-1,1,3,-4,13,-5,-8,15,6,2,-14,-2,-5,10,-8,8,1,-8,-13,12,5,-8,-10,6,11,9,-7,-3,6,13,4,-13,0,4,-11,4,-7,-6,0,0,10,-13,3,14,13,13,-14,-3,-15,-13,4,5,-8,-7,12,9,-11,12,3,5,5,-6,-1,3,3,10,7,0,-5,-5,14,0,-3,3,12,11,-5,-2,15,15,14,15,-4,1,5,-13,-11,4,-11,-8,15,9,-8,4,11,3,5,-9,-13,-4,-15,-1,1
        ].to_vec();

        let a_y = [
            26,-18,13,29,18,8,5,6,-10,-11,22,8,29,-26,-18,30,30,-13,-1,5,14,26,-23,8,1,15,18,-27,-3,-15,-5,30,-5,-20,-25,-11,7,7,10,-2,11,17,-22,20,-14,19,-21,-19,30,-3,25,-6,1,-14,30,23,-30,20,11,-21,4,11,15,10,-1,26,-19,-5,11,-27,13,-1,16,24,14,3,27,-23,-10,26,-21,-17,-9,-27,-14,-7,-30,17,22,-22,3,-19,28,15,-5,13,19,24,12,-3,-11,17,-27,-24,6,8,25,6,-10,13,14,-2,-2,15,15,7,7,3,8,19,-3,30,-12,12,-25,-29,12,13,6,2,-25,12,9,-3,6,27,-11,8,19,-5,-21,0,25,-9,-18,24,15,-15,2,-23,-8,-23,4,7,-13,-24,-23,-21,-27,25,4,-24,-28,17,-17,4,21,-24,-27,17,-6,19,-22,-9,27,10,-6,15,-9,12,12,15,25,-4,-27,-16,-10,1,-23,-5,13,22,-13,-27,4,17,13,-10,-2,26,9,-16,-14,-19,23,-20,25,20,1,14,19,6,14,22,21,0,18,19,-8,6,15,-10,23,6,6,1,-19,28,20,-1,-12,-9,4,25,14,25,-21,-27,-21,-26,-5,-19,19,12,16,0,19,16,-23,-19,-10,6,30,10,2,27,13,-24,-14,28,-20,-24,13,10,27,-12,-19,-7,21,25,20,1,-5,15,19,23,-2,-13,-18,25,20,21,-28,23,14,-28,9,17,10,13,-6,25,11,13,-3,3,-6,5,30,26,-6,27,26,-29,21,6,-18,-14,19,-11,17,-28,-30,-17,22,13,10,-22,-7,-4,-3,-22,4,7,5,-22,-20,20,-28,0,-26,-14,-8,12,-7,-11,-11,-8,3,-3,-19,11,-29,13,-25,13,-28,-25,-18,-9,11,12,-15,-22,9,-15,-25,-26,6,19,10,17,-9,-14,14,30,5,-29,1,-12,-21,-10,-14,11,12,17,0,25,25,-29,-26,-29,15,11,-28,-21,22,26,-18,13,28,14,11,14,24,6,16,16,17,9,-27,25,-29,22,-15,-24,7,0,-24,15,16,11,22,-6,-8,-28,-10,-29,-9,9,6,-23,-21,-11,-6,11,-10,-23,-6,10,13,-16,3,-18,-4,6,-16,0,14,-18,-11,26,-7,18,28,-13,-4,-2,-23,15,-21,-11,9,-26,-25,0,-8,-30,22,1,-21,0,-10,-4,-8,23,7,2,-28,13,1,8,-12,-17,-6,-17,-11,-7,-13,-25,-29,-17,30,-17,-7,-4,-19,-14,-5,17,5,4,-8,19,2,20,-3,-12,-13,-30,-10,21,29,-22,-6,20,-19,-16,-19,8,8,-18,11,-19,-2,14,26,-4,-8,-17,20,-14,7,-28,14,26,5,-10,-2,4,5,9,-17,8,1,14,-14,-30,25,26,3,7,4,5,-27,-5,23,-7,-10,26,19,-19,0,18,28,-16,-29,-2,16,28,-30,2,18,27,19,-28,-9,-30,-23,20,-6,24,-10,4,-26,8,24,29,-17,3,-6,-17,-30,10,-13,0,17,20,1,9,-29,22,-8,-5,-5,-18,-20,-30,3,-5,-26,5,18,24,-19,1,3,-22,12,-4,12,-14,-30,-17,15,-14,-23,-28,-11,-21,17,13,5,-29,-30,30,-4,21,-8,1,-12,11,7,15,-3,9,4,-23,5,8,-25,-19,15,-16,-21,13,-30,3,-11,24,5,14,18,-22,-25,16,0,20,15,19,29,-23,11,10,-23,-27,-7,-15,-13,-16,-2,4,28,-6,3,4,-23,-20,15,18,27,20,13,4,12,16,-4,4,21,-10,30,28,21,29,-2,-15,6,-14,4,5,19,-12,-3,-9,-19,21,19,-28,-7,23,-7,-19,-10,-21,-26,-5,-19,-22,11,8,-23,-17,-2,29,-23,0,-24,-12,-9,-28,-4,24,23,-19,10,-29,-26,4,-17,8,17,-19,-1,8,18,11,-8,30,29,-25,27,-2,7,-19,-19,7,25,-19,12,26,-29,-24,17,28,-2,21,13,-27,-6,20,0,9,26,9,1,-4,-1,-15,7,-15,18,-18,11,11,-11,1,23,6,-17,-21,-29,3,24,-8,22,24,-16,26,10,4,-4,8,-24,21,20,18,19,-14,1,15,-24,28,-5,15,9,24,-10,11,5,9,2,-21,-2,28,-13,27,11,27,1,-3,-1,-27,-6,8,14,2,4,15,14,28,8,14,-24,-27,-13,30,-1,-15,27,2,26,17,27,3,-2,-24,29,-24,-7,-14,26,-10,-10,30,21,17,12,-9,-5,22,5,-11,-9,-18,-28,-5,11,2,-19,-1,0,-4,-12,-20,30,23,-14,-8,-11,-15,-25,-25,-6,-6,-9,-11,20,-24,18,-22,-14,-22,18,3,25,-8,25,20,25,30,16,10,3,-3,1,24,2,29,-19,-7,0,-13,-17,24,-27,25,-7,23,24,-18,-23,3,-24,24,-21,9,-6,4,0,-3,29,-4,-14,-25,-26,6,-25,5,12,7,-1,25,-24,-13,-10,17,18,-18,18,3,10,-18,26,4,8,-18,29,12,5,-11,-22,21,8,-2,15,20,24,-28,9,30,20,18,-13,23,18,-14,20,-9,17,-22,11,-9,-11,-4,12,14,11,0,-14,-17,8,8,-10,-9,14,0,3,-23,-26,10,9,-4,-21,-20,-16,-6,-5,17,-9,0,-2,-30,-27,-10
        ].to_vec();

        let b_x = [
            -3,-8,14,7,-13,-14,-7,4,5,-7,-11,6,-5,-12,6,7,-12,7,3,7,2,11,-10,-5,3,-11,-10,-10,-10,4,-10,3,-15,-15,-13,3,-2,-4,11,-13,-7,-5,2,-9,-11,0,-3,7,5,-9,-14,10,-14,-2,-4,-13,-15,7,-6,-15,3,-9,2,15,-15,11,3,-10,-5,14,10,-1,5,-13,-1,-10,10,1,0,0,6,1,4,-1,13,-11,14,-9,10,13,11,-1,-2,7,1,-5,-6,-5,-11,-11,-7,0,-9,-5,1,-11,-3,-14,-13,-2,-8,4,10,-11,0,3,13,-12,4,-7,-4,-7,-8,5,-6,-11,14,13,2,-13,15,-11,-13,7,-9,-2,12,10,14,7,-10,0,14,7,-14,-10,0,1,1,-13,2,-1,-12,3,2,-13,-12,4,-10,-15,12,0,-12,7,3,-15,-11,7,-4,-3,-1,-2,15,8,-3,2,12,13,-3,15,-7,2,1,15,13,-4,11,-5,12,14,1,-12,5,9,5,0,-7,12,9,-10,1,-4,-10,5,-15,14,4,2,-2,-2,-13,-2,3,9,-12,-9,-15,-4,-9,-1,10,-5,0,-10,1,-2,2,-10,-15,-9,-2,4,12,-9,-8,8,3,-11,10,-10,12,4,-14,-6,-13,-5,8,-1,-9,-15,-12,14,15,-12,-5,9,14,12,4,12,15,9,7,-4,-3,-11,-7,-12,9,8,13,0,10,11,-14,4,13,-7,3,-15,-11,-14,-3,-9,-13,-13,13,15,-1,4,-11,-12,-12,-2,15,-4,-3,10,-13,2,14,-14,-4,-1,-1,4,1,-15,0,11,5,8,1,1,0,-5,-2,7,8,-14,-15,9,-1,4,-11,-11,-4,7,-4,-1,-6,-9,-14,-14,-6,-14,9,-9,-10,-10,14,1,-7,13,5,-6,-6,-10,-3,9,-1,13,15,5,15,-6,5,-12,7,15,-5,1,2,-2,14,-5,-14,-3,7,-2,13,2,-5,-5,-10,-14,-9,5,10,-7,5,6,15,2,13,3,1,-7,-4,12,-7,3,-6,-2,-7,-12,-1,5,5,0,-6,-8,-14,2,-13,6,-2,8,15,5,-8,-9,6,-13,5,-4,-5,12,-5,-14,-4,13,-10,-6,8,14,12,-9,-6,-12,-4,-15,-10,5,10,-7,4,14,-1,12,2,-10,-11,-8,-3,6,-15,-11,-5,-10,9,14,15,15,-1,-4,-1,12,-5,1,5,7,-9,9,-13,3,-6,7,-1,-5,15,4,-6,6,9,-2,-1,3,4,10,6,-3,-15,10,-9,-13,0,-8,6,15,-11,1,10,6,6,4,14,3,13,-3,-14,9,12,11,-13,7,12,1,-10,3,9,-15,12,4,14,-12,-8,2,8,3,-15,-9,-10,-6,-15,-12,-3,14,-1,-11,7,11,-1,-8,2,2,2,9,-14,-8,4,-9,-11,14,6,-2,7,-8,-9,2,-14,8,-15,1,8,-5,2,0,-14,-12,9,-3,3,-4,-1,0,1,15,-1,-1,0,13,-5,7,12,-7,4,-15,0,-15,4,0,-8,-3,-8,15,6,14,-6,-10,-13,10,6,6,-4,11,11,-1,-5,-11,13,-2,-10,3,-14,10,-9,-11,-6,-11,-11,15,-5,0,-9,0,-4,-6,3,11,10,8,-1,0,6,-5,4,14,-13,1,-3,-15,15,-10,-1,-5,-15,8,10,11,-11,2,9,-7,15,-2,0,-2,-15,11,5,-6,-12,4,7,13,-9,0,-8,-15,8,4,15,-8,10,-10,-4,-3,-13,3,15,8,2,-2,-7,7,1,-12,7,10,-1,2,2,7,14,-1,-6,1,11,-1,2,-10,2,-10,15,9,14,0,-10,-7,-10,-15,10,0,-9,-10,-9,-2,12,-7,15,-5,-7,-12,6,-2,1,4,4,8,3,-8,14,-10,-5,-6,-9,3,2,-13,5,15,3,-3,7,-14,-4,13,-13,-15,-4,8,-5,-11,-14,6,-15,10,-2,-12,6,-13,3,-4,5,-13,7,-8,4,3,-14,6,5,-15,-12,-6,-15,-11,0,15,2,1,-12,-1,-8,-2,-6,7,4,1,2,-12,12,6,-11,-7,6,-15,-1,-4,7,2,6,1,13,-8,3,4,8,-1,-9,-9,7,-5,4,7,9,-7,14,-13,-7,8,9,11,6,3,-8,-3,-2,15,-3,15,7,12,10,-4,0,15,6,10,-12,15,11,9,6,-6,-8,-13,-5,-12,-12,-3,9,-12,9,-14,5,5,-4,3,-11,2,13,15,-7,-15,14,3,10,9,-1,0,1,4,7,13,-6,-3,-15,1,-13,13,0,0,5,-3,14,-8,1,12,14,-12,8,0,7,-4,-15,-12,-2,-13,-13,-12,10,14,14,-4,-15,-6,7,1,-6,9,-15,-8,-5,14,-2,-6,13,-15,-12,-12,9,14,11,-14,-7,2,-9,5,2,-4,4,-7,-14,14,-15,7,-12,11,5,-3,-13,3,11,11,-10,-2,3,2,-3,-4,9,6,-15,-10,-2,8,6,13,-5,-11,13,-2,13,-2,-7,5,1,7,8,7,7,-13,-9,8,-10,-1,-2,-6,7,-5,-13,2,-5,7,3,5,-12,3,-1,4,-3,13,-1,5,-12,-13,-11,3,15,-7,-9,-3,-6,15,-6,-6,-2,-3,2,-10,11,-3,-9,1,-2,11,13,-9,-14,13,-2,-11,-15,-8,15,9,11,-9,6,15,8,-4    
        ].to_vec();
        
        let b_y = [
            11,-4,-15,-8,20,-19,-21,-19,-18,26,11,26,-17,-23,21,17,25,6,-9,-12,17,18,-12,-19,-12,-27,20,-9,-16,12,17,-24,-26,-21,-23,-16,21,-5,8,-29,17,-1,-10,19,-29,-25,-12,-3,-27,28,16,14,-21,2,22,16,-3,-6,-21,-5,-23,5,-2,18,-10,-14,-16,-19,-14,-3,-12,2,-3,8,-15,2,16,-12,-26,15,-10,11,-12,-8,7,-28,12,-12,-29,-13,-25,-15,-12,10,-23,18,-3,-12,-6,-7,19,-21,27,-19,16,-4,-26,18,-28,21,-27,29,23,-26,8,-27,24,18,24,11,23,-12,-7,-25,-16,-24,-20,19,-6,4,23,10,-24,-5,-11,-26,-19,20,29,-10,12,15,-6,8,-23,-27,5,11,3,-11,26,-24,-16,-25,6,20,-4,0,-5,-18,-10,30,-27,-21,-17,-27,5,-11,29,24,-24,-15,12,14,-6,13,-10,-5,-17,-16,-23,4,28,-27,30,-3,-2,8,14,-5,29,8,-21,-22,-18,22,-27,-30,-19,-16,22,18,-11,7,-8,21,-2,0,3,-22,-4,-10,-19,8,11,-6,-7,-9,-30,19,-16,-4,8,-23,-29,6,-23,1,-18,23,6,-1,28,29,-30,18,-10,-22,-1,-26,18,-25,-28,-23,27,20,-27,28,-12,11,-2,0,23,1,13,-5,-27,18,12,13,2,19,7,30,-7,-14,-22,-6,19,24,-2,23,-26,3,21,23,18,14,28,-30,8,30,4,0,-19,-28,2,-13,10,-11,9,-29,19,8,14,13,9,-18,12,-8,-19,-11,4,-9,-30,-20,9,-7,4,-30,9,9,-1,-16,28,-26,22,-18,-15,-30,-5,-20,24,-29,5,-19,9,2,26,25,-20,16,11,-22,-11,23,-3,14,-19,-24,-20,6,-14,21,-30,-1,-20,0,5,25,11,-23,-18,17,-13,-21,26,-23,24,-1,13,-25,-11,2,-1,-2,22,-16,-1,-29,-8,26,-18,-20,20,-7,6,13,11,1,-25,14,-6,-10,-7,28,-13,-10,-28,-19,-12,28,3,-22,11,7,23,11,-15,-26,19,10,0,17,18,-11,30,20,-24,-2,-6,-18,-10,30,27,5,21,28,4,5,26,-23,2,21,-11,-5,-18,-9,21,-29,1,-18,4,-7,-15,15,-12,23,-10,20,2,12,-17,29,13,12,-25,24,-3,27,29,-12,9,7,-9,-11,3,14,0,-11,27,5,0,3,27,15,-28,8,-8,15,-20,-7,-22,1,10,17,11,7,-1,-29,26,-10,-2,17,3,22,-23,-22,15,-6,-18,-23,18,24,-30,-3,30,-24,-14,-16,-14,-25,6,12,24,-26,-27,-28,24,1,-15,30,5,19,-28,-16,-19,30,-13,-23,-27,-13,17,-30,17,-3,-7,22,-11,-15,26,12,-19,-26,-18,11,-23,22,-3,-16,25,2,29,19,-8,17,-10,-5,15,18,-29,26,22,23,-4,6,15,7,17,8,17,-25,8,16,-9,26,24,16,4,25,-29,16,20,16,-11,-30,3,-14,25,0,-9,13,-21,12,-10,-24,-3,11,-23,24,23,14,-26,24,-10,21,-28,15,-8,13,29,6,2,-15,29,-2,-2,-7,-21,18,8,-16,25,-7,-30,0,17,11,-26,-3,-5,-17,18,-1,3,-15,25,-21,2,-26,23,-19,1,-12,30,-2,7,9,-2,22,13,-29,-17,-15,-7,29,28,14,-6,13,5,-2,-27,-12,28,-14,-24,-10,23,-18,8,9,-12,18,2,-19,1,-1,-13,0,-11,-6,3,21,25,5,20,-1,-22,24,-20,-9,-27,-8,5,21,2,30,14,16,14,-13,-12,13,16,25,17,-15,2,2,-6,-20,-21,12,-24,4,13,24,24,-13,27,-4,-15,-17,29,-18,25,-13,-28,-25,26,-17,22,22,12,24,-16,-11,-15,6,-21,-23,-11,-16,-15,19,29,-17,17,-22,-19,-8,15,21,17,-24,19,-15,-22,-27,-13,25,-26,-19,25,16,-29,1,-10,-23,-18,-23,-14,-24,-12,-26,7,12,-26,5,30,-15,2,-29,-17,30,-29,-4,3,19,-7,-21,-18,-15,23,-26,-16,-20,-27,-16,1,8,-3,-8,-23,5,30,10,24,7,28,-4,26,24,3,13,25,23,20,20,8,-3,30,30,2,10,6,-19,-28,16,1,-27,6,14,15,-3,22,15,30,24,-10,-25,-22,-6,-16,-7,11,23,7,-11,-18,-18,1,-5,-23,-9,18,-9,18,16,10,21,26,-15,23,-5,-20,-13,-21,-12,30,-28,13,15,-19,-17,-14,-2,22,5,28,21,7,11,-18,13,28,-29,19,25,17,28,13,29,-30,-16,21,-8,-18,17,0,-16,-12,-30,-3,5,-5,10,-24,2,-11,-3,-22,25,-28,14,22,-20,-2,14,-29,30,20,16,-1,-13,0,-26,8,-1,14,7,-7,29,-3,14,4,-2,10,-2,12,27,-8,20,27,14,29,-7,1,24,22,21,-30,30,-26,11,20,0,18,-7,29,14,23,14,-16,-20,-27,29,-21,1,-29,-29,-11,10,-6,29,29,13,-15,-14,-5,22,30,8,-13,-6,22,-4,-15,-2,-24,-28,-7,-30,15,2,23,-16,-3,16,-14,-5,6,-1,0,5,-3,-29,-26,-27,5,-21,22,4,-6,6,-13,25,17,5,28,-20,-19,-18,-28,-1,2,-26,15,22,14,28,-14,-12
        ].to_vec();

        let n = 1024; 
        let mut vec = Vec::<u8>::with_capacity(1024);
        for i in 0..n {
            let mut ax = 0 as i32;
            let mut ay = 0 as i32;
            let mut bx = 0 as i32;
            let mut by = 0 as i32;

            match direction {
                Direction::Horizontal => {
                    ax = coordinate.0 as i32 + (a_x[i]*2) as i32;
                    ay = coordinate.1 as i32 + (a_y[i]*2) as i32;
                    bx = coordinate.0 as i32 + (b_x[i]*2) as i32;
                    by = coordinate.1 as i32 + (b_y[i]*2) as i32;
                },
                Direction::Vertical => {
                    ax = coordinate.0 as i32 + (a_y[i]*2) as i32;
                    ay = coordinate.1 as i32 + (a_x[i]*2) as i32;
                    bx = coordinate.0 as i32 + (b_y[i]*2) as i32;
                    by = coordinate.1 as i32 + (b_x[i]*2) as i32;
                }
            }

            if  ax >= 0 && 
                ax < src.cols as i32 &&
                ay >= 0 && 
                ay < src.rows as i32 &&
                bx >= 0 && 
                bx < src.cols as i32 &&
                by >= 0 && 
                by < src.rows as i32 {

                if src.get_pixel_by_xy(ax as usize, ay as usize)[0] > src.get_pixel_by_xy(bx as usize, by as usize)[0] {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
        }
        self.feature_pairs = vec;
    }

    pub fn maximum_value(&self)
        -> i32
    {
        let mut total = 0;
        for description_value in &self.description {
            total += (description_value.clone() as i32).abs();
        }
        total
    }

    pub fn remove(&mut self) {
        self.removed = true;
    }

    // greater threshold is more similar
    pub fn most_similar_desc(&self, others: &Vec<PixelDescription>, threshold: i32)
        -> Result<(PixelDescription, PixelDescription), &'static str>
    {
        let mut most_similarity = 0;
        let mut most_similar_one = PixelDescription::new();
        for other in others {
            let current_similarity = self.similarity(&other);
            if current_similarity > most_similarity {
                most_similarity = current_similarity;
                most_similar_one = other.clone();
            }
        }

        if most_similarity > threshold {
            return Ok((self.clone(), most_similar_one));
        }
        return Err("Can not find similar desc");
    }

    pub fn similarity(&self, other: &PixelDescription) -> i32 {
        let mut s = 0;
        
        // if self.feature_pairs.len() != 1024 {
        //     println!("Feature Pairs: {:?}", self.feature_pairs.len());
        // }

        for i in 0..self.feature_pairs.len() {
            if self.feature_pairs.len() == other.feature_pairs.len() && self.feature_pairs[i] == other.feature_pairs[i] {
                s += 1;
            }
        }
        s
    }

    // greater threshold is more similar
    pub fn match_points(descriptions: &Vec<PixelDescription>, others: &Vec<PixelDescription>, threshold: i32)
        -> Vec<(PixelDescription, PixelDescription)>
    {
        let now = Instant::now();
        let mut points = Vec::<(PixelDescription, PixelDescription)>::new();
        for desc in descriptions {
            let the_most = desc.most_similar_desc(others, threshold);
            match the_most {
                Ok(v) => {
                    points.push(v)
                },
                Err(e) => {},
            }
        }
        println!("Spend ms on match points:{}", now.elapsed().as_millis());
        PixelDescription::filter_pair(&points)
        // points
    }

    pub fn filter_pair(pairs: &Vec<(PixelDescription, PixelDescription)>)
        -> Vec<(PixelDescription, PixelDescription)>
    {
        fn get_distance(pair: &(PixelDescription, PixelDescription)) -> f32 {
            let ax = pair.0.coordinate.0 as f32;
            let ay = pair.0.coordinate.1 as f32;

            let bx = pair.1.coordinate.0 as f32;
            let by = pair.1.coordinate.1 as f32;
            ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
        }

        // Find the Standard Deviation of distances
        fn get_angle(pair: &(PixelDescription, PixelDescription)) -> f32 {
            
                let ax = pair.0.coordinate.0 as f32;
                let ay = pair.0.coordinate.1 as f32;

                let bx = pair.1.coordinate.0 as f32;
                let by = pair.1.coordinate.1 as f32;
                
                ((ay-by)/(ax-bx)).atan()
        }

        
        let mut total = 0.0;
        for pair in pairs {
            total += get_distance(&pair);
        }
        
        let avg_distance = total/(pairs.len() as f32);
    
        let mut total = 0.0;

        for pair in pairs {
            total += get_angle(&pair);
        }

        let avg_angle = total/(pairs.len() as f32);

        let mut new_pairs = Vec::<(PixelDescription, PixelDescription)>::new();

        for pair in pairs {
            if ((get_angle(&pair) - avg_angle)/3.14).abs() < 0.05 {
                if ((get_distance(&pair) - avg_distance)/avg_distance).abs() < 0.2 {
                    new_pairs.push(pair.clone());
                }
            }
        }

        new_pairs
    }
}

impl Clone for PixelDescription {
    fn clone(&self) -> PixelDescription {
        PixelDescription {
            coordinate: self.coordinate.clone(),
            description: self.description.clone(),
            value: self.value.clone(),
            removed: self.removed.clone(),
            feature_pairs: self.feature_pairs.to_vec()
        }
    }
}