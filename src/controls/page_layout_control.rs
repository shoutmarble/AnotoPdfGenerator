use iced::widget::{checkbox, column, container, row, slider, space, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_aw::number_input::NumberInput;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub autodetect: bool,
}

pub fn page_layout_control<'a, Message>(
    state: &'a State,
    width: usize,
    height: usize,
    on_width_change: impl Fn(usize) -> Message + 'static + Clone,
    on_height_change: impl Fn(usize) -> Message + 'static + Clone,
    on_autodetect_change: impl Fn(bool) -> Message + 'static + Clone,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    let on_width_change_slider = on_width_change.clone();
    let width_ctrl = row![
        slider(1.0..=200.0, width as f64, move |v| on_width_change_slider(
            v as usize
        ))
        .step(1.0)
        .width(Length::Fill),
        NumberInput::new(&width, 1..=200, on_width_change)
            .step(1)
            .width(Length::Fixed(60.0))
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let on_height_change_slider = on_height_change.clone();
    let height_ctrl = row![
        slider(
            1.0..=200.0,
            height as f64,
            move |v| on_height_change_slider(v as usize)
        )
        .step(1.0)
        .width(Length::Fill),
        NumberInput::new(&height, 1..=200, on_height_change)
            .step(1)
            .width(Length::Fixed(60.0))
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let content = container(column![
        text("Number of dots on X-axis horizontally across")
            .size(14)
            .color([0.2, 0.2, 0.8]),
        width_ctrl,
        space().height(Length::Fixed(10.0)),
        text("Number of dots on Y-axis vertically across")
            .size(14)
            .color([0.8, 0.2, 0.8]),
        height_ctrl,
        space().height(Length::Fixed(20.0)),
        checkbox(state.autodetect)
            .label(
                "Autodetect number of X-Axis and Y-Axis dots to fill an A4 page at set DPI level"
            )
            .on_toggle(on_autodetect_change)
            .size(20)
            .text_size(14)
            .text_shaping(iced::widget::text::Shaping::Advanced),
    ])
    .padding(10)
    .style(|_theme| container::Style {
        border: Border {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    });

    if state.autodetect {
        // If autodetect is on, we might want to disable the inputs,
        // but iced doesn't have a generic "disable" wrapper easily accessible for all widgets without custom styles or logic.
        // For now, we just render them. The logic in the main app should prevent updates or overwrite them.
        // Or we could not pass the `on_change` events?
        // But the signature requires them.
        // We can wrap them in a container that intercepts events? No.
        // We'll rely on the main app logic to enforce the values.
        content.into()
    } else {
        content.into()
    }
}
