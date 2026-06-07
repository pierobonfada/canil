use axum::body::Bytes;
use image::{imageops::FilterType, ImageFormat};
use uuid::Uuid;

// ==========================================
// 📸 ESTÚDIO FOTOGRÁFICO DO CANIL
// ==========================================
// Este arquivo pega aquelas fotos gigantes que o tutor tira do celular 
// e dá aquele "tapa" mágico (redimensiona e comprime pra JPEG) 
// pra garantir que o site carregue super rápido sem gastar a internet de ninguém!

// 🪄 A função mágica que recebe os bytes crus da imagem e devolve o caminho salvo
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

    // Força o gerenciador de memória do Linux (glibc) a devolver a memória RAM não utilizada 
    // imediatamente para o Sistema Operacional, prevenindo OOM Kills no Render.
    #[cfg(target_os = "linux")]
    unsafe {
        extern "C" { fn malloc_trim(pad: usize) -> i32; }
        malloc_trim(0);
    }

    Ok(filepath)
}