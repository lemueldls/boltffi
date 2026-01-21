use crate::model::Primitive;

pub trait KotlinBufferRead {
    fn buffer_getter(&self) -> &'static str;
    fn buffer_conversion(&self) -> &'static str;
}

pub trait KotlinBufferWrite {
    fn buffer_putter(&self) -> &'static str;
    fn buffer_value_expr(&self, value_expr: &str) -> String;
}

impl KotlinBufferRead for Primitive {
    fn buffer_getter(&self) -> &'static str {
        match self {
            Self::Bool | Self::I8 | Self::U8 => "get",
            Self::I16 | Self::U16 => "getShort",
            Self::I32 | Self::U32 => "getInt",
            Self::I64 | Self::U64 | Self::Usize | Self::Isize => "getLong",
            Self::F32 => "getFloat",
            Self::F64 => "getDouble",
        }
    }

    fn buffer_conversion(&self) -> &'static str {
        match self {
            Self::Bool => " != 0.toByte()",
            Self::U8 => ".toUByte()",
            Self::U16 => ".toUShort()",
            Self::U32 => ".toUInt()",
            Self::U64 | Self::Usize => ".toULong()",
            _ => "",
        }
    }
}

impl KotlinBufferWrite for Primitive {
    fn buffer_putter(&self) -> &'static str {
        match self {
            Self::Bool | Self::I8 | Self::U8 => "put",
            Self::I16 | Self::U16 => "putShort",
            Self::I32 | Self::U32 => "putInt",
            Self::I64 | Self::U64 | Self::Usize | Self::Isize => "putLong",
            Self::F32 => "putFloat",
            Self::F64 => "putDouble",
        }
    }

    fn buffer_value_expr(&self, value_expr: &str) -> String {
        match self {
            Self::Bool => format!("(if ({}) 1 else 0).toByte()", value_expr),
            Self::U8 => format!("({}).toByte()", value_expr),
            Self::U16 => format!("({}).toShort()", value_expr),
            Self::U32 => format!("({}).toInt()", value_expr),
            Self::U64 => format!("({}).toLong()", value_expr),
            _ => value_expr.to_string(),
        }
    }
}
