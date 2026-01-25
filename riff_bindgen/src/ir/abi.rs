use riff_ffi_rules::naming::{CreateFn, GlobalSymbol, Name, RegisterFn, VtableField, VtableType};

use crate::ir::codec::CodecPlan;
use crate::ir::contract::PackageInfo;
use crate::ir::ids::{
    CallbackId, ClassId, EnumId, FunctionId, MethodId, ParamName, RecordId,
};
use crate::ir::plan::{AbiType, CallbackStyle, Mutability};

#[derive(Debug, Clone)]
pub struct AbiContract {
    pub package: PackageInfo,
    pub calls: Vec<AbiCall>,
    pub callbacks: Vec<AbiCallbackInvocation>,
    pub record_codecs: Vec<(RecordId, CodecPlan)>,
    pub enum_codecs: Vec<(EnumId, CodecPlan)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallId {
    Function(FunctionId),
    Method { class_id: ClassId, method_id: MethodId },
    Constructor { class_id: ClassId, index: usize },
}

#[derive(Debug, Clone)]
pub struct AbiCall {
    pub id: CallId,
    pub symbol: Name<GlobalSymbol>,
    pub mode: CallMode,
    pub params: Vec<AbiParam>,
    pub return_: ReturnTransport,
    pub error: ErrorTransport,
}

#[derive(Debug, Clone)]
pub enum CallMode {
    Sync,
    Async(Box<AsyncCall>),
}

#[derive(Debug, Clone)]
pub struct AsyncCall {
    pub poll: Name<GlobalSymbol>,
    pub complete: Name<GlobalSymbol>,
    pub cancel: Name<GlobalSymbol>,
    pub free: Name<GlobalSymbol>,
    pub result: AsyncResultTransport,
    pub error: ErrorTransport,
}

#[derive(Debug, Clone)]
pub enum AsyncResultTransport {
    Void,
    Direct(AbiType),
    Encoded { codec: CodecPlan },
    Handle { class_id: ClassId, nullable: bool },
    Callback { callback_id: CallbackId, nullable: bool },
}

#[derive(Debug, Clone)]
pub struct AbiParam {
    pub name: ParamName,
    pub ffi_type: AbiType,
    pub role: ParamRole,
}

#[derive(Debug, Clone)]
pub enum ParamRole {
    InDirect,
    InBuffer {
        len_param: ParamName,
        mutability: Mutability,
        element_abi: AbiType,
    },
    InEncoded {
        len_param: ParamName,
        codec: CodecPlan,
    },
    InHandle {
        class_id: ClassId,
        nullable: bool,
    },
    InCallback {
        callback_id: CallbackId,
        nullable: bool,
        style: CallbackStyle,
    },
    OutDirect,
    OutBuffer {
        len_param: ParamName,
        codec: CodecPlan,
    },
    OutLen {
        for_param: ParamName,
    },
    StatusOut,
}

#[derive(Debug, Clone)]
pub enum ReturnTransport {
    Void,
    Direct(AbiType),
    Encoded { codec: CodecPlan },
    Handle { class_id: ClassId, nullable: bool },
    Callback { callback_id: CallbackId, nullable: bool },
}

#[derive(Debug, Clone)]
pub enum ErrorTransport {
    None,
    StatusCode,
    Encoded { codec: CodecPlan },
}

#[derive(Debug, Clone)]
pub struct AbiCallbackInvocation {
    pub callback_id: CallbackId,
    pub vtable_type: Name<VtableType>,
    pub register_fn: Name<RegisterFn>,
    pub create_fn: Name<CreateFn>,
    pub methods: Vec<AbiCallbackMethod>,
}

#[derive(Debug, Clone)]
pub struct AbiCallbackMethod {
    pub id: MethodId,
    pub vtable_field: Name<VtableField>,
    pub is_async: bool,
    pub params: Vec<AbiParam>,
    pub return_: ReturnTransport,
    pub error: ErrorTransport,
}
