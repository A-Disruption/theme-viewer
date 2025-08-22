use iced::{
    advanced::{
        layout,
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    }, 
    alignment::Vertical, border::Radius, event, keyboard, mouse, touch, 
    Alignment, Border, Color, Element, Event, Length, Point, Rectangle, Size, Theme, Vector, Pixels,
};

// Default Settings
const ROW_HEIGHT: f32 = 32.0;       // its the row height, are you even reading the const name? smh
const ARROW_X_PAD: f32 = 4.0;       // where the arrow box starts, relative to indent
const ARROW_W: f32 = 16.0;          // arrow font size
const HANDLE_BASE_W: f32 = 8.0;     // collapsed handle width
const HANDLE_HOVER_W: f32 = 24.0;   // expanded handle width
const HANDLE_STRIPE_W: f32 = 2.0;   // thin base stripe (matches selection strip)
const CONTENT_GAP: f32 = 4.0;       // gap between arrow/handle block and content



#[allow(missing_debug_implementations)]
pub struct TreeHandle<'a, Message,  Theme = iced::Theme, Renderer = iced::Renderer> 
where 
    Theme: Catalog,
{
    branches: Vec<Element<'a, Message, Theme, Renderer>>, 
    width: Length, 
    spacing: f32, 
    indent: f32, 
    padding_x: f32,
    padding_y: f32,
    separator_x: f32,
    separator_y: f32,
    class: Theme::Class<'a>,
}

struct Branch_ {
    align_x: iced::alignment::Horizontal,
    align_y: iced::alignment::Vertical,
}

impl<'a, Message, Theme, Renderer> 
    TreeHandle<'a, Message, Theme, Renderer>
where 
    Theme: Catalog,
    Renderer: iced::advanced::Renderer 
{

    /// Creates a new [`Tree`] with the given branches
    pub fn new<'b, T>( 
        branches: impl IntoIterator< 
            Item = Branch<'a, 'b, T, Message, Theme, Renderer>>,
        ) -> Self
    where 
        T: Clone,
    {
        let branches = branches.into_iter();

        let mut width = Length::Shrink;
        let mut height = Length::Shrink;

        let (mut branches, views): (Vec<_>, Vec<_>) = branches
            .map( |branch| {
                
            }).collect();

    }

    /// Sets the width of the [`Tree`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the padding of the cells of the [`Tree`].
    pub fn padding(self, padding: impl Into<Pixels>) -> Self {
        let padding = padding.into();

        self.padding_x(padding).padding_y(padding)
    }

    /// Sets the horizontal padding of the cells of the [`Tree`].
    pub fn padding_x(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_x = padding.into().0;
        self
    }

    /// Sets the vertical padding of the cells of the [`Tree`].
    pub fn padding_y(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding_y = padding.into().0;
        self
    }

    /// Sets the thickness of the line separator between the cells of the [`Tree`].
    pub fn separator(self, separator: impl Into<Pixels>) -> Self {
        let separator = separator.into();

        self.separator_x(separator).separator_y(separator)
    }

    /// Sets the thickness of the horizontal line separator between the cells of the [`Tree`].
    pub fn separator_x(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_x = separator.into().0;
        self
    }

    /// Sets the thickness of the vertical line separator between the cells of the [`Tree`].
    pub fn separator_y(mut self, separator: impl Into<Pixels>) -> Self {
        self.separator_y = separator.into().0;
        self
    }

}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TreeHandle<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {

    }

    fn tag(&self) -> widget::tree::Tag {

    }

    fn state(&self) -> widget::tree::State {

    }

    fn children(&self) -> Vec<widget::Tree> {

    }

    fn diff(&self, state: &mut widget::Tree) {

    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {

    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {

    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {

    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {

    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {

    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::overlay::Element<'b, Message, Theme, Renderer>> {

    }
}

impl<'a, Message, Theme, Renderer> From<TreeHandle<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(tree: TreeHandle<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tree)
    }
}


#[allow(missing_debug_implementations)]
pub struct Branch<
    'a,
    'b,
    T,
    Message,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> {
    content: Element<'a, Message, Theme, Renderer>,
    view: Box< dyn Fn(T) -> Element<'a, Message, Theme, Renderer>+ 'b>,
    align_x: iced::alignment::Horizontal,
    align_y: iced::alignment::Vertical,
}

impl<'a, 'b, Message, Theme, Renderer> 
    Branch<'a, 'b, Message, Theme, Renderer> {

        pub fn align_x(
            mut self, 
            alignment: impl Into<iced::alignment::Horizontal>
        ) -> Self {
            self.align_x = alignment.into();
            self
        }

        pub fn align_y(
            mut self,
            alignment: impl Into<iced::alignment::Vertical>
        ) -> Self {
            self.align_y = alignment.into();
            self
        }
}





/// The theme catalog for the tree widget
pub trait Catalog {
    /// The style class
    type Class<'a>;
    
    /// Default style
    fn default<'a>() -> Self::Class<'a>;
    
    /// Get the style for a class
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// Style for the tree widget
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Text color
    pub text: Color,
    /// Selection background color
    pub selection_background: Color,
    /// Selection text color
    pub selection_text: Color,
    /// Selection border color
    pub selection_border: Color,
    /// Focus border color
    pub focus_border: Color,
    /// Arrow color
    pub arrow_color: Color,
    /// Line color for connecting lines
    pub line_color: Color,
    /// Drop indicator color - Accept
    pub accept_drop_indicator_color: Color,
    /// Drop indicator color - Deny
    pub deny_drop_indicator_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text: Color::BLACK,
            selection_background: Color::from_rgba(0.0, 0.0, 0.0, 0.05), // Subtle background
            selection_text: Color::BLACK,
            selection_border: Color::from_rgb(0.0, 0.5, 1.0), // Blue border
            focus_border: Color::from_rgba(0.0, 0.5, 1.0, 0.5), // Semi-transparent blue
            arrow_color: Color::from_rgb(0.3, 0.3, 0.3),
            line_color: Color::from_rgb(0.3, 0.3, 0.3),
            accept_drop_indicator_color: Color::from_rgb(0.0, 0.8, 0.0),
            deny_drop_indicator_color: Color::from_rgb(1.0, 0.0, 0.0),
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
            let is_dark = palette.background.base.color.r < 0.5;
            
            Style {
                text: palette.background.base.text,
                selection_background: if is_dark {
                    // For dark themes, use a lighter background
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                } else {
                    // For light themes, use a darker background
                    Color::from_rgba(0.0, 0.0, 0.0, 0.05)
                },
                selection_text: palette.background.base.text,
                selection_border: palette.primary.base.color,
                focus_border: Color::from_rgba(
                    palette.primary.base.color.r,
                    palette.primary.base.color.g,
                    palette.primary.base.color.b,
                    0.5
                ),
                arrow_color: palette.background.strong.color,
                line_color: palette.background.strong.color,
                accept_drop_indicator_color: palette.primary.strong.color,
                deny_drop_indicator_color: palette.danger.strong.color,
            }
        })
    }
    
    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}


