use crate::fonts::JB_MONO;
use iced::mouse;
use iced::widget::{button, canvas, column, container, row, space, text};
use iced::{Alignment, Border, Color, Element, Length, Point, Rectangle, Renderer, Theme};
use iced_aw::color_picker::ColorPicker;
use iced_aw::number_input::NumberInput;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub show_up: bool,
    pub show_down: bool,
    pub show_left: bool,
    pub show_right: bool,
}

pub fn anoto_control<'a, Message>(
    state: &'a State,
    dot_size: f32,
    origin_dist: f32,
    dot_origin_dist: f32,
    color_up: Color,
    color_down: Color,
    color_left: Color,
    color_right: Color,
    on_toggle_up: impl Fn(bool) -> Message + 'static + Clone,
    on_toggle_down: impl Fn(bool) -> Message + 'static + Clone,
    on_toggle_left: impl Fn(bool) -> Message + 'static + Clone,
    on_toggle_right: impl Fn(bool) -> Message + 'static + Clone,
    on_color_up_change: impl Fn(Color) -> Message + 'static + Clone,
    on_color_down_change: impl Fn(Color) -> Message + 'static + Clone,
    on_color_left_change: impl Fn(Color) -> Message + 'static + Clone,
    on_color_right_change: impl Fn(Color) -> Message + 'static + Clone,
    on_dot_size_change: impl Fn(f32) -> Message + 'static + Clone,
    on_origin_dist_change: impl Fn(f32) -> Message + 'static + Clone,
    on_dot_origin_dist_change: impl Fn(f32) -> Message + 'static + Clone,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    let up_dot = dot_button(
        Direction::Up,
        color_up,
        state.show_up,
        on_toggle_up,
        on_color_up_change,
    );
    let down_dot = dot_button(
        Direction::Down,
        color_down,
        state.show_down,
        on_toggle_down,
        on_color_down_change,
    );
    let left_dot = dot_button(
        Direction::Left,
        color_left,
        state.show_left,
        on_toggle_left,
        on_color_left_change,
    );
    let right_dot = dot_button(
        Direction::Right,
        color_right,
        state.show_right,
        on_toggle_right,
        on_color_right_change,
    );

    // Round to 1 decimal place for display
    let dot_size = (dot_size * 10.0).round() / 10.0;

    let dot_size_input = column![
        text("Dot Size").size(12).font(JB_MONO),
        NumberInput::new(&dot_size, 0.1..=100.0, on_dot_size_change)
            .step(0.1)
            .width(Length::Fixed(70.0))
    ]
    .align_x(Alignment::Center);

    let origin_dist = (origin_dist * 10.0).round() / 10.0;
    let origin_dist_input = column![
        text("Origin Dist").size(12).font(JB_MONO),
        NumberInput::new(&origin_dist, 0.0..=100.0, on_origin_dist_change)
            .step(0.1)
            .width(Length::Fixed(70.0))
    ]
    .align_x(Alignment::Center);

    let dot_origin_dist = (dot_origin_dist * 10.0).round() / 10.0;
    let dot_origin_dist_input = column![
        text("Dot-Origin Dist").size(12).font(JB_MONO),
        NumberInput::new(&dot_origin_dist, 0.0..=100.0, on_dot_origin_dist_change)
            .step(0.1)
            .width(Length::Fixed(70.0))
    ]
    .align_x(Alignment::Center);

    // Layout
    // Row 1: Dot Size, Up Dot, Origin Dist
    let row1 = row![
        dot_size_input,
        space().width(Length::Fixed(20.0)),
        up_dot,
        space().width(Length::Fixed(20.0)),
        origin_dist_input
    ]
    .align_y(Alignment::Center);

    // Row 2: Left Dot, Dot-Origin Dist, Right Dot
    let row2 = row![
        left_dot,
        space().width(Length::Fixed(20.0)),
        dot_origin_dist_input,
        space().width(Length::Fixed(20.0)),
        right_dot
    ]
    .align_y(Alignment::Center);

    // Row 3: Down Dot
    let row3 = row![down_dot].align_y(Alignment::Center);

    container(
        column![
            row1,
            space().height(Length::Fixed(20.0)),
            row2,
            space().height(Length::Fixed(20.0)),
            row3
        ]
        .align_x(Alignment::Center),
    )
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

fn dot_button<'a, Message>(
    direction: Direction,
    color: Color,
    show_picker: bool,
    on_toggle: impl Fn(bool) -> Message + 'static + Clone,
    on_change: impl Fn(Color) -> Message + 'static + Clone,
) -> Element<'a, Message>
where
    Message: Clone + 'static,
{
    let canvas = canvas(DotProgram { direction, color })
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

    let btn = button(canvas)
        .on_press(on_toggle(true))
        .padding(0)
        .style(button::text);

    ColorPicker::new(show_picker, color, btn, on_toggle(false), on_change).into()
}

struct DotProgram {
    direction: Direction,
    color: Color,
}

impl<Message> canvas::Program<Message> for DotProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let center = frame.center();
        let radius = bounds.width.min(bounds.height) / 2.0;

        let background = canvas::Path::circle(center, radius);
        frame.fill(&background, self.color);

        // Draw arrow
        let arrow_color = Color::WHITE; // Or contrasting color
        let arrow_size = radius * 0.6;

        let arrow_path = canvas::Path::new(|p| match self.direction {
            Direction::Up => {
                p.move_to(Point::new(center.x, center.y + arrow_size / 2.0));
                p.line_to(Point::new(center.x, center.y - arrow_size / 2.0));
                p.line_to(Point::new(
                    center.x - arrow_size / 3.0,
                    center.y - arrow_size / 6.0,
                ));
                p.move_to(Point::new(center.x, center.y - arrow_size / 2.0));
                p.line_to(Point::new(
                    center.x + arrow_size / 3.0,
                    center.y - arrow_size / 6.0,
                ));
            }
            Direction::Down => {
                p.move_to(Point::new(center.x, center.y - arrow_size / 2.0));
                p.line_to(Point::new(center.x, center.y + arrow_size / 2.0));
                p.line_to(Point::new(
                    center.x - arrow_size / 3.0,
                    center.y + arrow_size / 6.0,
                ));
                p.move_to(Point::new(center.x, center.y + arrow_size / 2.0));
                p.line_to(Point::new(
                    center.x + arrow_size / 3.0,
                    center.y + arrow_size / 6.0,
                ));
            }
            Direction::Left => {
                p.move_to(Point::new(center.x + arrow_size / 2.0, center.y));
                p.line_to(Point::new(center.x - arrow_size / 2.0, center.y));
                p.line_to(Point::new(
                    center.x - arrow_size / 6.0,
                    center.y - arrow_size / 3.0,
                ));
                p.move_to(Point::new(center.x - arrow_size / 2.0, center.y));
                p.line_to(Point::new(
                    center.x - arrow_size / 6.0,
                    center.y + arrow_size / 3.0,
                ));
            }
            Direction::Right => {
                p.move_to(Point::new(center.x - arrow_size / 2.0, center.y));
                p.line_to(Point::new(center.x + arrow_size / 2.0, center.y));
                p.line_to(Point::new(
                    center.x + arrow_size / 6.0,
                    center.y - arrow_size / 3.0,
                ));
                p.move_to(Point::new(center.x + arrow_size / 2.0, center.y));
                p.line_to(Point::new(
                    center.x + arrow_size / 6.0,
                    center.y + arrow_size / 3.0,
                ));
            }
        });

        frame.stroke(
            &arrow_path,
            canvas::Stroke::default()
                .with_color(arrow_color)
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }
}
