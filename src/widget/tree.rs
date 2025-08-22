//! A draggable tree widget for iced-rs
use iced::advanced::widget::{self, Operation, Tree as WidgetTree};
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell};
use iced::event::Status;
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Vector};
use std::collections::{HashMap, HashSet};

/// A tree node that can contain any iced Element
pub struct TreeNode<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    id: NodeId,
    content: Element<'a, Message, Theme, Renderer>,
    children: Vec<TreeNode<'a, Message, Theme, Renderer>>,
    expanded: bool,
}

/// Unique identifier for tree nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl<'a, Message, Theme, Renderer> TreeNode<'a, Message, Theme, Renderer> {
    /// Create a new tree node with content
    pub fn new(
        id: NodeId,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            id,
            content: content.into(),
            children: Vec::new(),
            expanded: true,
        }
    }

    /// Add a child node
    pub fn with_child(mut self, child: TreeNode<'a, Message, Theme, Renderer>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn with_children(
        mut self,
        children: impl IntoIterator<Item = TreeNode<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.children.extend(children);
        self
    }

    /// Set whether this node is expanded
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// The main tree widget
pub struct Tree<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
{
    root: Option<TreeNode<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    indent_size: f32,
    node_height: f32,
    on_toggle: Option<Box<dyn Fn(NodeId, bool) -> Message + 'a>>,
    on_drag_start: Option<Box<dyn Fn(NodeId, Point) -> Message + 'a>>,
    on_drag_over: Option<Box<dyn Fn(NodeId, NodeId, Point) -> Message + 'a>>,
    on_drop: Option<Box<dyn Fn(NodeId, NodeId, Point) -> Message + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Tree<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    /// Create a new tree widget
    pub fn new() -> Self {
        Self {
            root: None,
            width: Length::Shrink,
            height: Length::Shrink,
            indent_size: 20.0,
            node_height: 30.0,
            on_toggle: None,
            on_drag_start: None,
            on_drag_over: None,
            on_drop: None,
            class: Theme::default(),
        }
    }

    /// Set the root node
    pub fn with_root(mut self, root: TreeNode<'a, Message, Theme, Renderer>) -> Self {
        self.root = Some(root);
        self
    }

    /// Set the width
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set the indentation size for child nodes
    pub fn indent_size(mut self, size: f32) -> Self {
        self.indent_size = size;
        self
    }

    /// Set the height for each node
    pub fn node_height(mut self, height: f32) -> Self {
        self.node_height = height;
        self
    }

    /// Set callback for when a node is toggled (expanded/collapsed)
    pub fn on_toggle(mut self, callback: impl Fn(NodeId, bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }

    /// Set callback for drag start
    pub fn on_drag_start(mut self, callback: impl Fn(NodeId, Point) -> Message + 'a) -> Self {
        self.on_drag_start = Some(Box::new(callback));
        self
    }

    /// Set callback for drag over
    pub fn on_drag_over(mut self, callback: impl Fn(NodeId, NodeId, Point) -> Message + 'a) -> Self {
        self.on_drag_over = Some(Box::new(callback));
        self
    }

    /// Set callback for drop
    pub fn on_drop(mut self, callback: impl Fn(NodeId, NodeId, Point) -> Message + 'a) -> Self {
        self.on_drop = Some(Box::new(callback));
        self
    }

    /// Set the style class
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// Internal state for the tree widget
#[derive(Default)]
struct State {
    /// Maps node IDs to their expanded state
    expanded_nodes: HashMap<NodeId, bool>,
    /// Currently hovered node
    hovered_node: Option<NodeId>,
    /// Currently dragged node
    dragged_node: Option<NodeId>,
    /// Position where drag started
    drag_start: Option<Point>,
    /// Current drag position
    drag_position: Option<Point>,
    /// Node bounds for hit testing
    node_bounds: HashMap<NodeId, Rectangle>,
    /// Nodes that have children
    nodes_with_children: HashSet<NodeId>,
}

impl<Message, Theme, Renderer> widget::Widget<Message, Theme, Renderer>
    for Tree<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<WidgetTree> {
        if let Some(root) = &self.root {
            collect_children(root)
        } else {
            Vec::new()
        }
    }

    fn diff(&self, tree: &mut WidgetTree) {
        if let Some(root) = &self.root {
            let children = collect_elements(root);
            tree.diff_children(&children);
        }
    }

    fn layout(
        &self,
        tree: &mut WidgetTree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state.node_bounds.clear();

        let limits = limits.width(self.width).height(self.height);

        if let Some(root) = &self.root {
            let mut children_iter = tree.children.iter_mut();
            let (node, total_height) = layout_node(
                root,
                &mut children_iter,
                renderer,
                &limits,
                0.0, // depth
                0.0, // y_offset
                self.indent_size,
                self.node_height,
                state,
            );

            let size = limits.resolve(
                self.width,
                self.height,
                Size::new(node.size().width, total_height),
            );

            layout::Node::with_children(size, vec![node])
        } else {
            layout::Node::new(limits.resolve(self.width, self.height, Size::ZERO))
        }
    }

    fn update(
        &mut self,
        tree: &mut WidgetTree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // Update hovered node
                state.hovered_node = find_node_at_position(&state.node_bounds, position.clone());

                // Update drag position if dragging
                if state.dragged_node.is_some() {
                    state.drag_position = Some(position.clone());
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    if let Some(node_id) = find_node_at_position(&state.node_bounds, position) {
                        if let Some(bounds) = state.node_bounds.get(&node_id) {
                            // Define specific control areas
                            let toggle_area = Rectangle::new(
                                Point::new(bounds.x - 20.0, bounds.y),
                                Size::new(15.0, bounds.height),
                            );

                            let drag_handle_area = Rectangle::new(
                                Point::new(bounds.x - 35.0, bounds.y),
                                Size::new(15.0, bounds.height),
                            );

                            if toggle_area.contains(position) && state.nodes_with_children.contains(&node_id) {
                                // Toggle node (only if it has children)
                                let current_expanded = state.expanded_nodes.get(&node_id).copied().unwrap_or(true);
                                let new_expanded = !current_expanded;
                                state.expanded_nodes.insert(node_id, new_expanded);

                                if let Some(on_toggle) = &self.on_toggle {
                                    shell.publish((on_toggle)(node_id, new_expanded));
                                }
                                return;
                            } else if drag_handle_area.contains(position) {
                                // Start drag operation from handle
                                state.dragged_node = Some(node_id);
                                state.drag_start = Some(position);
                                state.drag_position = Some(position);

                                if let Some(on_drag_start) = &self.on_drag_start {
                                    shell.publish((on_drag_start)(node_id, position));
                                }
                                return;
                            }
                        }
                    }
                }

                // Forward events to content elements first
                if let Some(root) = &mut self.root {
                    let mut children_iter = tree.children.iter_mut();
                    if let Some(child_layout) = layout.children().next() {
                        forward_event_to_node(
                            root,
                            &mut children_iter,
                            event,
                            child_layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }

                // If we reach here and clicked on a node content area, start drag from content
                if let Some(position) = cursor.position() {
                    if let Some(node_id) = find_node_at_position(&state.node_bounds, position) {
                        // Only start dragging if we're not already dragging
                        if state.dragged_node.is_none() {
                            state.dragged_node = Some(node_id);
                            state.drag_start = Some(position);
                            state.drag_position = Some(position);

                            if let Some(on_drag_start) = &self.on_drag_start {
                                shell.publish((on_drag_start)(node_id, position));
                            }
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(dragged_id) = state.dragged_node.take() {
                    if let (Some(drop_position), Some(target_id)) = (
                        state.drag_position,
                        find_node_at_position(&state.node_bounds, state.drag_position.unwrap_or_default())
                    ) {
                        if dragged_id != target_id {
                            if let Some(on_drop) = &self.on_drop {
                                shell.publish((on_drop)(dragged_id, target_id, drop_position));
                            }
                        }
                    }

                    state.drag_start = None;
                    state.drag_position = None;
                    return;
                }

                // Forward button release to content elements
                if let Some(root) = &mut self.root {
                    let mut children_iter = tree.children.iter_mut();
                    if let Some(child_layout) = layout.children().next() {
                        forward_event_to_node(
                            root,
                            &mut children_iter,
                            event,
                            child_layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }
            }
            _ => {
                // Forward all other events to content elements
                if let Some(root) = &mut self.root {
                    let mut children_iter = tree.children.iter_mut();
                    if let Some(child_layout) = layout.children().next() {
                        forward_event_to_node(
                            root,
                            &mut children_iter,
                            event,
                            child_layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }
            }
        }
    }

    fn draw(
        &self,
        tree: &WidgetTree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let tree_style = theme.style(&self.class);

        if let Some(root) = &self.root {
            let mut children_iter = tree.children.iter();
            if let Some(child_layout) = layout.children().next() {
                draw_node(
                    root,
                    &mut children_iter,
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    viewport,
                    &tree_style,
                    state,
                    0.0, // depth
                );
            }
        }

        // Draw drag overlay if dragging
        if let (Some(dragged_id), Some(drag_pos)) = (state.dragged_node, state.drag_position) {
            if let Some(bounds) = state.node_bounds.get(&dragged_id) {
                let drag_bounds = Rectangle::new(
                    Point::new(drag_pos.x - bounds.width / 2.0, drag_pos.y - bounds.height / 2.0),
                    bounds.size(),
                );

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: drag_bounds,
                        border: iced::Border {
                            radius: iced::border::Radius::new(4.0),
                            color: tree_style.selection_border,
                            ..iced::Border::default()
                        },
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    tree_style.selection_background,
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &WidgetTree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.dragged_node.is_some() {
            return mouse::Interaction::Grabbing;
        }

        if let Some(position) = cursor.position() {
            if find_node_at_position(&state.node_bounds, position).is_some() {
                return mouse::Interaction::Pointer;
            }
        }

        mouse::Interaction::default()
    }

    fn operate(
        &self,
        tree: &mut WidgetTree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if let Some(root) = &self.root {
            let mut children_iter = tree.children.iter_mut();
            if let Some(child_layout) = layout.children().next() {
                operate_on_node(root, &mut children_iter, child_layout, renderer, operation);
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut WidgetTree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // For the initial implementation, let's skip overlay support
        // This can be added later once the basic tree functionality is working
        None
    }
}

// Helper functions

fn collect_children<Message, Theme, Renderer>(
    node: &TreeNode<'_, Message, Theme, Renderer>,
) -> Vec<WidgetTree> 
where
    Renderer: iced::advanced::Renderer,
{
    let mut trees = vec![WidgetTree::new(&node.content)];
    for child in &node.children {
        trees.extend(collect_children(child));
    }
    trees
}

fn collect_elements<'a, Message, Theme, Renderer>(
    node: &'a TreeNode<'a, Message, Theme, Renderer>,
) -> Vec<&'a Element<'a, Message, Theme, Renderer>> {
    let mut elements = vec![&node.content];
    for child in &node.children {
        elements.extend(collect_elements(child));
    }
    elements
}

fn layout_node<Message, Theme, Renderer>(
    node: &TreeNode<'_, Message, Theme, Renderer>,
    children_iter: &mut std::slice::IterMut<WidgetTree>,
    renderer: &Renderer,
    limits: &layout::Limits,
    depth: f32,
    y_offset: f32,
    indent_size: f32,
    node_height: f32,
    state: &mut State,
) -> (layout::Node, f32)
where
    Renderer: iced::advanced::Renderer,
{
    let child_tree = children_iter.next().expect("Missing child tree");
    let x_offset = depth * indent_size;

    // Layout the content
    let content_limits = layout::Limits::new(
        Size::ZERO,
        Size::new(limits.max().width - x_offset, node_height),
    );
    let content_layout = node.content.as_widget().layout(child_tree, renderer, &content_limits);
    let content_size = content_layout.size();

    // Store bounds for hit testing - only the actual content area, not overlapping with children
    let node_bounds = Rectangle::new(
        Point::new(x_offset, y_offset),
        Size::new(content_size.width, node_height), // Only this node's height, not children
    );
    state.node_bounds.insert(node.id, node_bounds);
    if !node.children.is_empty() {
        state.nodes_with_children.insert(node.id);
    } else {
        state.nodes_with_children.remove(&node.id);
    }

    let mut total_height = node_height;
    let mut child_nodes = vec![content_layout.move_to(Point::new(x_offset, y_offset))];

    // Layout children if expanded
    let is_expanded = state.expanded_nodes.get(&node.id).copied().unwrap_or(node.expanded);
    if is_expanded {
        let mut child_y = y_offset + node_height;
        for child in &node.children {
            let (child_layout, child_height) = layout_node(
                child,
                children_iter,
                renderer,
                limits,
                depth + 1.0,
                child_y,
                indent_size,
                node_height,
                state,
            );
            child_nodes.push(child_layout);
            child_y += child_height;
            total_height += child_height;
        }
    }

    let node_size = Size::new(
        (content_size.width + x_offset).max(limits.max().width),
        total_height,
    );

    (layout::Node::with_children(node_size, child_nodes), total_height)
}

fn find_node_at_position(node_bounds: &HashMap<NodeId, Rectangle>, position: Point) -> Option<NodeId> {
    // Find all nodes that contain this position
    let mut containing_nodes: Vec<(NodeId, Rectangle)> = node_bounds
        .iter()
        .filter(|(_, bounds)| bounds.contains(position))
        .map(|(id, bounds)| (*id, *bounds))
        .collect();
    
    if containing_nodes.is_empty() {
        return None;
    }
    
    // Sort by y position (top to bottom) then by x position (left to right)
    // This should give us the deepest/most specific node
    containing_nodes.sort_by(|a, b| {
        let y_cmp = a.1.y.partial_cmp(&b.1.y).unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp == std::cmp::Ordering::Equal {
            a.1.x.partial_cmp(&b.1.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_cmp
        }
    });
    
    // Return the last one (bottommost, then rightmost)
    containing_nodes.last().map(|(id, _)| *id)
}

fn forward_event_to_node<Message, Theme, Renderer>(
    node: &mut TreeNode<'_, Message, Theme, Renderer>,
    children_iter: &mut std::slice::IterMut<WidgetTree>,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
)
where
    Renderer: iced::advanced::Renderer,
{
    let child_tree = children_iter.next().expect("Missing child tree");
    let mut child_layouts = layout.children();
    let content_layout = child_layouts.next().expect("Missing content layout");

    node.content.as_widget_mut().update(
        child_tree,
        event,
        content_layout,
        cursor,
        renderer,
        clipboard,
        shell,
        viewport,
    );

    // Forward to children
    for (child, child_layout) in node.children.iter_mut().zip(child_layouts) {
        forward_event_to_node(
            child,
            children_iter,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
}

fn draw_node<Message, Theme, Renderer>(
    node: &TreeNode<'_, Message, Theme, Renderer>,
    children_iter: &mut std::slice::Iter<WidgetTree>,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    tree_style: &Style,
    state: &State,
    depth: f32,
)
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    let child_tree = children_iter.next().expect("Missing child tree");
    let mut child_layouts = layout.children();
    let content_layout = child_layouts.next().expect("Missing content layout");

    // Draw selection/hover background
    if Some(node.id) == state.hovered_node || Some(node.id) == state.dragged_node {
        let bounds = content_layout.bounds();
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    radius: iced::border::Radius::new(4.0),
                    color: tree_style.selection_border,
                    ..iced::Border::default()
                },
                shadow: iced::Shadow::default(),
                snap: true,
            },
            tree_style.selection_background,
        );
    }

    // Draw expand/collapse arrow if has children
    if !node.children.is_empty() {
        let bounds = content_layout.bounds();
        let is_expanded = state.expanded_nodes.get(&node.id).copied().unwrap_or(node.expanded);
        
        let arrow_bounds = Rectangle::new(
            Point::new(bounds.x - 15.0, bounds.y + bounds.height / 2.0 - 5.0),
            Size::new(10.0, 10.0),
        );

        // Draw a proper triangle for expand/collapse
        let triangle_color = tree_style.arrow_color;
        
        if is_expanded {
            // Downward pointing triangle (expanded)
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 2.0, arrow_bounds.y + 3.0),
                        Size::new(6.0, 1.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 3.0, arrow_bounds.y + 4.0),
                        Size::new(4.0, 1.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 4.0, arrow_bounds.y + 5.0),
                        Size::new(2.0, 1.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
        } else {
            // Right pointing triangle (collapsed)
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 3.0, arrow_bounds.y + 2.0),
                        Size::new(1.0, 6.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 4.0, arrow_bounds.y + 3.0),
                        Size::new(1.0, 4.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(arrow_bounds.x + 5.0, arrow_bounds.y + 4.0),
                        Size::new(1.0, 2.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                triangle_color,
            );
        }
    }

    // Draw drag handle on hover (positioned further left to avoid content overlap)
    if Some(node.id) == state.hovered_node {
        let bounds = content_layout.bounds();
        let handle_bounds = Rectangle::new(
            Point::new(bounds.x - 30.0, bounds.y + bounds.height / 2.0 - 3.0),
            Size::new(12.0, 6.0),
        );
        
        // Draw handle background
        renderer.fill_quad(
            renderer::Quad {
                bounds: handle_bounds,
                border: iced::Border {
                    radius: iced::border::Radius::new(2.0),
                    color: tree_style.arrow_color,
                    width: 1.0,
                },
                shadow: iced::Shadow::default(),
                snap: true,
            },
            Color::from_rgba(
                tree_style.arrow_color.r,
                tree_style.arrow_color.g,
                tree_style.arrow_color.b,
                0.3,
            ),
        );
        
        // Draw grip dots to make it look like a handle
        for i in 0..3 {
            for j in 0..2 {
                let dot_x = handle_bounds.x + 3.0 + (j as f32 * 3.0);
                let dot_y = handle_bounds.y + 1.5 + (i as f32 * 1.5);
                let dot_bounds = Rectangle::new(
                    Point::new(dot_x, dot_y),
                    Size::new(1.0, 1.0),
                );
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: dot_bounds,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    tree_style.arrow_color,
                );
            }
        }
    }

    // Draw connecting lines
    if depth > 0.0 {
        let bounds = content_layout.bounds();
        
        // Vertical line from parent
        let line_x = bounds.x - 25.0;
        let vertical_line_bounds = Rectangle::new(
            Point::new(line_x, bounds.y),
            Size::new(1.0, bounds.height / 2.0),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: vertical_line_bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            },
            tree_style.line_color,
        );
        
        // Horizontal line to node
        let horizontal_line_bounds = Rectangle::new(
            Point::new(line_x, bounds.y + bounds.height / 2.0 - 0.5),
            Size::new(10.0, 1.0),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: horizontal_line_bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            },
            tree_style.line_color,
        );
    }

    // Draw the content
    node.content.as_widget().draw(
        child_tree,
        renderer,
        theme,
        style,
        content_layout,
        cursor,
        viewport,
    );

    // Draw children if expanded
    let is_expanded = state.expanded_nodes.get(&node.id).copied().unwrap_or(node.expanded);
    if is_expanded {
        for (child, child_layout) in node.children.iter().zip(child_layouts) {
            draw_node(
                child,
                children_iter,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
                tree_style,
                state,
                depth + 1.0,
            );
        }
    }
}

fn operate_on_node<Message, Theme, Renderer>(
    node: &TreeNode<'_, Message, Theme, Renderer>,
    children_iter: &mut std::slice::IterMut<WidgetTree>,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn Operation,
)
where
    Renderer: iced::advanced::Renderer,
{
    let child_tree = children_iter.next().expect("Missing child tree");
    let mut child_layouts = layout.children();
    let content_layout = child_layouts.next().expect("Missing content layout");

    node.content.as_widget().operate(child_tree, content_layout, renderer, operation);

    for (child, child_layout) in node.children.iter().zip(child_layouts) {
        operate_on_node(child, children_iter, child_layout, renderer, operation);
    }
}

impl<'a, Message, Theme, Renderer> From<Tree<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(tree: Tree<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tree)
    }
}

/// The theme catalog for the tree widget
pub trait Catalog {
    type Class<'a>;
    
    fn default<'a>() -> Self::Class<'a>;
    
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// Style for the tree widget
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub text: Color,
    pub selection_background: Color,
    pub selection_text: Color,
    pub selection_border: Color,
    pub focus_border: Color,
    pub arrow_color: Color,
    pub line_color: Color,
    pub accept_drop_indicator_color: Color,
    pub deny_drop_indicator_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text: Color::BLACK,
            selection_background: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            selection_text: Color::BLACK,
            selection_border: Color::from_rgb(0.0, 0.5, 1.0),
            focus_border: Color::from_rgba(0.0, 0.5, 1.0, 0.5),
            arrow_color: Color::from_rgb(0.3, 0.3, 0.3),
            line_color: Color::from_rgb(0.3, 0.3, 0.3),
            accept_drop_indicator_color: Color::from_rgb(0.0, 0.8, 0.0),
            deny_drop_indicator_color: Color::from_rgb(1.0, 0.0, 0.0),
        }
    }
}

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
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                } else {
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