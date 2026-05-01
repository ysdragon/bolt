// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! OpenAPI / Swagger

use ring_lang_rs::*;

use crate::HTTP_SERVER_TYPE;

use super::{HttpServer, RouteDefinition, convert_path_params};

/// bolt_openapi_spec(server, spec_json) - set OpenAPI spec JSON
ring_func!(bolt_openapi_spec, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let spec = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.openapi_spec = Some(spec.to_string());
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_openapi_route(server) - enable OpenAPI docs (spec generated at server start)
ring_func!(bolt_openapi_route, |p| {
    ring_check_paracount!(p, 1);
    ring_check_cpointer!(p, 1);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.openapi_enabled = true;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_openapi_info(server, title, version, description) - set OpenAPI metadata
ring_func!(bolt_openapi_info, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let title = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 2))
            .to_string_lossy()
            .to_string()
    };
    let version = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 3))
            .to_string_lossy()
            .to_string()
    };
    let description = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 4))
            .to_string_lossy()
            .to_string()
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.openapi_title = title;
        server.openapi_version = version;
        server.openapi_description = description;
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_route_describe(server, method, path, description) - set route description
ring_func!(bolt_route_describe, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let method = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 2))
            .to_string_lossy()
            .to_string()
    };
    let path = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 3))
            .to_string_lossy()
            .to_string()
    };
    let description = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 4))
            .to_string_lossy()
            .to_string()
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.set_route_description(&method, &path, &description);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_route_tag(server, method, path, tag) - add tag to route
ring_func!(bolt_route_tag, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let method = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 2))
            .to_string_lossy()
            .to_string()
    };
    let path = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 3))
            .to_string_lossy()
            .to_string()
    };
    let tag = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 4))
            .to_string_lossy()
            .to_string()
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.add_route_tag(&method, &path, &tag);
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_add_constraint(server, handler_name, param_name, pattern) - add route param constraint
ring_func!(bolt_add_constraint, |p| {
    ring_check_paracount!(p, 4);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);
    ring_check_string!(p, 4);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let handler_name = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 2))
            .to_string_lossy()
            .to_string()
    };
    let param_name = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 3))
            .to_string_lossy()
            .to_string()
    };
    let pattern = unsafe {
        std::ffi::CStr::from_ptr(ring_api_getstring(p, 4))
            .to_string_lossy()
            .to_string()
    };

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.add_constraint(&handler_name, &param_name, &pattern);
    }

    ring_ret_number!(p, 1.0);
});

pub(crate) fn generate_openapi_spec(
    routes: &[RouteDefinition],
    title: &str,
    version: &str,
    description: &str,
) -> String {
    use utoipa::openapi::{
        Info, OpenApiBuilder, PathsBuilder, ResponseBuilder, ResponsesBuilder,
        path::{HttpMethod, OperationBuilder, PathItem},
    };

    let mut paths_builder = PathsBuilder::new();

    for route in routes {
        let path_key = convert_path_params(&route.path);

        let mut op_builder = OperationBuilder::new()
            .summary(Some(format!("{} {}", route.method, route.path)))
            .responses(
                ResponsesBuilder::new()
                    .response("200", ResponseBuilder::new().description("Success").build())
                    .build(),
            );

        if let Some(ref desc) = route.description {
            op_builder = op_builder.description(Some(desc.clone()));
        }

        if !route.tags.is_empty() {
            op_builder = op_builder.tags(Some(route.tags.clone()));
        }

        let operation = op_builder.build();

        let http_method = match route.method.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => continue,
        };

        let path_item = PathItem::new(http_method, operation);
        paths_builder = paths_builder.path(path_key, path_item);
    }

    let mut info = Info::new(title, version);
    if !description.is_empty() {
        info.description = Some(description.to_string());
    }

    let openapi = OpenApiBuilder::new()
        .info(info)
        .paths(paths_builder.build())
        .build();

    openapi.to_json().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_openapi_spec_empty_routes() {
        let spec = generate_openapi_spec(&[], "Test API", "1.0.0", "A test API");
        assert!(spec.contains("\"Test API\""));
        assert!(spec.contains("\"1.0.0\""));
        assert!(spec.contains("\"A test API\""));
        assert!(spec.contains("\"paths\""));
    }

    #[test]
    fn test_generate_openapi_spec_single_route() {
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}".into(),
            handler_name: "get_user".into(),
            description: Some("Get user by ID".into()),
            tags: vec!["users".into()],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let spec = generate_openapi_spec(&routes, "My API", "2.0", "");
        assert!(spec.contains("\"/users/{id}\""));
        assert!(spec.contains("\"get\""));
        assert!(spec.contains("\"Get user by ID\""));
        assert!(spec.contains("\"users\""));
        assert!(spec.contains("\"My API\""));
        assert!(spec.contains("\"2.0\""));
    }

    #[test]
    fn test_generate_openapi_spec_multiple_routes() {
        let routes = [
            RouteDefinition {
                method: "GET".into(),
                path: "/users".into(),
                handler_name: "list_users".into(),
                description: None,
                tags: vec!["users".into()],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
            RouteDefinition {
                method: "POST".into(),
                path: "/users".into(),
                handler_name: "create_user".into(),
                description: Some("Create a new user".into()),
                tags: vec!["users".into(), "admin".into()],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            },
        ];

        let spec = generate_openapi_spec(&routes, "API", "1.0", "desc");
        assert!(spec.contains("\"/users\""));
        assert!(spec.contains("\"get\""));
        assert!(spec.contains("\"post\""));
        assert!(spec.contains("\"Create a new user\""));
    }

    #[test]
    fn test_generate_openapi_spec_unknown_method_skipped() {
        let routes = [RouteDefinition {
            method: "TRACE".into(),
            path: "/trace".into(),
            handler_name: "trace_handler".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let spec = generate_openapi_spec(&routes, "API", "1.0", "");
        assert!(!spec.contains("\"/trace\""));
    }

    #[test]
    fn test_generate_openapi_spec_all_methods() {
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        let mut routes = Vec::new();
        for method in &methods {
            routes.push(RouteDefinition {
                method: method.to_string(),
                path: format!("/{}", method.to_lowercase()),
                handler_name: format!("handler_{}", method.to_lowercase()),
                description: None,
                tags: vec![],
                constraints: vec![],
                rate_limit: None,
                before_middleware: vec![],
                after_middleware: vec![],
            });
        }

        let spec = generate_openapi_spec(&routes, "API", "1.0", "");
        for method in ["get", "post", "put", "delete", "patch", "head", "options"] {
            assert!(
                spec.contains(&format!("\"{}\"", method)),
                "Missing method: {}",
                method
            );
        }
    }

    #[test]
    fn test_generate_openapi_spec_route_without_description() {
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/health".into(),
            handler_name: "health".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let spec = generate_openapi_spec(&routes, "API", "1.0", "");
        assert!(spec.contains("\"/health\""));
    }

    #[test]
    fn test_generate_openapi_spec_route_without_tags() {
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/items".into(),
            handler_name: "list_items".into(),
            description: Some("List all items".into()),
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let spec = generate_openapi_spec(&routes, "API", "1.0", "");
        assert!(spec.contains("\"List all items\""));
    }

    #[test]
    fn test_convert_path_params_in_openapi() {
        let routes = [RouteDefinition {
            method: "GET".into(),
            path: "/users/{id}/posts/{pid}".into(),
            handler_name: "get_post".into(),
            description: None,
            tags: vec![],
            constraints: vec![],
            rate_limit: None,
            before_middleware: vec![],
            after_middleware: vec![],
        }];

        let spec = generate_openapi_spec(&routes, "API", "1.0", "");
        assert!(spec.contains("\"/users/{id}/posts/{pid}\""));
    }
}
