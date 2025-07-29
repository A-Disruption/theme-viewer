use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    },
    alignment::Vertical,
    event, keyboard, mouse, touch,
    Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, Theme,
};
use std::collections::HashMap;
use std::marker::PhantomData;

/// A tree widget that displays hierarchical data with expand/collapse functionality
#[allow(missing_debug_implementations)]
pub struct TreeWidget<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    nodes: Vec<TreeNode<'a>>,
    on_select: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_toggle: Option<Box<dyn Fn(String) -> Message + 'a>>,
    spacing: f32,
    indent: f32,
    class: Theme::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

/// Individual tree node
#[derive(Debug, Clone)]
pub struct TreeNode<'a> {
    pub id: String,
    pub label: &'a str,
    pub children: Vec<TreeNode<'a>>,
}

/// State to track expanded nodes and selections
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    pub expanded: HashMap<String, bool>,
    pub selected: Option<String>,
}

impl<'a> TreeNode<'a> {
    pub fn new(id: String, label: &'a str) -> Self {
        Self {
            id,
            label,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNode<'a>>) -> Self {
        self.children = children;
        self
    }

    pub fn push_child(mut self, child: TreeNode<'a>) -> Self {
        self.children.push(child);
        self
    }
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.get(id).copied().unwrap_or(false)
    }

    pub fn toggle(&mut self, id: String) {
        let current = self.is_expanded(&id);
        self.expanded.insert(id, !current);
    }

/*     pub fn expand(&mut self, id: String) {
        self.expanded.insert(id, true);
    }

    pub fn collapse(&mut self, id: String) {
        self.expanded.insert(id, false);
    } */

    pub fn select(&mut self, id: String) {
        self.selected = Some(id);
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.as_ref().map_or(false, |s| s == id)
    }
}

#[derive(Debug, Clone)]
struct State {
    tree_state: TreeState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tree_state: TreeState::new(),
        }
    }
}

impl<'a, Message, Theme, Renderer> TreeWidget<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    pub fn new(nodes: Vec<TreeNode<'a>>) -> Self {
        Self {
            nodes,
            on_select: None,
            on_toggle: None,
            spacing: 4.0,
            indent: 20.0,
            class: Theme::default(),
            _renderer: PhantomData,
        }
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_toggle = Some(Box::new(f));
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TreeWidget<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        let state = tree.state.downcast_ref::<State>();
        let mut y_offset = 0.0;
        let mut children = Vec::new();

        for node in &self.nodes {
            let (child_layout, height) = self.layout_node(
                node, 
                &state.tree_state, 
                renderer, 
                limits, 
                0.0, 
                y_offset
            );
            children.push(child_layout.move_to(Point::new(0.0, y_offset)));
            y_offset += height;
        }

        Node::with_children(
            Size::new(limits.max().width, y_offset),
            children,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let mut child_layouts = layout.children();
        
        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                self.draw_node(
                    node,
                    &state.tree_state,
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    viewport,
                    0.0,
                );
            }
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(cursor_position) = cursor.position() {
                let mut child_layouts = layout.children();
                
                for node in &self.nodes {
                    if let Some(child_layout) = child_layouts.next() {
                        if self.handle_node_click(
                            node,
                            &mut state.tree_state,
                            cursor_position,
                            child_layout,
                            0.0,
                            shell,
                        ) {
                            shell.invalidate_layout();
                        }
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.get_hovered_node(layout, cursor.position()).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::Idle
        }
    }
}

impl<'a, Message, Theme, Renderer> TreeWidget<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn layout_node(
        &self,
        node: &TreeNode<'a>,
        tree_state: &TreeState,
        renderer: &Renderer,
        limits: &Limits,
        indent_level: f32,
        y_offset: f32,
    ) -> (Node, f32) {
        let line_height = 24.0;
        let mut total_height = line_height + self.spacing;
        let mut children = Vec::new();

        // Add the node itself
        let node_layout = Node::new(Size::new(
            limits.max().width,
            line_height,
        ));
        children.push(node_layout);

        // Add children if expanded
        if tree_state.is_expanded(&node.id) {
            let mut child_y_offset = line_height + self.spacing;
            
            for child in &node.children {
                let (child_layout, child_height) = self.layout_node(
                    child,
                    tree_state,
                    renderer,
                    limits,
                    indent_level + 1.0,
                    child_y_offset,
                );
                children.push(child_layout.move_to(Point::new(0.0, child_y_offset)));
                child_y_offset += child_height;
                total_height += child_height;
            }
        }

        (
            Node::with_children(
                Size::new(limits.max().width, total_height),
                children,
            ),
            total_height,
        )
    }

    fn draw_node(
        &self,
        node: &TreeNode<'a>,
        tree_state: &TreeState,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        indent_level: f32,
    ) {
        let bounds = layout.bounds();
        let line_height = 24.0;
        let mut child_layouts = layout.children();

        // Get the FIRST child layout (which represents THIS node's drawing area)
        if let Some(node_layout) = child_layouts.next() {
            let node_bounds = node_layout.bounds(); 
            let x_offset = indent_level * self.indent;
            let is_expanded = tree_state.is_expanded(&node.id);
            let is_selected = tree_state.is_selected(&node.id);
            let has_children = !node.children.is_empty();
            let appearance = <Theme as Catalog>::style(theme, &self.class);

            // Draw selection background using the node's specific bounds
            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(node_bounds.x, node_bounds.y),
                            Size::new(node_bounds.width, line_height),
                        ),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    appearance.selection_background,
                );
            }

            // Draw expand/collapse arrow using node's y position
            if has_children {
                let arrow_x = node_bounds.x + x_offset + 4.0;
                
                let arrow_text = if is_expanded { "▼" } else { "▶" };
                
                renderer.fill_text(
                    iced::advanced::Text {
                        content: arrow_text.to_string(),
                        bounds: Size::new(16.0, line_height),
                        size: iced::Pixels(14.0),
                        font: iced::Font::default(),
                        align_x: iced::advanced::text::Alignment::Left,
                        align_y: Vertical::Top,
                        line_height: iced::advanced::text::LineHeight::default(),
                        shaping: iced::advanced::text::Shaping::Advanced,
                        wrapping: iced::advanced::text::Wrapping::default(),
                    },
                    //Point::new(arrow_x, node_bounds.y + (line_height - 14.0) / 2.0),
                    Point::new(arrow_x, node_bounds.y),
                    appearance.arrow_color,
                    Rectangle::new(Point::new(arrow_x, node_bounds.y), Size::new(16.0, line_height)),
                );
            }

            // Draw node text using node's y position
            let text_x = x_offset + if has_children { 20.0 } else { 4.0 };
            let text_color = if is_selected {
                appearance.selection_text
            } else {
                appearance.text
            };

            renderer.fill_text(
                iced::advanced::Text {
                    content: node.label.to_string(),
                    bounds: Size::new(node_bounds.width - text_x, line_height),
                    size: iced::Pixels(14.0),
                    font: iced::Font::default(),
                    align_x: iced::advanced::text::Alignment::Left,
                    align_y: Vertical::Top,
                    line_height: iced::advanced::text::LineHeight::default(),
                    shaping: iced::advanced::text::Shaping::Basic,
                    wrapping: iced::advanced::text::Wrapping::default(),
                },
                //Point::new(node_bounds.x + text_x, node_bounds.y + (line_height - 14.0) / 2.0),
                Point::new(node_bounds.x + text_x, node_bounds.y),
                text_color,
                node_bounds, // Use node_bounds for clipping
            );

            // Draw children if expanded
            if is_expanded && !node.children.is_empty() {
                // Calculate the total height of all visible children (including nested)
                let mut total_children_height = 0.0;
                let mut visible_child_count = 0;
                
                for child in &node.children {
                    visible_child_count += 1;
                    total_children_height += line_height + self.spacing;
                    
                    // Add height of nested children if this child is expanded
                    if tree_state.is_expanded(&child.id) {
                        total_children_height += self.calculate_expanded_height(child, tree_state);
                    }
                }
                
                if visible_child_count > 0 {
                    // Draw vertical line from top of first child to bottom of last visible child
                    let parent_arrow_x = node_bounds.x + x_offset + 10.0;
                    let first_child_y = node_bounds.y + line_height + self.spacing;
                    let last_child_bottom_y = first_child_y + total_children_height - self.spacing;
                    
                    // Draw vertical line
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(parent_arrow_x - 0.5, first_child_y),
                                Size::new(1.0, last_child_bottom_y - first_child_y),
                            ),
                            border: Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: true,
                        },
                        appearance.arrow_color,
                    );
                }
                
                // Draw children
                for child in &node.children {
                    if let Some(child_layout) = child_layouts.next() {
                        self.draw_node(
                            child,
                            tree_state,
                            renderer,
                            theme,
                            style,
                            child_layout,
                            cursor,
                            viewport,
                            indent_level + 1.0,
                        );
                    }
                }
            }
        }
    }

    fn handle_node_click(
        &self,
        node: &TreeNode<'a>,
        tree_state: &mut TreeState,
        cursor_position: Point,
        layout: Layout<'_>,
        indent_level: f32,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let bounds = layout.bounds();
        let line_height = 24.0;
        
        // Check if click is on this node
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            let x_offset = indent_level * self.indent;
            let arrow_x = bounds.x + x_offset + 4.0;
            
            // Check if click is on arrow (for expansion/collapse)
            if !node.children.is_empty() 
                && cursor_position.x >= arrow_x 
                && cursor_position.x < arrow_x + 16.0 {
                
                tree_state.toggle(node.id.clone());
                if let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(node.id.clone()));
                }
            } else {
                // Click on node text (for selection)
                tree_state.select(node.id.clone());
                if let Some(on_select) = &self.on_select {
                    shell.publish((on_select)(node.id.clone()));
                }
            }
            return true;
        }

        // Check children if expanded
        if tree_state.is_expanded(&node.id) {
            let mut child_layouts = layout.children();
            child_layouts.next(); // Skip the node layout
            
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if self.handle_node_click(
                        child,
                        tree_state,
                        cursor_position,
                        child_layout,
                        indent_level + 1.0,
                        shell,
                    ) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn calculate_expanded_height(&self, node: &TreeNode<'a>, tree_state: &TreeState) -> f32 {
        let line_height = 24.0;
        let mut height = 0.0;
        
        for child in &node.children {
            height += line_height + self.spacing;
            
            // Recursively calculate height of expanded grandchildren
            if tree_state.is_expanded(&child.id) {
                height += self.calculate_expanded_height(child, tree_state);
            }
        }
        
        height
    }

    fn get_hovered_node(&self, layout: Layout<'_>, cursor_position: Option<Point>) -> Option<&TreeNode<'a>> {
        let cursor_position = cursor_position?;
        let mut child_layouts = layout.children();
        
        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(hovered) = self.get_hovered_node_recursive(node, child_layout, cursor_position, 0.0) {
                    return Some(hovered);
                }
            }
        }
        
        None
    }

    fn get_hovered_node_recursive<'b>(
        &self,
        node: &'b TreeNode<'a>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> Option<&'b TreeNode<'a>> {
        let bounds = layout.bounds();
        let line_height = 24.0;
        
        // Check if cursor is over this node
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            return Some(node);
        }

        // Check children if they exist and are visible
        let mut child_layouts = layout.children();
        child_layouts.next(); // Skip the node layout
        
        for child in &node.children {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(hovered) = self.get_hovered_node_recursive(
                    child, 
                    child_layout, 
                    cursor_position, 
                    indent_level + 1.0
                ) {
                    return Some(hovered);
                }
            }
        }

        None
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
    /// Arrow color
    pub arrow_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text: Color::BLACK,
            selection_background: Color::from_rgb(0.0, 0.5, 1.0),
            selection_text: Color::WHITE,
            arrow_color: Color::from_rgb(0.3, 0.3, 0.3),
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
                text: palette.background.base.text,
                selection_background: palette.primary.base.color,
                selection_text: palette.primary.base.text,
                arrow_color: palette.background.strong.color,
            }
        })
    }
    
    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// Helper function to create the tree widget
pub fn tree<'a, Message, Theme, Renderer>(
    nodes: Vec<TreeNode<'a>>,
) -> TreeWidget<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    TreeWidget::new(nodes)
}

impl<'a, Message, Theme, Renderer> From<TreeWidget<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(tree: TreeWidget<'a, Message, Theme, Renderer>) -> Self {
        Self::new(tree)
    }
}