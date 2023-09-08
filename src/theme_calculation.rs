use image::Rgb as ImageRgb;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

#[derive(PartialEq,Copy,Clone)]
pub enum Centrality {
    Average,
    Median,
    Prevalent,
}

enum Rgb {
    Red,
    Green,
    Blue,
}

#[derive(Debug)]
pub struct RgbValues {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug)]
pub struct Theme {
    pub primary_color: RgbValues,
    pub secondary_color: RgbValues,
    pub active_text_color: RgbValues,
    pub normal_text_color: RgbValues,
}

trait Component {
    fn max(&self) -> Rgb;
    fn min(&self) -> Rgb;
}

impl Component for ImageRgb<u8> {
    fn max(&self) -> Rgb {
        let red = self.0[0];
        let green = self.0[1];
        let blue = self.0[2];
        if red >= green && red >= blue {
            Rgb::Red
        } else if green >= red && green >= blue {
            Rgb::Green
        } else {
            Rgb::Blue
        }
    }
    fn min(&self) -> Rgb {
        let red = self.0[0];
        let green = self.0[1];
        let blue = self.0[2];
        if red <= green && red <= blue {
            Rgb::Red
        } else if green <= red && green <= blue {
            Rgb::Green
        } else {
            Rgb::Blue
        }
    }
}

impl Component for RgbValues {
    fn max(&self) -> Rgb {
        if self.red >= self.green && self.red >= self.blue {
            Rgb::Red
        } else if self.green >= self.red && self.green >= self.blue {
            Rgb::Green
        } else {
            Rgb::Blue
        }
    }
    fn min(&self) -> Rgb {
        if self.red <= self.green && self.red <= self.blue {
            Rgb::Red
        } else if self.green <= self.red && self.green <= self.blue {
            Rgb::Green
        } else {
            Rgb::Blue
        }
    }
}

impl RgbValues {
    fn get(&self, rgb: Rgb) -> u8 {
        match rgb {
            Rgb::Red => self.red,
            Rgb::Green => self.green,
            Rgb::Blue => self.blue,
        }
    }
    pub fn hex(&self) -> String {
        format!("{:02x?}{:02x?}{:02x?}", self.red, self.green, self.blue).to_ascii_lowercase()
    }
}

pub fn calculate_theme(path: &PathBuf, centrality: Centrality) -> Theme {
    let pixels = image::io::Reader::open(path).unwrap().decode().unwrap();
    let scaled_height = ((pixels.height() as f64) / 4.0) as u32;
    let scaled_width = ((pixels.width() as f64) / 4.0) as u32;
    let pixels = pixels
        .thumbnail(scaled_height, scaled_width)
        .to_rgb8()
        .pixels()
        .copied()
        .collect::<Vec<_>>();
    let pixels = Arc::new(RwLock::new(pixels));

    let avg_red: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let avg_green: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let avg_blue: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));

    match centrality {
        Centrality::Average | Centrality::Median => {
            let red_handle = spawn_color_thread(pixels.clone(), 0, avg_red.clone(), centrality);
            let green_handle = spawn_color_thread(pixels.clone(), 1, avg_green.clone(), centrality);
            let blue_handle = spawn_color_thread(pixels.clone(), 2, avg_blue.clone(), centrality);

            red_handle.join().unwrap();
            green_handle.join().unwrap();
            blue_handle.join().unwrap();
        }
        Centrality::Prevalent => {
            let prevalent_pixel = prevalent(pixels.clone());
            *avg_red.lock().unwrap() = prevalent_pixel.red;
            *avg_green.lock().unwrap() = prevalent_pixel.green;
            *avg_blue.lock().unwrap() = prevalent_pixel.blue;
        }
    }

    let primary_color = RgbValues {
        red: *avg_red.lock().unwrap(),
        green: *avg_green.lock().unwrap(),
        blue: *avg_blue.lock().unwrap(),
    };
    let secondary_color = complementary_color(&primary_color);
    let active_text_color =
        if primary_color.red > 128 && primary_color.green > 128 && primary_color.blue > 128 {
            RgbValues {
                red: 0,
                green: 0,
                blue: 0,
            }
        } else {
            RgbValues {
                red: 255,
                green: 255,
                blue: 255,
            }
        };

    let normal_text_color = if active_text_color.red == 0
        && active_text_color.green == 0
        && active_text_color.blue == 0
    {
        RgbValues {
            red: 60,
            green: 60,
            blue: 60,
        }
    } else {
        RgbValues {
            red: 195,
            green: 195,
            blue: 195,
        }
    };

    Theme {
        primary_color,
        secondary_color,
        active_text_color,
        normal_text_color,
    }
}

fn average(pixels: &[u8]) -> u8 {
    let sum: usize = pixels.iter().map(|x| *x as usize).sum();
    let avg: u8 = ((sum as f64) / (pixels.len() as f64)) as u8;
    avg
}

fn median(color_slice: &[u8]) -> u8 {
    if color_slice.len() % 2 == 0 {
        let left_middle =
            color_slice[(((color_slice.len() as f64) / (2.0)) - 1.0).floor() as usize];
        let right_middle = color_slice[((color_slice.len() as f64) / (2.0)).floor() as usize];
        (((right_middle as f64) + (left_middle as f64)) / 2.0) as u8
    } else {
        color_slice[(((color_slice.len() as f64) / 2.0) - 1.0) as usize]
    }
}

fn prevalent(pixels: Arc<RwLock<Vec<image::Rgb<u8>>>>) -> RgbValues {
  let mut pixel_prevalence_count = std::collections::HashMap::new(); 
  let pixels_bind =  pixels.read().unwrap();
  for pixel in pixels_bind.iter() {
    let count = pixel_prevalence_count.entry(pixel).or_insert(0);
    *count += 1;
  }
  
  let prevalant = pixel_prevalence_count.iter().max_by(|x,y| x.1.cmp(y.1)).unwrap().0;
  RgbValues { red: prevalant.0[0], green: prevalant.0[1], blue: prevalant.0[2] }
}

fn spawn_color_thread(
    pixels: Arc<RwLock<Vec<image::Rgb<u8>>>>,
    color_index: usize,
    color: Arc<Mutex<u8>>,
    centrality: Centrality,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut slice = pixels
            .read()
            .unwrap()
            .iter()
            .map(|p| p.0[color_index])
            .collect::<Vec<_>>();
        slice.sort();
        let mut shared_avg = color.lock().unwrap();
        if centrality == Centrality::Median {
            *shared_avg = median(&slice);
        } else if centrality == Centrality::Average {
            *shared_avg = average(&slice);
        }
    })
}

fn complementary_color(rgb: &RgbValues) -> RgbValues {
    let magnitude = (rgb.get(rgb.max()) as usize) + (rgb.get(rgb.min()) as usize);
    let red = (magnitude - (rgb.red as usize)) as u8;
    let green = (magnitude - (rgb.green as usize)) as u8;
    let blue = (magnitude - (rgb.blue as usize)) as u8;
    RgbValues { red, green, blue }
}
