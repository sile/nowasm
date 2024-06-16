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
                    | "linking.9.wasm"
                    | "linking.10.wasm"
                    | "linking.26.wasm"
                    | "select.0.wasm"
                    | "select.29.wasm"
                    | "unreached-valid.0.wasm"
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

#[test]
pub fn decode_load() -> orfail::Result<()> {
    decode("load.json").or_fail()
}

#[test]
pub fn decode_local_get() -> orfail::Result<()> {
    decode("local_get.json").or_fail()
}

#[test]
pub fn decode_local_set() -> orfail::Result<()> {
    decode("local_set.json").or_fail()
}

#[test]
pub fn decode_local_tee() -> orfail::Result<()> {
    decode("local_tee.json").or_fail()
}

#[test]
pub fn decode_loop() -> orfail::Result<()> {
    decode("loop.json").or_fail()
}

#[test]
pub fn decode_memory() -> orfail::Result<()> {
    decode("memory.json").or_fail()
}

#[test]
pub fn decode_memory_copy() -> orfail::Result<()> {
    decode("memory_copy.json").or_fail()
}

#[test]
pub fn decode_memory_fill() -> orfail::Result<()> {
    decode("memory_fill.json").or_fail()
}

#[test]
pub fn decode_memory_grow() -> orfail::Result<()> {
    decode("memory_grow.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_memory_init() -> orfail::Result<()> {
//     decode("memory_init.json").or_fail()
// }

#[test]
pub fn decode_memory_redundancy() -> orfail::Result<()> {
    decode("memory_redundancy.json").or_fail()
}

#[test]
pub fn decode_memory_size() -> orfail::Result<()> {
    decode("memory_size.json").or_fail()
}

#[test]
pub fn decode_memory_trap() -> orfail::Result<()> {
    decode("memory_trap.json").or_fail()
}

#[test]
pub fn decode_names() -> orfail::Result<()> {
    decode("names.json").or_fail()
}

#[test]
pub fn decode_nop() -> orfail::Result<()> {
    decode("nop.json").or_fail()
}

#[test]
pub fn decode_obsolete_keywords() -> orfail::Result<()> {
    decode("obsolete-keywords.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_ref_func() -> orfail::Result<()> {
//     decode("ref_func.json").or_fail()
// }
//
// #[test]
// pub fn decode_ref_is_null() -> orfail::Result<()> {
//     decode("ref_is_null.json").or_fail()
// }
//
// #[test]
// pub fn decode_ref_null() -> orfail::Result<()> {
//     decode("ref_null.json").or_fail()
// }

#[test]
pub fn decode_return() -> orfail::Result<()> {
    decode("return.json").or_fail()
}

#[test]
pub fn decode_select() -> orfail::Result<()> {
    decode("select.json").or_fail()
}

#[test]
pub fn decode_skip_stack_guard_page() -> orfail::Result<()> {
    decode("skip-stack-guard-page.json").or_fail()
}

#[test]
pub fn decode_stack() -> orfail::Result<()> {
    decode("stack.json").or_fail()
}

#[test]
pub fn decode_start() -> orfail::Result<()> {
    decode("start.json").or_fail()
}

#[test]
pub fn decode_store() -> orfail::Result<()> {
    decode("store.json").or_fail()
}

#[test]
pub fn decode_switch() -> orfail::Result<()> {
    decode("switch.json").or_fail()
}

#[test]
pub fn decode_table_sub() -> orfail::Result<()> {
    decode("table-sub.json").or_fail()
}

#[test]
pub fn decode_table() -> orfail::Result<()> {
    decode("table.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_table_copy() -> orfail::Result<()> {
//     decode("table_copy.json").or_fail()
// }

#[test]
pub fn decode_table_fill() -> orfail::Result<()> {
    decode("table_fill.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_table_get() -> orfail::Result<()> {
//     decode("table_get.json").or_fail()
// }
//
// #[test]
// pub fn decode_table_grow() -> orfail::Result<()> {
//     decode("table_grow.json").or_fail()
// }
//
// #[test]
// pub fn decode_table_init() -> orfail::Result<()> {
//     decode("table_init.json").or_fail()
// }
//
// #[test]
// pub fn decode_table_set() -> orfail::Result<()> {
//     decode("table_set.json").or_fail()
// }

#[test]
pub fn decode_table_size() -> orfail::Result<()> {
    decode("table_size.json").or_fail()
}

// Unsupported in 1.0
//
// #[test]
// pub fn decode_token() -> orfail::Result<()> {
//     decode("token.json").or_fail()
// }

#[test]
pub fn decode_traps() -> orfail::Result<()> {
    decode("traps.json").or_fail()
}

#[test]
pub fn decode_type() -> orfail::Result<()> {
    decode("type.json").or_fail()
}

#[test]
pub fn decode_unreachable() -> orfail::Result<()> {
    decode("unreachable.json").or_fail()
}

#[test]
pub fn decode_unreached_invalid() -> orfail::Result<()> {
    decode("unreached-invalid.json").or_fail()
}

#[test]
pub fn decode_unreached_valid() -> orfail::Result<()> {
    decode("unreached-valid.json").or_fail()
}

#[test]
pub fn decode_unwind() -> orfail::Result<()> {
    decode("unwind.json").or_fail()
}

#[test]
pub fn decode_utf8_custom_section_id() -> orfail::Result<()> {
    decode("utf8-custom-section-id.json").or_fail()
}

#[test]
pub fn decode_utf8_import_field() -> orfail::Result<()> {
    decode("utf8-import-field.json").or_fail()
}

#[test]
pub fn decode_utf8_import_module() -> orfail::Result<()> {
    decode("utf8-import-module.json").or_fail()
}

#[test]
pub fn decode_utf8_invalid_encoding() -> orfail::Result<()> {
    decode("utf8-invalid-encoding.json").or_fail()
}
