extern crate eva_lib;
extern crate serde_json;
use eva_lib::mat::Mat;

use std::fs::File;
use std::io::Read;

fn main() {
    let dir = std::env::args().nth(1).expect("no pattern given");

    let mut file = File::open(format!("{}/merged_fragments.json", dir)).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let result: Vec<Vec<usize>> = serde_json::from_str(&data).unwrap();

    let image = Mat::load_png(&format!("{}/result.png", dir));

    for (i, ffp) in result.iter().enumerate() {
        let mut fragment_image = image.crop(ffp[0], ffp[1], ffp[2] - ffp[0], ffp[3] - ffp[1]);
        // fragment_image = fragment_image.add_padding(20);

        fragment_image.save_as_png(&format!("{}/fragment_{}.png", dir, i));
    }

    // let ffp = result[35].clone();
    // let fragment_image = image.crop(ffp[0], ffp[1], ffp[2] - ffp[0], ffp[3] - ffp[1]);
    // fragment_image.save_as_png(&format!("{}/fragment_{}.png", dir, 35));

}