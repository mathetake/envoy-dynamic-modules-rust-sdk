#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ptr;

mod abi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Define the init function for the module.
/// This macro should be used in the root of the module.
///
/// ## Arguments
///
/// * `$new_filter_fn` - The function that creates a new HttpFilter object: `fn(&str) -> Box<dyn HttpFilter>`.
///  This function is called for each new filter chain configuration and should return a new HttpFilter object
///  based on the configuration string.
///
/// ## Example
///
/// ```
/// use envoy_dynamic_modules_rust_sdk::*;
///
/// struct HelloWorldFilter {}
/// struct HelloWorldFilterInstance {}
///
/// impl HttpFilter for HelloWorldFilter {
///    fn new_instance(&mut self, _envoy_filter_instance: EnvoyFilterInstance) -> Box<dyn HttpFilterInstance> {
///       Box::new(HelloWorldFilterInstance {})
///   }
/// }
///
/// impl HttpFilterInstance for HelloWorldFilterInstance {}
///
/// fn new_http_filter(config: &str) -> Box<dyn HttpFilter> {
///    match config {
///       "helloworld" => Box::new(HelloWorldFilter {}),
///      _ => panic!("Unknown config: {}", config),
///    }
/// }
/// init!(new_http_filter);
#[macro_export]
macro_rules! init {
    ($new_filter_fn:expr) => {
        #[no_mangle]
        pub extern "C" fn __envoy_dynamic_module_v1_event_program_init() -> usize {
            unsafe {
                envoy_dynamic_modules_rust_sdk::NEW_HTTP_FILTER_FN = $new_filter_fn;
            }
            0
        }
    };
}

pub static mut NEW_HTTP_FILTER_FN: fn(&str) -> Box<dyn HttpFilter> = |_: &str| {
    panic!("NEW_HTTP_FILTER_FN is not set");
};

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_init(
    config_ptr: abi::__envoy_dynamic_module_v1_type_HttpFilterConfigPtr,
    config_size: abi::__envoy_dynamic_module_v1_type_HttpFilterConfigSize,
) -> abi::__envoy_dynamic_module_v1_type_HttpFilterPtr {
    // Convert the raw pointer to the str.
    let config = {
        let slice = std::slice::from_raw_parts(config_ptr as *const u8, config_size);
        std::str::from_utf8(slice).unwrap()
    };

    let boxed_filter = Box::into_raw(NEW_HTTP_FILTER_FN(config));
    let boxed_filter_ptr = Box::into_raw(Box::new(boxed_filter));
    boxed_filter_ptr as abi::__envoy_dynamic_module_v1_type_HttpFilterPtr
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_destroy(
    http_filter: abi::__envoy_dynamic_module_v1_type_HttpFilterPtr,
) {
    let http_filter = http_filter as *mut *mut dyn HttpFilter;
    (**http_filter).destroy();

    // Drop the Box<dyn HttpFilter> and the Box<*mut dyn HttpFilter>
    let _outer = Box::from_raw(http_filter);
    let _inner = Box::from_raw(*http_filter);
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_init(
    envoy_filter_instance_ptr: abi::__envoy_dynamic_module_v1_type_EnvoyFilterInstancePtr,
    http_filter: abi::__envoy_dynamic_module_v1_type_HttpFilterPtr,
) -> abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr {
    let http_filter = http_filter as *mut *mut dyn HttpFilter;

    let instance = {
        let instance_boxed = (**http_filter).new_instance(EnvoyFilterInstance {
            raw_addr: envoy_filter_instance_ptr,
        });
        Box::into_raw(instance_boxed)
    };

    let http_filter_instance = Box::into_raw(Box::new(instance));
    http_filter_instance as abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_request_headers(
    http_filter_instance: abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr,
    request_headers_ptr: abi::__envoy_dynamic_module_v1_type_HttpRequestHeadersMapPtr,
    end_of_stream: abi::__envoy_dynamic_module_v1_type_EndOfStream,
) -> abi::__envoy_dynamic_module_v1_type_EventHttpRequestHeadersStatus {
    let http_filter_instance = http_filter_instance as *mut *mut dyn HttpFilterInstance;
    let http_filter_instance = &mut **http_filter_instance;
    http_filter_instance
        .request_headers(
            &RequestHeaders {
                raw: request_headers_ptr,
            },
            end_of_stream == 1,
        )
        .into()
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_request_body(
    http_filter_instance: abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr,
    buffer: abi::__envoy_dynamic_module_v1_type_HttpRequestBodyBufferPtr,
    end_of_stream: abi::__envoy_dynamic_module_v1_type_EndOfStream,
) -> abi::__envoy_dynamic_module_v1_type_EventHttpRequestBodyStatus {
    let http_filter_instance = http_filter_instance as *mut *mut dyn HttpFilterInstance;
    let http_filter_instance = &mut **http_filter_instance;
    http_filter_instance
        .request_body(&RequestBodyBuffer { raw: buffer }, end_of_stream == 1)
        .into()
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_response_headers(
    http_filter_instance: abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr,
    response_headers_map_ptr: abi::__envoy_dynamic_module_v1_type_HttpResponseHeaderMapPtr,
    end_of_stream: abi::__envoy_dynamic_module_v1_type_EndOfStream,
) -> abi::__envoy_dynamic_module_v1_type_EventHttpResponseHeadersStatus {
    let http_filter_instance = http_filter_instance as *mut *mut dyn HttpFilterInstance;
    let http_filter_instance = &mut **http_filter_instance;
    http_filter_instance
        .response_headers(
            &ResponseHeaders {
                raw: response_headers_map_ptr,
            },
            end_of_stream == 1,
        )
        .into()
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_response_body(
    http_filter_instance: abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr,
    buffer: abi::__envoy_dynamic_module_v1_type_HttpResponseBodyBufferPtr,
    end_of_stream: abi::__envoy_dynamic_module_v1_type_EndOfStream,
) -> abi::__envoy_dynamic_module_v1_type_EventHttpResponseBodyStatus {
    let http_filter_instance = http_filter_instance as *mut *mut dyn HttpFilterInstance;
    let http_filter_instance = &mut **http_filter_instance;
    http_filter_instance
        .response_body(&ResponseBodyBuffer { raw: buffer }, end_of_stream == 1)
        .into()
}

#[no_mangle]
unsafe extern "C" fn __envoy_dynamic_module_v1_event_http_filter_instance_destroy(
    http_filter_instance: abi::__envoy_dynamic_module_v1_type_HttpFilterInstancePtr,
) {
    let http_filter_instance = http_filter_instance as *mut *mut dyn HttpFilterInstance;
    (**http_filter_instance).destroy();

    let _outer = Box::from_raw(http_filter_instance);
    let _inner = Box::from_raw(&mut **http_filter_instance);
}

/// A trait that represents a single HTTP filter in the Envoy filter chain.
/// It is used to create HttpFilterInstance(s) that correspond to each HTTP request.
///
/// This is only created once per module instance via the new_http_filter function.
pub trait HttpFilter {
    /// This is called for each new HTTP request. This should return a new HttpFilterInstance object to handle the request.
    ///
    /// Note that this must be concurrency-safe as it can be called concurrently for multiple requests.
    ///
    /// * `envoy_filter_instance` is the Envoy filter object that is used to interact with the underlying Envoy filter.
    ///   This object is unique for each HTTP request. The object is destroyed when the stream is destroyed.
    ///   Therefore, after event_http_destroy is called, the methods on this object become no-op.
    fn new_instance(
        &mut self,
        envoy_filter_instance: EnvoyFilterInstance,
    ) -> Box<dyn HttpFilterInstance>;

    /// destroy is called when this filter is destroyed. E.g. the filter chain configuration is updated and removed from the Envoy.
    ///
    /// After this returns, the filter object is destructed.
    fn destroy(&self) {}
}

/// HttpFilterInstance is a trait that represents each HTTP request.
///
/// This is created for each new HTTP request and is destroyed when the request is completed.
pub trait HttpFilterInstance {
    /// This is called when request headers are received.
    /// The function should return the status of the operation.
    ///
    /// * `request_headers` is the reference to the request headers map.
    /// * `end_of_stream` is a boolean that indicates if this is the headers-only request.
    fn request_headers(
        &mut self,
        _request_headers: &RequestHeaders,
        _end_of_stream: bool,
    ) -> EventHttpRequestHeadersStatus {
        EventHttpRequestHeadersStatus::Continue
    }

    /// This is called when request body data is received.
    /// The function should return the status of the operation.
    ///
    /// * `request_body` is the reference to the newly arrived request body buffer.
    /// * `end_of_stream` is a boolean that indicates if this is the last data frame.
    fn request_body(
        &mut self,
        _request_body: &RequestBodyBuffer,
        _end_of_stream: bool,
    ) -> EventHttpRequestBodyStatus {
        EventHttpRequestBodyStatus::Continue
    }

    /// This is called when response headers are received.
    /// The function should return the status of the operation.
    ///
    /// * `response_headers` is the reference to the response headers map.
    /// * `end_of_stream` is a boolean that indicates if this is the headers-only response.
    fn response_headers(
        &mut self,
        _response_headers: &ResponseHeaders,
        _end_of_stream: bool,
    ) -> EventHttpResponseHeadersStatus {
        EventHttpResponseHeadersStatus::Continue
    }

    /// This is called when response body data is received.
    /// The function should return the status of the operation.
    ///
    /// * `response_body` is the reference to the newly arrived response body buffer.
    /// * `end_of_stream` is a boolean that indicates if this is the last data frame.
    fn response_body(
        &mut self,
        _response_body: &ResponseBodyBuffer,
        _end_of_stream: bool,
    ) -> EventHttpResponseBodyStatus {
        EventHttpResponseBodyStatus::Continue
    }

    /// This is called when the stream is completed or when the stream is reset.
    ///
    /// After this returns, this object is destructed.
    fn destroy(&mut self) {}
}

/// An opaque object that represents the underlying Envoy Http filter instance.
/// This is used to interact with it from the module code.
///
/// This is a shallow wrapper around the raw pointer to the Envoy filter instance.
/// Can be copied and stored somewhere else. However, the object MUST NOT be used after the
/// [`HttpFilterInstance::destroy`] for the corresponding filter instance is called.
///
#[derive(Debug, Clone, Copy)]
pub struct EnvoyFilterInstance {
    raw_addr: abi::__envoy_dynamic_module_v1_type_EnvoyFilterInstancePtr,
}

impl EnvoyFilterInstance {
    /// continue_request is used to resume the request processing after the filter has stopped it.
    pub fn continue_request(&self) {
        unsafe { abi::__envoy_dynamic_module_v1_http_continue_request(self.raw_addr) }
    }

    /// continue_response is used to resume the response processing after the filter has stopped it.
    pub fn continue_response(&self) {
        unsafe { abi::__envoy_dynamic_module_v1_http_continue_response(self.raw_addr) }
    }
}

/// An opaque object that represents the underlying Envoy Http request headers map.
/// This is used to interact with it from the module code.
///
/// This is a shallow wrapper around the raw pointer to the Envoy request headers map.
/// However, the object MUST NOT be used after the [`HttpFilterInstance::request_headers`].
pub struct RequestHeaders {
    raw: abi::__envoy_dynamic_module_v1_type_HttpRequestHeadersMapPtr,
}

impl RequestHeaders {
    /// Returns the first header value for the given key. To handle multiple values, use the [`RequestHeaders::Values`] method.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        let mut result_ptr: *const u8 = ptr::null();
        let mut result_size: usize = 0;

        let total = unsafe {
            abi::__envoy_dynamic_module_v1_http_get_request_header_value(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                &mut result_ptr as *mut _ as usize,
                &mut result_size as *mut _ as usize,
            )
        };

        if total == 0 {
            return None;
        }

        // Convert the result to a Rust string slice
        let result_slice = unsafe { std::slice::from_raw_parts(result_ptr, result_size) };
        let result_str = std::str::from_utf8(result_slice).unwrap();

        Some(result_str)
    }

    /// Returns all the header values for the given key.
    pub fn values(&self, key: &str) -> Vec<&str> {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        let mut result_ptr: *const u8 = ptr::null();
        let mut result_size: usize = 0;

        let mut values = Vec::new();
        let total = unsafe {
            abi::__envoy_dynamic_module_v1_http_get_request_header_value(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                &mut result_ptr as *mut _ as usize,
                &mut result_size as *mut _ as usize,
            )
        };

        if total == 0 {
            return values;
        }

        values = Vec::with_capacity(total as usize);
        values.push(unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(result_ptr, result_size)).unwrap()
        });

        for i in 1..total {
            unsafe {
                abi::__envoy_dynamic_module_v1_http_get_request_header_value_nth(
                    self.raw,
                    key_ptr as *const _ as usize,
                    key_size,
                    &mut result_ptr as *mut _ as usize,
                    &mut result_size as *mut _ as usize,
                    i,
                );
            }
            values.push(unsafe {
                std::str::from_utf8(std::slice::from_raw_parts(result_ptr, result_size)).unwrap()
            });
        }

        values
    }

    /// Sets the value for the given key. If multiple values are set for the same key,
    /// this removes all the previous values and sets the new single value.
    pub fn set(&self, key: &str, value: &str) {
        let key_ptr = key.as_ptr();
        let key_size = key.len();
        let value_ptr = value.as_ptr();
        let value_size = value.len();

        unsafe {
            abi::__envoy_dynamic_module_v1_http_set_request_header(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                value_ptr as *const _ as usize,
                value_size,
            )
        }
    }

    /// Removes the value for the given key. If multiple values are set for the same key,
    /// this removes all the values.
    pub fn remove(&self, key: &str) {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        unsafe {
            abi::__envoy_dynamic_module_v1_http_set_request_header(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                0,
                0,
            )
        }
    }
}

/// An opaque object that represents the underlying Envoy Http request body buffer.
/// This is used to interact with it from the module code.
pub struct RequestBodyBuffer {
    raw: abi::__envoy_dynamic_module_v1_type_HttpRequestBodyBufferPtr,
}

/// An opaque object that represents the underlying Envoy Http response headers map.
/// This is used to interact with it from the module code.
pub struct ResponseHeaders {
    raw: abi::__envoy_dynamic_module_v1_type_HttpResponseHeaderMapPtr,
}

impl ResponseHeaders {
    /// Returns the first header value for the given key. To handle multiple values, use the [`ResponseHeaders::Values`] method.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        let mut result_ptr: *const u8 = ptr::null();
        let mut result_size: usize = 0;

        let total = unsafe {
            abi::__envoy_dynamic_module_v1_http_get_response_header_value(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                &mut result_ptr as *mut _ as usize,
                &mut result_size as *mut _ as usize,
            )
        };

        if total == 0 {
            return None;
        }

        // Convert the result to a Rust string slice
        let result_slice = unsafe { std::slice::from_raw_parts(result_ptr, result_size) };
        let result_str = std::str::from_utf8(result_slice).unwrap();

        Some(result_str)
    }

    /// Returns all the header values for the given key.
    pub fn values(&self, key: &str) -> Vec<&str> {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        let mut result_ptr: *const u8 = ptr::null();
        let mut result_size: usize = 0;

        let mut values = Vec::new();
        let total = unsafe {
            abi::__envoy_dynamic_module_v1_http_get_response_header_value(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                &mut result_ptr as *mut _ as usize,
                &mut result_size as *mut _ as usize,
            )
        };

        if total == 0 {
            return values;
        }

        values = Vec::with_capacity(total as usize);
        values.push(unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(result_ptr, result_size)).unwrap()
        });

        for i in 1..total {
            unsafe {
                abi::__envoy_dynamic_module_v1_http_get_response_header_value_nth(
                    self.raw,
                    key_ptr as *const _ as usize,
                    key_size,
                    &mut result_ptr as *mut _ as usize,
                    &mut result_size as *mut _ as usize,
                    i,
                );
            }
            values.push(unsafe {
                std::str::from_utf8(std::slice::from_raw_parts(result_ptr, result_size)).unwrap()
            });
        }

        values
    }

    /// Sets the value for the given key. If multiple values are set for the same key,
    pub fn set(&self, key: &str, value: &str) {
        let key_ptr = key.as_ptr();
        let key_size = key.len();
        let value_ptr = value.as_ptr();
        let value_size = value.len();

        unsafe {
            abi::__envoy_dynamic_module_v1_http_set_response_header(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                value_ptr as *const _ as usize,
                value_size,
            )
        }
    }

    /// Removes the value for the given key. If multiple values are set for the same key,
    pub fn remove(&self, key: &str) {
        let key_ptr = key.as_ptr();
        let key_size = key.len();

        unsafe {
            abi::__envoy_dynamic_module_v1_http_set_response_header(
                self.raw,
                key_ptr as *const _ as usize,
                key_size,
                0,
                0,
            )
        }
    }
}

/// An opaque object that represents the underlying Envoy Http response body buffer.
/// This is used to interact with it from the module code.
pub struct ResponseBodyBuffer {
    raw: abi::__envoy_dynamic_module_v1_type_HttpResponseBodyBufferPtr,
}

/// The status of the processing after the [`HttpFilterInstance::request_headers`] is called.
pub enum EventHttpRequestHeadersStatus {
    /// Should be returned when the operation should continue.
    Continue,
    /// This indicates that Envoy shouldn't continue from processing the headers and should
    /// stop filter iteration. In other words, [`HttpFilterInstance::request_body`]
    /// will be called while not sending headers to the upstream. The header processing can be
    /// resumed by either calling [`EnvoyFilterInstance::continue_request`], or returns
    /// continue status from the [`HttpFilterInstance::request_body`].
    StopIteration,
    _reserved,
    /// This indicates that Envoy should stop all iteration and continue to buffer the request body
    /// until the limit is reached. When the limit is reached, Envoy will stop buffering and returns 500
    /// to the client. This means that [`HttpFilterInstance::request_body`] will not be called.
    ///
    /// The header processing can be resumed by either calling [`EnvoyFilterInstance::continue_request`], or
    /// returns continue status from the [`HttpFilterInstance::request_body`].
    StopAllIterationAndBuffer,
}

impl From<EventHttpRequestHeadersStatus>
    for abi::__envoy_dynamic_module_v1_type_EventHttpRequestHeadersStatus
{
    fn from(val: EventHttpRequestHeadersStatus) -> Self {
        match val {
            EventHttpRequestHeadersStatus::Continue => {
                abi::__envoy_dynamic_module_v1_type_EventHttpRequestHeadersStatusContinue
            }
            EventHttpRequestHeadersStatus::StopIteration => {
                abi::__envoy_dynamic_module_v1_type_EventHttpRequestHeadersStatusStopIteration
            }
            EventHttpRequestHeadersStatus::StopAllIterationAndBuffer => {
                abi::__envoy_dynamic_module_v1_type_EventHttpRequestHeadersStatusStopAllIterationAndBuffer
            }
            _ => {
                panic!("Invalid EventHttpRequestHeadersStatus")
            }
        }
    }
}

/// The status of the processing after the [`HttpFilterInstance::response_headers`] is called.
pub enum EventHttpRequestBodyStatus {
    /// Should be returned when the operation should continue.
    Continue,
    /// This indicates that Envoy shouldn't continue from processing the body frame and should stop iteration,
    /// but continue buffering the body until the limit is reached. When the limit is reached,
    /// Envoy will stop buffering and returns 500 to the client.
    ///
    /// This stops sending body data to the upstream, so if the module wants to continue sending body
    /// data, it should call [`EnvoyFilterInstance::continue_request`] or return continue status in the
    /// subsequent [`HttpFilterInstance::request_body`] calls.
    StopIterationAndBuffer,
}

impl From<EventHttpRequestBodyStatus>
    for abi::__envoy_dynamic_module_v1_type_EventHttpRequestBodyStatus
{
    fn from(val: EventHttpRequestBodyStatus) -> Self {
        match val {
            EventHttpRequestBodyStatus::Continue => {
                abi::__envoy_dynamic_module_v1_type_EventHttpRequestBodyStatusContinue
            }
            EventHttpRequestBodyStatus::StopIterationAndBuffer => {
                abi::__envoy_dynamic_module_v1_type_EventHttpRequestBodyStatusStopIterationAndBuffer
            }
        }
    }
}

/// The status of the processing after the [`HttpFilterInstance::response_headers`] is called.
pub enum EventHttpResponseHeadersStatus {
    /// Should be returned when the operation should continue.
    Continue,
    /// This indicates that Envoy shouldn't continue from processing the headers and should
    /// stop filter iteration. In other words, [`HttpFilterInstance::response_body`]
    /// will be called while not sending headers to the upstream. The header processing can be
    /// resumed by either calling [`EnvoyFilterInstance::continue_response`], or returns
    /// continue status from the [`HttpFilterInstance::response_body`].
    StopIteration,
    _reserved,
    /// This indicates that Envoy should stop all iteration and continue to buffer the response body
    /// until the limit is reached. When the limit is reached, Envoy will stop buffering and returns 500
    /// to the client. This means that [`HttpFilterInstance::response_body`] will not be called.
    ///
    /// The header processing can be resumed by either calling [`EnvoyFilterInstance::continue_response`], or
    /// returns continue status from the [`HttpFilterInstance::response_body`].
    StopAllIterationAndBuffer,
}

impl From<EventHttpResponseHeadersStatus>
    for abi::__envoy_dynamic_module_v1_type_EventHttpResponseHeadersStatus
{
    fn from(val: EventHttpResponseHeadersStatus) -> Self {
        match val {
            EventHttpResponseHeadersStatus::Continue => {
                abi::__envoy_dynamic_module_v1_type_EventHttpResponseHeadersStatusContinue
            }
            EventHttpResponseHeadersStatus::StopIteration => {
                abi::__envoy_dynamic_module_v1_type_EventHttpResponseHeadersStatusStopIteration
            }
            EventHttpResponseHeadersStatus::StopAllIterationAndBuffer => {
                abi::__envoy_dynamic_module_v1_type_EventHttpResponseHeadersStatusStopAllIterationAndBuffer
            }
            _ => {
                panic!("Invalid EventHttpResponseHeadersStatus")
            }
        }
    }
}

/// The status of the processing after the [`HttpFilterInstance::response_body`] is called.
pub enum EventHttpResponseBodyStatus {
    /// Should be returned when the operation should continue.
    Continue,
    /// This indicates that Envoy shouldn't continue from processing the body frame and should stop iteration,
    /// but continue buffering the body until the limit is reached. When the limit is reached,
    /// Envoy will stop buffering and returns 500 to the client.
    ///
    /// This stops sending body data to the upstream, so if the module wants to continue sending body
    /// data, it should call [`EnvoyFilterInstance::continue_responses`] or return continue status in the
    /// subsequent [`HttpFilterInstance::response_body`] calls.
    StopIterationAndBuffer,
}

impl From<EventHttpResponseBodyStatus>
    for abi::__envoy_dynamic_module_v1_type_EventHttpResponseBodyStatus
{
    fn from(val: EventHttpResponseBodyStatus) -> Self {
        match val {
            EventHttpResponseBodyStatus::Continue => {
                abi::__envoy_dynamic_module_v1_type_EventHttpResponseBodyStatusContinue
            }
            EventHttpResponseBodyStatus::StopIterationAndBuffer => {
                abi::__envoy_dynamic_module_v1_type_EventHttpResponseBodyStatusStopIterationAndBuffer
            }
        }
    }
}
