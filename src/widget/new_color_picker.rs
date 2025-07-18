//! A simple color picker overlay for iced-rs
use iced::advanced::layout::{self, Layout};
use iced::advanced::{renderer, Clipboard, Shell};
use iced::advanced::overlay;
use iced::mouse::{self, Cursor};
use iced::widget::{button, container, column, row, text, Space};
use iced::{
    alignment, border, Background, Border, Color, Element, Event, 
    Length, Point, Rectangle, Size, Theme, Vector,
};

use crate::widget::style;

/// The state of the color picker
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// The currently selected color
    pub color: Color,
    /// The active view mode
    view_mode: ViewMode,
    /// Predefined palette colors
    palette: Vec<Color>,
    /// Currently selected palette index
    selected_palette: Option<usize>,
    /// Slider drag state
    dragging_slider: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Grid,
    Spectrum,
    Sliders,
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            view_mode: ViewMode::Grid,
            palette: vec![
                Color::BLACK,
                Color::from_rgb(0.2, 0.2, 0.2),
                Color::from_rgb(0.0, 0.0, 1.0), // Blue
                Color::from_rgb(0.0, 1.0, 0.0), // Green
                Color::from_rgb(1.0, 1.0, 0.0), // Yellow
                Color::from_rgb(1.0, 0.0, 0.0), // Red
            ],
            selected_palette: None,
            dragging_slider: None,
        }
    }
}

impl ColorPickerState {
    /// Creates a new color picker state
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the hex code of the current color
    pub fn hex_code(&self) -> String {
        let r = (self.color.r * 255.0) as u8;
        let g = (self.color.g * 255.0) as u8;
        let b = (self.color.b * 255.0) as u8;
        format!("{:02X}{:02X}{:02X}", r, g, b)
    }
}

/// A simple color picker overlay
pub struct ColorPickerOverlay<'a, Message> {
    state: &'a mut ColorPickerState,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    on_submit: Message,
    position: Point,
    width: f32,
    height: f32,
}

impl<'a, Message> ColorPickerOverlay<'a, Message> {
    pub fn new(
        state: &'a mut ColorPickerState,
        position: Point,
        on_change: impl Fn(Color) -> Message + 'a,
        on_submit: Message,
    ) -> Self {
        Self {
            state,
            on_change: Box::new(on_change),
            on_submit,
            position,
            width: 300.0,
            height: 450.0,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ColorPickerOverlay<'a, Message>
where
    Message: Clone,
    Theme: style::Catalog,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        let mut picker_bounds = Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.width,
            height: self.height,
        };

        // Keep on screen
        if picker_bounds.x + picker_bounds.width > bounds.width {
            picker_bounds.x = bounds.width - picker_bounds.width;
        }
        if picker_bounds.y + picker_bounds.height > bounds.height {
            picker_bounds.y = bounds.height - picker_bounds.height;
        }
        picker_bounds.x = picker_bounds.x.max(0.0);
        picker_bounds.y = picker_bounds.y.max(0.0);

        layout::Node::new(Size::new(picker_bounds.width, picker_bounds.height))
            .move_to(Point::new(picker_bounds.x, picker_bounds.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    // Tab buttons
                    if position.y < 50.0 {
                        let button_width = self.width / 3.0;
                        if position.x < button_width {
                            self.state.view_mode = ViewMode::Grid;
                        } else if position.x < button_width * 2.0 {
                            self.state.view_mode = ViewMode::Spectrum;
                        } else {
                            self.state.view_mode = ViewMode::Sliders;
                        }
                        shell.request_redraw();
                        return;
                    }

                    // Color area
                    let color_area_bottom = self.height - 100.0;
                    if position.y > 50.0 && position.y < color_area_bottom {
                        match self.state.view_mode {
                            ViewMode::Grid => {
                                if let Some(color) = pick_from_grid(position, self.width, self.height) {
                                    self.state.color = color;
                                    shell.publish((self.on_change)(color));
                                }
                            }
                            ViewMode::Spectrum => {
                                if let Some(color) = pick_from_spectrum(position, self.width, self.height) {
                                    self.state.color = color;
                                    shell.publish((self.on_change)(color));
                                }
                            }
                            ViewMode::Sliders => {
                                if let Some(slider) = get_slider_at_position(position, self.width, self.height) {
                                    self.state.dragging_slider = Some(slider);
                                }
                            }
                        }
                        shell.request_redraw();
                        return;
                    }

                    // Palette area
                    if position.y > self.height - 100.0 && position.y < self.height - 50.0 {
                        if let Some((index, color)) = pick_from_palette(position, &self.state.palette, self.width) {
                            self.state.color = color;
                            self.state.selected_palette = Some(index);
                            shell.publish((self.on_change)(color));
                            shell.request_redraw();
                        }
                        return;
                    }

                    // Confirm button
                    if position.y > self.height - 50.0 {
                        shell.publish(self.on_submit.clone());
                        return;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.state.dragging_slider = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(slider) = self.state.dragging_slider {
                    if let Some(position) = cursor.position_in(bounds) {
                        if let Some(new_color) = update_slider_value(slider, position, &self.state.color, self.width) {
                            self.state.color = new_color;
                            shell.publish((self.on_change)(new_color));
                            shell.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
    ) {
        let bounds = layout.bounds();
        let palette = theme;

        // Background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    width: 1.0,
                    radius: 8.0.into(),
                    color: palette.background.weak.color,
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
            },
            palette.background.strong.color.into(),
        );

        // Draw tabs
        draw_tabs(renderer, bounds, self.state.view_mode, theme);

        // Draw color area
        let color_bounds = Rectangle {
            x: bounds.x + 10.0,
            y: bounds.y + 60.0,
            width: bounds.width - 20.0,
            height: bounds.height - 160.0,
        };

        match self.state.view_mode {
            ViewMode::Grid => draw_grid(renderer, color_bounds),
            ViewMode::Spectrum => draw_spectrum(renderer, color_bounds),
            ViewMode::Sliders => draw_sliders(renderer, color_bounds, &self.state.color),
        }

        // Draw palette
        draw_palette(renderer, bounds, &self.state.palette, self.state.selected_palette);

        // Draw hex code
        draw_hex_code(renderer, bounds, &self.state.hex_code());

        // Draw confirm button
        let button_bounds = Rectangle {
            x: bounds.x + bounds.width / 2.0 - 50.0,
            y: bounds.y + bounds.height - 40.0,
            width: 100.0,
            height: 30.0,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: button_bounds,
                border: border::rounded(4.0),
                ..renderer::Quad::default()
            },
            palette.primary.strong.color.into(),
        );

        renderer.fill_text(
            iced::advanced::text::Text {
                content: "Confirm".to_string(),
                bounds: Size::new(button_bounds.width, button_bounds.height),
                size: 14.0.into(),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::advanced::text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::default(),
            },
            button_bounds.center(),
            palette.primary.strong.text,
            Rectangle::with_size(Size::INFINITY),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        layout.bounds().contains(cursor_position)
    }
}

// Helper functions
fn draw_tabs<Renderer>(renderer: &mut Renderer, bounds: Rectangle, active: ViewMode, theme: &Theme)
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    let palette = theme.extended_palette();
    let button_width = bounds.width / 3.0;
    let modes = [(ViewMode::Grid, "Grid"), (ViewMode::Spectrum, "Spectrum"), (ViewMode::Sliders, "Sliders")];
    
    for (i, (mode, label)) in modes.iter().enumerate() {
        let button_bounds = Rectangle {
            x: bounds.x + i as f32 * button_width,
            y: bounds.y + 10.0,
            width: button_width - 5.0,
            height: 40.0,
        };

        let is_active = active == *mode;
        let (bg, text_color) = if is_active {
            (palette.primary.strong.color.into(), palette.primary.strong.text)
        } else {
            (palette.background.weak.color.into(), palette.background.weak.text)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: button_bounds,
                border: border::rounded(4.0),
                ..renderer::Quad::default()
            },
            bg,
        );

        renderer.fill_text(
            iced::advanced::text::Text {
                content: label.to_string(),
                bounds: Size::new(button_bounds.width, button_bounds.height),
                size: 14.0.into(),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::advanced::text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::default(),
            },
            button_bounds.center(),
            text_color,
            Rectangle::with_size(Size::INFINITY),
        );
    }
}

fn draw_grid<Renderer>(renderer: &mut Renderer, bounds: Rectangle)
where
    Renderer: renderer::Renderer,
{
    let grid_size = 10;
    let cell_width = bounds.width / grid_size as f32;
    let cell_height = bounds.height / grid_size as f32;

    for row in 0..grid_size {
        for col in 0..grid_size {
            let color = if row == grid_size - 1 {
                // Grayscale row
                let gray = col as f32 / (grid_size - 1) as f32;
                Color::from_rgb(gray, gray, gray)
            } else {
                let hue = col as f32 / grid_size as f32;
                let lightness = 1.0 - (row as f32 / grid_size as f32);
                hsl_to_rgb(hue * 360.0, 1.0, lightness)
            };
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + col as f32 * cell_width,
                        y: bounds.y + row as f32 * cell_height,
                        width: cell_width,
                        height: cell_height,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(color),
            );
        }
    }
}

fn draw_spectrum<Renderer>(renderer: &mut Renderer, bounds: Rectangle)
where
    Renderer: renderer::Renderer,
{
    // Simplified spectrum - in production you'd want a proper gradient
    for x in 0..bounds.width as i32 {
        for y in 0..bounds.height as i32 {
            let hue = x as f32 / bounds.width * 360.0;
            let saturation = 1.0 - (y as f32 / bounds.height);
            let color = hsl_to_rgb(hue, saturation, 0.5);
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + x as f32,
                        y: bounds.y + y as f32,
                        width: 1.0,
                        height: 1.0,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(color),
            );
        }
    }
}

fn draw_sliders<Renderer>(renderer: &mut Renderer, bounds: Rectangle, color: &Color)
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    let sliders = [
        ("R", color.r, Color::from_rgb(1.0, 0.0, 0.0)),
        ("G", color.g, Color::from_rgb(0.0, 1.0, 0.0)),
        ("B", color.b, Color::from_rgb(0.0, 0.0, 1.0)),
    ];

    for (i, (label, value, slider_color)) in sliders.iter().enumerate() {
        let y = bounds.y + i as f32 * 40.0;
        
        // Label
        renderer.fill_text(
            iced::advanced::text::Text {
                content: label.to_string(),
                bounds: Size::new(20.0, 30.0),
                size: 12.0.into(),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::advanced::text::Alignment::Left,
                align_y: alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::default(),
            },
            Point::new(bounds.x, y + 15.0),
            Color::from_rgb(0.7, 0.7, 0.7),
            Rectangle::with_size(Size::INFINITY),
        );

        // Track
        let track_x = bounds.x + 30.0;
        let track_width = bounds.width - 80.0;
        let track_y = y + 10.0;
        
        // Gradient
        for x in 0..track_width as i32 {
            let t = x as f32 / track_width;
            let gradient_color = Color::from_rgb(
                slider_color.r * t,
                slider_color.g * t,
                slider_color.b * t,
            );
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: track_x + x as f32,
                        y: track_y,
                        width: 1.0,
                        height: 10.0,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(gradient_color),
            );
        }

        // Handle
        let handle_x = track_x + value * track_width;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: handle_x - 10.0,
                    y: y,
                    width: 20.0,
                    height: 30.0,
                },
                border: border::rounded(15.0),
                ..renderer::Quad::default()
            },
            Background::Color(Color::WHITE),
        );

        // Value
        renderer.fill_text(
            iced::advanced::text::Text {
                content: format!("{}", (*value * 255.0) as u8),
                bounds: Size::new(40.0, 30.0),
                size: 12.0.into(),
                line_height: iced::advanced::text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: iced::advanced::text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::default(),
            },
            Point::new(bounds.x + bounds.width - 40.0, y + 15.0),
            Color::WHITE,
            Rectangle::with_size(Size::INFINITY),
        );
    }
}

fn draw_palette<Renderer>(
    renderer: &mut Renderer, 
    bounds: Rectangle, 
    palette: &[Color],
    selected: Option<usize>
)
where
    Renderer: renderer::Renderer,
{
    let palette_size = 30.0;
    let spacing = 5.0;
    let total_width = palette.len() as f32 * (palette_size + spacing);
    let start_x = bounds.x + (bounds.width - total_width) / 2.0;
    let y = bounds.y + bounds.height - 90.0;

    for (i, &color) in palette.iter().enumerate() {
        let x = start_x + i as f32 * (palette_size + spacing);
        
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { x, y, width: palette_size, height: palette_size },
                border: border::rounded(palette_size / 2.0),
                ..renderer::Quad::default()
            },
            Background::Color(color),
        );

        if selected == Some(i) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: x - 2.0,
                        y: y - 2.0,
                        width: palette_size + 4.0,
                        height: palette_size + 4.0,
                    },
                    border: Border {
                        color: Color::WHITE,
                        width: 2.0,
                        radius: ((palette_size + 4.0) / 2.0).into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
    }
}

fn draw_hex_code<Renderer>(renderer: &mut Renderer, bounds: Rectangle, hex_code: &str)
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    let text_y = bounds.y + bounds.height - 120.0;
    
    renderer.fill_text(
        iced::advanced::text::Text {
            content: format!("Hex: #{}", hex_code),
            bounds: Size::new(bounds.width, 20.0),
            size: 14.0.into(),
            line_height: iced::advanced::text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: iced::advanced::text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            shaping: iced::advanced::text::Shaping::Basic,
            wrapping: iced::advanced::text::Wrapping::default(),
        },
        Point::new(bounds.center_x(), text_y),
        Color::from_rgb(0.7, 0.7, 0.7),
        Rectangle::with_size(Size::INFINITY),
    );
}

// Picking functions
fn pick_from_grid(position: Point, width: f32, height: f32) -> Option<Color> {
    let color_bounds = Rectangle {
        x: 10.0,
        y: 60.0,
        width: width - 20.0,
        height: height - 160.0,
    };

    let relative_x = position.x - color_bounds.x;
    let relative_y = position.y - color_bounds.y;

    if relative_x >= 0.0 && relative_x <= color_bounds.width &&
       relative_y >= 0.0 && relative_y <= color_bounds.height {
        let grid_size = 10;
        let col = (relative_x / color_bounds.width * grid_size as f32) as usize;
        let row = (relative_y / color_bounds.height * grid_size as f32) as usize;

        if row == grid_size - 1 {
            let gray = col as f32 / (grid_size - 1) as f32;
            Some(Color::from_rgb(gray, gray, gray))
        } else {
            let hue = col as f32 / grid_size as f32;
            let lightness = 1.0 - (row as f32 / grid_size as f32);
            Some(hsl_to_rgb(hue * 360.0, 1.0, lightness))
        }
    } else {
        None
    }
}

fn pick_from_spectrum(position: Point, width: f32, height: f32) -> Option<Color> {
    let color_bounds = Rectangle {
        x: 10.0,
        y: 60.0,
        width: width - 20.0,
        height: height - 160.0,
    };

    let relative_x = position.x - color_bounds.x;
    let relative_y = position.y - color_bounds.y;

    if relative_x >= 0.0 && relative_x <= color_bounds.width &&
       relative_y >= 0.0 && relative_y <= color_bounds.height {
        let hue = relative_x / color_bounds.width * 360.0;
        let saturation = 1.0 - (relative_y / color_bounds.height);
        Some(hsl_to_rgb(hue, saturation, 0.5))
    } else {
        None
    }
}

fn pick_from_palette(position: Point, palette: &[Color], width: f32) -> Option<(usize, Color)> {
    let palette_size = 30.0;
    let spacing = 5.0;
    let total_width = palette.len() as f32 * (palette_size + spacing);
    let start_x = (width - total_width) / 2.0;
    
    for (i, &color) in palette.iter().enumerate() {
        let x = start_x + i as f32 * (palette_size + spacing);
        if position.x >= x && position.x <= x + palette_size {
            return Some((i, color));
        }
    }
    None
}

fn get_slider_at_position(position: Point, width: f32, height: f32) -> Option<usize> {
    let color_bounds = Rectangle {
        x: 10.0,
        y: 60.0,
        width: width - 20.0,
        height: height - 160.0,
    };

    let relative_y = position.y - color_bounds.y;
    
    for i in 0..3 {
        let y = i as f32 * 40.0;
        if relative_y >= y && relative_y <= y + 30.0 {
            return Some(i);
        }
    }
    None
}

fn update_slider_value(slider: usize, position: Point, current_color: &Color, width: f32) -> Option<Color> {
    let track_x = 40.0;
    let track_width = width - 90.0;
    let value = ((position.x - track_x) / track_width).clamp(0.0, 1.0);
    
    match slider {
        0 => Some(Color::from_rgb(value, current_color.g, current_color.b)),
        1 => Some(Color::from_rgb(current_color.r, value, current_color.b)),
        2 => Some(Color::from_rgb(current_color.r, current_color.g, value)),
        _ => None,
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let h = h / 360.0;
    let (r, g, b) = if s == 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        
        let hue_to_rgb = |p: f32, q: f32, mut t: f32| {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0/6.0 { p + (q - p) * 6.0 * t }
            else if t < 1.0/2.0 { q }
            else if t < 2.0/3.0 { p + (q - p) * (2.0/3.0 - t) * 6.0 }
            else { p }
        };
        
        (
            hue_to_rgb(p, q, h + 1.0/3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0/3.0),
        )
    };
    
    Color::from_rgb(r, g, b)
}