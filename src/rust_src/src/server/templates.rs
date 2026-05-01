// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Template Engine (MiniJinja)

use ring_lang_rs::*;

use crate::HTTP_SERVER_TYPE;

use super::HttpServer;

/// bolt_render_template(server, template, data_json) - render template string with MiniJinja
ring_func!(bolt_render_template, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let _ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);

    let template_str = ring_get_string!(p, 2);
    let data_json = ring_get_string!(p, 3);

    let data: serde_json::Value = serde_json::from_str(data_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let mut env = minijinja::Environment::new();
    if let Err(e) = env.add_template("template", template_str) {
        ring_error!(p, &format!("Template error: {}", e));
        return;
    }

    let tmpl = match env.get_template("template") {
        Ok(t) => t,
        Err(e) => {
            ring_error!(p, &format!("Template error: {}", e));
            return;
        }
    };

    match tmpl.render(&data) {
        Ok(result) => ring_ret_string!(p, &result),
        Err(e) => {
            ring_error!(p, &format!("Render error: {}", e));
        }
    }
});

/// bolt_render_file(server, filepath, data_json) - render template file with MiniJinja
ring_func!(bolt_render_file, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let _ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);

    let filepath = ring_get_string!(p, 2);
    let data_json = ring_get_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);

    let template_str = unsafe {
        let server = &*(ptr as *const HttpServer);
        let cache = server.template_cache.read().unwrap();
        if let Some((cached_content, cached_mtime)) = cache.get(filepath) {
            if let Ok(meta) = std::fs::metadata(filepath) {
                if let Ok(mtime) = meta.modified() {
                    let mtime_secs = mtime
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if *cached_mtime == mtime_secs {
                        cached_content.clone()
                    } else {
                        drop(cache);
                        match std::fs::read_to_string(filepath) {
                            Ok(content) => {
                                let mut cache = server.template_cache.write().unwrap();
                                cache.insert(filepath.to_string(), (content.clone(), mtime_secs));
                                content
                            }
                            Err(_) => {
                                ring_ret_string!(p, "");
                                return;
                            }
                        }
                    }
                } else {
                    cached_content.clone()
                }
            } else {
                cached_content.clone()
            }
        } else {
            drop(cache);
            match std::fs::read_to_string(filepath) {
                Ok(content) => {
                    let mtime_secs = std::fs::metadata(filepath)
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH)
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let mut cache = server.template_cache.write().unwrap();
                    cache.insert(filepath.to_string(), (content.clone(), mtime_secs));
                    content
                }
                Err(_) => {
                    ring_ret_string!(p, "");
                    return;
                }
            }
        }
    };

    let data: serde_json::Value = serde_json::from_str(data_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let dir = std::path::Path::new(filepath)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let template_name = std::path::Path::new(filepath)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("template");

    let mut env = minijinja::Environment::new();
    let dir_clone = dir.clone();
    env.set_loader(move |name| {
        let path = dir_clone.join(name);
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    });

    if let Err(e) = env.add_template(template_name, &template_str) {
        ring_error!(p, &format!("Template error: {}", e));
        return;
    }

    let tmpl = match env.get_template(template_name) {
        Ok(t) => t,
        Err(e) => {
            ring_error!(p, &format!("Template error: {}", e));
            return;
        }
    };

    match tmpl.render(&data) {
        Ok(result) => ring_ret_string!(p, &result),
        Err(e) => {
            ring_error!(p, &format!("Render error: {}", e));
        }
    }
});
