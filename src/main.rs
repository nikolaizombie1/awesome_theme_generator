use std::{path::PathBuf, io::Write};
use regex::Regex;
use theme_calculation::RgbValues;
use std::fs;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(short, long, required = true, value_parser = is_file)]
    nitrogen_bg_saved: PathBuf,
    #[arg(short, long, required = true, value_parser = is_file)]
    theme_lua: PathBuf,
    #[arg(short, long, required = true)]
    wallpaper_index: usize,
    #[arg(short, long)]
    median: bool,
    #[arg(short, long)]
    restart: bool,
}

fn is_file(input: &str) -> anyhow::Result<PathBuf> {
    let path = input.parse::<PathBuf>()?;
    if path.is_file() {
	Ok(path)
    } else {
	eprintln!("{} is not a file",path.to_str().expect("Unable to convert path to string"));
	std::process::exit(1);
    }
}

mod theme_calculation;

fn main() {
    let args = Args::parse();

    let current_wallpaper = std::fs::read_to_string(&args.nitrogen_bg_saved)
	.unwrap()
	.lines()
	.filter(|l| l.contains("file="))
	.map(|l| l[5..l.len()].to_owned())
	.collect::<Vec<_>>().get(args.wallpaper_index).unwrap().to_owned();
    let theme = theme_calculation::calculate_theme(&PathBuf::from(current_wallpaper), args.median);

    let mut theme_lua = fs::read_to_string(&args.theme_lua).unwrap();

    theme_lua = replace_property("bg_normal", theme.primary_color, &theme_lua);
    theme_lua = replace_property("bg_focus", theme.secondary_color, &theme_lua);
    theme_lua = replace_property("fg_focus", theme.active_text_color, &theme_lua);
    theme_lua = replace_property("fg_normal", theme.normal_text_color, &theme_lua);

    let mut theme_lua_modified = fs::File::create(&args.theme_lua).unwrap();
    theme_lua_modified.write_all(theme_lua.as_bytes()).unwrap();
    if args.restart {
	std::process::Command::new("bash").arg("-c").arg("echo 'awesome.restart()' | awesome-client").output().unwrap();
    }
}

fn replace_property(prop: &str, color: RgbValues, theme_lua: &str) -> String {
    let patern = r#"theme\."#.to_owned() + prop + r##"\s*=\s*"#[0-9a-fA-F]{6}""##;
    let pattern = Regex::new(&patern).unwrap();
    pattern.replace(theme_lua, format!("theme.{} = \"#{}\"",prop,color.hex())).to_string()
}
