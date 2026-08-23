// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! File Upload Functions

use ring_lang_rs::*;

use crate::HTTP_SERVER_TYPE;

use super::HttpServer;

macro_rules! add_file_to_list {
    ($list:expr, $f:expr) => {
        let pair = ring_list_newlist($list);
        ring_list_addstring_str(pair, "name");
        ring_list_addstring_str(pair, &$f.filename);
        let pair = ring_list_newlist($list);
        ring_list_addstring_str(pair, "field");
        ring_list_addstring_str(pair, &$f.name);
        let pair = ring_list_newlist($list);
        ring_list_addstring_str(pair, "type");
        ring_list_addstring_str(pair, &$f.content_type);
        let pair = ring_list_newlist($list);
        ring_list_addstring_str(pair, "size");
        ring_list_adddouble(pair, $f.data.len() as f64);
    };
}

/// bolt_req_files_count(server) → number of uploaded files
ring_func!(bolt_req_files_count, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            ring_ret_number!(p, ctx.files.len() as f64);
        } else {
            ring_ret_number!(p, 0.0);
        }
    }
});

/// bolt_req_file(server, index) → Ring list [:name, :field, :type, :size]
ring_func!(bolt_req_file, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let idx_raw = ring_get_number!(p, 2);
    if idx_raw < 1.0 || !idx_raw.is_finite() {
        ring_ret_list!(p, ring_new_list!(p));
        return;
    }
    let index = (idx_raw as usize) - 1;

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            if index < ctx.files.len() {
                let f = &ctx.files[index];
                let list = ring_new_list!(p);
                add_file_to_list!(list, f);
                ring_ret_list!(p, list);
            } else {
                ring_ret_list!(p, ring_new_list!(p));
            }
        } else {
            ring_ret_list!(p, ring_new_list!(p));
        }
    }
});

/// bolt_req_files(server) → Ring list of file lists
ring_func!(bolt_req_files, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            let list = ring_new_list!(p);
            for f in &ctx.files {
                let file_list = ring_list_newlist(list);
                add_file_to_list!(file_list, f);
            }
            ring_ret_list!(p, list);
        } else {
            ring_ret_list!(p, ring_new_list!(p));
        }
    }
});

/// bolt_req_file_by_field(server, field_name) → Ring list for first matching file
ring_func!(bolt_req_file_by_field, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let field_name = ring_get_string!(p, 2);

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            if let Some(f) = ctx.files.iter().find(|f| f.name == field_name) {
                let list = ring_new_list!(p);
                add_file_to_list!(list, f);
                ring_ret_list!(p, list);
            } else {
                ring_ret_list!(p, ring_new_list!(p));
            }
        } else {
            ring_ret_list!(p, ring_new_list!(p));
        }
    }
});

/// bolt_req_file_save(server, index, path) → 1 on success
ring_func!(bolt_req_file_save, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_number!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let idx_raw = ring_get_number!(p, 2);
    if idx_raw < 1.0 || !idx_raw.is_finite() {
        ring_ret_number!(p, 0.0);
        return;
    }
    let index = (idx_raw as usize) - 1;
    let path = ring_get_string!(p, 3);

    // Prevent path traversal and absolute paths
    let path_obj = std::path::Path::new(path);
    for component in path_obj.components() {
        match component {
            std::path::Component::ParentDir => {
                ring_error!(p, "Invalid path: path traversal detected");
                return;
            }
            std::path::Component::RootDir => {
                ring_error!(p, "Invalid path: absolute paths not allowed");
                return;
            }
            std::path::Component::Prefix(_) => {
                ring_error!(p, "Invalid path: path prefix not allowed");
                return;
            }
            _ => {}
        }
    }

    // Reject NUL bytes in path
    if path.as_bytes().contains(&0) {
        ring_error!(p, "Invalid path: NUL byte detected");
        return;
    }

    unsafe {
        let server = &*(ptr as *const HttpServer);
        let guard = server.current_request.lock();
        if let Some(ref ctx) = *guard {
            if index < ctx.files.len() {
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .and_then(|mut f| {
                        use std::io::Write;
                        f.write_all(&ctx.files[index].data)
                    }) {
                    Ok(_) => {
                        ring_ret_number!(p, 1.0);
                        return;
                    }
                    Err(_) => {
                        ring_ret_number!(p, 0.0);
                        return;
                    }
                }
            }
        }
    }
    ring_ret_number!(p, 0.0);
});
