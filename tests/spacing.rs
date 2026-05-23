use iced_math::{ir::AtomClass, spacing};

#[test]
fn ord_op_is_thin() {
    let s = spacing::between(AtomClass::Ord, AtomClass::Op, true);
    assert_eq!(s, spacing::Spacing::Thin);
}

#[test]
fn bin_bin_is_none_per_tex() {
    let s = spacing::between(AtomClass::Bin, AtomClass::Bin, true);
    assert_eq!(s, spacing::Spacing::None);
}

#[test]
fn ord_rel_is_thick() {
    let s = spacing::between(AtomClass::Ord, AtomClass::Rel, true);
    assert_eq!(s, spacing::Spacing::Thick);
}

#[test]
fn script_style_suppresses_med_thick() {
    let s = spacing::between(AtomClass::Ord, AtomClass::Rel, false);
    assert_eq!(s, spacing::Spacing::None);
}
