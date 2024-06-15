use self::testcase::Testcase;
use orfail::OrFail;
use testcase::Command;

mod testcase;

fn decode(testcase_name: &str) -> orfail::Result<()> {
    let testcase = Testcase::load(testcase_name).or_fail()?;
    for command in &testcase.commands {
        let Command::Module(command) = command else {
            continue;
        };
        if matches!(
            command.filename.to_str(),
            Some(
                "call_indirect.1.wasm"
                    | "elem.0.wasm"
                    | "elem.35.wasm"
                    | "elem.65.wasm"
                    | "elem.66.wasm"
                    | "elem.67.wasm"
                    | "elem.68.wasm"
                    | "global.0.wasm"
                    | "i32.0.wasm"
                    | "i64.0.wasm"
            )
        ) {
            // Unsupported in 1.0
            continue;
        }

        // TODO: assert handling
        command.decode_module().or_fail()?;
    }
    Ok(())
}

#[test]
pub fn decode_address() -> orfail::Result<()> {
    decode("address.json").or_fail()
}

#[test]
pub fn decode_align() -> orfail::Result<()> {
    decode("align.json").or_fail()
}

#[test]
pub fn decode_binary_leb128() -> orfail::Result<()> {
    decode("binary-leb128.json").or_fail()
}

#[test]
pub fn decode_binary() -> orfail::Result<()> {
    decode("binary.json").or_fail()
}

#[test]
pub fn decode_block() -> orfail::Result<()> {
    decode("block.json").or_fail()
}

#[test]
pub fn decode_br() -> orfail::Result<()> {
    decode("br.json").or_fail()
}

#[test]
pub fn decode_br_if() -> orfail::Result<()> {
    decode("br_if.json").or_fail()
}

#[test]
pub fn decode_br_table() -> orfail::Result<()> {
    decode("br_table.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_bulk() -> orfail::Result<()> {
//     decode("bulk.json").or_fail()
// }

#[test]
pub fn decode_call() -> orfail::Result<()> {
    decode("call.json").or_fail()
}

#[test]
pub fn decode_call_indirect() -> orfail::Result<()> {
    decode("call_indirect.json").or_fail()
}

#[test]
pub fn decode_comments() -> orfail::Result<()> {
    decode("comments.json").or_fail()
}

#[test]
pub fn decode_const() -> orfail::Result<()> {
    decode("const.json").or_fail()
}

#[test]
pub fn decode_conversions() -> orfail::Result<()> {
    decode("conversions.json").or_fail()
}

#[test]
pub fn decode_custom() -> orfail::Result<()> {
    decode("custom.json").or_fail()
}

#[test]
pub fn decode_data() -> orfail::Result<()> {
    decode("data.json").or_fail()
}

#[test]
pub fn decode_elem() -> orfail::Result<()> {
    decode("elem.json").or_fail()
}

#[test]
pub fn decode_endianness() -> orfail::Result<()> {
    decode("endianness.json").or_fail()
}

#[test]
pub fn decode_exports() -> orfail::Result<()> {
    decode("exports.json").or_fail()
}

#[test]
pub fn decode_f32() -> orfail::Result<()> {
    decode("f32.json").or_fail()
}

#[test]
pub fn decode_f32_bitwise() -> orfail::Result<()> {
    decode("f32_bitwise.json").or_fail()
}

#[test]
pub fn decode_f32_cmp() -> orfail::Result<()> {
    decode("f32_cmp.json").or_fail()
}

#[test]
pub fn decode_f64() -> orfail::Result<()> {
    decode("f64.json").or_fail()
}

#[test]
pub fn decode_f64_bitwise() -> orfail::Result<()> {
    decode("f64_bitwise.json").or_fail()
}

#[test]
pub fn decode_f64_cmp() -> orfail::Result<()> {
    decode("f64_cmp.json").or_fail()
}

#[test]
pub fn decode_fac() -> orfail::Result<()> {
    decode("fac.json").or_fail()
}

#[test]
pub fn decode_float_exprs() -> orfail::Result<()> {
    decode("float_exprs.json").or_fail()
}

#[test]
pub fn decode_float_literals() -> orfail::Result<()> {
    decode("float_literals.json").or_fail()
}

#[test]
pub fn decode_float_memory() -> orfail::Result<()> {
    decode("float_memory.json").or_fail()
}

#[test]
pub fn decode_float_misc() -> orfail::Result<()> {
    decode("float_misc.json").or_fail()
}

#[test]
pub fn decode_forward() -> orfail::Result<()> {
    decode("forward.json").or_fail()
}

#[test]
pub fn decode_func() -> orfail::Result<()> {
    decode("func.json").or_fail()
}

#[test]
pub fn decode_func_ptrs() -> orfail::Result<()> {
    decode("func_ptrs.json").or_fail()
}

#[test]
pub fn decode_global() -> orfail::Result<()> {
    decode("global.json").or_fail()
}

#[test]
pub fn decode_i32() -> orfail::Result<()> {
    decode("i32.json").or_fail()
}

#[test]
pub fn decode_i64() -> orfail::Result<()> {
    decode("i64.json").or_fail()
}

#[test]
pub fn decode_if() -> orfail::Result<()> {
    decode("if.json").or_fail()
}

#[test]
pub fn decode_imports() -> orfail::Result<()> {
    decode("imports.json").or_fail()
}

#[test]
pub fn decode_inline_module() -> orfail::Result<()> {
    decode("inline-module.json").or_fail()
}

#[test]
pub fn decode_int_exprs() -> orfail::Result<()> {
    decode("int_exprs.json").or_fail()
}

#[test]
pub fn decode_int_literals() -> orfail::Result<()> {
    decode("int_literals.json").or_fail()
}

#[test]
pub fn decode_labels() -> orfail::Result<()> {
    decode("labels.json").or_fail()
}

#[test]
pub fn decode_left_to_right() -> orfail::Result<()> {
    decode("left-to-right.json").or_fail()
}

#[test]
pub fn decode_linking() -> orfail::Result<()> {
    decode("linking.json").or_fail()
}

// ../testdata/linking.json
// ../testdata/load.json
// ../testdata/local_get.json
// ../testdata/local_set.json
// ../testdata/local_tee.json
// ../testdata/loop.json
// ../testdata/memory.json
// ../testdata/memory_copy.json
// ../testdata/memory_fill.json
// ../testdata/memory_grow.json
// ../testdata/memory_init.json
// ../testdata/memory_redundancy.json
// ../testdata/memory_size.json
// ../testdata/memory_trap.json
// ../testdata/names.json
// ../testdata/nop.json
// ../testdata/obsolete-keywords.json
// ../testdata/ref_func.json
// ../testdata/ref_is_null.json
// ../testdata/ref_null.json
// ../testdata/return.json
// ../testdata/select.json
// ../testdata/skip-stack-guard-page.json
// ../testdata/stack.json
// ../testdata/start.json
// ../testdata/store.json
// ../testdata/switch.json
// ../testdata/table-sub.json
// ../testdata/table.json
// ../testdata/table_copy.json
// ../testdata/table_fill.json
// ../testdata/table_get.json
// ../testdata/table_grow.json
// ../testdata/table_init.json
// ../testdata/table_set.json
// ../testdata/table_size.json
// ../testdata/token.json
// ../testdata/traps.json
// ../testdata/type.json
// ../testdata/unreachable.json
// ../testdata/unreached-invalid.json
// ../testdata/unreached-valid.json
// ../testdata/unwind.json
// ../testdata/utf8-custom-section-id.json
// ../testdata/utf8-import-field.json
// ../testdata/utf8-import-module.json
// ../testdata/utf8-invalid-encoding.json
