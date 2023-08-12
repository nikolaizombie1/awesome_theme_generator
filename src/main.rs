use image::Rgb;

enum RGB {
    RED,
    GREEN,
    BLUE,
}

#[derive(Debug)]
struct ComponentCount {
    red: usize,
    green: usize,
    blue: usize,
}

trait Component {
    fn max(&self) -> RGB;
}

impl Component for Rgb<u8> {
    fn max(&self) -> RGB {
        let red = self.0[0];
        let green = self.0[1];
        let blue = self.0[2];
        if red >= green && red >= blue {
            RGB::RED
        } else if green >= red && green >= blue {
            RGB::GREEN
        } else {
            RGB::BLUE
        }
    }
}

impl Component for ComponentCount {
    fn max(&self) -> RGB {
        if self.red >= self.green && self.red >= self.blue {
            RGB::RED
        } else if self.green >= self.red && self.green >= self.blue {
            RGB::GREEN
        } else {
            RGB::BLUE
        }
    }
}

fn main() {
    let image = image::io::Reader::open("/home/uwu/Linux-Mass-Storage/Documents/Rust_Stuff/awesome_theme_generator/F3AbO5CbEAEnfe9.jpeg")
	.unwrap()
	.decode()
	.unwrap().
	thumbnail(1000, 1000)
	.to_rgb8();
    let pixels = image.pixels().collect::<Vec<_>>();
    let mut max_component_counts = ComponentCount {
        red: 0,
        green: 0,
        blue: 0,
    };
    for pixel in pixels.iter() {
        match pixel.max() {
            RGB::RED => max_component_counts.red += 1,
            RGB::GREEN => max_component_counts.green += 1,
            RGB::BLUE => max_component_counts.blue += 1,
        }
    }
}
