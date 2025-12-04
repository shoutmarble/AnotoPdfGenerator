use iced::widget::{button, column, container, row, scrollable, slider, text, text_editor, text_input, vertical_space};
use iced::widget::image::{self, Image};
use iced::{Element, Length, Task, ContentFit, Border, Color, Shadow};
use iced_aw::spinner::Spinner;
use anoto_pdf::pdf_dotpaper::gen_pdf::{PdfConfig, gen_pdf_from_matrix_data};
use anoto_pdf::anoto_matrix::generate_matrix_only;
use anoto_pdf::make_plots::{draw_preview_image, draw_dot_on_file};
use anoto_pdf::controls::{anoto_control, page_layout_control, section_control};
use tokio::sync::{oneshot, mpsc, Mutex};
use std::sync::Arc;

use anoto_pdf::codec::anoto_6x6_a4_fixed;
use serde_json::Value;

const JB_MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/ttf/JetBrainsMonoNL-Medium.ttf");

pub fn main() -> iced::Result {
    iced::application("Anoto PDF Generator", Gui::update, Gui::view)
        .window_size((800.0, 600.0))
        .centered()
        .scale_factor(|s| s.ui_scale as f64)
        .font(JB_MONO_BYTES)
        .run()
}

struct Gui {
    config: PdfConfig,
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
    status_message: String,
    sect_u_str: String,
    sect_v_str: String,
    ui_scale: f32,
    generated_image_handle: Option<image::Handle>,
    control_state: anoto_control::State,
    page_layout_state: page_layout_control::State,
    is_generating: bool,
    server_port: String,
    server_shutdown_tx: Option<oneshot::Sender<()>>,
    server_status_text: String,
    server_rx: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
    rest_post_content: text_editor::Content,
    json_input: text_editor::Content,
    decoded_result: String,
    lookup_sect_u: String,
    lookup_sect_v: String,
    lookup_x: String,
    lookup_y: String,
    lookup_result: text_editor::Content,
    draw_x: String,
    draw_y: String,
    draw_status: String,
    current_png_path: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    UiScaleChanged(f32),
    DpiChanged(f32),
    DotSizeChanged(f32),
    OffsetChanged(f32),
    SpacingChanged(f32),
    HeightChanged(usize),
    WidthChanged(usize),
    AutodetectChanged(bool),
    SectUChanged(i32),
    SectVChanged(i32),
    GeneratePressed,
    GenerationFinished(Result<(image::Handle, String), String>),
    ToggleUpPicker(bool),
    ToggleDownPicker(bool),
    ToggleLeftPicker(bool),
    ToggleRightPicker(bool),
    ColorUpPicked(Color),
    ColorDownPicked(Color),
    ColorLeftPicked(Color),
    ColorRightPicked(Color),
    ServerPortChanged(String),
    ToggleServer,
    ServerStarted(Result<(), String>),
    RestPostReceived(Option<String>),
    RestPostContentChanged(text_editor::Action),
    JsonInputChanged(text_editor::Action),
    DecodeJson,
    LookupSectUChanged(String),
    LookupSectVChanged(String),
    LookupXChanged(String),
    LookupYChanged(String),
    PerformLookup,
    LookupResultChanged(text_editor::Action),
    DrawXChanged(String),
    DrawYChanged(String),
    DrawDotPressed,
    DrawDotFinished(Result<image::Handle, String>),
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            config: PdfConfig::default(),
            height: 9,
            width: 16,
            sect_u: 10,
            sect_v: 10,
            status_message: "Ready".to_string(),
            sect_u_str: "10".to_string(),
            sect_v_str: "10".to_string(),
            ui_scale: 0.6,
            generated_image_handle: None,
            control_state: anoto_control::State::default(),
            page_layout_state: page_layout_control::State::default(),
            is_generating: false,
            server_port: "8080".to_string(),
            server_shutdown_tx: None,
            server_status_text: "Server Stopped".to_string(),
            server_rx: None,
            rest_post_content: text_editor::Content::new(),
            json_input: text_editor::Content::new(),
            decoded_result: "Ready to decode".to_string(),
            lookup_sect_u: "10".to_string(),
            lookup_sect_v: "10".to_string(),
            lookup_x: "0".to_string(),
            lookup_y: "0".to_string(),
            lookup_result: text_editor::Content::new(),
            draw_x: "0".to_string(),
            draw_y: "0".to_string(),
            draw_status: "Ready".to_string(),
            current_png_path: None,
        }
    }
}

#[derive(Clone)]
struct GenerationParams {
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
    config: PdfConfig,
}

impl Gui {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UiScaleChanged(val) => self.ui_scale = val,
            Message::DpiChanged(val) => self.config.dpi = val,
            Message::DotSizeChanged(val) => self.config.dot_size = (val * 10.0).round() / 10.0,
            Message::OffsetChanged(val) => self.config.offset_from_origin = (val * 10.0).round() / 10.0,
            Message::SpacingChanged(val) => {
                self.config.grid_spacing = (val * 10.0).round() / 10.0;
                if self.page_layout_state.autodetect {
                    self.recalculate_layout();
                }
            },
            Message::HeightChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.height = val;
                }
            },
            Message::WidthChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.width = val;
                }
            },
            Message::AutodetectChanged(val) => {
                self.page_layout_state.autodetect = val;
                if val {
                    self.recalculate_layout();
                }
            },
            Message::SectUChanged(val) => {
                self.sect_u = val;
                self.sect_u_str = val.to_string();
            },
            Message::SectVChanged(val) => {
                self.sect_v = val;
                self.sect_v_str = val.to_string();
            },
            Message::ToggleUpPicker(show) => self.control_state.show_up = show,
            Message::ToggleDownPicker(show) => self.control_state.show_down = show,
            Message::ToggleLeftPicker(show) => self.control_state.show_left = show,
            Message::ToggleRightPicker(show) => self.control_state.show_right = show,
            Message::ColorUpPicked(color) => {
                self.config.color_up = color_to_hex(color);
                self.control_state.show_up = false;
            },
            Message::ColorDownPicked(color) => {
                self.config.color_down = color_to_hex(color);
                self.control_state.show_down = false;
            },
            Message::ColorLeftPicked(color) => {
                self.config.color_left = color_to_hex(color);
                self.control_state.show_left = false;
            },
            Message::ColorRightPicked(color) => {
                self.config.color_right = color_to_hex(color);
                self.control_state.show_right = false;
            },
            Message::ServerPortChanged(port) => {
                if port.chars().all(|c| c.is_numeric()) {
                    self.server_port = port;
                }
            }
            Message::ToggleServer => {
                if self.server_shutdown_tx.is_some() {
                    // Stop server
                    if let Some(tx) = self.server_shutdown_tx.take() {
                        let _ = tx.send(());
                    }
                    self.server_status_text = "Server Stopped".to_string();
                    self.server_rx = None;
                } else {
                    // Start server
                    let port_str = self.server_port.clone();
                    let (tx, rx) = oneshot::channel();
                    let (msg_tx, msg_rx) = mpsc::channel(100);
                    self.server_shutdown_tx = Some(tx);
                    let rx_arc = Arc::new(Mutex::new(msg_rx));
                    self.server_rx = Some(rx_arc.clone());
                    self.server_status_text = "Starting Server...".to_string();

                    return Task::batch(vec![
                        Task::perform(async move {
                            start_server_task(port_str, rx, msg_tx).await
                        }, Message::ServerStarted),
                        listen_for_post(rx_arc)
                    ]);
                }
            }
            Message::RestPostReceived(content) => {
                if let Some(c) = content {
                    let clean_c = c.trim();
                    let json_candidate = if clean_c.starts_with('\'') && clean_c.ends_with('\'') {
                        &clean_c[1..clean_c.len()-1]
                    } else {
                        clean_c
                    };

                    let formatted_content = if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_candidate) {
                        if let Some(arr) = val.as_array() {
                            // Check if it's a 2D array
                            if arr.iter().all(|item| item.is_array()) {
                                let mut s = String::from("[\n");
                                for (i, row) in arr.iter().enumerate() {
                                    s.push_str("  ");
                                    s.push_str(&serde_json::to_string(row).unwrap_or_default());
                                    if i < arr.len() - 1 {
                                        s.push_str(",\n");
                                    } else {
                                        s.push_str("\n");
                                    }
                                }
                                s.push_str("]");
                                s
                            } else {
                                serde_json::to_string_pretty(&val).unwrap_or(json_candidate.to_string())
                            }
                        } else {
                            serde_json::to_string_pretty(&val).unwrap_or(json_candidate.to_string())
                        }
                    } else {
                        // Fallback heuristic formatting for when JSON parsing fails (e.g. missing quotes)
                        json_candidate.to_string()
                            .replace("],", "],\n  ")
                            .replace("], ", "],\n  ")
                            .replace("[[", "[\n  [")
                            .replace("]]", "]\n]")
                    };
                    self.rest_post_content = text_editor::Content::with_text(&formatted_content);
                    if let Some(rx) = &self.server_rx {
                        return listen_for_post(rx.clone());
                    }
                }
            }
            Message::RestPostContentChanged(action) => {
                self.rest_post_content.perform(action);
            }
            Message::ServerStarted(result) => {
                match result {
                    Ok(_) => {
                        self.server_status_text = "Server Running".to_string();
                    }
                    Err(e) => {
                        self.server_status_text = format!("Error: {}", e);
                        self.server_shutdown_tx = None;
                    }
                }
            }
            Message::JsonInputChanged(action) => {
                self.json_input.perform(action);
            }
            Message::DecodeJson => {
                self.decoded_result = decode_json_input(&self.json_input.text());
            }
            Message::LookupSectUChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.lookup_sect_u = val;
                }
            }
            Message::LookupSectVChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.lookup_sect_v = val;
                }
            }
            Message::LookupXChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.lookup_x = val;
                }
            }
            Message::LookupYChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.lookup_y = val;
                }
            }
            Message::LookupResultChanged(action) => {
                self.lookup_result.perform(action);
            }
            Message::PerformLookup => {
                let res = perform_pattern_lookup(&self.lookup_sect_u, &self.lookup_sect_v, &self.lookup_x, &self.lookup_y);
                self.lookup_result = text_editor::Content::with_text(&res);
            }
            Message::GeneratePressed => {
                if !self.is_generating {
                    self.is_generating = true;
                    self.status_message = "Generating PDF...".to_string();
                    let params = GenerationParams {
                        height: self.height,
                        width: self.width,
                        sect_u: self.sect_u,
                        sect_v: self.sect_v,
                        config: self.config.clone(),
                    };
                    return Task::perform(async move {
                        generate_and_save(params).await
                    }, Message::GenerationFinished);
                }
            },
            Message::GenerationFinished(result) => {
                self.is_generating = false;
                match result {
                    Ok((handle, path)) => {
                        self.status_message = "PDF Generated Successfully!".to_string();
                        self.generated_image_handle = Some(handle);
                        self.current_png_path = Some(path);
                    }
                    Err(e) => self.status_message = format!("Error: {}", e),
                }
            }
            Message::DrawXChanged(val) => {
                if val.chars().all(|c| c.is_numeric() || c == '.') {
                    self.draw_x = val;
                }
            }
            Message::DrawYChanged(val) => {
                if val.chars().all(|c| c.is_numeric() || c == '.') {
                    self.draw_y = val;
                }
            }
            Message::DrawDotPressed => {
                if let Some(path) = &self.current_png_path {
                    let x = self.draw_x.parse::<f64>().unwrap_or(0.0);
                    let y = self.draw_y.parse::<f64>().unwrap_or(0.0);
                    let config = self.config.clone();
                    let matrix_width = self.width;
                    let matrix_height = self.height;
                    let path_clone = path.clone();
                    
                    self.draw_status = "Drawing dot...".to_string();

                    return Task::perform(async move {
                        match draw_dot_on_file(&path_clone, x, y, matrix_height, matrix_width, &config) {
                            Ok(_) => {
                                // Reload the image
                                 match std::fs::read(&path_clone) {
                                    Ok(bytes) => Ok(image::Handle::from_bytes(bytes)),
                                    Err(e) => Err(e.to_string())
                                 }
                            },
                            Err(e) => Err(e.to_string())
                        }
                    }, Message::DrawDotFinished);
                } else {
                    self.draw_status = "Generate PDF first!".to_string();
                }
            }
            Message::DrawDotFinished(result) => {
                match result {
                    Ok(handle) => {
                        self.draw_status = "Dot drawn!".to_string();
                        self.generated_image_handle = Some(handle);
                    }
                    Err(e) => self.draw_status = format!("Error: {}", e),
                }
            }
        }
        Task::none()
    }

    fn recalculate_layout(&mut self) {
        let a4_width = 595.276;
        let a4_height = 841.89;
        let margin = 20.0;
        let spacing = self.config.grid_spacing;
        if spacing > 0.0 {
            self.width = ((a4_width - 2.0 * margin) / spacing) as usize + 1;
            self.height = ((a4_height - 2.0 * margin) / spacing) as usize + 1;
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let scale_slider = container(column![
            text(format!("UI Scale: {:.1}", self.ui_scale)),
            slider(0.5..=3.0, self.ui_scale, Message::UiScaleChanged).step(0.1)
        ].spacing(10))
        .padding(10)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

        let dpi_slider = container(column![
            text(format!("PDF DPI: {:.0}", self.config.dpi)),
            slider(300.0..=1200.0, self.config.dpi, Message::DpiChanged).step(10.0)
        ].spacing(10))
        .padding(10)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

        let anoto_ctrl = anoto_control::anoto_control(
            &self.control_state,
            self.config.dot_size,
            self.config.grid_spacing,
            self.config.offset_from_origin,
            hex_to_color(&self.config.color_up),
            hex_to_color(&self.config.color_down),
            hex_to_color(&self.config.color_left),
            hex_to_color(&self.config.color_right),
            Message::ToggleUpPicker,
            Message::ToggleDownPicker,
            Message::ToggleLeftPicker,
            Message::ToggleRightPicker,
            Message::ColorUpPicked,
            Message::ColorDownPicked,
            Message::ColorLeftPicked,
            Message::ColorRightPicked,
            Message::DotSizeChanged,
            Message::SpacingChanged,
            Message::OffsetChanged,
        );

        let page_layout = page_layout_control::page_layout_control(
            &self.page_layout_state,
            self.width,
            self.height,
            Message::WidthChanged,
            Message::HeightChanged,
            Message::AutodetectChanged,
        );

        let section_ctrl = section_control::section_control(
            self.sect_u,
            self.sect_v,
            Message::SectUChanged,
            Message::SectVChanged,
        );

        let matrix_inputs = column![
            text("Matrix Settings:"),
            page_layout,
            section_ctrl,
        ].spacing(10);

        let generate_btn = if self.is_generating {
            row![
                button("Generating...").width(Length::Fill),
                Spinner::new().width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            ].spacing(10)
        } else {
            row![
                button("Generate PDF").on_press(Message::GeneratePressed).width(Length::Fill)
            ]
        };

        let controls = column![
            text("Anoto PDF Generator").size(30),
            vertical_space().height(20),
            scale_slider,
            vertical_space().height(20),
            dpi_slider,
            vertical_space().height(20),
            anoto_ctrl,
            vertical_space().height(20),
            matrix_inputs,
            vertical_space().height(20),
            generate_btn,
            vertical_space().height(10),
            text(&self.status_message),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fixed(460.0));

        let image_preview = if let Some(handle) = &self.generated_image_handle {
             container(
                container(
                    Image::new(handle.clone())
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .padding(5)
                .style(|_theme| container::Style {
                    border: Border {
                        color: Color::from_rgb(0.2, 0.2, 0.2),
                        width: 5.0,
                        radius: 10.0.into(),
                    },
                    background: Some(Color::from_rgb(0.95, 0.95, 0.95).into()),
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                        offset: iced::Vector::new(5.0, 5.0),
                        blur_radius: 10.0,
                    },
                    ..container::Style::default()
                })
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(text("No image generated yet").size(20))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        };

        let server_controls = container(column![
            text("Web Server").size(20),
            vertical_space().height(10),
            row![
                text("Port: "),
                text_input("8080", &self.server_port)
                    .on_input(Message::ServerPortChanged)
                    .padding(5)
                    .width(Length::Fixed(80.0))
            ].spacing(10).align_y(iced::Alignment::Center),
            vertical_space().height(10),
            button(if self.server_shutdown_tx.is_some() { "Stop Server" } else { "Start Server" })
                .on_press(Message::ToggleServer)
                .padding(10)
                .width(Length::Fill),
            vertical_space().height(10),
            text(&self.server_status_text).size(14).color(if self.server_shutdown_tx.is_some() { Color::from_rgb(0.0, 0.8, 0.0) } else { Color::from_rgb(0.8, 0.0, 0.0) }),
        ]
        .spacing(10))
        .padding(20)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 2.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(200.0));

        let decoder_controls = container(column![
            text("Decoder").size(20),
            vertical_space().height(10),
            text_editor(&self.json_input)
                .on_action(Message::JsonInputChanged)
                .height(Length::Fixed(200.0))
                .font(iced::font::Font::MONOSPACE)
                .wrapping(iced::widget::text::Wrapping::None),
            vertical_space().height(10),
            button("Decode Position")
                .on_press(Message::DecodeJson)
                .padding(10)
                .width(Length::Fill),
            vertical_space().height(10),
            text(&self.decoded_result).size(14),
        ]
        .spacing(10))
        .padding(20)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 2.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(200.0));

        let lookup_controls = container(column![
            text("Pattern Lookup").size(20),
            vertical_space().height(10),
            row![
                column![
                    text("Sect U"),
                    text_input("10", &self.lookup_sect_u).on_input(Message::LookupSectUChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill),
                column![
                    text("Sect V"),
                    text_input("10", &self.lookup_sect_v).on_input(Message::LookupSectVChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill)
            ].spacing(10),
            row![
                column![
                    text("X"),
                    text_input("0", &self.lookup_x).on_input(Message::LookupXChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill),
                column![
                    text("Y"),
                    text_input("0", &self.lookup_y).on_input(Message::LookupYChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill)
            ].spacing(10),
            vertical_space().height(10),
            button("Lookup Pattern")
                .on_press(Message::PerformLookup)
                .padding(10)
                .width(Length::Fill),
            vertical_space().height(10),
            text_editor(&self.lookup_result)
                .on_action(Message::LookupResultChanged)
                .height(Length::Fixed(200.0))
                .font(iced::font::Font::MONOSPACE)
                .wrapping(iced::widget::text::Wrapping::None),
        ]
        .spacing(10))
        .padding(20)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 2.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(200.0));

        let draw_dot_controls = container(column![
            text("Draw Dot").size(20),
            vertical_space().height(10),
            row![
                column![
                    text("Position X:"),
                    text_input("0", &self.draw_x).on_input(Message::DrawXChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill),
                column![
                    text("Position Y:"),
                    text_input("0", &self.draw_y).on_input(Message::DrawYChanged).padding(5).width(Length::Fill)
                ].spacing(5).width(Length::Fill)
            ].spacing(10),
            vertical_space().height(10),
            button("Draw Dot")
                .on_press(Message::DrawDotPressed)
                .padding(10)
                .width(Length::Fill),
            vertical_space().height(10),
            text(&self.draw_status).size(14),
        ]
        .spacing(10))
        .padding(20)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 2.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(200.0));

        let rest_post_controls = container(column![
            text("JSON 6x6+ REST Service Post").size(20),
            vertical_space().height(10),
            text_editor(&self.rest_post_content)
                .on_action(Message::RestPostContentChanged)
                .height(Length::Fixed(300.0))
                .font(iced::font::Font::MONOSPACE)
                .wrapping(iced::widget::text::Wrapping::None),
            vertical_space().height(10),
            button("JSON 6x6 REST post")
                .padding(10)
                .width(Length::Fill),
            vertical_space().height(10),
            text("Ready to decode").size(14),
        ]
        .spacing(10))
        .padding(20)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 2.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(250.0));

        let right_column = column![
            server_controls,
            vertical_space().height(20),
            decoder_controls,
            vertical_space().height(20),
            lookup_controls,
            vertical_space().height(20),
            draw_dot_controls
        ]
        .padding(20);

        let new_column = column![
            rest_post_controls
        ]
        .padding(20);

        row![
            image_preview,
            scrollable(controls).width(Length::Fixed(480.0)),
            scrollable(right_column),
            scrollable(new_column)
        ]
        .into()
    }
}

fn decode_json_input(input: &str) -> String {
    let parsed: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return format!("JSON Parse Error: {}", e),
    };

    let mut bits = ndarray::Array3::<i8>::zeros((6, 6, 2));

    // Helper to map direction to bits
    let map_direction = |dir: &str| -> Option<(i8, i8)> {
        match dir {
            "↑" | "Up" | "up" => Some((0, 0)),
            "←" | "Left" | "left" => Some((1, 0)),
            "→" | "Right" | "right" => Some((0, 1)),
            "↓" | "Down" | "down" => Some((1, 1)),
            _ => None,
        }
    };

    let map_coords = |x: i64, y: i64| -> Option<(i8, i8)> {
        match (x, y) {
            (0, 0) => Some((0, 0)), // Up
            (1, 0) => Some((1, 0)), // Left
            (0, 1) => Some((0, 1)), // Right
            (1, 1) => Some((1, 1)), // Down
            _ => None,
        }
    };

    if let Some(arr) = parsed.as_array() {
        if arr.len() != 6 {
            return "JSON must be a 6x6 array".to_string();
        }

        for (r, row) in arr.iter().enumerate() {
            if let Some(row_arr) = row.as_array() {
                if row_arr.len() != 6 {
                    return format!("Row {} must have 6 elements", r);
                }

                for (c, cell) in row_arr.iter().enumerate() {
                    let (b0, b1) = if let Some(cell_arr) = cell.as_array() {
                        // Case: [[0,0], ...]
                        if cell_arr.len() == 2 {
                            let x = cell_arr[0].as_i64();
                            let y = cell_arr[1].as_i64();
                            if let (Some(x), Some(y)) = (x, y) {
                                if let Some(bits) = map_coords(x, y) {
                                    bits
                                } else {
                                    return format!("Invalid coordinate ({}, {}) at [{}, {}]", x, y, r, c);
                                }
                            } else {
                                // Maybe it's ["↑"]
                                if let Some(s) = cell_arr[0].as_str() {
                                     if let Some(bits) = map_direction(s) {
                                        bits
                                     } else {
                                        return format!("Invalid direction string '{}' at [{}, {}]", s, r, c);
                                     }
                                } else {
                                    return format!("Invalid cell format at [{}, {}]", r, c);
                                }
                            }
                        } else if cell_arr.len() == 1 {
                             // Case: [["↑"], ...]
                             if let Some(s) = cell_arr[0].as_str() {
                                 if let Some(bits) = map_direction(s) {
                                    bits
                                 } else {
                                    return format!("Invalid direction string '{}' at [{}, {}]", s, r, c);
                                 }
                             } else {
                                return format!("Invalid cell format at [{}, {}]", r, c);
                             }
                        } else {
                            return format!("Invalid cell array length at [{}, {}]", r, c);
                        }
                    } else if let Some(s) = cell.as_str() {
                        // Case: ["↑", ...]
                        if let Some(bits) = map_direction(s) {
                            bits
                        } else {
                            return format!("Invalid direction string '{}' (bytes: {:?}) at [{}, {}]", s, s.as_bytes(), r, c);
                        }
                    } else {
                        return format!("Invalid cell type at [{}, {}]", r, c);
                    };

                    bits[[r, c, 0]] = b0;
                    bits[[r, c, 1]] = b1;
                }
            } else {
                return format!("Row {} is not an array", r);
            }
        }
    } else {
        return "JSON must be an array".to_string();
    }

    let codec = anoto_6x6_a4_fixed();
    match codec.decode_position(&bits) {
        Ok((x, y)) => format!("Position: ({}, {})", x, y),
        Err(e) => format!("Decoding Error: {}", e),
    }
}

fn perform_pattern_lookup(sect_u_str: &str, sect_v_str: &str, x_str: &str, y_str: &str) -> String {
    let sect_u = match sect_u_str.parse::<i32>() { Ok(v) => v, Err(_) => return "Invalid Sect U".to_string() };
    let sect_v = match sect_v_str.parse::<i32>() { Ok(v) => v, Err(_) => return "Invalid Sect V".to_string() };
    let x = match x_str.parse::<i32>() { Ok(v) => v, Err(_) => return "Invalid X".to_string() };
    let y = match y_str.parse::<i32>() { Ok(v) => v, Err(_) => return "Invalid Y".to_string() };

    let codec = anoto_6x6_a4_fixed();
    
    let start_roll_x = sect_u % codec.mns_length as i32;
    let start_roll_y = sect_v % codec.mns_length as i32;
    
    let bitmatrix = codec.encode_patch((x, y), (6, 6), (start_roll_x, start_roll_y));
    
    let mut result = String::new();
    result.push_str("[\n");
    for r in 0..6 {
        result.push_str("  [");
        for c in 0..6 {
            let b0 = bitmatrix[[r, c, 0]];
            let b1 = bitmatrix[[r, c, 1]];
            let arrow = match (b0, b1) {
                (0, 0) => "\"↑\"",
                (1, 0) => "\"←\"",
                (0, 1) => "\"→\"",
                (1, 1) => "\"↓\"",
                _ => "\"?\"",
            };
            result.push_str(arrow);
            if c < 5 { result.push_str(", "); }
        }
        result.push_str("]");
        if r < 5 { result.push_str(",\n"); }
    }
    result.push_str("\n]");
    
    result
}

fn listen_for_post(rx: Arc<Mutex<mpsc::Receiver<String>>>) -> Task<Message> {
    Task::perform(async move {
        let mut lock = rx.lock().await;
        lock.recv().await
    }, Message::RestPostReceived)
}

async fn start_server_task(port_str: String, rx: oneshot::Receiver<()>, msg_tx: mpsc::Sender<String>) -> Result<(), String> {
    let port = port_str.parse::<u16>().map_err(|_| "Invalid port")?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let (startup_tx, startup_rx) = oneshot::channel();

    std::thread::spawn(move || {
        // Create a dedicated Tokio runtime for the server
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build() 
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = startup_tx.send(Err(format!("Failed to create Tokio runtime: {}", e)));
                return;
            }
        };

        rt.block_on(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => {
                    let _ = startup_tx.send(Ok(()));
                    l
                },
                Err(e) => {
                    let _ = startup_tx.send(Err(e.to_string()));
                    return;
                }
            };

            let app = axum::Router::new()
                .route("/", axum::routing::get(index_handler))
                .route("/decode", axum::routing::post(decode_handler))
                .layer(axum::Extension(msg_tx));

            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async { rx.await.ok(); })
                .await 
            {
                eprintln!("Server error: {}", e);
            }
        });
    });

    match startup_rx.await {
        Ok(result) => result,
        Err(_) => Err("Server startup channel closed unexpectedly".to_string()),
    }
}

async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

async fn decode_handler(
    axum::Extension(msg_tx): axum::Extension<mpsc::Sender<String>>,
    body: String
) -> impl axum::response::IntoResponse {
    let _ = msg_tx.send(body).await;
    axum::http::StatusCode::OK
}

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Anoto PDF Generator</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #1e1e1e;
            color: #ffffff;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
        }
        .container {
            text-align: center;
            padding: 3rem;
            border: 1px solid #333;
            border-radius: 15px;
            background-color: #252526;
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
            max-width: 500px;
            width: 90%;
        }
        h1 {
            color: #61dafb;
            margin-bottom: 1.5rem;
            font-size: 2.5rem;
        }
        p {
            font-size: 1.2rem;
            color: #cccccc;
            line-height: 1.6;
            margin-bottom: 2rem;
        }
        .status {
            display: inline-block;
            padding: 0.75rem 1.5rem;
            background-color: #28a745;
            color: white;
            border-radius: 50px;
            font-weight: bold;
            font-size: 1.1rem;
            box-shadow: 0 4px 6px rgba(40, 167, 69, 0.3);
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0% {
                box-shadow: 0 0 0 0 rgba(40, 167, 69, 0.7);
            }
            70% {
                box-shadow: 0 0 0 10px rgba(40, 167, 69, 0);
            }
            100% {
                box-shadow: 0 0 0 0 rgba(40, 167, 69, 0);
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Anoto PDF Generator</h1>
        <p>The Anoto PDF Generator server is currently running and listening for requests.</p>
        <div class="status">System Online</div>
    </div>
</body>
</html>
"#;

async fn generate_and_save(params: GenerationParams) -> Result<(image::Handle, String), String> {
    // This is a blocking operation, but we run it in an async block.
    // In a real async runtime, we should use spawn_blocking.
    // Since we don't have easy access to spawn_blocking without adding tokio dependency explicitly,
    // we will just run it here. It might block the UI thread if the executor is single threaded.
    // However, for the purpose of this task, we are using Task::perform.
    
    let result = (|| -> Result<(image::Handle, String), Box<dyn std::error::Error>> {
        let bitmatrix = generate_matrix_only(params.height, params.width, params.sect_u, params.sect_v)?;
        let base_filename = format!("GUI_G__{}__{}__{}__{}", params.height, params.width, params.sect_u, params.sect_v);
        
        // Generate PDF
        gen_pdf_from_matrix_data(&bitmatrix, &format!("{}.pdf", base_filename), &params.config)?;

        // Generate PNG
        if !std::path::Path::new("output").exists() {
            std::fs::create_dir("output")?;
        }
        let bitmatrix_i8 = bitmatrix.mapv(|x| x as i8);
        let png_path = format!("output/{}__X.png", base_filename);
        draw_preview_image(&bitmatrix_i8, &params.config, &png_path)?;

        // Load image bytes to force refresh
        let bytes = std::fs::read(&png_path)?;
        Ok((image::Handle::from_bytes(bytes), png_path))
    })();

    result.map_err(|e| e.to_string())
}

fn color_to_hex(color: Color) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn hex_to_color(hex: &str) -> Color {
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
        Color::from_rgb8(r, g, b)
    } else {
        Color::BLACK
    }
}


