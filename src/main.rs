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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Screen {
    screen_index: usize,
    wallpaper_path: String,
}

enum Property {
    BgNormal,
    BgFocus,
    FgNormal,
    FgFocus,
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::BgNormal => write!(f, "bg_normal"),
            Self::BgFocus => write!(f, "bg_focus"),
            Self::FgNormal => write!(f, "fg_normal"),
            Self::FgFocus => write!(f, "fg_focus"),
        }
    }
}

enum Generality {
    Global,
    Bar,
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

    let wallpapers = std::fs::read_to_string(&args.nitrogen_bg_saved)
        .unwrap()
        .lines()
        .filter(|l| l.contains("file=") || l.contains("xin"))
        .map(|l| l[5..l.len()].to_owned())
        .collect::<Vec<_>>();

    let mut screens: Vec<Screen> = vec![];

    for i in (0..wallpapers.len()).filter(|x| x % 2 == 0) {
        let index = wallpapers[i]
            .split('_')
            .collect::<String>()
            .chars()
            .filter(|c| c != &']')
            .collect::<String>()
            .parse::<i64>()
            .unwrap();
        let index: usize = (index + 2) as usize;
        let wallpaper = &wallpapers[i + 1];
        screens.push(Screen {
            screen_index: index,
            wallpaper_path: wallpaper.to_owned(),
        });
    }

    screens.sort();

    let screens = screens
        .into_iter()
        .enumerate()
        .map(|s| Screen {
            screen_index: s.0 + 1,
            wallpaper_path: s.1.wallpaper_path,
        })
        .collect::<Vec<Screen>>();

    let mut theme_lua = fs::read_to_string(&args.theme_lua).unwrap();

    let theme = theme_calculation::calculate_theme(
        &PathBuf::from(screens.clone()[0].clone().wallpaper_path),
        centrality,
    );
    theme_lua = replace_property(
        Property::BgNormal,
        theme.primary_color,
        &theme_lua,
        Generality::Global,
        &screens[0],
    );
    theme_lua = replace_property(
        Property::BgFocus,
        theme.secondary_color,
        &theme_lua,
        Generality::Global,
        &screens[0],
    );
    theme_lua = replace_property(
        Property::FgFocus,
        theme.active_text_color,
        &theme_lua,
        Generality::Global,
        &screens[0],
    );
    theme_lua = replace_property(
        Property::FgNormal,
        theme.normal_text_color,
        &theme_lua,
        Generality::Global,
        &screens[0],
    );

    for screen in screens.clone() {
        let theme = theme_calculation::calculate_theme(
            &PathBuf::from(screen.clone().wallpaper_path),
            centrality,
        );
        if !Regex::new(&format!(
            "local bar{} = {}\ntheme.bar{} = bar{}",
            screen.screen_index, r#"\{\}"#, screen.screen_index, screen.screen_index
        ))
        .unwrap()
        .is_match(&theme_lua)
        {
            theme_lua = Regex::new("return theme")
                .unwrap()
                .replace(
                    &theme_lua,
                    format!(
                        "local bar{} = {}\ntheme.bar{} = bar{}\nreturn theme",
                        screen.screen_index, r#"{}"#, screen.screen_index, screen.screen_index
                    ),
                )
                .to_string();
        }
        theme_lua = replace_property(
            Property::FgNormal,
            theme.normal_text_color,
            &theme_lua,
            Generality::Bar,
            &screen,
        );
        theme_lua = replace_property(
            Property::BgNormal,
            theme.primary_color,
            &theme_lua,
            Generality::Bar,
            &screen,
        );
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

fn replace_property(
    prop: Property,
    color: RgbValues,
    theme_lua: &str,
    generality: Generality,
    screen: &Screen,
) -> String {
    let general = match generality {
        Generality::Global => "".to_owned(),
        Generality::Bar => format!("{}{}{}", "bar", screen.screen_index, r#"\."#),
    };
    let patern = format!(
        "{}{}{}{}",
        r#"theme\."#.to_owned(),
        general,
        prop,
        r##"\s*=\s*"#[0-9a-fA-F]{6}""##
    );
    let pattern = Regex::new(&patern).unwrap();
    let general = match generality {
        Generality::Global => "".to_owned(),
        Generality::Bar => format!("bar{}.", screen.screen_index),
    };
    let replacement_text = format!("theme.{}{} = \"#{}\"", general, prop, color.hex());
    match pattern.is_match(theme_lua) {
        true => {
            return pattern.replace(theme_lua, replacement_text).to_string();
        }
        false => {
            return Regex::new("return theme")
                .unwrap()
                .replace(theme_lua, format!("{}\nreturn theme", replacement_text))
                .to_string();
        }
    }
}
