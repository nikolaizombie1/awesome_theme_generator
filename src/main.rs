use std::{path::PathBuf, io::Write};
use regex::Regex;
use theme_calculation::RgbValues;
use std::fs;

mod theme_calculation;

fn main() {
    let current_wallpaper = std::fs::read_to_string("/home/uwu/.config/nitrogen/bg-saved.cfg")
	.unwrap()
	.lines()
	.filter(|l| l.contains("file="))
	.map(|l| l[5..l.len()].to_owned())
	.collect::<Vec<_>>().first().unwrap().to_owned();
    let theme = theme_calculation::calculate_theme(&PathBuf::from(current_wallpaper));

    let mut theme_lua = fs::read_to_string("/home/uwu/.config/awesome/theme.lua").unwrap();

    theme_lua = replace_property("bg_normal", theme.primary_color, &theme_lua);
    theme_lua = replace_property("bg_focus", theme.secondary_color, &theme_lua);
    theme_lua = replace_property("fg_focus", theme.active_text_color, &theme_lua);
    theme_lua = replace_property("fg_normal", theme.normal_text_color, &theme_lua);

    let mut theme_lua_modified = fs::File::create("/home/uwu/.config/awesome/theme.lua").unwrap();
    theme_lua_modified.write_all(theme_lua.as_bytes()).unwrap();
    std::process::Command::new("bash").arg("-c").arg("echo 'awesome.restart()' | awesome-client").output().unwrap();
}

fn replace_property(prop: &str, color: RgbValues, theme_lua: &str) -> String {
    let patern = r#"theme\."#.to_owned() + prop + r##"\s*=\s*"#[0-9a-fA-F]{6}""##;
    let pattern = Regex::new(&patern).unwrap();
    pattern.replace(&theme_lua, format!("theme.{} = \"#{}\"",prop,color.hex())).to_string()
}
