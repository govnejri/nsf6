use actix_web::{HttpRequest, HttpResponse, Result, web, http::header};
use actix_files::NamedFile;
use std::path::PathBuf;
use dashmap::DashMap;
use std::sync::Arc;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// Структура для кэша сжатых изображений
#[derive(Clone)]
pub struct ImageCache {
    cache: Arc<DashMap<String, CachedImage>>,
    max_size: usize, // Максимальный размер кэша в байтах
    current_size: Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Clone)]
struct CachedImage {
    data: Vec<u8>,
    content_type: String,
    last_modified: u64,
    original_modified: u64,
}

impl ImageCache {
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_size: max_size_mb * 1024 * 1024,
            current_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    async fn get_or_create_webp(&self, image_path: &PathBuf, cache_key: &str) -> Result<CachedImage> {
        // Проверяем время модификации файла
        let metadata = fs::metadata(image_path)
            .map_err(|_| actix_web::error::ErrorNotFound("Image not found"))?;
        
        let modified_time = metadata.modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Проверяем кэш
        if let Some(cached) = self.cache.get(cache_key) {
            if cached.original_modified >= modified_time {
                return Ok(cached.clone());
            }
        }

        // Читаем и конвертируем изображение
        let webp_data = self.convert_to_webp(image_path).await?;
        
        let cached_image = CachedImage {
            data: webp_data,
            content_type: "image/webp".to_string(),
            last_modified: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            original_modified: modified_time,
        };

        // Проверяем размер кэша перед добавлением
        let data_size = cached_image.data.len();
        let current = self.current_size.load(std::sync::atomic::Ordering::Relaxed);
        
        if current + data_size > self.max_size {
            self.cleanup_cache().await;
        }

        self.current_size.fetch_add(data_size, std::sync::atomic::Ordering::Relaxed);
        self.cache.insert(cache_key.to_string(), cached_image.clone());

        Ok(cached_image)
    }

    async fn convert_to_webp(&self, image_path: &PathBuf) -> Result<Vec<u8>> {
        // Используем tokio::task::spawn_blocking для CPU-интенсивной операции
        let path = image_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
            let img = image::open(&path)?;
            
            // Конвертируем в WebP с качеством 85%
            let encoder = webp::Encoder::from_image(&img)?;
            let webp_data = encoder.encode(85.0);
            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Task join error"))?
        .map_err(|_| actix_web::error::ErrorInternalServerError("WebP conversion failed"))
    }

    async fn cleanup_cache(&self) {
        // Простая стратегия: удаляем 25% самых старых записей
        let mut entries: Vec<_> = self.cache.iter()
            .map(|entry| (entry.key().clone(), entry.value().last_modified))
            .collect();
        
        entries.sort_by_key(|(_, modified)| *modified);
        let to_remove = entries.len() / 4;
        
        for (key, _) in entries.into_iter().take(to_remove) {
            if let Some((_, cached)) = self.cache.remove(&key) {
                self.current_size.fetch_sub(cached.data.len(), std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}

// Глобальный кэш изображений
static IMAGE_CACHE: once_cell::sync::Lazy<ImageCache> = once_cell::sync::Lazy::new(|| {
    ImageCache::new(100) // 100 MB кэш
});

pub async fn serve_image(req: HttpRequest, path: web::Path<String>) -> Result<HttpResponse> {
    let image_path = PathBuf::from("web/out/static/assets/img").join(path.as_str());
    
    // Проверяем, существует ли файл
    if !image_path.exists() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Создаем NamedFile с оптимизированными заголовками
    let file = NamedFile::open(image_path)?
        .use_etag(true)
        .use_last_modified(true);

    // Добавляем заголовки кэширования для изображений
    let mut response = file.into_response(&req);
    
    // Кэшируем изображения на 1 год
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    Ok(response)
}

pub async fn serve_optimized_image(
    req: HttpRequest, 
    path: web::Path<String>
) -> Result<HttpResponse> {
    let image_path = PathBuf::from("web/out/static/assets/img").join(path.as_str());
    
    if !image_path.exists() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Проверяем Accept заголовок для WebP поддержки
    let accepts_webp = req
        .headers()
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .map(|accept| accept.contains("image/webp"))
        .unwrap_or(false);

    // Если браузер поддерживает WebP, конвертируем на лету
    if accepts_webp {
        let cache_key = format!("webp:{}", path.as_str());
        
        match IMAGE_CACHE.get_or_create_webp(&image_path, &cache_key).await {
            Ok(cached_image) => {
                return Ok(HttpResponse::Ok()
                    .content_type(cached_image.content_type.as_str())
                    .insert_header((header::CACHE_CONTROL, "public, max-age=31536000, immutable"))
                    .insert_header((header::ETAG, format!("\"{}\"", cached_image.last_modified)))
                    .body(cached_image.data));
            }
            Err(e) => {
                println!("Failed to convert to WebP: {:?}", e);
                // Fallback к оригинальному изображению
            }
        }
    }

    // Возвращаем оригинальное изображение
    serve_image(req, path).await
}
