use minijinja::{path_loader, Environment};
use minijinja_autoreload::AutoReloader;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::Serialize;
use actix_web::{Error, HttpResponse};

pub static TEMPLATES: Lazy<AutoReloader> = Lazy::new(|| {
    AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        let template_path = "web/out";
        env.set_loader(path_loader(template_path));
        notifier.watch_path(template_path, true);
        Ok(env)
    })
});

pub struct TemplateManager {
    templates: HashMap<String, String>,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.load_templates();
        manager
    }

    fn load_templates(&mut self) {
        let template_dir = Path::new("web/out");
        
        if let Ok(entries) = fs::read_dir(template_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
                    if let Some(file_stem) = path.file_stem() {
                        if let Some(template_name) = file_stem.to_str() {
                            let template_path = path.file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("")
                                .to_string();
                            self.templates.insert(template_name.to_string(), template_path);
                        }
                    }
                }
            }
        }
    }

    pub fn get_template_file(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    pub fn render<T: Serialize>(&self, template_name: &str, ctx: T) -> Result<HttpResponse, Error> {
        let template_file = self.get_template_file(template_name)
            .ok_or_else(|| {
                actix_web::error::ErrorNotFound(format!("Template '{}' not found", template_name))
            })?;

        let env = TEMPLATES
            .acquire_env()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        let tmpl = env
            .get_template(template_file)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        let html = tmpl
            .render(ctx)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html))
    }
}

pub static TEMPLATE_MANAGER: Lazy<TemplateManager> = Lazy::new(|| TemplateManager::new());

pub fn render_template<T: Serialize>(template_name: &str, ctx: T) -> Result<HttpResponse, Error> {
    TEMPLATE_MANAGER.render(template_name, ctx)
}