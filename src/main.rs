use std::env;
use std::fs;
use std::path::Path;
use apk_info::Apk;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <path_to_apk> <output_path>", &args[0]);
        std::process::exit(1);
    }

    let apk_path = &args[1];
    let output_path = &args[2];

    println!("APK path provided: {}", apk_path);
    println!("Output path for extracted images: {}", output_path);
   
    get_app_icon(apk_path, output_path);
}

fn get_app_icon(apk_path: &str, output_dir: &str) {
    let apk = match Apk::new(apk_path) {
        Ok(apk) => apk,
        Err(e) => {
            eprintln!("Cannot open apk file: {}", e);
            std::process::exit(1);
        }
    };
    let package_name = apk.get_package_name().unwrap_or_default();

    let app_icon_path = match apk.get_application_icon() {
        Some(path) => path,
        None => {
            eprintln!("No application icon path found in APK");
            std::process::exit(1);
        }
    };

    let icon_data = if app_icon_path.ends_with(".xml") {
        // Adaptive icon detected, search for a fallback PNG.
        println!("Adaptive icon detected. Searching for a fallback PNG icon.");
        const DENSITY_ORDER: &[&str] = &["xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi"];

        let (data, _) = DENSITY_ORDER.iter().find_map(|density| {
            let fallback_path = format!("res/mipmap-{}/ic_launcher.png", density);
            apk.read(&fallback_path).ok()
        }).expect("No suitable PNG fallback icon found for adaptive icon.");
        data
    } else {
        let (data, _) = apk.read(&app_icon_path).unwrap_or_else(|e| {
            eprintln!("Cannot read icon file '{}': {}", app_icon_path, e);
            std::process::exit(1);
        });
        data
    };

    let output_file_path = Path::new(output_dir).join(format!("{}.png", package_name));
    if let Err(e) = fs::write(&output_file_path, &icon_data) {
        eprintln!("Failed to write icon to '{}': {}", output_file_path.display(), e);
        std::process::exit(1);
    };

    println!("Package name: {}", package_name);
    println!("Icon successfully extracted to: {}", output_file_path.display());
}