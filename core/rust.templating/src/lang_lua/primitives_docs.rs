pub fn document_primitives() -> Vec<templating_docgen::Primitive> {
    templating_docgen::PrimitiveListBuilder::default()
        .add("u8", "number", "An unsigned 8-bit integer. **Note: u8 arrays (`{u8}`) are often used to represent an array of bytes in AntiRaid**", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("0-{}", u8::MAX),
            )
        })
        .add("u16", "number", "An unsigned 16-bit integer.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("0-{}", u16::MAX),
            )
        })
        .add("u32", "number", "An unsigned 32-bit integer.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("0-{}", u32::MAX),
            )
        })
        .add("u64", "number", "An unsigned 64-bit integer. **Note that most, if not all, cases of `i64` in the actual API are either `string` or the `I64` custom type from typesext**", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("0-{}", u64::MAX),
            )
        })
        .add("i8", "number", "A signed 8-bit integer.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("{}-{}", i8::MIN, i8::MAX),
            )
        })
        .add("i16", "number", "A signed 16-bit integer.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("{}-{}", i16::MIN, i16::MAX),
            )
        })
        .add("i32", "number", "A signed 32-bit integer.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("{}-{}", i32::MIN, i32::MAX),
            )
        })
        .add("i64", "number", "A signed 64-bit integer. **Note that most, if not all, cases of `i64` in the actual API are either `string` or the `I64` custom type from typesext**", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                &format!("{}-{}", i64::MIN, i64::MAX),
            )
        })
        .add("f32", "number", "A 32-bit floating point number.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                "IEEE 754 single-precision floating point",
            )
        })
        .add("f64", "number", "A 64-bit floating point number.", |p| {
            p.add_constraint(
                "range",
                "The range of values this number can take on",
                "IEEE 754 double-precision floating point",
            )
        })
        .add("bool", "boolean", "A boolean value.", |p| p)
        .add("char", "string", "A single Unicode character.", |p| {
            p.add_constraint(
                "length",
                "The length of the string",
                "1",
            )
        })
        .add("string", "string", "A UTF-8 encoded string.", |p| {
            p.add_constraint(
                "encoding",
                "Accepted character encoding",
                "UTF-8 *only*",
            )
        })
        .build()
}
