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
        if command.filename.to_str() == Some("call_indirect.1.wasm") {
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

// ../testdata/data.json
// ../testdata/elem.json
// ../testdata/endianness.json
// ../testdata/exports.json
// ../testdata/f32.json
// ../testdata/f32_bitwise.json
// ../testdata/f32_cmp.json
// ../testdata/f64.json
// ../testdata/f64_bitwise.json
// ../testdata/f64_cmp.json
// ../testdata/fac.json
// ../testdata/float_exprs.json
// ../testdata/float_literals.json
// ../testdata/float_memory.json
// ../testdata/float_misc.json
// ../testdata/forward.json
// ../testdata/func.json
// ../testdata/func_ptrs.json
// ../testdata/global.json
// ../testdata/i32.json
// ../testdata/i64.json
// ../testdata/if.json
// ../testdata/imports.json
// ../testdata/inline-module.json
// ../testdata/int_exprs.json
// ../testdata/int_literals.json
// ../testdata/labels.json
// ../testdata/left-to-right.json
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
