use image::{Rgb as ImageRgb};
use std::{sync::{Arc,RwLock,Mutex}, thread, path::PathBuf};

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
struct ComponentCount {
    red: usize,
    green: usize,
    blue: usize,
}

#[derive(Debug)]
pub struct Theme {
    pub primary_color: RgbValues,
    pub secondary_color: RgbValues,
}

trait Component {
    fn max(&self) -> Rgb;
    fn middle(&self) -> Rgb;
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
    fn middle(&self) -> Rgb {
        let red = self.0[0];
        let green = self.0[1];
        let blue = self.0[2];
        if red >= green && red <= blue {
            Rgb::Red
        } else if green >= red && green <= blue {
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

impl Component for ComponentCount {
    fn max(&self) -> Rgb {
        if self.red >= self.green && self.red >= self.blue {
            Rgb::Red
        } else if self.green >= self.red && self.green >= self.blue {
            Rgb::Green
        } else {
            Rgb::Blue
        }
    }
    fn middle(&self) -> Rgb {
        if self.red >= self.green && self.red <= self.blue {
            Rgb::Red
        } else if self.green >= self.red && self.green <= self.blue {
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

impl ComponentCount {
    fn red(&self) -> usize {
	self.red
    } 
    fn green(&self) -> usize {
	self.green
    }
    fn blue(&self) -> usize {
	self.blue
    }
    fn get(&self, rgb: Rgb) -> usize {
	match rgb {
	    Rgb::Red => self.red(),
	    Rgb::Green => self.green(),
	    Rgb::Blue => self.blue(),
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
    fn middle(&self) -> Rgb {
        if self.red >= self.green && self.red <= self.blue {
            Rgb::Red
        } else if self.green >= self.red && self.green <= self.blue {
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
	format!("{:x?}{:x?}{:x?}",self.red,self.green,self.blue).to_ascii_uppercase()
    }
}


pub fn calculate_theme(path: &PathBuf) -> Theme {
    let pixels = image::io::Reader::open(path)
	.unwrap()
	.decode()
	.unwrap()
        .thumbnail(1000, 1000)
	.to_rgb8().pixels().copied().collect::<Vec<_>>();
    let pixels = Arc::new(RwLock::new(pixels));
    let mut max_component_counts = ComponentCount {
        red: 0,
        green: 0,
        blue: 0,
    };
    for pixel in pixels.read().unwrap().iter() {
        match pixel.max() {
            Rgb::Red => max_component_counts.red += 1,
            Rgb::Green => max_component_counts.green += 1,
            Rgb::Blue => max_component_counts.blue += 1,
        }
    }
    let avg_red: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let avg_green: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let avg_blue: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));

    let shared_pixels = pixels.clone();
    let shared_avg_red =  avg_red.clone();
    let red_handle = thread::spawn(move || {
	let mut red_slice = shared_pixels.read().unwrap().iter().map(|p| p.0[0]).collect::<Vec<_>>();
	red_slice.sort();
	let mut shared_avg_red = shared_avg_red.lock().unwrap();
	*shared_avg_red = average(&red_slice);
    });

    let shared_pixels = pixels.clone();
    let shared_avg_green =  avg_green.clone();
    let green_handle = thread::spawn(move || {
	let mut green_slice = shared_pixels.read().unwrap().iter().map(|p| p.0[1]).collect::<Vec<_>>();
	green_slice.sort();
	let mut shared_avg_green = shared_avg_green.lock().unwrap();
	*shared_avg_green = average(&green_slice);
    });

    let shared_pixels = pixels.clone();
    let shared_avg_blue =  avg_blue.clone();
    let blue_handle = thread::spawn(move || {
	let mut blue_slice = shared_pixels.read().unwrap().iter().map(|p| p.0[2]).collect::<Vec<_>>();
	blue_slice.sort();
	let mut shared_avg_blue = shared_avg_blue.lock().unwrap();
	*shared_avg_blue = average(&blue_slice);
    });

    red_handle.join().unwrap();
    green_handle.join().unwrap();
    blue_handle.join().unwrap();

    let primary_color = RgbValues {red: *avg_red.lock().unwrap(), green: *avg_green.lock().unwrap(), blue: *avg_blue.lock().unwrap()};
    let secondary_color = complementary_color(&primary_color);

    Theme { primary_color, secondary_color }
}

fn average(pixels: &[u8]) -> u8 {
    let sum: usize = pixels.iter().map(|x| *x as usize).sum();
    let avg: u8 = ((sum as f64)/(pixels.len() as f64)) as u8;
    avg
}

fn complementary_color(rgb: &RgbValues) -> RgbValues {
    let magnitude = (rgb.get(rgb.max()) as usize) + (rgb.get(rgb.min()) as usize);
    let red = (magnitude  - (rgb.red as usize)) as u8;
    let green = (magnitude  - (rgb.green as usize)) as u8;
    let blue = (magnitude  - (rgb.blue as usize)) as u8;
    RgbValues { red, green, blue }
}
