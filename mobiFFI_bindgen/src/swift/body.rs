use askama::Template;

use crate::model::{Class, Enumeration, Method, Module, Record, StreamMethod};

use super::names::NamingConvention;
use super::templates::{StreamBodyTemplate, SyncMethodBodyTemplate};
use super::types::TypeMapper;

pub struct BodyRenderer;

impl BodyRenderer {
    pub fn render_method(method: &Method, class: &Class, module: &Module) -> String {
        let class_prefix = class.ffi_prefix(&module.ffi_prefix());

        match (method.is_async, method.throws()) {
            (true, true) => Self::async_throwing_body(method, &class_prefix),
            (true, false) => Self::async_body(method, &class_prefix),
            (false, true) => Self::throwing_body(method, &class_prefix),
            (false, false) => SyncMethodBodyTemplate::from_method(method, class, module)
                .render()
                .expect("sync method template failed"),
        }
    }

    pub fn render_stream(stream: &StreamMethod, class: &Class, module: &Module) -> String {
        StreamBodyTemplate::from_stream(stream, class, module)
            .render()
            .expect("stream body template failed")
    }

    pub fn render_record(record: &Record) -> String {
        let name = NamingConvention::class_name(&record.name);
        let fields = record
            .fields
            .iter()
            .map(|field| {
                format!(
                    "    public var {}: {}",
                    NamingConvention::property_name(&field.name),
                    TypeMapper::map_type(&field.field_type)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let init_params = record
            .fields
            .iter()
            .map(|field| {
                format!(
                    "{}: {}",
                    NamingConvention::param_name(&field.name),
                    TypeMapper::map_type(&field.field_type)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let init_body = record
            .fields
            .iter()
            .map(|field| {
                let prop = NamingConvention::property_name(&field.name);
                let param = NamingConvention::param_name(&field.name);
                format!("        self.{prop} = {param}")
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"public struct {name}: Equatable {{
{fields}

    public init({init_params}) {{
{init_body}
    }}
}}"#
        )
    }

    pub fn render_enum(enumeration: &Enumeration) -> String {
        let name = NamingConvention::class_name(&enumeration.name);

        if enumeration.is_c_style() {
            Self::render_c_style_enum(enumeration, &name)
        } else {
            Self::render_data_enum(enumeration, &name)
        }
    }

    fn render_c_style_enum(enumeration: &Enumeration, name: &str) -> String {
        let cases = enumeration
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let case_name = NamingConvention::enum_case_name(&variant.name);
                let value = variant.discriminant.unwrap_or(index as i64);
                format!("    case {case_name} = {value}")
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"public enum {name}: Int32 {{
{cases}
}}"#
        )
    }

    fn render_data_enum(enumeration: &Enumeration, name: &str) -> String {
        let cases = enumeration
            .variants
            .iter()
            .map(|variant| {
                let case_name = NamingConvention::enum_case_name(&variant.name);
                if variant.is_unit() {
                    format!("    case {case_name}")
                } else {
                    let fields = variant
                        .fields
                        .iter()
                        .map(|field| {
                            let field_name = NamingConvention::param_name(&field.name);
                            let field_type = TypeMapper::map_type(&field.field_type);
                            format!("{field_name}: {field_type}")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("    case {case_name}({fields})")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"public enum {name} {{
{cases}
}}"#
        )
    }

    fn sync_body(method: &Method, class_prefix: &str) -> String {
        let ffi_name = method.ffi_name(class_prefix);
        let args = Self::build_ffi_args(method);
        let return_handling = Self::build_return_handling(method);

        format!("{return_handling}{ffi_name}(handle{args})")
    }

    fn throwing_body(method: &Method, class_prefix: &str) -> String {
        let ffi_name = method.ffi_name(class_prefix);
        let args = Self::build_ffi_args(method);
        let return_type = method
            .output
            .as_ref()
            .map(TypeMapper::map_type)
            .unwrap_or_else(|| "Void".into());

        format!(
            r#"var status = FfiStatus()
let result = {ffi_name}(handle{args}, &status)
if status.code != 0 {{
    throw FfiError(status: status)
}}
return result as {return_type}"#
        )
    }

    fn async_body(method: &Method, class_prefix: &str) -> String {
        let ffi_name = method.ffi_name(class_prefix);
        let ffi_poll = method.ffi_poll(class_prefix);
        let ffi_complete = method.ffi_complete(class_prefix);
        let ffi_cancel = method.ffi_cancel(class_prefix);
        let ffi_free = method.ffi_free(class_prefix);
        let args = Self::build_ffi_args(method);
        let return_type = method
            .output
            .as_ref()
            .map(TypeMapper::map_type)
            .unwrap_or_else(|| "Void".into());

        format!(
            r#"let futureHandle = {ffi_name}(handle{args})
defer {{ {ffi_free}(futureHandle) }}

return try await withTaskCancellationHandler {{
    try await withCheckedThrowingContinuation {{ (continuation: CheckedContinuation<{return_type}, Error>) in
        func poll() {{
            {ffi_poll}(futureHandle, Unmanaged.passRetained(Continuation(continuation)).toOpaque()) {{ userData, pollResult in
                let cont = Unmanaged<Continuation<{return_type}>>.fromOpaque(userData!).takeRetainedValue()
                if pollResult == 0 {{
                    let result = {ffi_complete}(futureHandle)
                    cont.resume(returning: result)
                }} else {{
                    poll()
                }}
            }}
        }}
        poll()
    }}
}} onCancel: {{
    {ffi_cancel}(futureHandle)
}}"#
        )
    }

    fn async_throwing_body(method: &Method, class_prefix: &str) -> String {
        let ffi_name = method.ffi_name(class_prefix);
        let ffi_poll = method.ffi_poll(class_prefix);
        let ffi_complete = method.ffi_complete(class_prefix);
        let ffi_cancel = method.ffi_cancel(class_prefix);
        let ffi_free = method.ffi_free(class_prefix);
        let args = Self::build_ffi_args(method);
        let return_type = method
            .output
            .as_ref()
            .map(TypeMapper::map_type)
            .unwrap_or_else(|| "Void".into());

        format!(
            r#"let futureHandle = {ffi_name}(handle{args})
defer {{ {ffi_free}(futureHandle) }}

return try await withTaskCancellationHandler {{
    try await withCheckedThrowingContinuation {{ (continuation: CheckedContinuation<{return_type}, Error>) in
        func poll() {{
            {ffi_poll}(futureHandle, Unmanaged.passRetained(Continuation(continuation)).toOpaque()) {{ userData, pollResult in
                let cont = Unmanaged<Continuation<{return_type}>>.fromOpaque(userData!).takeRetainedValue()
                if pollResult == 0 {{
                    var status = FfiStatus()
                    let result = {ffi_complete}(futureHandle, &status)
                    if status.code != 0 {{
                        cont.resume(throwing: FfiError(status: status))
                    }} else {{
                        cont.resume(returning: result)
                    }}
                }} else {{
                    poll()
                }}
            }}
        }}
        poll()
    }}
}} onCancel: {{
    {ffi_cancel}(futureHandle)
}}"#
        )
    }

    fn build_ffi_args(method: &Method) -> String {
        if method.inputs.is_empty() {
            String::new()
        } else {
            let args = method
                .inputs
                .iter()
                .map(|param| NamingConvention::param_name(&param.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!(", {args}")
        }
    }

    fn build_return_handling(method: &Method) -> &'static str {
        match &method.output {
            Some(ty) if !ty.is_void() => "return ",
            _ => "",
        }
    }
}
