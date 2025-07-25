use iced::{
    advanced::{
        layout::{Limits, Node}, overlay, renderer, text::Renderer as _, widget::{self, tree::Tree}, Clipboard, Layout, Overlay, Renderer as _, Shell, Widget
    },
    alignment::{Horizontal, Vertical},
    event, keyboard, mouse, touch,
    widget::{button, column, container, row, scrollable::Viewport, slider, text, text_input, Button, Column, Container, Row, Slider, Space, Text},
    Border, Color, Element, Event, Length, Padding, Point, Rectangle, 
    Renderer, Shadow, Size, Vector,
};

static mut ACTIVE_COLOR_PICKER: Option<*mut bool> = None;

/// A button that displays a color and opens a color picker when clicked
pub struct ColorButton<'a, Message> {
    color: Color,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    width: Length,
    height: Length,
    padding: Padding,
    border_radius: f32,
    border_width: f32,
    title: String,
}

impl<'a, Message> ColorButton<'a, Message> {
    /// Creates a new color button with the given color
    pub fn new(color: Color, on_change: impl Fn(Color) -> Message + 'a) -> Self {
        Self {
            color,
            on_change: Box::new(on_change),
            width: Length::Fixed(30.0),
            height: Length::Fixed(20.0),
            padding: Padding::ZERO,
            border_radius: 4.0,
            border_width: 1.0,
            title: "Color".to_string(),
        }
    }

    /// Sets the title for the color picker overlay
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the width of the button
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the button
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding of the button
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Sets the border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }
}

#[derive(Debug, Clone)]
struct State {
    is_open: bool,
    color: Color,
    overlay_state: OverlayState,
    title: String,
    overlay_position: Point,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_open: false,
            color: Color::WHITE,
            overlay_state: OverlayState::from_color(Color::WHITE),
            title: "Color".to_string(),
            overlay_position: Point::new(0.0, 0.0),
        }
    }
}

impl<'a, Message: Clone + 'a> Widget<Message, iced::Theme, Renderer> for ColorButton<'a, Message> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            color: self.color,
            overlay_state: OverlayState::from_color(self.color),
            title: self.title.clone(),
            ..State::default()
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        Node::new(size)
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = state.state.downcast_ref::<State>();

        // Draw the color button
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: if state.is_open { 
                        theme.palette().primary 
                    } else { 
                        Color::from_rgb(0.5, 0.5, 0.5) 
                    },
                    width: self.border_width,
                    radius: self.border_radius.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            state.color,
        );
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = state.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(bounds) {
                    state.is_open = !state.is_open;
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, Renderer>> {
        let widget_state = state.state.downcast_mut::<State>();
        
        if widget_state.is_open {

            unsafe {   // Doesn't seem like a good idea?
                if let Some(active) = ACTIVE_COLOR_PICKER {
                    if active != &mut widget_state.is_open as *mut bool {
                        // Close the other picker
                        *active = false;
                    }
                }
                
                widget_state.is_open = true;
                ACTIVE_COLOR_PICKER = Some(&mut widget_state.is_open as *mut bool);
            }

            // Calculate centered position
            let overlay_width = 320.0;
            let overlay_height = 440.0;

            let mut position = Point::new(
                (viewport.width - overlay_width) / 2.0,
                (viewport.height - overlay_height) / 2.0,
            );

            position.x = position.x.max(10.0).min(viewport.width - overlay_width - 10.0);
            position.y = position.y.max(10.0).min(viewport.height - overlay_height - 10.0);
            
            // We need to handle the state updates through a wrapper
            let overlay_state = &mut widget_state.overlay_state;
            let is_open = &mut widget_state.is_open;
            let color = &mut widget_state.color;
            let position = &mut widget_state.overlay_position;
            let on_change = &self.on_change;

            if position.x == 0.0 && position.y == 0.0 {
                *position = Point::new(
                    (viewport.width - overlay_width) / 2.0,
                    (viewport.height - overlay_height) / 2.0,
                );
            }
            
            Some(
                ModernColorPickerOverlay {
                    overlay_state,
                    is_open,
                    color,
                    on_change,
                    position: position,
                    title: widget_state.title.clone(),
                    viewport_size: viewport.size(),
                }
                .overlay()
            )
        } else {
            None
        }
    }
}

impl<'a, Message: Clone + 'a> From<ColorButton<'a, Message>> for Element<'a, Message, iced::Theme, Renderer> {
    fn from(button: ColorButton<'a, Message>) -> Self {
        Self::new(button)
    }
}

/// Helper function to create a color button
pub fn color_button<'a, Message>(
    color: Color,
    on_change: impl Fn(Color) -> Message + 'a,
) -> ColorButton<'a, Message> {
    ColorButton::new(color, on_change)
}

// Modern overlay implementation with tabs
#[derive(Debug, Clone)]
struct OverlayState {
    active_tab: ColorPickerTab,
    // Grid tab state
    // Spectrum tab state
    hue: f32,
    saturation: f32,
    value: f32,
    spectrum_dragging: bool,
    // Sliders tab state
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    hex_input: String,
    // Common
    preset_colors: Vec<Color>,
    // Dragging sliders
    hue_dragging: bool,
    dragging_slider: Option<SliderType>,
    // Dragging state for the overlay window
    is_dragging: bool,
    drag_offset: Vector,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColorPickerTab {
    Grid,
    Spectrum,
    Sliders,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SliderType {
    Red,
    Green,
    Blue,
    Alpha,
}

impl OverlayState {
    fn from_color(color: Color) -> Self {
        let (h, s, v) = rgb_to_hsv(color);
        Self {
            active_tab: ColorPickerTab::Grid,
            hue: h,
            saturation: s,
            value: v,
            spectrum_dragging: false,
            red: color.r,
            green: color.g,
            blue: color.b,
            alpha: color.a,
            hex_input: color_to_hex(color),
            preset_colors: vec![
                Color::BLACK,
                Color::WHITE,
                Color::from_rgb8(0x00, 0x7A, 0xFF), // Blue
                Color::from_rgb8(0x00, 0xC8, 0x00), // Green
                Color::from_rgb8(0xFF, 0xD7, 0x00), // Yellow
                Color::from_rgb8(0xFF, 0x00, 0x00), // Red
            ],
            hue_dragging: false,
            dragging_slider: None,
            is_dragging: false,
            drag_offset: Vector::new(0.0, 0.0),
        }
    }

    fn update_from_hsv(&mut self) {
        let color = hsv_to_rgb(self.hue, self.saturation, self.value);
        self.red = color.r;
        self.green = color.g;
        self.blue = color.b;
        //self.hex_input = color_to_hex(color);
        self.update_from_rgb();
    }

    fn update_from_rgb(&mut self) {
        let color = Color::from_rgba(self.red, self.green, self.blue, self.alpha);
        let (h, s, v) = rgb_to_hsv(color);
        self.hue = h;
        self.saturation = s;
        self.value = v;
        self.hex_input = color_to_hex(color);
    }

    fn update_from_hex(&mut self) {
        if let Ok(color) = hex_to_color(&self.hex_input) {
            self.red = color.r;
            self.green = color.g;
            self.blue = color.b;
            let (h, s, v) = rgb_to_hsv(color);
            self.hue = h;
            self.saturation = s;
            self.value = v;
        }
    }

    fn current_color(&self) -> Color {
        Color::from_rgba(self.red, self.green, self.blue, self.alpha)
    }
}

#[derive(Debug, Clone)]
enum OverlayMessage {
    TabChanged(ColorPickerTab),
    PresetSelected(Color),
    SpectrumDragStarted,
    SpectrumDragEnded,
    SpectrumDragged(Point),
    RedChanged(f32),
    GreenChanged(f32),
    BlueChanged(f32),
    HexInputChanged(String),
    Close,
}

struct ModernColorPickerOverlay<'a, Message> {
    overlay_state: &'a mut OverlayState,
    is_open: &'a mut bool,
    color: &'a mut Color,
    on_change: &'a dyn Fn(Color) -> Message,
    position: &'a mut Point,
    title: String,
    viewport_size: Size,
}

impl<'a, Message> ModernColorPickerOverlay<'a, Message> 
where
    Message: Clone
{
    fn overlay(self) -> overlay::Element<'a, Message, iced::Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }
}

impl<'a, Message: Clone> Overlay<Message, iced::Theme, Renderer> for ModernColorPickerOverlay<'a, Message> {
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> Node {
        let size = Size::new(320.0, 440.0);
        let node = Node::new(size);
        let node = node.move_to(*self.position);
        node
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        
        // Draw background with shadow
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: theme.extended_palette().background.weak.color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                snap: true,
            },
            theme.extended_palette().background.base.color,
        );

        // Header
        let header_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: 50.0,
        };

        renderer.fill_text(
            iced::advanced::Text {
                content: self.title.clone(),
                bounds: Size::new(header_bounds.width, header_bounds.height),
                size: iced::Pixels(18.0),
                font: iced::Font::default(),
                align_x: text::Alignment::Center,
                align_y: Vertical::Center,
                line_height: iced::advanced::text::LineHeight::default(),
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::widget::text::Wrapping::default(),
            },
            Point::new(header_bounds.center_x(), header_bounds.center_y()),
            style.text_color,
            header_bounds,
        );

        // Close button (X)
        let close_bounds = Rectangle {
            x: bounds.x + bounds.width - 40.0,
            y: bounds.y + 10.0,
            width: 30.0,
            height: 30.0,
        };

        if cursor.is_over(close_bounds) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: close_bounds,
                    border: Border {
                        radius: 15.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            );
        }

        renderer.fill_text(
            iced::advanced::Text {
                content: "×".to_string(),
                bounds: Size::new(close_bounds.width, close_bounds.height),
                size: iced::Pixels(24.0),
                font: iced::Font::default(),
                align_x: text::Alignment::Center,
                align_y: Vertical::Center,
                line_height: iced::advanced::text::LineHeight::default(),
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::default(),
            },
            Point::new(close_bounds.center_x(), close_bounds.center_y()),
            style.text_color,
            close_bounds,
        );

        // Tab buttons
        let tab_y = bounds.y + 60.0;
        let tab_width = 80.0;
        let tab_height = 35.0;
        let tab_spacing = 10.0;
        
        let tabs = [
            (ColorPickerTab::Grid, "Grid"),
            (ColorPickerTab::Spectrum, "Spectrum"),
            (ColorPickerTab::Sliders, "Sliders"),
        ];

        for (i, (tab, label)) in tabs.iter().enumerate() {
            let tab_x = bounds.x + 20.0 + (tab_width + tab_spacing) * i as f32;
            let tab_bounds = Rectangle {
                x: tab_x,
                y: tab_y,
                width: tab_width,
                height: tab_height,
            };

            let is_active = self.overlay_state.active_tab == *tab;
            let is_hovered = cursor.is_over(tab_bounds);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: tab_bounds,
                    border: Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                if is_active {
                    theme.extended_palette().primary.base.color
                } else if is_hovered {
                    theme.extended_palette().background.weak.color
                } else {
                    Color::TRANSPARENT
                },
            );

            renderer.fill_text(
                iced::advanced::Text {
                    content: label.to_string(),
                    bounds: Size::new(tab_bounds.width, tab_bounds.height),
                    size: iced::Pixels(14.0),
                    font: iced::Font::default(),
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::default(),
                },
                Point::new(tab_bounds.center_x(), tab_bounds.center_y()),
                if is_active {
                    Color::WHITE
                } else {
                    style.text_color
                },
                tab_bounds,
            );
        }

        // Content area
        let content_bounds = Rectangle {
            x: bounds.x + 20.0,
            y: bounds.y + 110.0,
            width: bounds.width - 40.0,
            height: 230.0,
        };

        match self.overlay_state.active_tab {
            ColorPickerTab::Grid => self.draw_grid_tab(renderer, theme, content_bounds, cursor),
            ColorPickerTab::Spectrum => self.draw_spectrum_tab(renderer, theme, content_bounds, cursor),
            ColorPickerTab::Sliders => self.draw_sliders_tab(renderer, theme, style, content_bounds),
        }

        // Preset colors
        let preset_y = bounds.y + 360.0;
        let preset_size = 30.0;
        let preset_spacing = 8.0;
        let preset_per_row = ((bounds.width - 40.0) / (preset_size + preset_spacing)) as usize;

        for (i, color) in self.overlay_state.preset_colors.iter().enumerate() {
            let row = i / preset_per_row;
            let col = i % preset_per_row;

            let preset_x = bounds.x + 20.0 + (preset_size + preset_spacing) * col as f32;
            let preset_y = preset_y + (preset_size + preset_spacing) * row as f32;

            let preset_bounds = Rectangle {
                x: preset_x,
                y: preset_y,
                width: preset_size,
                height: preset_size,
            };

            let is_hovered = cursor.is_over(preset_bounds);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: preset_bounds,
                    border: Border {
                        color: if is_hovered {
                            theme.palette().primary
                        } else {
                            Color::from_rgba(0.5, 0.5, 0.5, 0.9)
                        },
                        width: if is_hovered { 2.0 } else { 1.0 },
                        radius: 15.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                *color,
            );
        }

        // Add button (+)
        let last_preset_idx = self.overlay_state.preset_colors.len();
        let add_row = last_preset_idx / preset_per_row;
        let add_col = last_preset_idx % preset_per_row;

        
        if add_row < 2 && add_col < preset_per_row {
            let add_preset_bounds = Rectangle {
                x: bounds.x + 20.0 + (preset_size + preset_spacing) * add_col as f32,
                y: preset_y + (preset_size + preset_spacing) * add_row as f32,
                width: preset_size,
                height: preset_size,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: add_preset_bounds,
                    border: Border {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                        width: 1.0,
                        radius: 20.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                theme.extended_palette().background.weak.color,
            );

            renderer.fill_text(
                iced::advanced::Text {
                    content: "+".to_string(),
                    bounds: Size::new(add_preset_bounds.width, add_preset_bounds.height),
                    size: iced::Pixels(24.0),
                    font: iced::Font::default(),
                    align_x: text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::default(),
                },
                Point::new(add_preset_bounds.center_x(), add_preset_bounds.center_y()),
                style.text_color,
                add_preset_bounds,
            );

        }

        
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {

                // Check if we should start dragging the overlay
                let header_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: 50.0,
                };
                
                let close_bounds = Rectangle {
                    x: bounds.x + bounds.width - 40.0,
                    y: bounds.y + 10.0,
                    width: 30.0,
                    height: 30.0,
                };

                if cursor.is_over(header_bounds) && !cursor.is_over(close_bounds) && !self.overlay_state.is_dragging {
                    if !self.overlay_state.spectrum_dragging && 
                        !self.overlay_state.hue_dragging && 
                        self.overlay_state.dragging_slider.is_none() {
                            if let Some(position) = cursor.position() {
                                self.overlay_state.is_dragging = true;
                                self.overlay_state.drag_offset = Vector::new(
                                    position.x - bounds.x,
                                    position.y - bounds.y,
                                );
                                return;
                            }
                        }
                }

                if cursor.is_over(close_bounds) {
                    *self.is_open = false;
                    //shell.request_redraw();
                    shell.invalidate_layout();
                    shell.invalidate_widgets();
                    shell.capture_event();
                    return;
                }

                // Check tabs
                let tab_y = bounds.y + 60.0;
                let tab_width = 80.0;
                let tab_height = 35.0;
                let tab_spacing = 10.0;
                
                let tabs = [
                    ColorPickerTab::Grid,
                    ColorPickerTab::Spectrum,
                    ColorPickerTab::Sliders,
                ];

                for (i, tab) in tabs.iter().enumerate() {
                    let tab_x = bounds.x + 20.0 + (tab_width + tab_spacing) * i as f32;
                    let tab_bounds = Rectangle {
                        x: tab_x,
                        y: tab_y,
                        width: tab_width,
                        height: tab_height,
                    };

                    if cursor.is_over(tab_bounds) {
                        self.overlay_state.active_tab = *tab;
                        //shell.request_redraw();
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        shell.capture_event();
                        return;
                    }
                }

                // Check preset colors
                let preset_y = bounds.y + 360.0;
                let preset_size = 30.0;
                let preset_spacing = 8.0;
                let presets_per_row = ((bounds.width - 40.0) / (preset_size + preset_spacing)) as usize;

                for (i, color) in self.overlay_state.preset_colors.clone().iter().enumerate() {
                    let row = i / presets_per_row;
                    let col = i % presets_per_row;
                    
                    if row >= 2 {
                        continue;
                    }
                    
                    let preset_x = bounds.x + 20.0 + (preset_size + preset_spacing) * col as f32;
                    let preset_y = preset_y + (preset_size + preset_spacing) * row as f32;
                    
                    let preset_bounds = Rectangle {
                        x: preset_x,
                        y: preset_y,
                        width: preset_size,
                        height: preset_size,
                    };

                    if cursor.is_over(preset_bounds) {

                        self.overlay_state.red = color.r;
                        self.overlay_state.green = color.g;
                        self.overlay_state.blue = color.b;
                        self.overlay_state.alpha = color.a;
                        self.overlay_state.update_from_rgb();

                        *self.color = *color;
                        shell.publish((self.on_change)(*color));
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        shell.capture_event();
                        return;
                    }
                }

                // Check add preset button
                let last_preset_idx = self.overlay_state.preset_colors.len();
                let add_row = last_preset_idx / presets_per_row;
                let add_col = last_preset_idx % presets_per_row;

                if add_row < 2 {  // Only check if we haven't exceeded 2 rows
                    let add_preset_bounds = Rectangle {
                        x: bounds.x + 20.0 + (preset_size + preset_spacing) * add_col as f32,
                        y: preset_y + (preset_size + preset_spacing) * add_row as f32,
                        width: preset_size,
                        height: preset_size,
                    };

                    if cursor.is_over(add_preset_bounds) {
                        let current_color = self.overlay_state.current_color();
                        if !self.overlay_state.preset_colors.contains(&current_color) {
                            self.overlay_state.preset_colors.push(current_color);
                            //shell.request_redraw();
                            shell.invalidate_layout();
                            shell.invalidate_widgets();
                            shell.capture_event();
                        }
                        return;
                    }
                }

                // Handle tab-specific clicks
                let content_bounds = Rectangle {
                    x: bounds.x + 20.0,
                    y: bounds.y + 110.0,
                    width: bounds.width - 40.0,
                    height: 230.0,
                };

                match self.overlay_state.active_tab {
                    ColorPickerTab::Grid => {
                        self.handle_grid_click(content_bounds, cursor, shell);
                    }
                    ColorPickerTab::Spectrum => {
                        self.handle_spectrum_click(content_bounds, cursor, shell);
                    }
                    ColorPickerTab::Sliders => {
                        self.handle_slider_click(content_bounds, cursor, shell);
                    }
                    _ => {}
                }
                shell.invalidate_layout();
                shell.invalidate_widgets();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.overlay_state.is_dragging = false;
                self.overlay_state.spectrum_dragging = false;
                self.overlay_state.hue_dragging = false;
                self.overlay_state.dragging_slider = None;
                shell.invalidate_layout();
                shell.invalidate_widgets();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.overlay_state.is_dragging {
                    if let Some(position) = cursor.position() {
                        let new_x = position.x - self.overlay_state.drag_offset.x;
                        let new_y = position.y - self.overlay_state.drag_offset.y;
                        
                        // Keep within viewport bounds
                        self.position.x = new_x.max(0.0).min(self.viewport_size.width - bounds.width);
                        self.position.y = new_y.max(0.0).min(self.viewport_size.height - bounds.height);
                        
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        shell.capture_event();
                    }
                } else if self.overlay_state.spectrum_dragging || self.overlay_state.hue_dragging {
                    let content_bounds = Rectangle {
                        x: bounds.x + 20.0,
                        y: bounds.y + 110.0,
                        width: bounds.width - 40.0,
                        height: 220.0,
                    };
                    self.handle_spectrum_drag(content_bounds, cursor, shell);
                    shell.invalidate_layout();
                    shell.invalidate_widgets();
                    shell.capture_event();
                } else if self.overlay_state.dragging_slider.is_some() {
                    let content_bounds = Rectangle {
                        x: bounds.x + 20.0,
                        y: bounds.y + 110.0,
                        width: bounds.width - 40.0,
                        height: 220.0,
                    };
                    self.handle_slider_drag(content_bounds, cursor, shell);
                    shell.invalidate_layout();
                    shell.invalidate_widgets();
                    shell.capture_event();
                }

                shell.invalidate_layout();
                shell.invalidate_widgets();
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed { 
                key: keyboard::Key::Named(keyboard::key::Named::Escape), 
                .. 
            }) => {
                *self.is_open = false;
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
            
        // Check if cursor is over close button
        let close_bounds = Rectangle {
            x: bounds.x + bounds.width - 40.0,
            y: bounds.y + 10.0,
            width: 30.0,
            height: 30.0,
        };

        if cursor.is_over(close_bounds) {
            return mouse::Interaction::Pointer;
        }
        
        // Check if cursor is over header (for dragging)
        let header_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: 50.0,
        };

        if cursor.is_over(header_bounds) {
            return mouse::Interaction::Grab;
        }
        
        mouse::Interaction::default()
            
    }
}

impl<'a, Message: Clone> ModernColorPickerOverlay<'a, Message> {
        fn draw_grid_tab(
        &self,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) {
        let cell_size = bounds.width / 12.0;
        let rows = 8;
        let cols = 12;

        for row in 0..rows {
            for col in 0..cols {
                let x = bounds.x + col as f32 * cell_size;
                let y = bounds.y + row as f32 * cell_size;
                
                let hue = (col as f32 / cols as f32) * 360.0;
                let saturation = 1.0 - (row as f32 / rows as f32) * 0.7;
                let value = 1.0 - (row as f32 / rows as f32) * 0.5;
                
                let color = hsv_to_rgb(hue, saturation, value);
                
                let cell_bounds = Rectangle {
                    x,
                    y,
                    width: cell_size - 1.0,
                    height: cell_size - 1.0,
                };

                let is_hovered = cursor.is_over(cell_bounds);

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: cell_bounds,
                        border: if is_hovered {
                            Border {
                                color: Color::WHITE,
                                width: 2.0,
                                radius: 0.0.into(),
                            }
                        } else {
                            Border::default()
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    color,
                );
            }
        }

        // Add grayscale row at the bottom
        let gray_y = bounds.y + rows as f32 * cell_size + 10.0;
        for col in 0..cols {
            let x = bounds.x + col as f32 * cell_size;
            let gray_value = col as f32 / (cols - 1) as f32;
            let color = Color::from_rgb(gray_value, gray_value, gray_value);
            
            let cell_bounds = Rectangle {
                x,
                y: gray_y,
                width: cell_size - 1.0,
                height: cell_size - 1.0,
            };

            let is_hovered = cursor.is_over(cell_bounds);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: cell_bounds,
                    border: if is_hovered {
                        Border {
                            color: if gray_value > 0.5 { Color::BLACK } else { Color::WHITE },
                            width: 2.0,
                            radius: 0.0.into(),
                        }
                    } else {
                        Border::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                color,
            );
        }
    }

    fn draw_spectrum_tab(
        &self,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) {
        // Draw HSV spectrum
//        let spectrum_size = bounds.width.min(bounds.height);
        let spectrum_height = bounds.height - 30.0;
        let spectrum_size = bounds.width.min(spectrum_height);

        let spectrum_bounds = Rectangle {
            x: bounds.x + (bounds.width - spectrum_size) / 2.0,
            y: bounds.y,
            width: spectrum_size,
            height: spectrum_size,
        };

        // Draw saturation/value gradient
        for y in 0..spectrum_size as u32 {
            for x in 0..spectrum_size as u32 {
                let saturation = x as f32 / spectrum_size;
                let value = 1.0 - (y as f32 / spectrum_size);
                let color = hsv_to_rgb(self.overlay_state.hue, saturation, value);
                
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: spectrum_bounds.x + x as f32,
                            y: spectrum_bounds.y + y as f32,
                            width: 1.0,
                            height: 1.0,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    color,
                );
            }
        }

        // Draw selection indicator
        let indicator_x = spectrum_bounds.x + self.overlay_state.saturation * spectrum_size;
        let indicator_y = spectrum_bounds.y + (1.0 - self.overlay_state.value) * spectrum_size;
        
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: indicator_x - 8.0,
                    y: indicator_y - 8.0,
                    width: 16.0,
                    height: 16.0,
                },
                border: Border {
                    color: Color::WHITE,
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Color::TRANSPARENT,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: indicator_x - 6.0,
                    y: indicator_y - 6.0,
                    width: 12.0,
                    height: 12.0,
                },
                border: Border {
                    color: Color::BLACK,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Color::TRANSPARENT,
        );

        // Draw hue slider
        let hue_y = spectrum_bounds.y + spectrum_bounds.height + 20.0;
        let hue_bounds = Rectangle {
            x: spectrum_bounds.x,
            y: hue_y,
            width: spectrum_bounds.width,
            height: 20.0,
        };

        // Draw hue gradient
        for x in 0..spectrum_bounds.width as u32 {
            let hue = (x as f32 / spectrum_bounds.width) * 360.0;
            let color = hsv_to_rgb(hue, 1.0, 1.0);
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: hue_bounds.x + x as f32,
                        y: hue_bounds.y,
                        width: 1.0,
                        height: hue_bounds.height,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                color,
            );
        }

        // Draw hue indicator
        let hue_indicator_x = hue_bounds.x + (self.overlay_state.hue / 360.0) * hue_bounds.width;
        
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: hue_indicator_x - 2.0,
                    y: hue_bounds.y - 2.0,
                    width: 4.0,
                    height: hue_bounds.height + 4.0,
                },
                border: Border {
                    color: Color::WHITE,
                    width: 2.0,
                    radius: 2.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Color::BLACK,
        );
    }

    fn draw_sliders_tab(
        &self,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        bounds: Rectangle,
    ) {
        let slider_height = 30.0;
        let spacing = 35.0;
        let label_width = 60.0;
        let value_width = 40.0;
        let slider_width = bounds.width - label_width - value_width - 20.0;

        // RGB sliders
        let sliders = [
            ("RED", self.overlay_state.red, Color::from_rgb(1.0, 0.0, 0.0)),
            ("GREEN", self.overlay_state.green, Color::from_rgb(0.0, 1.0, 0.0)),
            ("BLUE", self.overlay_state.blue, Color::from_rgb(0.0, 0.0, 1.0)),
            ("ALPHA", self.overlay_state.alpha, Color::from_rgba(1.0, 1.0, 1.0, 0.5))
        ];

        for (i, (label, value, color)) in sliders.iter().enumerate() {
            let y = bounds.y + i as f32 * spacing;

            // Label
            renderer.fill_text(
                iced::advanced::Text {
                    content: label.to_string(),
                    bounds: Size::new(label_width, slider_height),
                    size: iced::Pixels(12.0),
                    font: iced::Font::default(),
                    align_x: text::Alignment::Left,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::default(),
                },
                Point::new(bounds.x, y + slider_height / 2.0),
                style.text_color,
                Rectangle {
                    x: bounds.x,
                    y,
                    width: label_width,
                    height: slider_height,
                },
            );

            // Slider track
            let track_bounds = Rectangle {
                x: bounds.x + label_width,
                y: y + slider_height / 2.0 - 2.0,
                width: slider_width,
                height: 4.0,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: track_bounds,
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                theme.extended_palette().background.weak.color,
            );

            // Slider fill
            let fill_bounds = Rectangle {
                x: track_bounds.x,
                y: track_bounds.y,
                width: track_bounds.width * value,
                height: track_bounds.height,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: fill_bounds,
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                *color,
            );

            // Slider handle
            let handle_x = track_bounds.x + track_bounds.width * value;
            let handle_bounds = Rectangle {
                x: handle_x - 8.0,
                y: y + slider_height / 2.0 - 8.0,
                width: 16.0,
                height: 16.0,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: handle_bounds,
                    border: Border {
                        color: theme.extended_palette().background.weak.color,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                Color::WHITE,
            );

            // Value text
            let value_text = format!("{}", (*value * 255.0) as u8);
            renderer.fill_text(
                iced::advanced::Text {
                    content: value_text,
                    bounds: Size::new(value_width, slider_height),
                    size: iced::Pixels(12.0),
                    font: iced::Font::default(),
                    align_x: text::Alignment::Right,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::widget::text::Wrapping::default(),
                },
                Point::new(bounds.x + bounds.width - value_width / 2.0, y + slider_height / 2.0),
                style.text_color,
                Rectangle {
                    x: bounds.x + bounds.width - value_width,
                    y,
                    width: value_width,
                    height: slider_height,
                },
            );
        }

        // Hex input
        let hex_y = bounds.y + 4.0 * spacing + 20.0;
        
        // Hex label
        renderer.fill_text(
            iced::advanced::Text {
                content: "sRGB Hex Color #".to_string(),
                bounds: Size::new(bounds.width / 2.0, 30.0),
                size: iced::Pixels(12.0),
                font: iced::Font::default(),
                align_x: text::Alignment::Left,
                align_y: Vertical::Center,
                line_height: iced::advanced::text::LineHeight::default(),
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::default(),
            },
            Point::new(bounds.x, hex_y + 15.0),
            theme.palette().primary,
            Rectangle {
                x: bounds.x,
                y: hex_y,
                width: bounds.width / 2.0,
                height: 30.0,
            },
        );

        // Hex value
        renderer.fill_text(
            iced::advanced::Text {
                content: self.overlay_state.hex_input.trim_start_matches('#').to_string(),
                bounds: Size::new(bounds.width / 2.0, 30.0),
                size: iced::Pixels(14.0),
                font: iced::Font::MONOSPACE,
                align_x: text::Alignment::Right,
                align_y: Vertical::Center,
                line_height: iced::advanced::text::LineHeight::default(),
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::default(),
            },
            Point::new(bounds.x + bounds.width - bounds.width / 4.0, hex_y + 15.0),
            style.text_color,
            Rectangle {
                x: bounds.x + bounds.width / 2.0,
                y: hex_y,
                width: bounds.width / 2.0,
                height: 30.0,
            },
        );
    }

    fn handle_grid_click(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(position) = cursor.position_in(bounds) {
            let cell_size = bounds.width / 12.0;
            let col = (position.x / cell_size) as usize;
            let row = (position.y / cell_size) as usize;
            
            if row < 8 && col < 12 {
                let hue = (col as f32 / 12.0) * 360.0;
                let saturation = 1.0 - (row as f32 / 8.0) * 0.7;
                let value = 1.0 - (row as f32 / 8.0) * 0.5;
                
                self.overlay_state.hue = hue;
                self.overlay_state.saturation = saturation;
                self.overlay_state.value = value;
                self.overlay_state.update_from_hsv();
                
                let color = self.overlay_state.current_color();
                *self.color = color;
                shell.publish((self.on_change)(color));
            } else {
                // Check for grayscale row with proper bounds
                let gray_y_start = 8.0 * cell_size + 10.0;
                let gray_col = ((position.x / cell_size) as usize).min(11);
                
                // Check if we're within the grayscale row Y bounds
                if position.y >= gray_y_start && position.y < gray_y_start + cell_size {
                    let gray_value = gray_col as f32 / 11.0;
                    let color = Color::from_rgb(gray_value, gray_value, gray_value);
                    
                    self.overlay_state.red = gray_value;
                    self.overlay_state.green = gray_value;
                    self.overlay_state.blue = gray_value;
                    self.overlay_state.update_from_rgb();
                    
                    *self.color = color;
                    shell.publish((self.on_change)(color));
                }
            }
        }
    }

    fn handle_spectrum_drag(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let spectrum_height = bounds.height - 30.0;
        let spectrum_size = bounds.width.min(spectrum_height);
        let spectrum_bounds = Rectangle {
            x: bounds.x + (bounds.width - spectrum_size) / 2.0,
            y: bounds.y,
            width: spectrum_size,
            height: spectrum_size,
        };

        if self.overlay_state.spectrum_dragging {
            if let Some(spectrum_position) = cursor.position_in(spectrum_bounds) {
                self.overlay_state.saturation = (spectrum_position.x / spectrum_size).clamp(0.0, 1.0);
                self.overlay_state.value = (1.0 - spectrum_position.y / spectrum_size).clamp(0.0, 1.0);
                self.overlay_state.update_from_hsv();
                
                let color = self.overlay_state.current_color();
                *self.color = color;
                shell.publish((self.on_change)(color));
                shell.request_redraw();
            }
        } else if self.overlay_state.hue_dragging {
            let hue_bounds = Rectangle {
                x: spectrum_bounds.x,
                y: spectrum_bounds.y + spectrum_bounds.height + 30.0,
                width: spectrum_bounds.width,
                height: 20.0,
            };

            if let Some(hue_position) = cursor.position_in(hue_bounds) {
                self.overlay_state.hue = (hue_position.x / hue_bounds.width * 360.0).clamp(0.0, 360.0);
                self.overlay_state.update_from_hsv();
                
                let color = self.overlay_state.current_color();
                *self.color = color;
                shell.publish((self.on_change)(color));
                shell.request_redraw();
            }
        }
    }

    fn handle_spectrum_click(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let spectrum_height = bounds.height - 30.0;
        let spectrum_size = bounds.width.min(spectrum_height);
        let spectrum_bounds = Rectangle {
            x: bounds.x + (bounds.width - spectrum_size) / 2.0,
            y: bounds.y,
            width: spectrum_size,
            height: spectrum_size,
        };

        let hue_bounds = Rectangle {
            x: spectrum_bounds.x,
            y: spectrum_bounds.y + spectrum_bounds.height + 30.0,
            width: spectrum_bounds.width,
            height: 20.0,
        };

        if cursor.is_over(spectrum_bounds) {
            self.overlay_state.spectrum_dragging = true;
            self.handle_spectrum_drag(bounds, cursor, shell);
        } else if cursor.is_over(hue_bounds) {
            self.overlay_state.hue_dragging = true;
            self.handle_spectrum_drag(bounds, cursor, shell);
        }
    }

    fn handle_slider_click(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let slider_height = 30.0;
        let spacing = 35.0;
        let label_width = 60.0;
        let value_width = 40.0;
        let slider_width = bounds.width - label_width - value_width - 20.0;

        for i in 0..4 {
            let y = bounds.y + i as f32 * spacing;
            let track_bounds = Rectangle {
                x: bounds.x + label_width,
                y: y,
                width: slider_width,
                height: slider_height,
            };

            if cursor.is_over(track_bounds) {
                self.overlay_state.dragging_slider = Some(match i {
                    0 => SliderType::Red,
                    1 => SliderType::Green,
                    2 => SliderType::Blue,
                    3 => SliderType::Alpha,
                    _ => unreachable!(),
                });
                self.handle_slider_drag(bounds, cursor, shell);
                break;
            }
        }
    }

    fn handle_slider_drag(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(slider_type) = self.overlay_state.dragging_slider {
            let slider_height = 30.0;
            let spacing = 35.0;
            let label_width = 60.0;
            let value_width = 40.0;
            let slider_width = bounds.width - label_width - value_width - 20.0;

            let slider_index = match slider_type {
                SliderType::Red => 0,
                SliderType::Green => 1,
                SliderType::Blue => 2,
                SliderType::Alpha => 3,
            };

            let y = bounds.y + slider_index as f32 * spacing;
            let track_bounds = Rectangle {
                x: bounds.x + label_width,
                y: y,
                width: slider_width,
                height: slider_height,
            };

            if let Some(position) = cursor.position_in(track_bounds) {
                let value = (position.x / track_bounds.width).clamp(0.0, 1.0);
                
                match slider_type {
                    SliderType::Red => self.overlay_state.red = value,
                    SliderType::Green => self.overlay_state.green = value,
                    SliderType::Blue => self.overlay_state.blue = value,
                    SliderType::Alpha => self.overlay_state.alpha = value,
                }
                
                self.overlay_state.update_from_rgb();
                let color = self.overlay_state.current_color();
                *self.color = color;
                shell.publish((self.on_change)(color));
                shell.request_redraw();
            }
        }
    }
    
}

// Helper functions
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Color::from_rgb(r + m, g + m, b + m)
}

fn rgb_to_hsv(color: Color) -> (f32, f32, f32) {
    let r = color.r;
    let g = color.g;
    let b = color.b;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    
    let h = if h < 0.0 { h + 360.0 } else { h };
    
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;
    
    (h, s, v)
}

fn color_to_hex(color: Color) -> String {
    if color.a < 1.0 {
        format!("#{:02X}{:02X}{:02X}{:02X}", 
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8
        )
    } else {
        format!("#{:02X}{:02X}{:02X}", 
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8
        )
    }
}

fn hex_to_color(hex: &str) -> Result<Color, ()> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        6 => {
            u32::from_str_radix(hex, 16)
                .map(|rgb| {
                    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
                    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
                    let b = (rgb & 0xFF) as f32 / 255.0;
                    Color::from_rgb(r, g, b)
                })
                .map_err(|_| ())
        }
        8 => {
            u32::from_str_radix(hex, 16)
                .map(|rgba| {
                    let r = ((rgba >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((rgba >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((rgba >> 8) & 0xFF) as f32 / 255.0;
                    let a = (rgba & 0xFF) as f32 / 255.0;
                    Color { r, g, b, a }
                })
                .map_err(|_| ())
        }
        _ => Err(()),
    }
}

// to handle slider interactions in the Sliders tab:
impl<'a, Message: Clone> ModernColorPickerOverlay<'a, Message> {
    fn handle_slider_interaction(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) {
        let slider_height = 30.0;
        let spacing = 35.0;
        let label_width = 60.0;
        let value_width = 40.0;
        let slider_width = bounds.width - label_width - value_width - 20.0;

        // Check if we're interacting with any slider
        for i in 0..3 {
            let y = bounds.y + i as f32 * spacing;
            let track_bounds = Rectangle {
                x: bounds.x + label_width,
                y: y,
                width: slider_width,
                height: slider_height,
            };

            if cursor.is_over(track_bounds) {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
                    if let Some(position) = cursor.position_in(track_bounds) {
                        let value = (position.x / track_bounds.width).clamp(0.0, 1.0);
                        
                        match i {
                            0 => self.overlay_state.red = value,
                            1 => self.overlay_state.green = value,
                            2 => self.overlay_state.blue = value,
                            _ => {}
                        }
                        
                        self.overlay_state.update_from_rgb();
                        let color = self.overlay_state.current_color();
                        *self.color = color;
                        shell.publish((self.on_change)(color));
                        shell.request_redraw();
                    }
                }
            }
        }
    }

    fn handle_hue_slider_interaction(
        &mut self,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        let spectrum_size = bounds.width.min(bounds.height);
        let hue_y = bounds.y + spectrum_size + 20.0;
        let hue_bounds = Rectangle {
            x: bounds.x + (bounds.width - spectrum_size) / 2.0,
            y: hue_y,
            width: spectrum_size,
            height: 20.0,
        };

        if cursor.is_over(hue_bounds) {
            if let Some(position) = cursor.position_in(hue_bounds) {
                self.overlay_state.hue = (position.x / hue_bounds.width * 360.0).clamp(0.0, 360.0);
                self.overlay_state.update_from_hsv();
                
                let color = self.overlay_state.current_color();
                *self.color = color;
                shell.publish((self.on_change)(color));
                shell.request_redraw();
            }
        }
    }
}

impl<'a, Message> std::fmt::Debug for ModernColorPickerOverlay<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModernColorPickerOverlay")
            .field("position", &self.position)
            .field("overlay_state", &self.overlay_state)
            .finish()
    }
}

impl<'a, Message> ColorButton<'a, Message> {
    /// Creates a small color button suitable for inline use
    pub fn small(color: Color, on_change: impl Fn(Color) -> Message + 'a) -> Self {
        Self::new(color, on_change)
            .width(20)
            .height(20)
            .border_radius(3.0)
    }

    /// Creates a large color button for prominent display
    pub fn large(color: Color, on_change: impl Fn(Color) -> Message + 'a) -> Self {
        Self::new(color, on_change)
            .width(60)
            .height(40)
            .border_radius(6.0)
    }

    /// Creates a circular color button
    pub fn circle(color: Color, on_change: impl Fn(Color) -> Message + 'a) -> Self {
        Self::new(color, on_change)
            .width(30)
            .height(30)
            .border_radius(15.0)
    }
}

// Add this to your update method in ModernColorPickerOverlay to handle slider interactions:
// (This would go in the Event::Mouse(mouse::Event::ButtonPressed) match arm)
/*
ColorPickerTab::Sliders => {
    self.handle_slider_interaction(content_bounds, cursor, event, shell);
}
*/

// Also add drag support for sliders by tracking which slider is being dragged:
// You'd need to add a field to OverlayState:
/*
#[derive(Debug, Clone)]
struct OverlayState {
    // ... existing fields ...
    dragging_slider: Option<SliderType>,
}

#[derive(Debug, Clone, Copy)]
enum SliderType {
    Red,
    Green,
    Blue,
    Hue,
}
*/