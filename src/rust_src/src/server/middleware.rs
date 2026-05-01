// Bolt Framework
// A blazing-fast HTTP framework for Ring
// Copyright (c) 2026, Youssef Saeed

//! Middleware (Error Handler + Per-Route + Global Before/After)

use ring_lang_rs::*;

use crate::HTTP_SERVER_TYPE;

use super::HttpServer;

/// bolt_set_error_handler(server, handler_name) - set global error handler
ring_func!(bolt_set_error_handler, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "set error handler: invalid server pointer");
        return;
    }

    let handler_name = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.error_handler = Some(handler_name.to_string());
    }

    // Define ringvm_errorhandler() in Ring so the VM calls it on
    // unhandled errors instead of calling ring_vm_shutdown (exit).
    // cCatchError holds the error message; ringvm_passerror() tells
    // the VM the error was handled and to continue execution.
    let vm = p as RingVM;
    let code = format!(
        "func ringvm_errorhandler\n{}\nringvm_passerror()\n",
        handler_name
    );
    ring_vm_runcode_str(vm, &code);

    ring_ret_number!(p, 1.0);
});

/// bolt_route_before(server, handler_name, middleware_name) - add per-route before middleware
ring_func!(bolt_route_before, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "route before middleware: invalid server pointer");
        return;
    }

    let handler_name = ring_get_string!(p, 2);
    let middleware = ring_get_string!(p, 3);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        for route in &mut server.routes {
            if route.handler_name == handler_name {
                route.before_middleware.push(middleware.to_string());
                break;
            }
        }
    }
    ring_ret_number!(p, 1.0);
});

/// bolt_route_after(server, handler_name, middleware_name) - add per-route after middleware
ring_func!(bolt_route_after, |p| {
    ring_check_paracount!(p, 3);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);
    ring_check_string!(p, 3);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        ring_error!(p, "route after middleware: invalid server pointer");
        return;
    }

    let handler_name = ring_get_string!(p, 2);
    let middleware = ring_get_string!(p, 3);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        for route in &mut server.routes {
            if route.handler_name == handler_name {
                route.after_middleware.push(middleware.to_string());
                break;
            }
        }
    }
    ring_ret_number!(p, 1.0);
});

/// bolt_before(server, handler) - add before middleware (runs before each request)
ring_func!(bolt_before, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let handler = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.before_handlers.push(handler.to_string());
    }

    ring_ret_number!(p, 1.0);
});

/// bolt_after(server, handler) - add after middleware (runs after each request)
ring_func!(bolt_after, |p| {
    ring_check_paracount!(p, 2);
    ring_check_cpointer!(p, 1);
    ring_check_string!(p, 2);

    let ptr = ring_api_getcpointer(p, 1, HTTP_SERVER_TYPE);
    if ptr.is_null() {
        return;
    }

    let handler = ring_get_string!(p, 2);

    unsafe {
        let server = &mut *(ptr as *mut HttpServer);
        server.after_handlers.push(handler.to_string());
    }

    ring_ret_number!(p, 1.0);
});
