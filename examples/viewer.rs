//! Standalone Iced viewer showing the v0.1 corpus.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Task, Theme};

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
            (r"\frac{1}{2} + \frac{1}{3} = \frac{5}{6}", false),
            (r"\sqrt{x^2 + y^2}", false),
            (r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}", true),
            (r"\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}", true),
            (r"\left( \frac{a+b}{c} \right)^2", false),
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
                row![lhs, container(rhs).padding(10)].spacing(20).into()
            })
            .collect();

        scrollable(column(items).spacing(15).padding(20)).into()
    }
}
