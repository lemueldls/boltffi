use crate::ir::contract::{FfiContract, TypeCatalog};
use crate::ir::definitions::{EnumRepr, ParamDef, ParamPassing, ReturnDef, VariantPayload};
use crate::ir::ids::{EnumId, FieldName, FunctionId, ParamName, VariantName};
use crate::ir::types::TypeExpr;

#[derive(Debug, Clone)]
pub enum ValidationError {
    UnresolvedType { context: String, error: String },
    InvalidParamPassing { context: String, message: String },
    InvalidPrimitive { context: String, message: String },
    NonEncodableInData { context: String, message: String },
}

pub fn validate_contract(contract: &FfiContract) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let catalog = &contract.catalog;

    for record in catalog.all_records() {
        for field in &record.fields {
            validate_field_type(
                &field.type_expr,
                catalog,
                &record.id,
                &field.name,
                &mut errors,
            );
        }
    }

    for enumeration in catalog.all_enums() {
        if let EnumRepr::Data { variants, .. } = &enumeration.repr {
            for variant in variants {
                validate_variant_payload(
                    &variant.payload,
                    catalog,
                    &enumeration.id,
                    &variant.name,
                    &mut errors,
                );
            }
        }
    }

    for func in &contract.functions {
        validate_callable(&func.id, &func.params, &func.returns, catalog, &mut errors);
    }

    for class in catalog.all_classes() {
        for ctor in &class.constructors {
            let ctx = format!(
                "{}::{}",
                class.id,
                ctor.name.as_ref().map(|n| n.as_str()).unwrap_or("new")
            );
            for param in &ctor.params {
                validate_param_type(
                    &param.type_expr,
                    &param.passing,
                    catalog,
                    &ctx,
                    &param.name,
                    &mut errors,
                );
            }
        }
        for method in &class.methods {
            let ctx = format!("{}::{}", class.id, method.id);
            validate_callable_inner(&ctx, &method.params, &method.returns, catalog, &mut errors);
        }
    }

    for callback in catalog.all_callbacks() {
        for method in &callback.methods {
            let ctx = format!("{}::{}", callback.id, method.id);
            validate_callable_inner(&ctx, &method.params, &method.returns, catalog, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_callable(
    id: &FunctionId,
    params: &[ParamDef],
    returns: &ReturnDef,
    catalog: &TypeCatalog,
    errors: &mut Vec<ValidationError>,
) {
    let ctx = id.as_str();
    validate_callable_inner(ctx, params, returns, catalog, errors);
}

fn validate_callable_inner(
    ctx: &str,
    params: &[ParamDef],
    returns: &ReturnDef,
    catalog: &TypeCatalog,
    errors: &mut Vec<ValidationError>,
) {
    for param in params {
        validate_param_type(
            &param.type_expr,
            &param.passing,
            catalog,
            ctx,
            &param.name,
            errors,
        );
    }
    validate_return_type(returns, catalog, ctx, errors);
}

fn validate_param_type(
    expr: &TypeExpr,
    passing: &ParamPassing,
    catalog: &TypeCatalog,
    ctx: &str,
    param_name: &ParamName,
    errors: &mut Vec<ValidationError>,
) {
    if let Err(e) = validate_type_expr(expr, catalog) {
        errors.push(ValidationError::UnresolvedType {
            context: format!("{}::{}", ctx, param_name),
            error: e,
        });
    }
    if let Err(e) = validate_param_passing(expr, passing, param_name) {
        errors.push(e);
    }
    if let Err(e) = reject_nested_non_encodable(expr, &format!("{}::{}", ctx, param_name)) {
        errors.push(e);
    }
}

fn reject_nested_non_encodable(expr: &TypeExpr, context: &str) -> Result<(), ValidationError> {
    match expr {
        TypeExpr::Vec(inner) | TypeExpr::Option(inner) => {
            if matches!(inner.as_ref(), TypeExpr::Handle(_) | TypeExpr::Callback(_)) {
                return Err(ValidationError::NonEncodableInData {
                    context: context.to_string(),
                    message: "Vec<Handle>/Vec<Callback>/Option<Handle>/Option<Callback> cannot be encoded - use direct Handle or Callback parameters".to_string(),
                });
            }
            reject_nested_non_encodable(inner, context)
        }
        TypeExpr::Result { ok, err } => {
            reject_nested_non_encodable(ok, context)?;
            reject_nested_non_encodable(err, context)
        }
        _ => Ok(()),
    }
}

fn validate_return_type(
    returns: &ReturnDef,
    catalog: &TypeCatalog,
    ctx: &str,
    errors: &mut Vec<ValidationError>,
) {
    match returns {
        ReturnDef::Void => {}
        ReturnDef::Value(ty) => {
            let return_ctx = format!("{} return", ctx);
            if let Err(e) = validate_type_expr(ty, catalog) {
                errors.push(ValidationError::UnresolvedType {
                    context: return_ctx.clone(),
                    error: e,
                });
            }
            if let Err(e) = reject_nested_non_encodable(ty, &return_ctx) {
                errors.push(e);
            }
        }
        ReturnDef::Result { ok, err } => {
            let ok_ctx = format!("{} return (ok)", ctx);
            let err_ctx = format!("{} return (err)", ctx);
            if let Err(e) = validate_type_expr(ok, catalog) {
                errors.push(ValidationError::UnresolvedType {
                    context: ok_ctx.clone(),
                    error: e,
                });
            }
            if let Err(e) = reject_nested_non_encodable(ok, &ok_ctx) {
                errors.push(e);
            }
            if let Err(e) = validate_type_expr(err, catalog) {
                errors.push(ValidationError::UnresolvedType {
                    context: err_ctx.clone(),
                    error: e,
                });
            }
            if let Err(e) = reject_nested_non_encodable(err, &err_ctx) {
                errors.push(e);
            }
        }
    }
}

fn validate_variant_payload(
    payload: &VariantPayload,
    catalog: &TypeCatalog,
    enum_id: &EnumId,
    variant_name: &VariantName,
    errors: &mut Vec<ValidationError>,
) {
    match payload {
        VariantPayload::Unit => {}
        VariantPayload::Tuple(types) => {
            for (idx, type_expr) in types.iter().enumerate() {
                let ctx = format!("{}::{}::{}", enum_id, variant_name, idx);
                if let Err(e) = validate_type_expr(type_expr, catalog) {
                    errors.push(ValidationError::UnresolvedType {
                        context: ctx.clone(),
                        error: e,
                    });
                }
                if let Err(e) = reject_non_encodable_in_data(type_expr, &ctx) {
                    errors.push(e);
                }
            }
        }
        VariantPayload::Struct(fields) => {
            for field in fields {
                validate_field_type(
                    &field.type_expr,
                    catalog,
                    format!("{}::{}", enum_id, variant_name),
                    &field.name,
                    errors,
                );
            }
        }
    }
}

fn validate_field_type(
    expr: &TypeExpr,
    catalog: &TypeCatalog,
    parent_id: impl std::fmt::Display,
    field_name: &FieldName,
    errors: &mut Vec<ValidationError>,
) {
    if let Err(e) = validate_type_expr(expr, catalog) {
        errors.push(ValidationError::UnresolvedType {
            context: format!("{}::{}", parent_id, field_name),
            error: e,
        });
    }
    if let Err(e) = reject_non_encodable_in_data(expr, &format!("{}::{}", parent_id, field_name)) {
        errors.push(e);
    }
}

fn validate_type_expr(expr: &TypeExpr, catalog: &TypeCatalog) -> Result<(), String> {
    match expr {
        TypeExpr::Primitive(_) | TypeExpr::String | TypeExpr::Bytes => Ok(()),
        TypeExpr::Record(id) => catalog
            .resolve_record(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved record: {}", id)),
        TypeExpr::Enum(id) => catalog
            .resolve_enum(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved enum: {}", id)),
        TypeExpr::Callback(id) => catalog
            .resolve_callback(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved callback: {}", id)),
        TypeExpr::Custom(id) => catalog
            .resolve_custom(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved custom type: {}", id)),
        TypeExpr::Builtin(id) => catalog
            .resolve_builtin(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved builtin: {:?}", id)),
        TypeExpr::Handle(id) => catalog
            .resolve_class(id)
            .map(|_| ())
            .ok_or_else(|| format!("unresolved class handle: {}", id)),
        TypeExpr::Vec(inner) | TypeExpr::Option(inner) => validate_type_expr(inner, catalog),
        TypeExpr::Result { ok, err } => {
            validate_type_expr(ok, catalog)?;
            validate_type_expr(err, catalog)
        }
    }
}

fn reject_non_encodable_in_data(expr: &TypeExpr, context: &str) -> Result<(), ValidationError> {
    match expr {
        TypeExpr::Handle(_) => Err(ValidationError::NonEncodableInData {
            context: context.to_string(),
            message: "Handle (class reference) cannot appear inside records/enums - it's an opaque pointer, not serializable".to_string(),
        }),
        TypeExpr::Callback(_) => Err(ValidationError::NonEncodableInData {
            context: context.to_string(),
            message: "Callback trait cannot appear inside records/enums - use as function parameter only".to_string(),
        }),
        TypeExpr::Vec(inner) | TypeExpr::Option(inner) => {
            reject_non_encodable_in_data(inner, context)
        }
        TypeExpr::Result { ok, err } => {
            reject_non_encodable_in_data(ok, context)?;
            reject_non_encodable_in_data(err, context)
        }
        _ => Ok(()),
    }
}

fn validate_param_passing(
    expr: &TypeExpr,
    passing: &ParamPassing,
    param_name: &ParamName,
) -> Result<(), ValidationError> {
    match (passing, expr) {
        (ParamPassing::ImplTrait, TypeExpr::Callback(_)) => Ok(()),
        (ParamPassing::BoxedDyn, TypeExpr::Callback(_)) => Ok(()),
        (ParamPassing::ImplTrait | ParamPassing::BoxedDyn, _) => {
            Err(ValidationError::InvalidParamPassing {
                context: param_name.to_string(),
                message: "impl Trait or Box<dyn Trait> requires a callback trait".to_string(),
            })
        }
        _ => Ok(()),
    }
}
