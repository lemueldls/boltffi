use boltffi_ffi_rules::callable::ExecutionKind;
use heck::ToLowerCamelCase;

use crate::ir::plan::{CompositeLayout, ScalarOrigin, Transport};
use crate::ir::{
    AbiContract,
    abi::{
        AbiCall, AbiCallbackInvocation, AbiCallbackMethod, AbiEnum, AbiEnumField, AbiEnumPayload,
        AbiStream, CallId, CallMode, StreamItemTransport,
    },
    codec::{CodecPlan, EnumLayout, VariantPayloadLayout},
    contract::FfiContract,
    definitions::{
        CallbackMethodDef, CallbackTraitDef, ClassDef, DefaultValue, EnumDef, FieldDef,
        FunctionDef, MethodDef, RecordDef, ReturnDef, StreamDef, StreamMode,
    },
    ids::BuiltinId,
    ops::{ReadOp, ReadSeq, WriteOp},
    types::{PrimitiveType, TypeExpr},
};
use crate::render::kotlin::{NamingConvention, emit_size_expr_for_write_seq, emit_write_expr};
use boltffi_ffi_rules::naming;
use boltffi_ffi_rules::transport::EnumTagStrategy;

use super::plan::{
    KmpCallback, KmpCallbackMethod, KmpClass, KmpClassConstructor, KmpClassMethod, KmpClassStream,
    KmpEnum, KmpEnumField, KmpEnumVariant, KmpFunction, KmpModule, KmpParam, KmpRecord,
    KmpRecordField, KmpStreamMode,
};

pub struct KmpLowerer<'a> {
    contract: &'a FfiContract,
    abi: &'a AbiContract,
}

impl<'a> KmpLowerer<'a> {
    pub fn new(contract: &'a FfiContract, abi: &'a AbiContract) -> Self {
        Self { contract, abi }
    }

    pub fn lower(&self) -> KmpModule {
        let mut call_map = std::collections::HashMap::new();
        for call in &self.abi.calls {
            if let CallId::Function(function_id) = &call.id {
                call_map.insert(function_id.as_str(), call);
            }
        }

        let records = self
            .contract
            .catalog
            .all_records()
            .map(|record| self.lower_record(record))
            .collect::<Vec<_>>();
        let enums = self
            .contract
            .catalog
            .all_enums()
            .map(|enumeration| self.lower_enum(enumeration))
            .collect::<Vec<_>>();
        let classes = self
            .contract
            .catalog
            .all_classes()
            .map(|class| self.lower_class(class))
            .collect::<Vec<_>>();
        let callbacks = self
            .contract
            .catalog
            .all_callbacks()
            .map(|callback| self.lower_callback(callback))
            .collect::<Vec<_>>();
        let mut functions = Vec::new();

        for function in &self.contract.functions {
            let Some(call) = call_map.get(function.id.as_str()) else {
                continue;
            };

            match self.supported_function(function, call) {
                Some(kmp_function) => functions.push(kmp_function),
                None => continue,
            }
        }

        KmpModule {
            records,
            enums,
            classes,
            callbacks,
            functions,
        }
    }

    fn supported_function(&self, function: &FunctionDef, call: &AbiCall) -> Option<KmpFunction> {
        if call.params.len() != function.params.len() {
            return None;
        }

        let mut params = Vec::with_capacity(function.params.len());
        for param in &function.params {
            params.push(KmpParam {
                name: NamingConvention::param_name(param.name.as_str()),
                kotlin_type: self.kotlin_type(&param.type_expr),
            });
        }

        let return_type = match &function.returns {
            ReturnDef::Void => None,
            ReturnDef::Value(type_expr) => Some(self.kotlin_type(type_expr)),
            ReturnDef::Result { ok, err } => Some(format!(
                "BoltFFIResult<{}, {}>",
                self.kotlin_type(ok),
                self.kotlin_type(err)
            )),
        };

        Some(KmpFunction {
            public_name: function.id.as_str().to_lower_camel_case(),
            ffi_symbol: call.symbol.as_str().to_string(),
            params,
            return_type,
            is_async: function.execution_kind() == ExecutionKind::Async
                || matches!(call.mode, CallMode::Async(_)),
        })
    }

    fn kotlin_type(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Void => "Unit".to_string(),
            TypeExpr::Primitive(primitive) => Self::kotlin_primitive_type(*primitive),
            TypeExpr::String => "String".to_string(),
            TypeExpr::Bytes => "ByteArray".to_string(),
            TypeExpr::Vec(inner) => format!("List<{}>", self.kotlin_type(inner)),
            TypeExpr::Option(inner) => format!("{}?", self.kotlin_type(inner)),
            TypeExpr::Result { ok, err } => format!(
                "BoltFFIResult<{}, {}>",
                self.kotlin_type(ok),
                self.kotlin_type(err)
            ),
            TypeExpr::Record(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Enum(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Handle(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Callback(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Custom(id) => NamingConvention::class_name(id.as_str()),
            TypeExpr::Builtin(id) => self.kotlin_builtin_type(id),
        }
    }

    fn kotlin_primitive_type(primitive: PrimitiveType) -> String {
        match primitive {
            PrimitiveType::Bool => "Boolean".to_string(),
            PrimitiveType::I8 => "Byte".to_string(),
            PrimitiveType::U8 => "UByte".to_string(),
            PrimitiveType::I16 => "Short".to_string(),
            PrimitiveType::U16 => "UShort".to_string(),
            PrimitiveType::I32 => "Int".to_string(),
            PrimitiveType::U32 => "UInt".to_string(),
            PrimitiveType::I64 | PrimitiveType::ISize => "Long".to_string(),
            PrimitiveType::U64 | PrimitiveType::USize => "ULong".to_string(),
            PrimitiveType::F32 => "Float".to_string(),
            PrimitiveType::F64 => "Double".to_string(),
        }
    }

    fn kotlin_builtin_type(&self, builtin_id: &BuiltinId) -> String {
        match builtin_id.as_str() {
            "Duration" => "Duration".to_string(),
            "SystemTime" => "Instant".to_string(),
            "Uuid" => "UUID".to_string(),
            "Url" => "URI".to_string(),
            _ => NamingConvention::class_name(builtin_id.as_str()),
        }
    }

    fn lower_record(&self, record: &RecordDef) -> KmpRecord {
        let abi_record = self.abi_record_for(record);
        KmpRecord {
            class_name: NamingConvention::class_name(record.id.as_str()),
            fields: record
                .fields
                .iter()
                .map(|field| self.lower_record_field(field))
                .collect::<Vec<_>>(),
            decode_source: self.render_record_decode_source(record, abi_record),
            encode_source: self.render_record_encode_source(record, abi_record),
            doc: record.doc.clone(),
        }
    }

    fn abi_record_for(&self, record: &RecordDef) -> &crate::ir::abi::AbiRecord {
        self.abi
            .records
            .iter()
            .find(|abi_record| abi_record.id == record.id)
            .expect("abi record missing")
    }

    fn lower_record_field(&self, field: &FieldDef) -> KmpRecordField {
        let kotlin_type = self.kotlin_type(&field.type_expr);
        KmpRecordField {
            name: NamingConvention::param_name(field.name.as_str()),
            kotlin_type: kotlin_type.clone(),
            default_value: field
                .default
                .as_ref()
                .map(|default| Self::kotlin_default_literal(default, &kotlin_type)),
        }
    }

    fn kotlin_default_literal(default: &DefaultValue, kotlin_type: &str) -> String {
        use heck::ToUpperCamelCase;

        match default {
            DefaultValue::Bool(true) => "true".to_string(),
            DefaultValue::Bool(false) => "false".to_string(),
            DefaultValue::Integer(v) => match kotlin_type {
                "Double" => format!("{}.0", v),
                "Float" => format!("{}.0f", v),
                "UInt" => format!("{}u", v),
                "ULong" => format!("{}uL", v),
                "UShort" => format!("{}u", v),
                "UByte" => format!("{}u", v),
                "Long" => format!("{}L", v),
                _ => v.to_string(),
            },
            DefaultValue::Float(v) => {
                let has_decimal = v.fract() != 0.0;
                let base = if has_decimal {
                    format!("{}", v)
                } else {
                    format!("{}.0", v)
                };
                match kotlin_type {
                    "Float" => format!("{}f", base),
                    _ => base,
                }
            }
            DefaultValue::String(v) => format!("\"{}\"", v),
            DefaultValue::EnumVariant {
                enum_name,
                variant_name,
            } => format!(
                "{}.{}",
                enum_name.to_upper_camel_case(),
                NamingConvention::enum_entry_name(variant_name)
            ),
            DefaultValue::Null => "null".to_string(),
        }
    }

    fn abi_enum_for(&self, enumeration: &EnumDef) -> &AbiEnum {
        self.abi
            .enums
            .iter()
            .find(|abi_enum| abi_enum.id == enumeration.id)
            .expect("abi enum missing")
    }

    fn lower_enum(&self, enumeration: &EnumDef) -> KmpEnum {
        let abi_enum = self.abi_enum_for(enumeration);
        let class_name = NamingConvention::class_name(enumeration.id.as_str());
        let is_c_style = abi_enum.is_c_style;
        let is_error = enumeration.is_error;
        let value_type = match &enumeration.repr {
            crate::ir::definitions::EnumRepr::CStyle { tag_type, .. } => {
                Some(Self::kotlin_primitive_type(*tag_type))
            }
            _ => None,
        };
        let variant_docs = enumeration.variant_docs();
        let variants = abi_enum
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let mut lowered = self.lower_enum_variant(abi_enum, variant, index, is_c_style);
                lowered.doc = variant_docs.get(index).cloned().flatten();
                lowered
            })
            .collect::<Vec<_>>();

        KmpEnum {
            class_name,
            is_c_style,
            is_error,
            value_type,
            variants,
            decode_source: self.render_enum_decode_source(enumeration, abi_enum),
            encode_source: self.render_enum_encode_source(enumeration, abi_enum),
            doc: enumeration.doc.clone(),
        }
    }

    fn lower_enum_variant(
        &self,
        abi_enum: &AbiEnum,
        variant: &crate::ir::abi::AbiEnumVariant,
        ordinal: usize,
        is_c_style: bool,
    ) -> KmpEnumVariant {
        let fields = match &variant.payload {
            AbiEnumPayload::Unit => Vec::new(),
            AbiEnumPayload::Tuple(fields) | AbiEnumPayload::Struct(fields) => fields
                .iter()
                .enumerate()
                .map(|(index, field)| self.lower_enum_field(field, index))
                .collect(),
        };
        let name = if is_c_style {
            NamingConvention::enum_entry_name(variant.name.as_str())
        } else {
            NamingConvention::class_name(variant.name.as_str())
        };

        KmpEnumVariant {
            name,
            tag: self.kotlin_enum_variant_tag(abi_enum, ordinal, variant.discriminant),
            fields,
            doc: None,
        }
    }

    fn lower_enum_field(&self, field: &AbiEnumField, index: usize) -> KmpEnumField {
        KmpEnumField {
            name: if field.name.as_str().is_empty() {
                format!("value_{}", index)
            } else {
                NamingConvention::property_name(field.name.as_str())
            },
            kotlin_type: self.kotlin_type(&field.type_expr),
        }
    }

    fn kotlin_enum_variant_tag(
        &self,
        abi_enum: &AbiEnum,
        ordinal: usize,
        discriminant: i128,
    ) -> i128 {
        match abi_enum.codec_tag_strategy {
            EnumTagStrategy::Discriminant => discriminant,
            EnumTagStrategy::OrdinalIndex => abi_enum.resolve_codec_tag(ordinal, discriminant),
        }
    }

    fn lower_class(&self, class: &ClassDef) -> KmpClass {
        KmpClass {
            class_name: NamingConvention::class_name(class.id.as_str()),
            doc: class.doc.clone(),
            constructors: class
                .constructors
                .iter()
                .map(|constructor| KmpClassConstructor {
                    ffi_symbol: naming::class_ffi_new(class.id.as_str()).into_string(),
                    params: constructor
                        .params()
                        .into_iter()
                        .map(|param| KmpParam {
                            name: NamingConvention::param_name(param.name.as_str()),
                            kotlin_type: self.kotlin_type(&param.type_expr),
                        })
                        .collect::<Vec<_>>(),
                    doc: constructor.doc().map(String::from),
                })
                .collect::<Vec<_>>(),
            methods: class
                .methods
                .iter()
                .map(|method| self.lower_class_method(class, method))
                .collect::<Vec<_>>(),
            streams: class
                .streams
                .iter()
                .map(|stream| self.lower_class_stream(class, stream))
                .collect::<Vec<_>>(),
        }
    }

    fn abi_stream_for<'b>(&'b self, class: &ClassDef, stream: &StreamDef) -> &'b AbiStream {
        self.abi
            .streams
            .iter()
            .find(|item| item.class_id == class.id && item.stream_id == stream.id)
            .expect("abi stream missing")
    }

    fn lower_class_stream(&self, class: &ClassDef, stream: &StreamDef) -> KmpClassStream {
        let abi_stream = self.abi_stream_for(class, stream);
        let item_type = self.kotlin_type(&stream.item_type);
        KmpClassStream {
            name: NamingConvention::method_name(stream.id.as_str()),
            item_type: item_type.clone(),
            mode: match stream.mode {
                StreamMode::Async => KmpStreamMode::Async,
                StreamMode::Batch => KmpStreamMode::Batch,
                StreamMode::Callback => KmpStreamMode::Callback,
            },
            pop_batch_items_expr: self.stream_pop_batch_items_expr(abi_stream, &item_type),
            subscribe_symbol: abi_stream.subscribe.as_str().to_string(),
            poll_symbol: abi_stream.poll.as_str().to_string(),
            pop_batch_symbol: abi_stream.pop_batch.as_str().to_string(),
            wait_symbol: abi_stream.wait.as_str().to_string(),
            unsubscribe_symbol: abi_stream.unsubscribe.as_str().to_string(),
            free_symbol: abi_stream.free.as_str().to_string(),
            doc: stream.doc.clone(),
        }
    }

    fn stream_pop_batch_items_expr(&self, stream: &AbiStream, item_type: &str) -> String {
        match &stream.item_transport {
            Transport::Scalar(origin) => Self::direct_scalar_stream_items_expr(origin),
            Transport::Composite(layout) => Self::direct_composite_stream_items_expr(layout),
            _ => self.wire_encoded_stream_items_expr(stream, item_type),
        }
    }

    fn wire_encoded_stream_items_expr(&self, stream: &AbiStream, _item_type: &str) -> String {
        let StreamItemTransport::WireEncoded { decode_ops } = &stream.item;
        let item_expr = self.wire_read_expr(decode_ops);
        format!(
            "run {{ val reader = boltffiWireReader(bytes); val count = reader.readI32(); List(count) {{ {item_expr} }} }}"
        )
    }

    fn wire_read_expr(&self, seq: &ReadSeq) -> String {
        let Some(op) = seq.ops.first() else {
            return "Unit".to_string();
        };
        match op {
            ReadOp::Primitive { primitive, .. } => Self::wire_primitive_read_expr(*primitive),
            ReadOp::String { .. } => "reader.readString()".to_string(),
            ReadOp::Bytes { .. } => "reader.readBytes()".to_string(),
            ReadOp::Option { some, .. } => {
                let some_expr = self.wire_read_expr(some);
                format!("if (reader.readU8().toInt() == 0) null else {some_expr}")
            }
            ReadOp::Vec { element, .. } => {
                let element_expr = self.wire_read_expr(element);
                format!("run {{ val len = reader.readI32(); List(len) {{ {element_expr} }} }}")
            }
            ReadOp::Record { id, .. } => {
                format!("{}(reader)", Self::record_decode_helper_name(id.as_str()))
            }
            ReadOp::Enum { id, layout, .. } => match layout {
                EnumLayout::CStyle { tag_type, .. } => {
                    let enum_name = NamingConvention::class_name(id.as_str());
                    let tag_read = Self::wire_primitive_read_expr(*tag_type);
                    format!(
                        "run {{ val tag = {}; {}.entries.first {{ it.value == tag }} }}",
                        tag_read, enum_name
                    )
                }
                EnumLayout::Data {
                    tag_type,
                    tag_strategy,
                    variants,
                } => self.wire_read_data_enum_expr(id.as_str(), *tag_type, *tag_strategy, variants),
                EnumLayout::Recursive => {
                    format!("{}(reader)", Self::enum_decode_helper_name(id.as_str()))
                }
            },
            ReadOp::Result { ok, err, .. } => {
                let ok_expr = self.wire_read_expr(ok);
                let err_expr = self.wire_read_expr(err);
                format!(
                    "if (reader.readU8().toInt() == 0) BoltFFIResult.Ok({ok_expr}) else BoltFFIResult.Err({err_expr})"
                )
            }
            ReadOp::Builtin { id, .. } => Self::wire_builtin_read_expr(id),
            ReadOp::Custom { underlying, .. } => self.wire_read_expr(underlying),
        }
    }

    fn wire_read_codec_expr(&self, codec: &CodecPlan) -> String {
        match codec {
            CodecPlan::Void => "Unit".to_string(),
            CodecPlan::Primitive(primitive) => Self::wire_primitive_read_expr(*primitive),
            CodecPlan::String => "reader.readString()".to_string(),
            CodecPlan::Bytes => "reader.readBytes()".to_string(),
            CodecPlan::Builtin(id) => Self::wire_builtin_read_expr(id),
            CodecPlan::Option(inner) => {
                let inner_expr = self.wire_read_codec_expr(inner);
                format!("if (reader.readU8().toInt() == 0) null else {inner_expr}")
            }
            CodecPlan::Vec { element, .. } => {
                let item_expr = self.wire_read_codec_expr(element);
                format!("run {{ val len = reader.readI32(); List(len) {{ {item_expr} }} }}")
            }
            CodecPlan::Result { ok, err } => {
                let ok_expr = self.wire_read_codec_expr(ok);
                let err_expr = self.wire_read_codec_expr(err);
                format!(
                    "if (reader.readU8().toInt() == 0) BoltFFIResult.Ok({ok_expr}) else BoltFFIResult.Err({err_expr})"
                )
            }
            CodecPlan::Record { id, layout } => match layout {
                crate::ir::codec::RecordLayout::Blittable { fields, .. } => {
                    let _ = fields;
                    format!("{}(reader)", Self::record_decode_helper_name(id.as_str()))
                }
                crate::ir::codec::RecordLayout::Encoded { fields } => {
                    let _ = fields;
                    format!("{}(reader)", Self::record_decode_helper_name(id.as_str()))
                }
                crate::ir::codec::RecordLayout::Recursive => {
                    format!("{}(reader)", Self::record_decode_helper_name(id.as_str()))
                }
            },
            CodecPlan::Enum { id, layout } => match layout {
                EnumLayout::CStyle { tag_type, .. } => {
                    let enum_name = NamingConvention::class_name(id.as_str());
                    let tag_read = Self::wire_primitive_read_expr(*tag_type);
                    format!(
                        "run {{ val tag = {}; {}.entries.first {{ it.value == tag }} }}",
                        tag_read, enum_name
                    )
                }
                EnumLayout::Data {
                    tag_type,
                    tag_strategy,
                    variants,
                } => self.wire_read_data_enum_expr(id.as_str(), *tag_type, *tag_strategy, variants),
                EnumLayout::Recursive => {
                    format!("{}(reader)", Self::enum_decode_helper_name(id.as_str()))
                }
            },
            CodecPlan::Custom { underlying, .. } => self.wire_read_codec_expr(underlying),
        }
    }

    fn wire_read_data_enum_expr(
        &self,
        enum_id: &str,
        tag_type: PrimitiveType,
        tag_strategy: EnumTagStrategy,
        variants: &[crate::ir::codec::VariantLayout],
    ) -> String {
        let enum_name = NamingConvention::class_name(enum_id);
        let tag_read = Self::wire_primitive_read_expr(tag_type);
        let arms = variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let tag_value = match tag_strategy {
                    EnumTagStrategy::Discriminant => variant.discriminant,
                    EnumTagStrategy::OrdinalIndex => index as i128,
                };
                let tag_literal = Self::kotlin_tag_literal(tag_value, tag_type);
                let variant_name = NamingConvention::class_name(variant.name.as_str());
                let ctor_expr = match &variant.payload {
                    VariantPayloadLayout::Unit => format!("{}.{}", enum_name, variant_name),
                    VariantPayloadLayout::Fields(fields) => {
                        let args = fields
                            .iter()
                            .map(|field| {
                                let field_name = NamingConvention::param_name(field.name.as_str());
                                let value_expr = self.wire_read_codec_expr(&field.codec);
                                format!("{} = {}", field_name, value_expr)
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{}.{}({})", enum_name, variant_name, args)
                    }
                };
                format!("{} -> {}", tag_literal, ctor_expr)
            })
            .collect::<Vec<_>>()
            .join("; ");

        format!(
            "run {{ val tag = {}; when (tag) {{ {}; else -> error(\"Unknown enum tag for {}: $tag\") }} }}",
            tag_read, arms, enum_name
        )
    }

    fn wire_builtin_read_expr(builtin_id: &BuiltinId) -> String {
        match builtin_id.as_str() {
            "Duration" => "reader.readDuration()".to_string(),
            "SystemTime" => "reader.readInstant()".to_string(),
            "Uuid" => "reader.readUuid()".to_string(),
            "Url" => "reader.readUri()".to_string(),
            _ => "reader.readString()".to_string(),
        }
    }

    fn record_decode_helper_name(id: &str) -> String {
        format!("boltffiDecodeRecord{}", NamingConvention::class_name(id))
    }

    fn enum_decode_helper_name(id: &str) -> String {
        format!("boltffiDecodeEnum{}", NamingConvention::class_name(id))
    }

    fn render_record_decode_source(
        &self,
        record: &RecordDef,
        abi_record: &crate::ir::abi::AbiRecord,
    ) -> String {
        let record_name = NamingConvention::class_name(record.id.as_str());
        let body = match abi_record.decode_ops.ops.first() {
            Some(ReadOp::Record { fields, .. }) => {
                let args = fields
                    .iter()
                    .map(|field| {
                        let field_name = NamingConvention::param_name(field.name.as_str());
                        let value_expr = self.wire_read_expr(&field.seq);
                        format!("{} = {}", field_name, value_expr)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", record_name, args)
            }
            _ => self.wire_read_expr(&abi_record.decode_ops),
        };

        format!(
            "private fun {}(reader: boltffiWireReader): {} = {}",
            Self::record_decode_helper_name(record.id.as_str()),
            record_name,
            body
        )
    }

    fn render_enum_decode_source(&self, enumeration: &EnumDef, abi_enum: &AbiEnum) -> String {
        let enum_name = NamingConvention::class_name(enumeration.id.as_str());
        let body = match &enumeration.repr {
            crate::ir::definitions::EnumRepr::CStyle { tag_type, .. } => {
                let tag_read = Self::wire_primitive_read_expr(*tag_type);
                let arms = abi_enum
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        let _tag_value = abi_enum.resolve_codec_tag(index, variant.discriminant);
                        let tag_literal = Self::kotlin_tag_literal(_tag_value, *tag_type);
                        format!(
                            "{} -> {}.{}",
                            tag_literal,
                            enum_name,
                            NamingConvention::enum_entry_name(variant.name.as_str())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!(
                    "run {{ val tag = {}; when (tag) {{ {}; else -> error(\"Unknown enum tag for {}: $tag\") }} }}",
                    tag_read, arms, enum_name
                )
            }
            crate::ir::definitions::EnumRepr::Data { tag_type, variants } => {
                let arms = variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        let tag_value = abi_enum.resolve_codec_tag(index, variant.discriminant);
                        let tag_literal = Self::kotlin_tag_literal(tag_value, *tag_type);
                        let abi_variant = &abi_enum.variants[index];
                        let variant_name = NamingConvention::class_name(variant.name.as_str());
                        let ctor_expr = match &abi_variant.payload {
                            AbiEnumPayload::Unit => format!("{}.{}", enum_name, variant_name),
                            AbiEnumPayload::Tuple(fields) | AbiEnumPayload::Struct(fields) => {
                                let args = fields
                                    .iter()
                                    .map(|field| {
                                        let field_name =
                                            NamingConvention::param_name(field.name.as_str());
                                        let value_expr = self.wire_read_expr(&field.decode);
                                        format!("{} = {}", field_name, value_expr)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("{}.{}({})", enum_name, variant_name, args)
                            }
                        };
                        format!("{} -> {}", tag_literal, ctor_expr)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!(
                    "run {{ val tag = {}; when (tag) {{ {}; else -> error(\"Unknown enum tag for {}: $tag\") }} }}",
                    Self::wire_primitive_read_expr(*tag_type),
                    arms,
                    enum_name
                )
            }
        };

        format!(
            "private fun {}(reader: boltffiWireReader): {} = {}",
            Self::enum_decode_helper_name(enumeration.id.as_str()),
            enum_name,
            body
        )
    }

    fn render_record_encode_source(
        &self,
        _record: &RecordDef,
        abi_record: &crate::ir::abi::AbiRecord,
    ) -> String {
        let Some(WriteOp::Record { fields, .. }) = abi_record.encode_ops.ops.first() else {
            return format!(
                "fun {}.wireEncodedSize(): Int = 0\n\nfun {}.wireEncodeTo(wire: boltffiWireWriter) {{\n}}",
                NamingConvention::class_name(_record.id.as_str()),
                NamingConvention::class_name(_record.id.as_str())
            );
        };

        let size_expr = if fields.is_empty() {
            "0".to_string()
        } else {
            fields
                .iter()
                .map(|field| emit_size_expr_for_write_seq(&field.seq))
                .collect::<Vec<_>>()
                .join(" + ")
        };
        let encode_lines = fields
            .iter()
            .map(|field| format!("        {}", emit_write_expr(&field.seq)))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "fun {}.wireEncodedSize(): Int = {size_expr}\n\nfun {}.wireEncodeTo(wire: boltffiWireWriter) {{\n{encode_lines}\n}}",
            NamingConvention::class_name(_record.id.as_str()),
            NamingConvention::class_name(_record.id.as_str()),
        )
    }

    fn render_enum_encode_source(&self, enumeration: &EnumDef, abi_enum: &AbiEnum) -> String {
        let enum_name = NamingConvention::class_name(enumeration.id.as_str());
        match &enumeration.repr {
            crate::ir::definitions::EnumRepr::CStyle { tag_type, .. } => {
                let tag_write = Self::wire_primitive_write_expr(*tag_type, "this.value");
                let size_expr = tag_type.wire_size_bytes().to_string();
                format!(
                    "fun {enum_name}.wireEncodedSize(): Int = {size_expr}\n\nfun {enum_name}.wireEncodeTo(wire: boltffiWireWriter) {{\n    {tag_write}\n}}"
                )
            }
            crate::ir::definitions::EnumRepr::Data { tag_type, variants } => {
                let tag_size = tag_type.wire_size_bytes();
                let size_arms = variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| match &abi_enum.variants[index].payload {
                        AbiEnumPayload::Unit => format!(
                            "is {}.{} -> 0",
                            enum_name,
                            NamingConvention::class_name(variant.name.as_str())
                        ),
                        AbiEnumPayload::Tuple(fields) | AbiEnumPayload::Struct(fields) => {
                            let field_sizes = fields
                                .iter()
                                .map(|field| emit_size_expr_for_write_seq(&field.encode))
                                .collect::<Vec<_>>()
                                .join(" + ");
                            format!(
                                "is {}.{} -> {}",
                                enum_name,
                                NamingConvention::class_name(variant.name.as_str()),
                                field_sizes
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n        ");
                let encode_arms = variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        let tag_value = abi_enum.resolve_codec_tag(index, variant.discriminant);
                        let tag_literal = Self::kotlin_tag_literal(tag_value, *tag_type);
                        let tag_write = Self::wire_primitive_write_expr(*tag_type, &tag_literal);
                        let variant_name = NamingConvention::class_name(variant.name.as_str());
                        let abi_variant = &abi_enum.variants[index];
                        let field_lines = match &abi_variant.payload {
                            AbiEnumPayload::Unit => String::new(),
                            AbiEnumPayload::Tuple(fields) | AbiEnumPayload::Struct(fields) => fields
                                .iter()
                                .map(|field| format!("            {}", emit_write_expr(&field.encode)))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        };
                        if field_lines.is_empty() {
                            format!("            is {}.{} -> {}", enum_name, variant_name, tag_write)
                        } else {
                            format!("            is {}.{} -> {{\n                {}\n{}\n            }}", enum_name, variant_name, tag_write, field_lines)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "fun {enum_name}.wireEncodedSize(): Int = {tag_size} + when (this) {{\n        {size_arms}\n    }}\n\nfun {enum_name}.wireEncodeTo(wire: boltffiWireWriter) {{\n    when (this) {{\n{encode_arms}\n    }}\n}}"
                )
            }
        }
    }

    fn wire_primitive_write_expr(primitive: PrimitiveType, value_expr: &str) -> String {
        match primitive {
            PrimitiveType::Bool => format!("wire.writeBool({value_expr})"),
            PrimitiveType::I8 => format!("wire.writeI8({value_expr})"),
            PrimitiveType::U8 => format!("wire.writeU8({value_expr})"),
            PrimitiveType::I16 => format!("wire.writeI16({value_expr})"),
            PrimitiveType::U16 => format!("wire.writeU16({value_expr})"),
            PrimitiveType::I32 => format!("wire.writeI32({value_expr})"),
            PrimitiveType::U32 => format!("wire.writeU32({value_expr})"),
            PrimitiveType::I64 | PrimitiveType::ISize => format!("wire.writeI64({value_expr})"),
            PrimitiveType::U64 | PrimitiveType::USize => format!("wire.writeU64({value_expr})"),
            PrimitiveType::F32 => format!("wire.writeF32({value_expr})"),
            PrimitiveType::F64 => format!("wire.writeF64({value_expr})"),
        }
    }

    fn wire_primitive_read_expr(primitive: PrimitiveType) -> String {
        match primitive {
            PrimitiveType::Bool => "reader.readBool()".to_string(),
            PrimitiveType::I8 => "reader.readI8()".to_string(),
            PrimitiveType::U8 => "reader.readU8()".to_string(),
            PrimitiveType::I16 => "reader.readI16()".to_string(),
            PrimitiveType::U16 => "reader.readU16()".to_string(),
            PrimitiveType::I32 => "reader.readI32()".to_string(),
            PrimitiveType::U32 => "reader.readU32()".to_string(),
            PrimitiveType::I64 | PrimitiveType::ISize => "reader.readI64()".to_string(),
            PrimitiveType::U64 | PrimitiveType::USize => "reader.readU64()".to_string(),
            PrimitiveType::F32 => "reader.readF32()".to_string(),
            PrimitiveType::F64 => "reader.readF64()".to_string(),
        }
    }

    fn kotlin_tag_literal(value: i128, primitive: PrimitiveType) -> String {
        match primitive {
            PrimitiveType::Bool => {
                if value == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            PrimitiveType::I8 => format!("{}.toByte()", value),
            PrimitiveType::U8 => format!("{}u.toUByte()", value),
            PrimitiveType::I16 => format!("{}.toShort()", value),
            PrimitiveType::U16 => format!("{}u.toUShort()", value),
            PrimitiveType::I32 => value.to_string(),
            PrimitiveType::U32 => format!("{}u", value),
            PrimitiveType::I64 | PrimitiveType::ISize => format!("{}L", value),
            PrimitiveType::U64 | PrimitiveType::USize => format!("{}uL", value),
            PrimitiveType::F32 => format!("{}f", value),
            PrimitiveType::F64 => format!("{}.0", value),
        }
    }

    fn direct_composite_stream_items_expr(layout: &CompositeLayout) -> String {
        let record_name = NamingConvention::class_name(layout.record_id.as_str());
        let field_exprs = layout
            .fields
            .iter()
            .map(|field| {
                let field_name = NamingConvention::param_name(field.name.as_str());
                let read_expr = Self::primitive_read_expr_at(
                    "bytes",
                    &format!("base + {}", field.offset),
                    field.primitive,
                );
                format!("{field_name} = {read_expr}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "run {{ val stride = {}; List(bytes.size / stride) {{ index -> val base = index * stride; {}({}) }} }}",
            layout.total_size, record_name, field_exprs
        )
    }

    fn primitive_read_expr_at(
        bytes_ident: &str,
        offset_expr: &str,
        primitive: PrimitiveType,
    ) -> String {
        match primitive {
            PrimitiveType::Bool => format!("{bytes_ident}[{offset_expr}].toInt() != 0"),
            PrimitiveType::I8 => format!("{bytes_ident}[{offset_expr}]"),
            PrimitiveType::U8 => format!("{bytes_ident}[{offset_expr}].toUByte()"),
            PrimitiveType::I16 => format!("boltffiReadLeI16({bytes_ident}, {offset_expr})"),
            PrimitiveType::U16 => {
                format!("boltffiReadLeI16({bytes_ident}, {offset_expr}).toUShort()")
            }
            PrimitiveType::I32 => format!("boltffiReadLeI32({bytes_ident}, {offset_expr})"),
            PrimitiveType::U32 => {
                format!("boltffiReadLeI32({bytes_ident}, {offset_expr}).toUInt()")
            }
            PrimitiveType::I64 | PrimitiveType::ISize => {
                format!("boltffiReadLeI64({bytes_ident}, {offset_expr})")
            }
            PrimitiveType::U64 | PrimitiveType::USize => {
                format!("boltffiReadLeI64({bytes_ident}, {offset_expr}).toULong()")
            }
            PrimitiveType::F32 => {
                format!("Float.fromBits(boltffiReadLeI32({bytes_ident}, {offset_expr}))")
            }
            PrimitiveType::F64 => {
                format!("Double.fromBits(boltffiReadLeI64({bytes_ident}, {offset_expr}))")
            }
        }
    }

    fn direct_scalar_stream_items_expr(origin: &ScalarOrigin) -> String {
        match origin {
            ScalarOrigin::Primitive(primitive) => match primitive {
                PrimitiveType::Bool => {
                    "List(bytes.size) { index -> bytes[index].toInt() != 0 }".to_string()
                }
                PrimitiveType::I8 => "List(bytes.size) { index -> bytes[index] }".to_string(),
                PrimitiveType::U8 => {
                    "List(bytes.size) { index -> bytes[index].toUByte() }".to_string()
                }
                PrimitiveType::I16 => "boltffiDecodeI16List(bytes)".to_string(),
                PrimitiveType::U16 => "boltffiDecodeU16List(bytes)".to_string(),
                PrimitiveType::I32 => "boltffiDecodeI32List(bytes)".to_string(),
                PrimitiveType::U32 => "boltffiDecodeU32List(bytes)".to_string(),
                PrimitiveType::I64 | PrimitiveType::ISize => {
                    "boltffiDecodeI64List(bytes)".to_string()
                }
                PrimitiveType::U64 | PrimitiveType::USize => {
                    "boltffiDecodeU64List(bytes)".to_string()
                }
                PrimitiveType::F32 => "boltffiDecodeF32List(bytes)".to_string(),
                PrimitiveType::F64 => "boltffiDecodeF64List(bytes)".to_string(),
            },
            ScalarOrigin::CStyleEnum { enum_id, tag_type } => {
                let enum_name = NamingConvention::class_name(enum_id.as_str());
                let values_expr =
                    Self::direct_scalar_stream_items_expr(&ScalarOrigin::Primitive(*tag_type));
                format!(
                    "{}.map {{ value -> {}.entries.first {{ it.value == value }} }}",
                    values_expr, enum_name
                )
            }
        }
    }

    fn lower_class_method(&self, class: &ClassDef, method: &MethodDef) -> KmpClassMethod {
        KmpClassMethod {
            ffi_symbol: naming::method_ffi_name(class.id.as_str(), method.id.as_str())
                .into_string(),
            name: NamingConvention::method_name(method.id.as_str()),
            params: method
                .params
                .iter()
                .map(|param| KmpParam {
                    name: NamingConvention::param_name(param.name.as_str()),
                    kotlin_type: self.kotlin_type(&param.type_expr),
                })
                .collect::<Vec<_>>(),
            return_type: match &method.returns {
                ReturnDef::Void => None,
                ReturnDef::Value(type_expr) => Some(self.kotlin_type(type_expr)),
                ReturnDef::Result { ok, err } => Some(format!(
                    "BoltFFIResult<{}, {}>",
                    self.kotlin_type(ok),
                    self.kotlin_type(err)
                )),
            },
            is_async: method.execution_kind() == ExecutionKind::Async,
            doc: method.doc.clone(),
        }
    }

    fn lower_callback(&self, callback: &CallbackTraitDef) -> KmpCallback {
        let abi_callback = self.abi_callback_for(callback);
        KmpCallback {
            interface_name: NamingConvention::class_name(callback.id.as_str()),
            methods: callback
                .methods
                .iter()
                .map(|method| {
                    let abi_method = self.abi_callback_method_for(abi_callback, method);
                    self.lower_callback_method(method, abi_method)
                })
                .collect::<Vec<_>>(),
            is_closure: matches!(callback.kind, crate::ir::definitions::CallbackKind::Closure),
            doc: callback.doc.clone(),
        }
    }

    fn abi_callback_for<'b>(&'b self, callback: &CallbackTraitDef) -> &'b AbiCallbackInvocation {
        self.abi
            .callbacks
            .iter()
            .find(|item| item.callback_id == callback.id)
            .expect("abi callback missing")
    }

    fn abi_callback_method_for<'b>(
        &'b self,
        callback: &'b AbiCallbackInvocation,
        method: &CallbackMethodDef,
    ) -> &'b AbiCallbackMethod {
        callback
            .methods
            .iter()
            .find(|item| item.id == method.id)
            .expect("abi callback method missing")
    }

    fn lower_callback_method(
        &self,
        method: &CallbackMethodDef,
        abi_method: &AbiCallbackMethod,
    ) -> KmpCallbackMethod {
        let method_name_pascal = NamingConvention::class_name(method.id.as_str());
        let invoker_suffix = self.invoker_suffix_from_return_shape(&abi_method.returns);
        let (complete_name, fail_name, invoker_symbol, invoker_failure_symbol) =
            if method.execution_kind() == ExecutionKind::Async {
                (
                    Some(format!("complete{method_name_pascal}")),
                    Some(format!("fail{method_name_pascal}")),
                    Some(format!("invokeAsyncCallback{invoker_suffix}")),
                    Some(format!("invokeAsyncCallback{invoker_suffix}Failure")),
                )
            } else {
                (None, None, None, None)
            };
        let async_invoke_result_expr = if method.execution_kind() == ExecutionKind::Async {
            self.callback_async_invoker_result_expr(&abi_method.returns)
        } else {
            None
        };
        KmpCallbackMethod {
            ffi_name: abi_method.vtable_field.as_str().to_string(),
            name: NamingConvention::method_name(method.id.as_str()),
            complete_name,
            fail_name,
            invoker_symbol,
            invoker_failure_symbol,
            async_invoke_result_expr,
            params: method
                .params
                .iter()
                .map(|param| KmpParam {
                    name: NamingConvention::param_name(param.name.as_str()),
                    kotlin_type: self.kotlin_type(&param.type_expr),
                })
                .collect::<Vec<_>>(),
            return_type: match &method.returns {
                ReturnDef::Void => None,
                ReturnDef::Value(type_expr) => Some(self.kotlin_type(type_expr)),
                ReturnDef::Result { ok, err } => Some(format!(
                    "BoltFFIResult<{}, {}>",
                    self.kotlin_type(ok),
                    self.kotlin_type(err)
                )),
            },
            is_async: method.execution_kind() == ExecutionKind::Async,
            doc: method.doc.clone(),
        }
    }

    fn callback_async_invoker_result_expr(
        &self,
        ret_shape: &crate::ir::abi::ReturnShape,
    ) -> Option<String> {
        use boltffi_ffi_rules::transport::ValueReturnStrategy;

        match ret_shape.value_return_strategy() {
            ValueReturnStrategy::Void => None,
            ValueReturnStrategy::Scalar(_) => {
                let Some(transport) = &ret_shape.transport else {
                    return Some("result".to_string());
                };
                match transport {
                    Transport::Scalar(ScalarOrigin::Primitive(primitive)) => {
                        let expr = match primitive {
                            PrimitiveType::Bool => "result",
                            PrimitiveType::I8 | PrimitiveType::U8 => "result.toByte()",
                            PrimitiveType::I16 | PrimitiveType::U16 => "result.toShort()",
                            PrimitiveType::I32 | PrimitiveType::U32 => "result.toInt()",
                            PrimitiveType::I64
                            | PrimitiveType::U64
                            | PrimitiveType::ISize
                            | PrimitiveType::USize => "result.toLong()",
                            PrimitiveType::F32 => "result",
                            PrimitiveType::F64 => "result",
                        };
                        Some(expr.to_string())
                    }
                    Transport::Scalar(ScalarOrigin::CStyleEnum { .. }) => {
                        Some("result.value".to_string())
                    }
                    _ => Some("result".to_string()),
                }
            }
            ValueReturnStrategy::ObjectHandle => {
                let Some(Transport::Handle { nullable, .. }) = &ret_shape.transport else {
                    return Some("result".to_string());
                };
                if *nullable {
                    Some("result?.__boltffiHandle() ?: 0L".to_string())
                } else {
                    Some("result.__boltffiHandle()".to_string())
                }
            }
            ValueReturnStrategy::CallbackHandle => {
                let Some(Transport::Callback {
                    callback_id,
                    nullable,
                    ..
                }) = &ret_shape.transport
                else {
                    return Some("result".to_string());
                };
                let callback_name = NamingConvention::class_name(callback_id.as_str());
                if *nullable {
                    Some(format!(
                        "result?.let {{ {}Bridge.retain(it) }} ?: 0L",
                        callback_name
                    ))
                } else {
                    Some(format!("{}Bridge.retain(result)", callback_name))
                }
            }
            ValueReturnStrategy::CompositeValue | ValueReturnStrategy::Buffer(_) => {
                Some("result".to_string())
            }
        }
    }

    fn invoker_suffix_from_return_shape(&self, ret_shape: &crate::ir::abi::ReturnShape) -> String {
        use boltffi_ffi_rules::transport::ValueReturnStrategy;

        match ret_shape.value_return_strategy() {
            ValueReturnStrategy::Void => "Void".to_string(),
            ValueReturnStrategy::Scalar(_) => {
                let Some(Transport::Scalar(origin)) = &ret_shape.transport else {
                    unreachable!("scalar return strategy requires scalar transport");
                };
                self.invoker_suffix_from_primitive(origin.primitive())
            }
            ValueReturnStrategy::ObjectHandle | ValueReturnStrategy::CallbackHandle => {
                "Handle".to_string()
            }
            ValueReturnStrategy::CompositeValue | ValueReturnStrategy::Buffer(_) => {
                "Wire".to_string()
            }
        }
    }

    fn invoker_suffix_from_primitive(&self, primitive: PrimitiveType) -> String {
        match primitive {
            PrimitiveType::Bool => "Bool".to_string(),
            PrimitiveType::I8 | PrimitiveType::U8 => "I8".to_string(),
            PrimitiveType::I16 | PrimitiveType::U16 => "I16".to_string(),
            PrimitiveType::I32 | PrimitiveType::U32 => "I32".to_string(),
            PrimitiveType::I64
            | PrimitiveType::U64
            | PrimitiveType::ISize
            | PrimitiveType::USize => "I64".to_string(),
            PrimitiveType::F32 => "F32".to_string(),
            PrimitiveType::F64 => "F64".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::ir::{Lowerer, build_contract};
    use crate::scan::scan_crate_with_pointer_width;

    use super::KmpLowerer;

    fn demo_source_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/demo")
    }

    #[test]
    fn lower_demo_contract_collects_supported_functions() {
        let mut scanned_module =
            scan_crate_with_pointer_width(&demo_source_directory(), "demo", None)
                .expect("demo crate should scan");
        let ffi_contract = build_contract(&mut scanned_module);
        let abi_contract = Lowerer::new(&ffi_contract).to_abi_contract();

        let module = KmpLowerer::new(&ffi_contract, &abi_contract).lower();

        assert!(!module.records.is_empty());
        assert!(!module.enums.is_empty());
        assert!(!module.classes.is_empty());
        assert!(!module.callbacks.is_empty());
        assert!(!module.functions.is_empty());
        assert!(
            module
                .functions
                .iter()
                .any(|function| function.public_name == "echoI32")
        );
        assert!(
            module
                .functions
                .iter()
                .all(|function| !function.public_name.is_empty())
        );
    }
}
