use iced_math::font;

#[test]
fn reads_axis_height() {
    let h = font::math_constant(font::MathConstant::AxisHeight, 16.0);
    assert!(h > 0.0 && h < 16.0, "AxisHeight should be small positive px, got {}", h);
}

#[test]
fn reads_fraction_rule_thickness() {
    let t = font::math_constant(font::MathConstant::FractionRuleThickness, 16.0);
    assert!(t > 0.0 && t < 2.0, "FractionRuleThickness should be ~1px, got {}", t);
}
