use axum::body::Bytes;
use image::{imageops::FilterType, ImageFormat};
use uuid::Uuid;

pub fn process_and_save_image(bytes: Bytes) -> Result<String, String> {
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Falha ao ler formato da imagem: {}", e))?;

    let resized = img.resize(800, 800, FilterType::Lanczos3);

    let filename = format!("{}.jpg", Uuid::new_v4());
    let filepath = format!("uploads/{}", filename);

    let mut file = std::fs::File::create(&filepath)
        .map_err(|e| format!("Falha ao criar arquivo: {}", e))?;
    
    resized
        .write_to(&mut file, ImageFormat::Jpeg)
        .map_err(|e| format!("Falha ao salvar JPEG: {}", e))?;

    Ok(filepath)
}