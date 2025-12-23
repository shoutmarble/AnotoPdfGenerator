#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anoto_pdf::anoto_matrix::generate_matrix_only;
use anoto_pdf::controls::{anoto_control, page_layout_control, section_control};
use anoto_pdf::make_plots::{draw_dot_on_file, draw_dots_on_file, draw_preview_image};
use anoto_pdf::pdf_dotpaper::gen_pdf::{PdfConfig, gen_pdf_from_matrix_data};
use anoto_pdf::persist_json::write_json_grid_rows;
use iced::widget::image;
use iced::widget::{
    Action, button, canvas, column, container, row, scrollable, slider, space, text, text_editor,
    text_input,
};
use iced::widget::pane_grid;
use iced::{
    Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Task, Theme, Vector, mouse,
};
use iced_aw::spinner::Spinner;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};

use anoto_pdf::codec::anoto_6x6_a4_fixed;
use serde_json::Value;

const JB_MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/ttf/JetBrainsMonoNL-Medium.ttf");

pub fn main() -> iced::Result {
    iced::application(Gui::default, Gui::update, Gui::view)
        .title("Anoto PDF Generator")
        .window_size((800.0, 600.0))
        .centered()
        .scale_factor(|s| s.ui_scale)
        .font(iced_aw::ICED_AW_FONT_BYTES)
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
    server_thread: Option<std::thread::JoinHandle<()>>,
    server_status_text: String,
    server_rx: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
    rest_post_content: text_editor::Content,
    json_input: text_editor::Content,
    decoded_result: String,
    lookup_sect_u: String,
    lookup_sect_v: String,
    lookup_x: String,
    lookup_y: String,
    lookup_size: f32,
    lookup_result: text_editor::Content,
    draw_x: String,
    draw_y: String,
    draw_status: String,
    current_png_path: Option<String>,
    preview_zoom: f32,
    preview_pan: Vector,
    image_width: u32,
    image_height: u32,
    points_input: text_editor::Content,
    points_status: String,
    seen_points: HashSet<(i64, i64)>,

    panes: pane_grid::State<PaneKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneKind {
    Preview,
    Controls,
    Tools,
    Rest,
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
    GenerationFinished(Result<(image::Handle, String, u32, u32), String>),
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
    ServerStopped,
    RestPostReceived(Option<String>),
    RestPostContentChanged(text_editor::Action),
    JsonInputChanged(text_editor::Action),
    DecodeJson,
    LookupSectUChanged(String),
    LookupSectVChanged(String),
    LookupXChanged(String),
    LookupYChanged(String),
    LookupSizeChanged(f32),
    PerformLookup,
    LookupResultChanged(text_editor::Action),
    DrawXChanged(String),
    DrawYChanged(String),
    DrawDotPressed,
    DrawDotFinished(Result<image::Handle, String>),
    PreviewZoomed(f32, Point, (f32, f32)),
    PreviewPanned(Vector, (f32, f32)),
    PointsInputChanged(text_editor::Action),
    PlotAllPoints,
    PlotAllPointsFinished(Result<image::Handle, String>),
    PaneResized(pane_grid::Split, f32),
}

impl Default for Gui {
    fn default() -> Self {
        let (mut panes, preview_pane) = pane_grid::State::new(PaneKind::Preview);
        let (controls_pane, _) = panes
            .split(pane_grid::Axis::Vertical, preview_pane, PaneKind::Controls)
            .expect("split preview->controls");
        let (tools_pane, _) = panes
            .split(pane_grid::Axis::Vertical, controls_pane, PaneKind::Tools)
            .expect("split controls->tools");
        let (_rest_pane, _) = panes
            .split(pane_grid::Axis::Vertical, tools_pane, PaneKind::Rest)
            .expect("split tools->rest");

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
            server_thread: None,
            server_status_text: "Server Stopped".to_string(),
            server_rx: None,
            rest_post_content: text_editor::Content::new(),
            json_input: text_editor::Content::new(),
            decoded_result: "Ready to decode".to_string(),
            lookup_sect_u: "10".to_string(),
            lookup_sect_v: "10".to_string(),
            lookup_x: "0".to_string(),
            lookup_y: "0".to_string(),
            lookup_size: 6.0,
            lookup_result: text_editor::Content::new(),
            draw_x: "0".to_string(),
            draw_y: "0".to_string(),
            draw_status: "Ready".to_string(),
            current_png_path: None,
            preview_zoom: 1.0,
            preview_pan: Vector::new(0.0, 0.0),
            image_width: 0,
            image_height: 0,
            points_input: text_editor::Content::new(),
            points_status: "Ready".to_string(),
            seen_points: HashSet::new(),

            panes,
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

#[derive(Default)]
struct ImageViewerState {
    is_dragging: bool,
    last_cursor_pos: Option<Point>,
}

struct ImageViewer<'a> {
    handle: &'a image::Handle,
    zoom: f32,
    pan: Vector,
    image_size: (f32, f32),
}

impl<'a> canvas::Program<Message> for ImageViewer<'a> {
    type State = ImageViewerState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                cursor.position_in(bounds).map(|p| {
                    Action::publish(Message::PreviewZoomed(
                        delta_y,
                        p,
                        (bounds.width, bounds.height),
                    ))
                    .and_capture()
                })
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.position_in(bounds).is_some() {
                    state.is_dragging = true;
                    state.last_cursor_pos = cursor.position();
                    Some(Action::capture())
                } else {
                    None
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.last_cursor_pos = None;
                    Some(Action::capture())
                } else {
                    None
                }
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging
                    && let Some(current_pos) = cursor.position()
                    && let Some(last_pos) = state.last_cursor_pos
                {
                    let delta = current_pos - last_pos;
                    state.last_cursor_pos = Some(current_pos);
                    return Some(
                        Action::publish(Message::PreviewPanned(
                            delta,
                            (bounds.width, bounds.height),
                        ))
                        .and_capture(),
                    );
                }
                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let (img_w, img_h) = self.image_size;
        let w = img_w * self.zoom;
        let h = img_h * self.zoom;
        let dest = Rectangle {
            x: self.pan.x,
            y: self.pan.y,
            width: w,
            height: h,
        };

        frame.draw_image(dest, canvas::Image::new(self.handle.clone()));

        vec![frame.into_geometry()]
    }
}

impl Gui {
    fn clamp_pan_for_viewport(&mut self, viewport: (f32, f32)) {
        let (vw, vh) = viewport;
        if vw <= 0.0 || vh <= 0.0 || self.image_width == 0 || self.image_height == 0 {
            return;
        }

        let img_w = self.image_width as f32 * self.preview_zoom;
        let img_h = self.image_height as f32 * self.preview_zoom;

        let pan_x = if img_w <= vw {
            (vw - img_w) * 0.5
        } else {
            self.preview_pan.x.clamp(vw - img_w, 0.0)
        };

        let pan_y = if img_h <= vh {
            (vh - img_h) * 0.5
        } else {
            self.preview_pan.y.clamp(vh - img_h, 0.0)
        };

        self.preview_pan = Vector::new(pan_x, pan_y);
    }

    fn zoom_at(&mut self, delta: f32, cursor: Point, viewport: (f32, f32)) {
        let old_zoom = self.preview_zoom;
        let new_zoom = (old_zoom * (1.0 + delta * 0.1)).clamp(0.1, 20.0);
        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        // Keep the point under the cursor stable while zooming.
        let cursor_v = Vector::new(cursor.x, cursor.y);
        let world = (cursor_v - self.preview_pan) * (1.0 / old_zoom);
        self.preview_zoom = new_zoom;
        self.preview_pan = cursor_v - world * new_zoom;

        self.clamp_pan_for_viewport(viewport);
    }

    fn pan_by(&mut self, delta: Vector, viewport: (f32, f32)) {
        self.preview_pan += delta;
        self.clamp_pan_for_viewport(viewport);
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UiScaleChanged(val) => self.ui_scale = val,
            Message::DpiChanged(val) => self.config.dpi = val,
            Message::DotSizeChanged(val) => self.config.dot_size = (val * 10.0).round() / 10.0,
            Message::OffsetChanged(val) => {
                self.config.offset_from_origin = (val * 10.0).round() / 10.0
            }
            Message::SpacingChanged(val) => {
                self.config.grid_spacing = (val * 10.0).round() / 10.0;
                if self.page_layout_state.autodetect {
                    self.recalculate_layout();
                }
            }
            Message::HeightChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.height = val;
                }
            }
            Message::WidthChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.width = val;
                }
            }
            Message::AutodetectChanged(val) => {
                self.page_layout_state.autodetect = val;
                if val {
                    self.recalculate_layout();
                }
            }
            Message::SectUChanged(val) => {
                self.sect_u = val;
                self.sect_u_str = val.to_string();
            }
            Message::SectVChanged(val) => {
                self.sect_v = val;
                self.sect_v_str = val.to_string();
            }
            Message::ToggleUpPicker(show) => self.control_state.show_up = show,
            Message::ToggleDownPicker(show) => self.control_state.show_down = show,
            Message::ToggleLeftPicker(show) => self.control_state.show_left = show,
            Message::ToggleRightPicker(show) => self.control_state.show_right = show,
            Message::ColorUpPicked(color) => {
                self.config.color_up = color_to_hex(color);
                self.control_state.show_up = false;
            }
            Message::ColorDownPicked(color) => {
                self.config.color_down = color_to_hex(color);
                self.control_state.show_down = false;
            }
            Message::ColorLeftPicked(color) => {
                self.config.color_left = color_to_hex(color);
                self.control_state.show_left = false;
            }
            Message::ColorRightPicked(color) => {
                self.config.color_right = color_to_hex(color);
                self.control_state.show_right = false;
            }
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
                    self.server_rx = None;
                    self.server_status_text = "Stopping Server...".to_string();

                    let join_handle = self.server_thread.take();
                    return Task::perform(
                        async move {
                            if let Some(handle) = join_handle {
                                let _ = handle.join();
                            }
                        },
                        |_| Message::ServerStopped,
                    );
                } else {
                    // Start server
                    let port_str = self.server_port.clone();

                    let port = match port_str.parse::<u16>() {
                        Ok(p) => p,
                        Err(_) => {
                            self.server_status_text = "Error: Invalid port".to_string();
                            return Task::none();
                        }
                    };

                    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
                    let (tx, rx) = oneshot::channel();
                    let (msg_tx, msg_rx) = mpsc::channel(100);

                    let (startup_tx, startup_rx) = oneshot::channel::<Result<(), String>>();

                    self.server_shutdown_tx = Some(tx);
                    let rx_arc = Arc::new(Mutex::new(msg_rx));
                    self.server_rx = Some(rx_arc.clone());
                    self.server_status_text = "Starting Server...".to_string();

                    let handle = std::thread::spawn(move || {
                        let rt = match tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                let _ = startup_tx
                                    .send(Err(format!("Failed to create Tokio runtime: {}", e)));
                                return;
                            }
                        };

                        rt.block_on(async move {
                            let listener = match tokio::net::TcpListener::bind(addr).await {
                                Ok(l) => {
                                    let _ = startup_tx.send(Ok(()));
                                    l
                                }
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
                                .with_graceful_shutdown(async {
                                    rx.await.ok();
                                })
                                .await
                            {
                                eprintln!("Server error: {}", e);
                            }
                        });
                    });

                    self.server_thread = Some(handle);

                    return Task::batch(vec![
                        Task::perform(
                            async move {
                                match startup_rx.await {
                                    Ok(r) => r,
                                    Err(_) => {
                                        Err("Server startup channel closed unexpectedly"
                                            .to_string())
                                    }
                                }
                            },
                            Message::ServerStarted,
                        ),
                        listen_for_post(rx_arc),
                    ]);
                }
            }
            Message::ServerStopped => {
                self.server_status_text = "Server Stopped".to_string();
            }
            Message::RestPostReceived(content) => {
                // If the server isn't running anymore, ignore any late/queued messages.
                if self.server_shutdown_tx.is_none() {
                    return Task::none();
                }
                if let Some(c) = content {
                    let clean_c = c.trim();
                    let json_candidate = if clean_c.starts_with('\'') && clean_c.ends_with('\'') {
                        &clean_c[1..clean_c.len() - 1]
                    } else {
                        clean_c
                    };

                    let formatted_content = if let Ok(val) =
                        serde_json::from_str::<serde_json::Value>(json_candidate)
                    {
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
                                        s.push('\n');
                                    }
                                }
                                s.push(']');
                                s
                            } else {
                                serde_json::to_string_pretty(&val)
                                    .unwrap_or(json_candidate.to_string())
                            }
                        } else {
                            serde_json::to_string_pretty(&val).unwrap_or(json_candidate.to_string())
                        }
                    } else {
                        // Fallback heuristic formatting for when JSON parsing fails (e.g. missing quotes)
                        json_candidate
                            .to_string()
                            .replace("],", "],\n  ")
                            .replace("], ", "],\n  ")
                            .replace("[[", "[\n  [")
                            .replace("]]", "]\n]")
                    };
                    self.rest_post_content = text_editor::Content::with_text(&formatted_content);

                    // Attempt to decode and add to points list
                    let decode_res = decode_json_input(json_candidate);

                    let positions = extract_positions_from_decode_output(&decode_res);
                    if !positions.is_empty() {
                        let mut new_lines: Vec<String> = Vec::new();
                        for (x, y) in positions {
                            if self.seen_points.insert((x, y)) {
                                new_lines.push(format!("({}, {})", x, y));
                            }
                        }

                        if !new_lines.is_empty() {
                            let current_text = self.points_input.text();
                            let appended = new_lines.join("\n");
                            let new_text = if current_text.is_empty() {
                                appended
                            } else {
                                format!("{}\n{}", current_text, appended)
                            };
                            self.points_input = text_editor::Content::with_text(&new_text);
                            self.points_status =
                                format!("Decoded {} new point(s)", new_lines.len());
                        } else {
                            self.points_status = "Decoded (no new points)".to_string();
                        }
                    } else {
                        println!("Decode failed: {}", decode_res);
                        println!("Received input (first 500 chars): {:.500}", json_candidate);
                        self.points_status = format!("Decode failed: {}", decode_res);
                    }

                    if let Some(rx) = &self.server_rx {
                        return listen_for_post(rx.clone());
                    }
                }
            }
            Message::RestPostContentChanged(action) => {
                self.rest_post_content.perform(action);
            }
            Message::ServerStarted(result) => match result {
                Ok(_) => {
                    self.server_status_text = "Server Running".to_string();
                }
                Err(e) => {
                    self.server_status_text = format!("Error: {}", e);
                    self.server_shutdown_tx = None;
                    self.server_rx = None;
                    self.server_thread = None;
                }
            },
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
            Message::LookupSizeChanged(val) => {
                // Keep integer sizes in [6, 100]
                let clamped = val.clamp(6.0, 100.0).round();
                self.lookup_size = clamped;
            }
            Message::LookupResultChanged(action) => {
                self.lookup_result.perform(action);
            }
            Message::PerformLookup => {
                let res = perform_pattern_lookup(
                    &self.lookup_sect_u,
                    &self.lookup_sect_v,
                    &self.lookup_x,
                    &self.lookup_y,
                    self.lookup_size as usize,
                );
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
                    return Task::perform(
                        async move { generate_and_save(params).await },
                        Message::GenerationFinished,
                    );
                }
            }
            Message::GenerationFinished(result) => {
                self.is_generating = false;
                match result {
                    Ok((handle, path, w, h)) => {
                        self.status_message = "PDF Generated Successfully!".to_string();
                        self.generated_image_handle = Some(handle);
                        self.current_png_path = Some(path);
                        self.image_width = w;
                        self.image_height = h;
                        self.preview_zoom = 1.0;
                        self.preview_pan = Vector::new(0.0, 0.0);
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

                    return Task::perform(
                        async move {
                            match draw_dot_on_file(
                                &path_clone,
                                x,
                                y,
                                matrix_height,
                                matrix_width,
                                &config,
                            ) {
                                Ok(_) => {
                                    // Reload the image
                                    match std::fs::read(&path_clone) {
                                        Ok(bytes) => Ok(image::Handle::from_bytes(bytes)),
                                        Err(e) => Err(e.to_string()),
                                    }
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::DrawDotFinished,
                    );
                } else {
                    self.draw_status = "Generate PDF first!".to_string();
                }
            }
            Message::DrawDotFinished(result) => match result {
                Ok(handle) => {
                    self.draw_status = "Dot drawn!".to_string();
                    self.generated_image_handle = Some(handle);
                }
                Err(e) => self.draw_status = format!("Error: {}", e),
            },
            Message::PointsInputChanged(action) => {
                self.points_input.perform(action);

                // Keep the dedupe set consistent with what's currently in the editor.
                // This prevents a mismatch where the user clears/edits the box but we still
                // treat old points as "seen".
                self.seen_points.clear();
                for (x, y) in parse_points(&self.points_input.text()) {
                    let rx = x.round();
                    let ry = y.round();
                    if (x - rx).abs() < 1e-9 && (y - ry).abs() < 1e-9 {
                        self.seen_points.insert((rx as i64, ry as i64));
                    }
                }
            }
            Message::PlotAllPoints => {
                if let Some(path) = &self.current_png_path {
                    let text = self.points_input.text();
                    let mut points = parse_points(&text);

                    // If the text isn't (x,y) points, try decoding it as Anoto JSON (arrows/binary)
                    if points.is_empty() {
                        let decoded = decode_json_input(&text);
                        points = decoded
                            .lines()
                            .filter_map(|l| {
                                let l = l.trim();
                                if !l.starts_with("Position: (") {
                                    return None;
                                }
                                let coords = l.trim_start_matches("Position: ").trim();
                                parse_points(coords).into_iter().next()
                            })
                            .collect();
                    }
                    if points.is_empty() {
                        self.points_status = "No valid points found".to_string();
                    } else {
                        // Deduplicate points before plotting
                        let mut seen: HashSet<(u64, u64)> = HashSet::new();
                        points.retain(|(x, y)| {
                            // Deduplicate exact float values without truncation.
                            // This keeps distinct points distinct even if they share the same integer part.
                            seen.insert((x.to_bits(), y.to_bits()))
                        });

                        let config = self.config.clone();
                        let matrix_width = self.width;
                        let matrix_height = self.height;
                        let path_clone = path.clone();

                        self.points_status = format!("Plotting {} points...", points.len());

                        return Task::perform(
                            async move {
                                match draw_dots_on_file(
                                    &path_clone,
                                    &points,
                                    matrix_height,
                                    matrix_width,
                                    &config,
                                ) {
                                    Ok(_) => {
                                        // Reload the image
                                        match std::fs::read(&path_clone) {
                                            Ok(bytes) => Ok(image::Handle::from_bytes(bytes)),
                                            Err(e) => Err(e.to_string()),
                                        }
                                    }
                                    Err(e) => Err(e.to_string()),
                                }
                            },
                            Message::PlotAllPointsFinished,
                        );
                    }
                } else {
                    self.points_status = "Generate PDF first!".to_string();
                }
            }
            Message::PlotAllPointsFinished(result) => match result {
                Ok(handle) => {
                    self.points_status = "Points plotted!".to_string();
                    self.generated_image_handle = Some(handle);
                }
                Err(e) => self.points_status = format!("Error: {}", e),
            },
            Message::PreviewZoomed(delta, cursor, viewport) => {
                self.zoom_at(delta, cursor, viewport);
            }
            Message::PreviewPanned(delta, viewport) => {
                self.pan_by(delta, viewport);
            }
            Message::PaneResized(split, ratio) => {
                self.panes.resize(split, ratio);
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
        let preview_pane = || -> Element<'_, Message> {
            if let Some(handle) = &self.generated_image_handle {
                container(
                    container(
                        canvas::Canvas::new(ImageViewer {
                            handle,
                            zoom: self.preview_zoom,
                            pan: self.preview_pan,
                            image_size: (self.image_width as f32, self.image_height as f32),
                        })
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
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
                    }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                container(text("No image generated yet").size(20))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
        };

        let controls_pane = || -> Element<'_, Message> {
            let scale_slider = container(
                column![
                    text(format!("UI Scale: {:.1}", self.ui_scale)),
                    slider(0.5..=3.0, self.ui_scale, Message::UiScaleChanged).step(0.1)
                ]
                .spacing(10),
            )
            .padding(10)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            });

            let dpi_slider = container(
                column![
                    text(format!("PDF DPI: {:.0}", self.config.dpi)),
                    slider(300.0..=1200.0, self.config.dpi, Message::DpiChanged).step(10.0)
                ]
                .spacing(10),
            )
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

            let matrix_inputs =
                column![text("Matrix Settings:"), page_layout, section_ctrl,].spacing(10);

            let generate_btn = if self.is_generating {
                row![
                    button("Generating...").width(Length::Fill),
                    Spinner::new()
                        .width(Length::Fixed(20.0))
                        .height(Length::Fixed(20.0)),
                ]
                .spacing(10)
            } else {
                row![
                    button("Generate PDF")
                        .on_press(Message::GeneratePressed)
                        .width(Length::Fill)
                ]
            };

            let controls = column![
                text("Anoto PDF Generator").size(30),
                space().height(Length::Fixed(20.0)),
                scale_slider,
                space().height(Length::Fixed(20.0)),
                dpi_slider,
                space().height(Length::Fixed(20.0)),
                anoto_ctrl,
                space().height(Length::Fixed(20.0)),
                matrix_inputs,
                space().height(Length::Fixed(20.0)),
                generate_btn,
                space().height(Length::Fixed(10.0)),
                text(&self.status_message),
            ]
            .spacing(10)
            .padding(20)
            .width(Length::Fill);

            scrollable(controls)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let tools_pane = || -> Element<'_, Message> {
            let server_controls = container(
                column![
                    text("Web Server").size(20),
                    space().height(Length::Fixed(10.0)),
                    row![
                        text("Port: "),
                        text_input("8080", &self.server_port)
                            .on_input(Message::ServerPortChanged)
                            .padding(5)
                            .width(Length::Fixed(80.0))
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                    space().height(Length::Fixed(10.0)),
                    button(if self.server_shutdown_tx.is_some() {
                        "Stop Server"
                    } else {
                        "Start Server"
                    })
                    .on_press(Message::ToggleServer)
                    .padding(10)
                    .width(Length::Fill),
                    space().height(Length::Fixed(10.0)),
                    text(&self.server_status_text).size(14).color(
                        if self.server_shutdown_tx.is_some() {
                            Color::from_rgb(0.0, 0.8, 0.0)
                        } else {
                            Color::from_rgb(0.8, 0.0, 0.0)
                        }
                    ),
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .width(Length::Fill);

            let decoder_controls = container(
                column![
                    text("Decoder").size(20),
                    space().height(Length::Fixed(10.0)),
                    text_editor(&self.json_input)
                        .on_action(Message::JsonInputChanged)
                        .height(Length::Fixed(200.0))
                        .font(iced::font::Font::MONOSPACE)
                        .wrapping(iced::widget::text::Wrapping::None),
                    space().height(Length::Fixed(10.0)),
                    button("Decode Position")
                        .on_press(Message::DecodeJson)
                        .padding(10)
                        .width(Length::Fill),
                    space().height(Length::Fixed(10.0)),
                    text(&self.decoded_result).size(14),
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .width(Length::Fill);

            let lookup_controls = container(
                column![
                    text("Pattern Lookup").size(20),
                    space().height(Length::Fixed(10.0)),
                    row![
                        column![
                            text("Sect U"),
                            text_input("10", &self.lookup_sect_u)
                                .on_input(Message::LookupSectUChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill),
                        column![
                            text("Sect V"),
                            text_input("10", &self.lookup_sect_v)
                                .on_input(Message::LookupSectVChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill)
                    ]
                    .spacing(10),
                    row![
                        column![
                            text("X"),
                            text_input("0", &self.lookup_x)
                                .on_input(Message::LookupXChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill),
                        column![
                            text("Y"),
                            text_input("0", &self.lookup_y)
                                .on_input(Message::LookupYChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill)
                    ]
                    .spacing(10),
                    column![
                        text(format!(
                            "Matrix Size: {}x{}",
                            self.lookup_size as u32,
                            self.lookup_size as u32
                        )),
                        slider(6.0..=100.0, self.lookup_size, Message::LookupSizeChanged)
                            .step(1.0)
                    ]
                    .spacing(5),
                    space().height(Length::Fixed(10.0)),
                    button("Lookup Pattern")
                        .on_press(Message::PerformLookup)
                        .padding(10)
                        .width(Length::Fill),
                    space().height(Length::Fixed(10.0)),
                    {
                        let size_text = format!(
                            "{}x{}",
                            self.lookup_size as u32,
                            self.lookup_size as u32
                        );
                        text_input("", &size_text)
                            .padding(5)
                            .width(Length::Fill)
                    },
                    text_editor(&self.lookup_result)
                        .on_action(Message::LookupResultChanged)
                        .height(Length::Fixed(200.0))
                        .font(iced::font::Font::MONOSPACE)
                        .wrapping(iced::widget::text::Wrapping::None),
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .width(Length::Fill);

            let draw_dot_controls = container(
                column![
                    text("Draw Dot").size(20),
                    space().height(Length::Fixed(10.0)),
                    row![
                        column![
                            text("Position X:"),
                            text_input("0", &self.draw_x)
                                .on_input(Message::DrawXChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill),
                        column![
                            text("Position Y:"),
                            text_input("0", &self.draw_y)
                                .on_input(Message::DrawYChanged)
                                .padding(5)
                                .width(Length::Fill)
                        ]
                        .spacing(5)
                        .width(Length::Fill)
                    ]
                    .spacing(10),
                    space().height(Length::Fixed(10.0)),
                    button("Draw Dot")
                        .on_press(Message::DrawDotPressed)
                        .padding(10)
                        .width(Length::Fill),
                    space().height(Length::Fixed(10.0)),
                    text(&self.draw_status).size(14),
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .width(Length::Fill);

            let right_column = column![
                server_controls,
                space().height(Length::Fixed(20.0)),
                decoder_controls,
                space().height(Length::Fixed(20.0)),
                lookup_controls,
                space().height(Length::Fixed(20.0)),
                draw_dot_controls
            ]
            .padding(20);

            scrollable(right_column)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let rest_pane = || -> Element<'_, Message> {
            let rest_post_controls = container(
                column![
                    text("JSON 6x6+ REST Listener").size(20),
                    space().height(Length::Fixed(10.0)),
                    text_editor(&self.rest_post_content)
                        .on_action(Message::RestPostContentChanged)
                        .height(Length::Fixed(300.0))
                        .font(iced::font::Font::MONOSPACE)
                        .wrapping(iced::widget::text::Wrapping::None),
                    space().height(Length::Fixed(10.0)),
                    text("Ready to decode").size(14),
                    space().height(Length::Fixed(20.0)),
                    text("(X,Y) Decoder").size(20),
                    text_editor(&self.points_input)
                        .on_action(Message::PointsInputChanged)
                        .height(Length::Fixed(150.0))
                        .font(iced::font::Font::MONOSPACE),
                    button("Plot on A4")
                        .on_press(Message::PlotAllPoints)
                        .padding(10)
                        .width(Length::Fill),
                    text(&self.points_status).size(14),
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 2.0,
                    radius: 5.0.into(),
                },
                ..container::Style::default()
            })
            .width(Length::Fill);

            let new_column = column![rest_post_controls].padding(20);

            scrollable(new_column)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        pane_grid::PaneGrid::new(&self.panes, move |_, kind, _| {
            let content: Element<'_, Message> = match *kind {
                PaneKind::Preview => preview_pane(),
                PaneKind::Controls => controls_pane(),
                PaneKind::Tools => tools_pane(),
                PaneKind::Rest => rest_pane(),
            };
            pane_grid::Content::new(content)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .on_resize(10, |e| Message::PaneResized(e.split, e.ratio))
        .into()
    }
}

fn parse_points(input: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    // Split by closing parenthesis to separate potential points
    let parts: Vec<&str> = input.split(')').collect();
    for part in parts {
        // Find the last opening parenthesis in this part
        if let Some(start) = part.rfind('(') {
            let content = &part[start + 1..];
            // Split by comma
            if let Some(comma) = content.find(',') {
                let x_str = &content[..comma].trim();
                let y_str = &content[comma + 1..].trim();
                if let (Ok(x), Ok(y)) = (x_str.parse::<f64>(), y_str.parse::<f64>()) {
                    points.push((x, y));
                }
            } else {
                // Maybe space separated?
                let coords: Vec<&str> = content.split_whitespace().collect();
                if coords.len() >= 2
                    && let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>())
                {
                    points.push((x, y));
                }
            }
        }
    }
    points
}

fn extract_positions_from_decode_output(s: &str) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if !line.starts_with("Position: (") {
            continue;
        }
        let rest = line.trim_start_matches("Position: ").trim();
        // rest should look like: (x, y)
        let rest = rest.strip_prefix('(').and_then(|r| r.strip_suffix(')'));
        let Some(inner) = rest else {
            continue;
        };

        let mut parts = inner.split(',');
        let x = parts
            .next()
            .map(str::trim)
            .and_then(|p| p.parse::<i64>().ok());
        let y = parts
            .next()
            .map(str::trim)
            .and_then(|p| p.parse::<i64>().ok());
        if let (Some(x), Some(y)) = (x, y) {
            out.push((x, y));
        }
    }
    out
}

fn decode_json_input(input: &str) -> String {
    let mut parsed: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return format!("JSON Parse Error: {}", e),
    };

    // If parsed is a string, try to parse it as JSON again (handles shells that quote JSON)
    if let Some(s) = parsed.as_str()
        && let Ok(v) = serde_json::from_str(s)
    {
        parsed = v;
    }

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
            (0, 0) => Some((0, 0)),
            (1, 0) => Some((1, 0)),
            (0, 1) => Some((0, 1)),
            (1, 1) => Some((1, 1)),
            _ => None,
        }
    };

    let map_cell = |cell: &Value| -> Option<(i8, i8)> {
        if let Some(cell_arr) = cell.as_array() {
            // Case: [0,0]
            if cell_arr.len() == 2 {
                let x = cell_arr[0].as_i64();
                let y = cell_arr[1].as_i64();
                if let (Some(x), Some(y)) = (x, y) {
                    return map_coords(x, y);
                }

                // Case: ["↑", ...] or ["Up", ...]
                if let Some(s) = cell_arr[0].as_str() {
                    return map_direction(s);
                }
            }

            // Case: ["↑"] or ["Up"]
            if cell_arr.len() == 1
                && let Some(s) = cell_arr[0].as_str()
            {
                return map_direction(s);
            }
        } else if let Some(s) = cell.as_str() {
            // Case: "↑"
            return map_direction(s);
        }

        None
    };

    // Find the matrix (allow arbitrary wrapping arrays until we reach a 2D array of cells)
    let mut matrix_val = &parsed;
    while let Some(arr) = matrix_val.as_array() {
        if arr.is_empty() {
            return "Empty array".to_string();
        }

        let Some(first_row) = arr[0].as_array() else {
            return "Invalid structure".to_string();
        };

        if first_row.is_empty() {
            return "Empty row".to_string();
        }

        let is_cell = |v: &Value| {
            if v.is_string() {
                true
            } else if let Some(a) = v.as_array() {
                // It's a cell if it's [num, num] or ["str"], but NOT if it's nested arrays
                !a.is_empty() && !a[0].is_array()
            } else {
                false
            }
        };

        if is_cell(&first_row[0]) {
            break;
        }

        matrix_val = &arr[0];
    }

    let rows = match matrix_val.as_array() {
        Some(a) => a,
        None => return "Could not find matrix array".to_string(),
    };

    let mut invalid_cells = 0usize;
    let mut grid: Vec<Vec<Option<(i8, i8)>>> = Vec::new();

    for (r_idx, row_val) in rows.iter().enumerate() {
        let row_arr = match row_val.as_array() {
            Some(a) => a,
            None => return format!("Row {} is not an array", r_idx),
        };

        let mut row_bits: Vec<Option<(i8, i8)>> = Vec::with_capacity(row_arr.len());
        for cell_val in row_arr.iter() {
            let bits = map_cell(cell_val);
            if bits.is_none() {
                invalid_cells += 1;
            }
            row_bits.push(bits);
        }
        grid.push(row_bits);
    }

    let height = grid.len();
    if height < 6 {
        return "Matrix too small (height < 6)".to_string();
    }
    let width = grid[0].len();
    if width < 6 {
        return "Matrix too small (width < 6)".to_string();
    }
    if grid.iter().any(|r| r.len() != width) {
        return "Matrix rows have inconsistent lengths".to_string();
    }

    let codec = anoto_6x6_a4_fixed();
    let mut seen: HashSet<(i64, i64)> = HashSet::new();
    let mut decoded: Vec<(i64, i64)> = Vec::new();
    let mut decode_errors = 0usize;

    // Reuse the buffer to avoid allocating for every 6x6 window.
    let mut bits = ndarray::Array3::<i8>::zeros((6, 6, 2));

    for r in 0..=(height - 6) {
        for c in 0..=(width - 6) {
            let mut ok = true;

            for i in 0..6 {
                for j in 0..6 {
                    match grid[r + i][c + j] {
                        Some((b0, b1)) => {
                            bits[[i, j, 0]] = b0;
                            bits[[i, j, 1]] = b1;
                        }
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    break;
                }
            }

            if !ok {
                continue;
            }

            match codec.decode_position(&bits) {
                Ok((x, y)) => {
                    let x = x as i64;
                    let y = y as i64;
                    if seen.insert((x, y)) {
                        decoded.push((x, y));
                    }
                }
                Err(_) => {
                    decode_errors += 1;
                }
            }
        }
    }

    if decoded.is_empty() {
        if invalid_cells > 0 {
            return format!(
                "No valid positions found (invalid cells: {})",
                invalid_cells
            );
        }
        if decode_errors > 0 {
            return format!(
                "No valid positions found (decode errors: {})",
                decode_errors
            );
        }
        return "No valid positions found".to_string();
    }

    decoded
        .into_iter()
        .map(|(x, y)| format!("Position: ({}, {})", x, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn perform_pattern_lookup(
    sect_u_str: &str,
    sect_v_str: &str,
    x_str: &str,
    y_str: &str,
    size: usize,
) -> String {
    let sect_u = match sect_u_str.parse::<i32>() {
        Ok(v) => v,
        Err(_) => return "Invalid Sect U".to_string(),
    };
    let sect_v = match sect_v_str.parse::<i32>() {
        Ok(v) => v,
        Err(_) => return "Invalid Sect V".to_string(),
    };
    let x = match x_str.parse::<i32>() {
        Ok(v) => v,
        Err(_) => return "Invalid X".to_string(),
    };
    let y = match y_str.parse::<i32>() {
        Ok(v) => v,
        Err(_) => return "Invalid Y".to_string(),
    };

    let size = size.clamp(6, 100);

    let codec = anoto_6x6_a4_fixed();
    let patch = codec.encode_patch((x, y), (size, size), (sect_u, sect_v));

    let mut arrows: Vec<Vec<&'static str>> = Vec::with_capacity(size);

    for r in 0..size {
        let mut row_arrows: Vec<&'static str> = Vec::with_capacity(size);
        for c in 0..size {
            let b0 = patch[[r, c, 0]];
            let b1 = patch[[r, c, 1]];
            let arrow = match (b0, b1) {
                (0, 0) => "↑",
                (1, 0) => "←",
                (0, 1) => "→",
                (1, 1) => "↓",
                _ => "?",
            };
            row_arrows.push(arrow);
        }
        arrows.push(row_arrows);
    }

    pretty_print_arrow_grid_json(&arrows)
}

fn pretty_print_arrow_grid_json(arrows: &[Vec<&'static str>]) -> String {
    // Format as valid JSON, one row per line, matching the requested style:
    // [
    //   ["↓","↓",...],
    //   ["..."],
    // ]
    let mut out = String::new();
    out.push_str("[\n");

    for (row_idx, row) in arrows.iter().enumerate() {
        out.push_str("  [");
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(cell);
            out.push('"');
        }
        out.push(']');
        if row_idx + 1 != arrows.len() {
            out.push(',');
        }
        out.push('\n');
    }

    out.push(']');
    out
}

fn listen_for_post(rx: Arc<Mutex<mpsc::Receiver<String>>>) -> Task<Message> {
    Task::perform(
        async move {
            let mut guard = rx.lock().await;
            guard.recv().await
        },
        Message::RestPostReceived,
    )
}

async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

async fn decode_handler(
    axum::Extension(msg_tx): axum::Extension<mpsc::Sender<String>>,
    body: String,
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

async fn generate_and_save(
    params: GenerationParams,
) -> Result<(image::Handle, String, u32, u32), String> {
    // This is a blocking operation, but we run it in an async block.
    // In a real async runtime, we should use spawn_blocking.
    // Since we don't have easy access to spawn_blocking without adding tokio dependency explicitly,
    // we will just run it here. It might block the UI thread if the executor is single threaded.
    // However, for the purpose of this task, we are using Task::perform.

    fn bitmatrix_to_arrow_json(bitmatrix: &ndarray::Array3<i32>) -> Vec<Vec<&'static str>> {
        let (h, w, _d) = bitmatrix.dim();
        let mut arrows: Vec<Vec<&'static str>> = Vec::with_capacity(h);
        for y in 0..h {
            let mut row_arrows: Vec<&'static str> = Vec::with_capacity(w);
            for x in 0..w {
                let b0 = bitmatrix[[y, x, 0]];
                let b1 = bitmatrix[[y, x, 1]];
                let arrow = match (b0, b1) {
                    (0, 0) => "↑",
                    (1, 0) => "←",
                    (0, 1) => "→",
                    (1, 1) => "↓",
                    _ => "?",
                };
                row_arrows.push(arrow);
            }
            arrows.push(row_arrows);
        }
        arrows
    }

    let result = (|| -> Result<(image::Handle, String, u32, u32), Box<dyn std::error::Error>> {
        let bitmatrix =
            generate_matrix_only(params.height, params.width, params.sect_u, params.sect_v)?;
        let base_filename = format!(
            "GUI_G__{}__{}__{}__{}",
            params.height, params.width, params.sect_u, params.sect_v
        );

        if !std::path::Path::new("output").exists() {
            std::fs::create_dir("output")?;
        }

        // Generate X (normal) PDF/PNG
        gen_pdf_from_matrix_data(
            &bitmatrix,
            &format!("{}__X.pdf", base_filename),
            &params.config,
        )?;

        // Also emit Arrow JSON (X)
        let arrows_x = bitmatrix_to_arrow_json(&bitmatrix);
        let arrows_x_path = format!("output/{}__X.json", base_filename);
        let arrows_x_file = std::fs::File::create(&arrows_x_path)?;
        write_json_grid_rows(arrows_x_file, &arrows_x)?;

        let bitmatrix_i8 = bitmatrix.mapv(|x| x as i8);
        let png_path_x = format!("output/{}__X.png", base_filename);
        draw_preview_image(&bitmatrix_i8, &params.config, &png_path_x)?;

        // Generate Y (Y-axis flipped) PDF/PNG
        let (h, w, d) = bitmatrix.dim();
        let mut bitmatrix_y = bitmatrix.clone();
        for y in 0..h {
            for x in 0..w {
                for k in 0..d {
                    bitmatrix_y[[y, x, k]] = bitmatrix[[h - 1 - y, x, k]];
                }
            }
        }

        gen_pdf_from_matrix_data(
            &bitmatrix_y,
            &format!("{}__Y.pdf", base_filename),
            &params.config,
        )?;

        // Also emit Arrow JSON (Y, matching the flipped-Y PDF/PNG)
        let arrows_y = bitmatrix_to_arrow_json(&bitmatrix_y);
        let arrows_y_path = format!("output/{}__Y.json", base_filename);
        let arrows_y_file = std::fs::File::create(&arrows_y_path)?;
        write_json_grid_rows(arrows_y_file, &arrows_y)?;

        let bitmatrix_y_i8 = bitmatrix_y.mapv(|x| x as i8);
        let png_path_y = format!("output/{}__Y.png", base_filename);
        draw_preview_image(&bitmatrix_y_i8, &params.config, &png_path_y)?;

        // Load Y image bytes to force refresh in UI (Y is the default preview)
        let bytes = std::fs::read(&png_path_y)?;
        let img = ::image::load_from_memory(&bytes)?;
        Ok((
            image::Handle::from_bytes(bytes),
            png_path_y,
            img.width(),
            img.height(),
        ))
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
