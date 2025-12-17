use iced::widget::{column, container, row, slider, space, text};
use iced::{Alignment, Border, Color, Element, Length};
use iced_aw::number_input::NumberInput;

pub fn section_control<'a, Message>(
    sect_u: i32,
    sect_v: i32,
    on_sect_u_change: impl Fn(i32) -> Message + 'static + Clone,
    on_sect_v_change: impl Fn(i32) -> Message + 'static + Clone,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    let on_sect_u_change_slider = on_sect_u_change.clone();
    let sect_u_ctrl = row![
        slider(
            1.0..=100.0,
            sect_u as f64,
            move |v| on_sect_u_change_slider(v as i32)
        )
        .step(1.0)
        .width(Length::Fill),
        NumberInput::new(&sect_u, 1..=100, on_sect_u_change)
            .step(1)
            .width(Length::Fixed(60.0))
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let on_sect_v_change_slider = on_sect_v_change.clone();
    let sect_v_ctrl = row![
        slider(
            1.0..=100.0,
            sect_v as f64,
            move |v| on_sect_v_change_slider(v as i32)
        )
        .step(1.0)
        .width(Length::Fill),
        NumberInput::new(&sect_v, 1..=100, on_sect_v_change)
            .step(1)
            .width(Length::Fixed(60.0))
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(column![
        text("Sect U").size(14),
        sect_u_ctrl,
        space().height(Length::Fixed(10.0)),
        text("Sect V").size(14),
        sect_v_ctrl,
    ])
    .padding(10)
    .style(|_theme| container::Style {
        border: Border {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}
