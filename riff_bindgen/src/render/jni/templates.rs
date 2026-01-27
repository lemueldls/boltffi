use std::fmt::{self, Display, Formatter};

use askama::Template;

use super::plan::{
    JniAsyncCallbackInvoker, JniAsyncFunction, JniCallbackTrait, JniClass, JniClosureTrampoline,
    JniFunction, JniModule, JniParam, JniReturnAbi, JniWireCtor, JniWireFunction, JniWireMethod,
};

#[derive(Template)]
#[template(path = "render/jni/templates/jni_glue.txt", escape = "none")]
pub struct JniGlueTemplate<'a> {
    pub prefix: &'a str,
    pub jni_prefix: &'a str,
    pub package_path: &'a str,
    pub module_name: &'a str,
    pub class_name: &'a str,
    pub has_async: bool,
    pub has_async_callbacks: bool,
    pub functions: &'a [JniFunction],
    pub wire_functions: &'a [JniWireFunction],
    pub async_functions: &'a [JniAsyncFunction],
    pub classes: &'a [JniClass],
    pub callback_traits: &'a [JniCallbackTrait],
    pub async_callback_invokers: &'a [JniAsyncCallbackInvoker],
    pub closure_trampolines: &'a [JniClosureTrampoline],
}

impl<'a> JniGlueTemplate<'a> {
    pub fn new(module: &'a JniModule) -> Self {
        Self {
            prefix: module.prefix.as_str(),
            jni_prefix: module.jni_prefix.as_str(),
            package_path: module.package_path.as_str(),
            module_name: module.module_name.as_str(),
            class_name: module.class_name.as_str(),
            has_async: module.has_async,
            has_async_callbacks: module.has_async_callbacks,
            functions: &module.functions,
            wire_functions: &module.wire_functions,
            async_functions: &module.async_functions,
            classes: &module.classes,
            callback_traits: &module.callback_traits,
            async_callback_invokers: &module.async_callback_invokers,
            closure_trampolines: &module.closure_trampolines,
        }
    }
}

#[derive(Template)]
#[template(path = "render/jni/templates/jni_wire_function.txt", escape = "none")]
pub struct JniWireFunctionTemplate<'a> {
    pub ffi_name: &'a str,
    pub jni_name: &'a str,
    pub jni_params: &'a str,
    pub params: &'a [JniParam],
    pub return_abi: &'a JniReturnAbi,
}

impl<'a> JniWireFunctionTemplate<'a> {
    pub fn new(func: &'a JniWireFunction) -> Self {
        Self {
            ffi_name: func.ffi_name.as_str(),
            jni_name: func.jni_name.as_str(),
            jni_params: func.jni_params.as_str(),
            params: &func.params,
            return_abi: &func.return_abi,
        }
    }
}

impl Display for JniWireFunction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        JniWireFunctionTemplate::new(self)
            .render()
            .map_err(|_| fmt::Error)
            .and_then(|rendered| formatter.write_str(&rendered))
    }
}

#[derive(Template)]
#[template(path = "render/jni/templates/jni_wire_method.txt", escape = "none")]
pub struct JniWireMethodTemplate<'a> {
    pub ffi_name: &'a str,
    pub jni_name: &'a str,
    pub jni_params: &'a str,
    pub params: &'a [JniParam],
    pub return_abi: &'a JniReturnAbi,
    pub include_handle: bool,
}

impl<'a> JniWireMethodTemplate<'a> {
    pub fn new(method: &'a JniWireMethod) -> Self {
        Self {
            ffi_name: method.ffi_name.as_str(),
            jni_name: method.jni_name.as_str(),
            jni_params: method.jni_params.as_str(),
            params: &method.params,
            return_abi: &method.return_abi,
            include_handle: method.include_handle,
        }
    }
}

impl Display for JniWireMethod {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        JniWireMethodTemplate::new(self)
            .render()
            .map_err(|_| fmt::Error)
            .and_then(|rendered| formatter.write_str(&rendered))
    }
}

#[derive(Template)]
#[template(path = "render/jni/templates/jni_wire_ctor.txt", escape = "none")]
pub struct JniWireCtorTemplate<'a> {
    pub ffi_name: &'a str,
    pub jni_name: &'a str,
    pub jni_params: &'a str,
    pub params: &'a [JniParam],
}

impl<'a> JniWireCtorTemplate<'a> {
    pub fn new(ctor: &'a JniWireCtor) -> Self {
        Self {
            ffi_name: ctor.ffi_name.as_str(),
            jni_name: ctor.jni_name.as_str(),
            jni_params: ctor.jni_params.as_str(),
            params: &ctor.params,
        }
    }
}

impl Display for JniWireCtor {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        JniWireCtorTemplate::new(self)
            .render()
            .map_err(|_| fmt::Error)
            .and_then(|rendered| formatter.write_str(&rendered))
    }
}
