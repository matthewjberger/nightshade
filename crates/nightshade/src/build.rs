use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::imageops::FilterType;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn compile_windows_icon(png_bytes: &[u8]) {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let ico_path = Path::new(&out_dir).join("icon.ico");

    let img = image::load_from_memory(png_bytes).expect("Failed to load icon PNG");

    let sizes = [16, 32, 48, 256];
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    for size in sizes {
        let resized = img.resize_exact(size, size, FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        let icon_image = IconImage::from_rgba_data(width, height, rgba.into_raw());
        icon_dir.add_entry(IconDirEntry::encode(&icon_image).expect("Failed to encode icon"));
    }

    let file = File::create(&ico_path).expect("Failed to create icon.ico");
    let writer = BufWriter::new(file);
    icon_dir.write(writer).expect("Failed to write icon.ico");

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ico_path.to_str().expect("Invalid ICO path"));
    resource
        .compile()
        .expect("Failed to compile Windows resources");
}
