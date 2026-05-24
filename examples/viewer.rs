//! Standalone Iced viewer showing the v0.1 corpus.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Task, Theme};

fn title(_: &App) -> String {
    String::from("iced_math viewer")
}

fn theme(_: &App) -> Theme {
    Theme::Light
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(title)
        .theme(theme)
        .run()
}

#[derive(Default)]
struct App;

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> Self {
        App
    }

    fn update(&mut self, _: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let entries = [
            ("E = mc^2", false),
            (r"e^{i\pi} + 1 = 0", false),
            (r"\frac{1}{2} + \frac{1}{3} = \frac{5}{6}", true),
            (r"\sqrt{x^2 + y^2}", false),
            (r"\left( \frac{a+b}{c} \right)^2", true),
            (r"\frac{d}{dx}\left( x^2 \right) = 2x", true),
            (r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}", true),
            (r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}", true),
            (r"\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}", true),
            (r"\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}", true),
            (r"f(x) = \frac{1}{\sqrt{2\pi\sigma^2}} e^{-\frac{(x-\mu)^2}{2\sigma^2}}", true),
            // v0.3: named operators, accents, matrices.
            (r"\lim_{x \to 0} \frac{\sin x}{x} = 1", true),
            (r"\cos^2\theta + \sin^2\theta = 1", false),
            (r"\vec{F} = m\,\vec{a}", false),
            (r"\hat{p}\,\psi = -i\hbar\,\nabla\psi", false),
            (r"A = \begin{pmatrix} a & b \\ c & d \end{pmatrix}", true),
            (r"|x| = \begin{cases} x & x \ge 0 \\ -x & x < 0 \end{cases}", true),
        ];
        let items: Vec<Element<'_, Message>> = entries
            .iter()
            .map(|(src, display)| {
                let lhs = text((*src).to_string()).size(14);
                let rhs: Element<'_, Message> = if *display {
                    iced_math::block(src)
                } else {
                    iced_math::inline(src)
                };
                row![
                    container(lhs).width(Length::Fixed(360.0)),
                    container(rhs).width(Length::Fill).padding(10)
                ]
                .spacing(20)
                .into()
            })
            .collect();

        scrollable(column(items).spacing(15).padding(20)).into()
    }
}
