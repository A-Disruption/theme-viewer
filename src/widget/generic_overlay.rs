use iced::{
    advanced::{
        layout::{Limits, Node},
        overlay,
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Overlay as _, Renderer as _, Shell, Widget,
    }, alignment::Vertical, border::Radius, event, keyboard, mouse, touch, widget::button, Border, Color, Element, Event, Length, Padding, Point, Rectangle, Shadow, Size, Theme, Vector
};

/// A button that opens a draggable overlay with custom content
#[allow(missing_debug_implementations)]
pub struct OverlayButton<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> 
where
    Theme: Catalog + button::Catalog,
{
    /// The button label
    label: String,
    /// The overlay title
    title: String,
    /// Function to create the overlay content (called each time)
    content: Element<'a, Message, Theme, Renderer>,
    /// Optional width for the overlay (defaults to 400px)
    overlay_width: Option<f32>,
    /// Optional height for the overlay (defaults to content height)
    overlay_height: Option<f32>,
    /// Button width
    width: Length,
    /// Button height
    height: Length,
    /// Button padding
    padding: Padding,
    /// Callback when the overlay is opened
    on_open: Option<Box<dyn Fn() -> Message + 'a>>,
    /// Callback when the overlay is closed
    on_close: Option<Box<dyn Fn() -> Message + 'a>>,
    /// Class of the Overlay
    class: <Theme as Catalog>::Class<'a>,
    /// Get full window size for overlay bounds
    window_size: Option<Rectangle>,
    /// Status from button widget to match style
    status: Option<button::Status>,
    /// Button class
    button_class: <Theme as button::Catalog>::Class<'a>,
    /// is_press to match button status
    is_pressed: bool,
}

impl<'a, Message, Theme, Renderer> OverlayButton<'a, Message, Theme, Renderer> 
where 
    Renderer: iced::advanced::Renderer,
    Theme: Catalog + button::Catalog,
{
    /// Creates a new overlay button with the given label and content function
    pub fn new(
        label: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {

        Self {
            label: label.into(),
            title: title.into(),
            content: content.into(),
            overlay_width: None,
            overlay_height: None,
            width: Length::Fixed(50.0),
            height: Length::Fixed(30.0),
            padding: DEFAULT_PADDING,
            on_open: None,
            on_close: None,
            class: <Theme as Catalog>::default(),
            window_size: None,
            status: None,
            button_class: <Theme as button::Catalog>::default(),
            is_pressed: false,
        }
    }

    /// Sets the overlay width
    pub fn overlay_width(mut self, width: f32) -> Self {
        self.overlay_width = Some(width);
        self
    }

    /// Sets the overlay height
    pub fn overlay_height(mut self, height: f32) -> Self {
        self.overlay_height = Some(height);
        self
    }

    /// Sets the button width
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the button height
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the button padding
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets a callback for when the overlay is opened
    pub fn on_open(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_open = Some(Box::new(callback));
        self
    }

    /// Sets a callback for when the overlay is closed
    pub fn on_close(mut self, callback: impl Fn() -> Message + 'a) -> Self {
        self.on_close = Some(Box::new(callback));
        self
    }

    /// Sets the style of the button using button's styling system
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self
    where
        <Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, Theme>>,
    {
        self.button_class = (Box::new(style) as button::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the button class directly
    #[must_use]
    pub fn button_class(mut self, class: impl Into<<Theme as button::Catalog>::Class<'a>>) -> Self {
        self.button_class = class.into();
        self
    }

    /// Sets the overlay style
    #[must_use]
    pub fn overlay_style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the class of the Overlay
    #[must_use]
    pub fn overlay_class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

#[derive(Debug, Clone)]
struct State {
    is_open: bool,
    position: Point,
    is_dragging: bool,
    drag_offset: Vector,
    window_size: Size,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_open: false,
            position: Point::new(0.0, 0.0),
            is_dragging: false,
            drag_offset: Vector::new(0.0, 0.0),
            window_size: Size::new(0.0, 0.0),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> 
    for OverlayButton<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: iced::widget::button::Catalog + iced::widget::text::Catalog + iced::widget::container::Catalog + Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&(self.content))]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self, 
        _tree: &mut Tree, 
        _renderer: &Renderer, 
        limits: &Limits
    ) -> Node {
        let size = limits
            .width(self.width)
            .height(self.height)
            .resolve(self.width, self.height, Size::ZERO);
        Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) 
    where 
        Theme: Catalog + button::Catalog,
    {
        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();
//        let is_hovered = cursor.is_over(bounds);
        let style = <Theme as button::Catalog>::style(theme, &self.button_class, self.status.unwrap_or(button::Status::Active));

        // Draw button background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: style.border.color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            style.background.unwrap()
        );

        // Draw button text
        renderer.fill_text(
            iced::advanced::Text {
                content: self.label.clone(),
                bounds: Size::new(bounds.width, bounds.height),
                size: iced::Pixels(16.0),
                font: iced::Font::default(),
                align_x: iced::advanced::text::Alignment::Center,
                align_y: Vertical::Center,
                line_height: iced::advanced::text::LineHeight::default(),
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::default(),
            },
            Point::new(bounds.center_x(), bounds.center_y()),
            style.text_color,
            bounds,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Window(iced::window::Event::Opened { size, .. })
            | Event::Window(iced::window::Event::Resized(size)) => {
                state.window_size = Size::new(size.width, size.height);
                // No need to invalidate layout unless your layout depends on it while open
            }
            _ => {}
        }


        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if self.is_pressed {
                        self.is_pressed = false;
                        self.status = Some(button::Status::Active);
                    }
            }
            Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                if cursor.is_over(layout.bounds()) {
                    self.status = Some(button::Status::Hovered);
                    shell.invalidate_layout();
                } else {
                    self.status = Some(button::Status::Active);
                    shell.invalidate_layout();
                }
            }
            _ => {}
        }

        if state.is_open {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    self.status = Some(button::Status::Pressed);
                    self.is_pressed = true;
                    state.is_open = true;
                    shell.invalidate_layout();
                }
            }

            Event::Window(iced::window::Event::Opened { position: _, size }) => {
                let window_size = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: size.width,
                    height: size.height,
                };

                self.window_size = Some(window_size);
            }
            Event::Window(iced::window::Event::Resized(size)) => {
                let window_size = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: size.width,
                    height: size.height,
                };

                self.window_size = Some(window_size);
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        
        // Only show interaction when overlay is closed
        if state.is_open {
            return mouse::Interaction::None;
        }

        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        offset: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        if !state.is_open {
            return None;
        }

        if state.position == Point::ORIGIN {
            let overlay_width = self.overlay_width.unwrap_or(400.0);
            let overlay_height = self.overlay_height.unwrap_or(300.0);
            let window = state.window_size;

            // Fallback if we haven't seen a window event yet
            let (window_width, window_height) = if window.width > 0.0 && window.height > 0.0 {
                (window.width, window.height)
            } else {
                // Default if no window events yet
                (800.0, 800.0)
            };

            state.position = Point::new(
                (window_width - overlay_width) / 2.0 + offset.x,
                (window_height - overlay_height) / 2.0 + offset.y,
            );
        }

        let fullscreen = {
            let win = state.window_size;
            if win.width > 0.0 && win.height > 0.0 {
                Rectangle::new(Point::ORIGIN, win)
            } else {
                // defensive fallback
                Rectangle::new(Point::ORIGIN, Size::new(99999.0, 99999.0))
            }
        };

        let content_tree = &mut tree.children[0];

        // Pre-compute content layout here where we have mutable access
        let header_height = 50.0;
        let padding = 20.0;
        let overlay_width = self.overlay_width.unwrap_or(400.0);
        let overlay_height = self.overlay_height.unwrap_or(300.0);
        let overlay_size = Size::new(
                overlay_width - padding * 2.0, 
                overlay_height + header_height - padding * 2.0
            );
        
        let content_limits = Limits::new(Size::ZERO, overlay_size);

        let content_layout = self.content
            .as_widget_mut()
            .layout(content_tree, renderer, &content_limits);

        Some(overlay::Element::new(Box::new(Overlay {
            state,
            title: &self.title,
            class: <Theme as Catalog>::default(),
            content: &mut self.content,
            tree: content_tree,
            width: self.overlay_width.unwrap_or(400.0),
            height: self.overlay_height,
            viewport: fullscreen,
            on_close: self.on_close.as_deref(),
            content_layout: Some(content_layout),
        })))
    }
}

/// The default [`Padding`] of a [`Button`]. Using for Overlay Button to match iced::widget::button
pub(crate) const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

struct Overlay<'a, 'b, Message, Theme, Renderer> 
where 
    Theme: Catalog,
{
    state: &'a mut State,
    class: Theme::Class<'a>,
    title: &'a str,
    content: &'a mut Element<'b, Message, Theme, Renderer>,
    tree: &'a mut Tree,
    width: f32,
    height: Option<f32>,
    viewport: Rectangle,
    on_close: Option<&'a dyn Fn() -> Message>,
    content_layout: Option<Node>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: iced::widget::container::Catalog 
        + iced::widget::button::Catalog 
        + iced::widget::text::Catalog
        + Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let header_height = 50.0;
        let padding = 20.0;
        
        // Use the pre-computed content layout passed from OverlayButton
        let content_height = if let Some(ref content_layout) = self.content_layout {
            content_layout.size().height
        } else {
            100.0  // fallback
        };
        
        let total_height = if let Some(height) = self.height {
            height
        } else {
            header_height + content_height + padding * 2.0
        };

        let size = Size::new(self.width, total_height);
        Node::new(size).move_to(self.state.position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let draw_style = <Theme as Catalog>::style(&theme, &self.class);

        // Use layer rendering for proper overlay isolation
        renderer.with_layer(self.viewport, |renderer| {
            // Draw background with shadow
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        color: draw_style.border_color,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    shadow: draw_style.shadow,
                    snap: true,
                },
                draw_style.background,
            );

            // Draw header background
            let header_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: 50.0,
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: header_bounds,
                    border: Border {
                        color: draw_style.border_color,
                        width: 1.0,
                        radius: Radius {
                            top_left: 12.0,
                            top_right: 12.0,
                            bottom_left: 0.0,
                            bottom_right: 0.0,
                        },
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                draw_style.header_background,
            );

            // Draw title
            renderer.fill_text(
                iced::advanced::Text {
                    content: self.title.to_string(),
                    bounds: Size::new(header_bounds.width - 50.0, header_bounds.height),
                    size: iced::Pixels(18.0),
                    font: iced::Font::default(),
                    align_x: iced::advanced::text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Advanced,
                    wrapping: iced::advanced::text::Wrapping::default(),
                },
                Point::new(header_bounds.center_x() - 25.0, header_bounds.center_y()),
                draw_style.text_color,
                header_bounds,
            );

            // Draw close button
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
                    align_x: iced::advanced::text::Alignment::Center,
                    align_y: Vertical::Center,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::advanced::text::Wrapping::default(),
                },
                Point::new(close_bounds.center_x(), close_bounds.center_y()),
                style.text_color,
                close_bounds,
            );

            // Draw content with proper translation like float.rs
            let content_bounds = Rectangle {
                x: bounds.x + 20.0,
                y: bounds.y + 60.0,
                width: bounds.width - 40.0,
                height: bounds.height - 80.0,
            };

            renderer.with_translation(
                Vector::new(content_bounds.x, content_bounds.y),
                |renderer| {
                    // Adjust cursor to content coordinate space
                    let adjusted_cursor = cursor.position().map(|position| {
                        mouse::Cursor::Available(Point::new(
                            position.x - content_bounds.x,
                            position.y - content_bounds.y,
                        ))
                    }).unwrap_or(mouse::Cursor::Unavailable);

                    // Use the pre-computed layout from the layout() method
                    if let Some(ref content_layout) = self.content_layout {
                        self.content.as_widget().draw(
                            self.tree,
                            renderer,
                            theme,
                            style,
                            Layout::new(content_layout),
                            adjusted_cursor,
                            &Rectangle::new(Point::ORIGIN, content_bounds.size()),
                        );
                    }
                },
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        // Handle header interactions first
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
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

                if cursor.is_over(close_bounds) {
                    self.state.is_open = false;
                    if let Some(on_close) = self.on_close {
                        shell.publish(on_close());
                    }
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return;
                }

                if cursor.is_over(header_bounds) {
                    if let Some(position) = cursor.position() {
                        self.state.is_dragging = true;
                        self.state.drag_offset = Vector::new(
                            position.x - bounds.x,
                            position.y - bounds.y,
                        );
                    }
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return; // Don't forward to content if dragging header
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                self.state.is_dragging = false;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.state.is_dragging {
                    if let Some(position) = cursor.position() {
                        let new_x = position.x - self.state.drag_offset.x;
                        let new_y = position.y - self.state.drag_offset.y;

                        self.state.position.x = new_x
                            .max(0.0)
                            .min(self.viewport.width - bounds.width);
                        self.state.position.y = new_y
                            .max(0.0)
                            .min(self.viewport.height - bounds.height);

                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                    return; // Don't forward to content while dragging
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.state.is_open = false;
                if let Some(on_close) = self.on_close {
                    shell.publish(on_close());
                }
                return;
            }
            _ => {}
        }

        // Forward events to content
        let content_bounds = Rectangle {
            x: bounds.x + 20.0,
            y: bounds.y + 60.0,
            width: bounds.width - 40.0,
            height: bounds.height - 80.0,
        };

        // Always forward keyboard events and mouse events over content area
        let should_forward_event = match event {
            // Always forward keyboard events
            Event::Keyboard(_) => true,
            // Forward mouse events when over content area and not dragging
            Event::Mouse(_) | Event::Touch(_) => {
                !self.state.is_dragging && cursor.is_over(content_bounds)
            },
            // Forward other events
            _ => true,
        };

        if should_forward_event {
            let adjusted_cursor = if cursor.is_over(content_bounds) {
                cursor.position().map(|position| {
                    mouse::Cursor::Available(Point::new(
                        position.x - content_bounds.x,
                        position.y - content_bounds.y,
                    ))
                }).unwrap_or(mouse::Cursor::Unavailable)
            } else {
                mouse::Cursor::Unavailable
            };

            // Use the stored content layout
            if let Some(ref content_layout) = self.content_layout {
                self.content.as_widget_mut().update(
                    self.tree,
                    event,
                    Layout::new(content_layout),
                    adjusted_cursor,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle::new(Point::ORIGIN, content_bounds.size()),
                );
            }
        }

    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        let close_bounds = Rectangle {
            x: bounds.x + bounds.width - 40.0,
            y: bounds.y + 10.0,
            width: 30.0,
            height: 30.0,
        };

        if cursor.is_over(close_bounds) {
            return mouse::Interaction::Pointer;
        }

        let header_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: 50.0,
        };

        if cursor.is_over(header_bounds) {
            return mouse::Interaction::Grab;
        }

        // Forward to content for interaction
        let content_bounds = Rectangle {
            x: bounds.x + 20.0,
            y: bounds.y + 60.0,
            width: bounds.width - 40.0,
            height: bounds.height - 80.0,
        };

        if cursor.is_over(content_bounds) {
            let adjusted_cursor = cursor.position().map(|position| {
                mouse::Cursor::Available(Point::new(
                    position.x - content_bounds.x,
                    position.y - content_bounds.y,
                ))
            }).unwrap_or(mouse::Cursor::Unavailable);

            // Use the stored content layout
            if let Some(ref content_layout) = self.content_layout {
                return self.content.as_widget().mouse_interaction(
                    self.tree,
                    Layout::new(content_layout),
                    adjusted_cursor,
                    &Rectangle::new(Point::ORIGIN, content_bounds.size()),
                    renderer,
                );
            }
        }

        mouse::Interaction::default()
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        // Get the actual bounds of the overlay window
        let bounds = layout.bounds();
        
        // Calculate the actual position of the content area
        let content_offset = Vector::new(
            bounds.x + 20.0,  // overlay position + content padding
            bounds.y + 60.0,  // overlay position + header height + padding
        );
        
        let content_bounds = Rectangle {
            x: content_offset.x,
            y: content_offset.y,
            width: self.width - 40.0,
            height: self.height.unwrap_or(300.0) - 80.0,
        };
        
        // Use the stored content layout
        if let Some(ref content_layout) = self.content_layout {
            self.content.as_widget_mut().overlay(
                self.tree,
                Layout::new(content_layout),
                renderer,
                &content_bounds,
                content_offset,  // Pass the actual screen position
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<OverlayButton<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: iced::widget::button::Catalog + iced::widget::text::Catalog + iced::widget::container::Catalog + Catalog + 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(button: OverlayButton<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

/// Helper function to create an overlay button
pub fn overlay_button<'a, Message, Theme, Renderer>(
    label: impl Into<String>,
    title: impl Into<String>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> OverlayButton<'a, Message, Theme, Renderer> 
where 
    Renderer: iced::advanced::Renderer,
    Theme: Catalog + button::Catalog,
{
    OverlayButton::new(label, title, content)
}

/// The theme catalog of a draggable overlay
pub trait Catalog {
    /// The style class
    type Class<'a>;
    
    /// Default style
    fn default<'a>() -> Self::Class<'a>;
    
    /// Get the style for a class
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// Style for the overlay
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Background color
    pub background: Color,
    /// Header background color  
    pub header_background: Color,
    /// Border color
    pub border_color: Color,
    /// Text color
    pub text_color: Color,
    /// Shadow
    pub shadow: Shadow,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Color::from_rgb8(245, 245, 245),
            header_background: Color::from_rgb8(230, 230, 230),
            border_color: Color::from_rgb8(200, 200, 200),
            text_color: Color::BLACK,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
        }
    }
}

/// Styling function
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;
    
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| {
            let palette = theme.extended_palette();
            Style {
                background: palette.background.base.color,
                header_background: palette.background.weak.color,
                border_color: palette.background.strong.color,
                text_color: palette.background.base.text,
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
            }
        })
    }
    
    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}



// #[cfg(test)]
// mod example {
//     use super::*;
//     use iced::{
//         widget::{button, checkbox, column, text, text_input},
//         Element, Length, Task,
//     };
// 
//     #[derive(Debug, Clone)]
//     enum Message {
//         OverlayCheckboxToggled(bool),
//         TextInputChanged(String),
//         ButtonPressed,
//         OverlayOpened,
//         OverlayClosed,
//     }
// 
//     struct App {
//         overlay_checkbox: bool,
//         text_input_value: String,
//     }
// 
//     impl Default for App {
//         fn default() -> Self {
//             Self {
//                 overlay_checkbox: false,
//                 text_input_value: String::new(),
//             }
//         }
//     }
// 
//     impl App {
//         fn update(&mut self, message: Message) -> Task<Message> {
//             match message {
//                 Message::OverlayCheckboxToggled(checked) => {
//                     self.overlay_checkbox = checked;
//                 }
//                 Message::TextInputChanged(value) => {
//                     self.text_input_value = value;
//                 }
//                 Message::ButtonPressed => {
//                     println!("Button pressed! Checkbox: {}, Text: {}", 
//                             self.overlay_checkbox, self.text_input_value);
//                 }
//                 Message::OverlayOpened => {
//                     println!("Overlay opened");
//                 }
//                 Message::OverlayClosed => {
//                     println!("Overlay closed");
//                 }
//             }
//             Task::none()
//         }
// 
//         fn view(&self) -> Element<Message> {
//             // Create the overlay content - this will stay connected to your app state
//             let overlay_content = column![
//                 text("Overlay Dialog").size(20),
//                 text("This content is connected to the main app state:"),
//                 checkbox("Enable Feature", self.overlay_checkbox)
//                     .on_toggle(Message::OverlayCheckboxToggled),
//                 text_input("Type something...", &self.text_input_value)
//                     .on_input(Message::TextInputChanged),
//                 button("Do Something")
//                     .on_press(Message::ButtonPressed),
//                 text(format!("Current state - Checkbox: {}, Text: '{}'", 
//                            self.overlay_checkbox, self.text_input_value))
//                     .size(12),
//             ]
//             .spacing(15)
//             .padding(20)
//             .into();
// 
//             // Create the overlay button
//             let overlay_btn = overlay_button(
//                 "Open Modal Dialog", 
//                 "Example Overlay",
//                 overlay_content,
//             )
//             .overlay_width(450.0)
//             .overlay_height(350.0)
//             .on_open(|| Message::OverlayOpened)
//             .on_close(|| Message::OverlayClosed);
// 
//             column![
//                 text("Overlay Button Example").size(24),
//                 text("Click the button to open a modal with interactive content:"),
//                 overlay_btn,
//                 text(format!("Main app state - Checkbox: {}, Text: '{}'", 
//                            self.overlay_checkbox, self.text_input_value))
//                     .size(14),
//             ]
//             .spacing(20)
//             .padding(40)
//             .into()
//         }
//     }
// 
//     // This would be your main function if this were a standalone app
//     #[allow(dead_code)]
//     fn run_example() -> iced::Result {
//         iced::application(|| {App::default()}, App::update, App::view)
//             .run()
//     }
// }