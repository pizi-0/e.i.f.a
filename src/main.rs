use std::env;
use std::fs;
use std::path::Path;
use apk_info::Apk;
use std::fs::File;
use zip::ZipArchive;

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
    
    // Prioritize searching for the highest resolution ic_launcher.png first.
    println!("Searching for the highest resolution ic_launcher.png...");
    const DENSITY_ORDER: &[&str] = &["xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi"];
    
    let file = File::open(apk_path).expect("Failed to open APK file");
    let mut archive = ZipArchive::new(file).expect("Failed to read APK as zip");
    
    let primary_search_result = DENSITY_ORDER.iter().find_map(|density| {
        // Find a file entry that starts with the mipmap density folder and ends with the icon name
        for i in 0..archive.len() {
            let mut file_entry = archive.by_index(i).ok()?;
            let file_name = file_entry.name().to_owned(); 
    
            let prefix = format!("res/mipmap-{}", density);
            if file_name.starts_with(&prefix) && file_name.ends_with("/ic_launcher.png") {
                // Found a match, now read its contents
                let mut buffer = Vec::new();
                if std::io::copy(&mut file_entry, &mut buffer).is_ok() {
                    println!("Found ic_launcher.png in: {}", file_name);
                    return Some(buffer);
                }
            }
        }
        None  
    });

    let icon_data = primary_search_result.unwrap_or_else(|| {
        // Fallback to the icon path from the manifest if ic_launcher.png is not found.
        println!("No ic_launcher.png found. Falling back to manifest icon path.");
        let app_icon_path = apk.get_application_icon()
            .expect("No application icon path found in APK manifest.");

        if app_icon_path.ends_with(".xml") {
            eprintln!("Manifest icon is an XML file, and no suitable PNG fallback was found.");
            std::process::exit(1);
        }

        println!("Found non-XML icon in manifest: {}", app_icon_path);
        let (data, _) = apk.read(&app_icon_path).expect("Failed to read manifest icon file");
        data 
    });

    let output_file_path = Path::new(output_dir).join(format!("{}.png", package_name));
    if let Err(e) = fs::write(&output_file_path, &icon_data) {
        eprintln!("Failed to write icon to '{}': {}", output_file_path.display(), e);
        std::process::exit(1);
    };
    println!("Package name: {}", package_name);
    println!("Icon successfully extracted to: {}", output_file_path.display());
}