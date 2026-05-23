#[allow(dead_code)]
fn must_compile_with_default_theme() -> iced::Element<'static, ()> {
    iced_math::inline::<(), iced::Theme, iced::Renderer>("E = mc^2")
}

#[allow(dead_code)]
fn block_must_compile() -> iced::Element<'static, ()> {
    iced_math::block::<(), iced::Theme, iced::Renderer>(r"\frac{a}{b}")
}

#[test]
fn smoke() {
    let _ = must_compile_with_default_theme();
    let _ = block_must_compile();
}
