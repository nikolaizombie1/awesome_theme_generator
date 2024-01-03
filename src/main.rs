use clap::{ArgGroup, Parser};
use regex::Regex;
use std::fs;
use std::{io::Write, path::PathBuf};
use theme_calculation::RgbValues;

#[derive(Parser)]
#[clap(group(
    ArgGroup::new("centrality")
    .required(true)
    .args(&["average","median","prevalance"])
))]
struct Args {
    #[arg(short, long, required = true, value_parser = is_file)]
    nitrogen_bg_saved: PathBuf,
    #[arg(short, long, required = true, value_parser = is_file)]
    theme_lua: PathBuf,
    #[arg(short, long, group = "centrality")]
    average: bool,
    #[arg(short, long, group = "centrality")]
    median: bool,
    #[arg(short, long, group = "centrality")]
    prevalance: bool,
    #[arg(short, long)]
    restart: bool,
}

fn is_file(input: &str) -> anyhow::Result<PathBuf> {
    let path = input.parse::<PathBuf>()?;
    if path.is_file() {
        Ok(path)
    } else {
        eprintln!(
            "{} is not a file",
            path.to_str().expect("Unable to convert path to string")
        );
        std::process::exit(1);
    }
}

mod theme_calculation;

fn main() {
    let args = Args::parse();
    let centrality = if args.average {
        theme_calculation::Centrality::Average
    } else if args.median {
        theme_calculation::Centrality::Median
    } else {
        theme_calculation::Centrality::Prevalent
    };

    let mut current_wallpapers = std::fs::read_to_string(&args.nitrogen_bg_saved)
        .unwrap()
        .lines()
        .filter(|l| l.contains("file="))
        .map(|l| l[5..l.len()].to_owned())
        .collect::<Vec<_>>();

    current_wallpapers.sort();

    let mut theme_lua = fs::read_to_string(&args.theme_lua).unwrap();

    for wallpaper in current_wallpapers {
        let theme = theme_calculation::calculate_theme(&PathBuf::from(wallpaper), centrality);
        theme_lua = replace_property("bg_normal", theme.primary_color, &theme_lua);
        theme_lua = replace_property("bg_focus", theme.secondary_color, &theme_lua);
        theme_lua = replace_property("fg_focus", theme.active_text_color, &theme_lua);
        theme_lua = replace_property("fg_normal", theme.normal_text_color, &theme_lua);
    }

    let mut theme_lua_modified = fs::File::create(&args.theme_lua).unwrap();
    theme_lua_modified.write_all(theme_lua.as_bytes()).unwrap();
    if args.restart {
        std::process::Command::new("bash")
            .arg("-c")
            .arg("echo 'awesome.restart()' | awesome-client")
            .output()
            .unwrap();
    }
}

fn replace_property(prop: &str, color: RgbValues, theme_lua: &str) -> String {
    let patern = r#"theme\."#.to_owned() + prop + r##"\s*=\s*"#[0-9a-fA-F]{6}""##;
    let pattern = Regex::new(&patern).unwrap();
    pattern
        .replace(theme_lua, format!("theme.{} = \"#{}\"", prop, color.hex()))
        .to_string()
}
