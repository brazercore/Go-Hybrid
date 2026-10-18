//! const_overflow.rs — regression for issue #148: a constant initializer
//! outside a declared fixed-width integer type used to pass `hyb check`
//! (0 diagnostics) and then fail `go vet` / `go build` with
//! `cannot use N as T value (overflows)`.

use hy_check::{check_file, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn error_codes(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

const OVERFLOW_INT8: &str = "\
fn main() {
    var n int8 = 300
    fmt.println(\"{n}\")
}
";

const OVERFLOW_UINT8: &str = "\
fn main() {
    var n uint8 = 256
    fmt.println(\"{n}\")
}
";

const OVERFLOW_BYTE_HEX: &str = "\
fn main() {
    var n byte = 0x100
    fmt.println(\"{n}\")
}
";

const OVERFLOW_NEG_INT8: &str = "\
fn main() {
    var n int8 = -129
    fmt.println(\"{n}\")
}
";

const OVERFLOW_FILE_CONST: &str = "\
const n int8 = 300

fn main() {
    fmt.println(\"{n}\")
}
";

const FITS_INT8: &str = "\
fn main() {
    var n int8 = 127
    var m int8 = -128
    fmt.println(\"{n} {m}\")
}
";

const FITS_BYTE: &str = "\
fn main() {
    var n byte = 255
    fmt.println(\"{n}\")
}
";

const VARIABLE_OPERAND_OK: &str = "\
fn main() {
    x := 300
    y := byte(x)
    fmt.println(\"{y}\")
}
";

const PAREN_OVERFLOW: &str = "\
fn main() {
    var n int8 = (300)
    fmt.println(\"{n}\")
}
";

#[test]
fn int8_constant_300_is_rejected() {
    let r = check(OVERFLOW_INT8);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for var n int8 = 300, got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn uint8_constant_256_is_rejected() {
    let r = check(OVERFLOW_UINT8);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002, got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn byte_hex_0x100_is_rejected() {
    let r = check(OVERFLOW_BYTE_HEX);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for byte = 0x100, got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn int8_neg_129_is_rejected() {
    let r = check(OVERFLOW_NEG_INT8);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for -129, got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn file_level_const_overflow_is_rejected() {
    let r = check(OVERFLOW_FILE_CONST);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for file-level const, got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn paren_wrapped_overflow_is_rejected() {
    let r = check(PAREN_OVERFLOW);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for (300), got {:?}",
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject);
}

#[test]
fn int8_boundary_values_accept() {
    let r = check(FITS_INT8);
    assert!(
        error_codes(&r).is_empty(),
        "127 / -128 must fit int8, got {:?}",
        error_codes(&r)
    );
    assert_eq!(r.verdict, Verdict::Accept);
}

#[test]
fn byte_255_accepts() {
    let r = check(FITS_BYTE);
    assert!(
        error_codes(&r).is_empty(),
        "255 must fit byte, got {:?}",
        error_codes(&r)
    );
    assert_eq!(r.verdict, Verdict::Accept);
}

#[test]
fn variable_conversion_truncation_still_accepts() {
    // Issue #148 control: conversion from a *variable* is the runtime
    // truncation path (SPEC X4) and must stay accepted.
    let r = check(VARIABLE_OPERAND_OK);
    assert!(
        error_codes(&r).is_empty(),
        "variable conversion must stay clean, got {:?}",
        error_codes(&r)
    );
    assert_eq!(r.verdict, Verdict::Accept);
}

// ---------------------------------------------------------------------------
// Issue #514 — the same constant-range rule through a *place* target.
// Every program below used to pass `hyb check` with 0 diagnostics and then
// fail `go build` with `cannot use N as T value (overflows)`.
// ---------------------------------------------------------------------------

const FIELD_OVERFLOW: &str = "\
struct user {
    age int8
}

fn main() {
    var u user
    u.age = 300
    fmt.println(\"{u.age}\")
}
";

const NESTED_FIELD_OVERFLOW: &str = "\
struct addr {
    zip int16
}

struct person {
    addr addr
}

fn main() {
    var p person
    p.addr.zip = 70000
    fmt.println(\"{p.addr.zip}\")
}
";

const POINTER_FIELD_OVERFLOW: &str = "\
struct user {
    age int8
}

fn bump(p *user) {
    p.age = 300
}

fn main() {
    var u user
    bump(&u)
    fmt.println(\"{u.age}\")
}
";

const SLICE_OF_STRUCT_FIELD_OVERFLOW: &str = "\
struct item {
    code uint8
}

fn main() {
    var items []item
    items[0].code = 300
    fmt.println(\"{items[0].code}\")
}
";

const COMPOUND_FIELD_OVERFLOW: &str = "\
struct user {
    age int8
}

fn main() {
    var u user
    u.age += 200
    fmt.println(\"{u.age}\")
}
";

const FIELD_FITS: &str = "\
struct user {
    age int8
}

fn main() {
    var u user
    u.age = 100
    fmt.println(\"{u.age}\")
}
";

const FIELD_FROM_VARIABLE_OK: &str = "\
struct user {
    age int8
}

fn main() {
    var u user
    var n int8 = 100
    u.age = n
    fmt.println(\"{u.age}\")
}
";

fn rejects_e6002(src: &str, what: &str) {
    let r = check(src);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E6002"),
        "expected HYB-E6002 for {}, got {:?}",
        what,
        cs
    );
    assert_eq!(r.verdict, Verdict::Reject, "{} must reject", what);
}

#[test]
fn struct_field_constant_overflow_is_rejected() {
    rejects_e6002(FIELD_OVERFLOW, "u.age = 300 with age int8");
}

#[test]
fn nested_struct_field_overflow_is_rejected() {
    rejects_e6002(NESTED_FIELD_OVERFLOW, "p.addr.zip = 70000 with zip int16");
}

#[test]
fn pointer_struct_field_overflow_is_rejected() {
    rejects_e6002(POINTER_FIELD_OVERFLOW, "p.age = 300 through *user");
}

#[test]
fn slice_of_struct_field_overflow_is_rejected() {
    rejects_e6002(
        SLICE_OF_STRUCT_FIELD_OVERFLOW,
        "items[0].code = 300 with code uint8",
    );
}

#[test]
fn compound_field_assign_overflow_is_rejected() {
    rejects_e6002(COMPOUND_FIELD_OVERFLOW, "u.age += 200 with age int8");
}

#[test]
fn struct_field_in_range_accepts() {
    let r = check(FIELD_FITS);
    assert!(
        error_codes(&r).is_empty(),
        "u.age = 100 must fit int8, got {:?}",
        error_codes(&r)
    );
    assert_eq!(r.verdict, Verdict::Accept);
}

#[test]
fn struct_field_from_variable_accepts() {
    // Non-constant operands stay out of scope (SPEC X4): Go truncates the
    // variable at run time, so this must remain accepted.
    let r = check(FIELD_FROM_VARIABLE_OK);
    assert!(
        error_codes(&r).is_empty(),
        "u.age = n (n int8) must stay clean, got {:?}",
        error_codes(&r)
    );
    assert_eq!(r.verdict, Verdict::Accept);
}
