use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    },
    alignment::Vertical,
    border::Radius,
    event, keyboard, mouse, touch, Border, Color, Element, Event, Length, Point, Rectangle, Size,
    Theme, Vector,
};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// A trait for creating tree node content on demand
pub trait TreeNodeContent<'a, Message, Theme, Renderer>: 'a {
    fn view(&self) -> Element<'a, Message, Theme, Renderer>;
    fn id(&self) -> &str;
    fn accepts_drops(&self) -> bool {
        false
    }
}

/// A simple implementation for static content
pub struct StaticNodeContent<'a, Message, Theme, Renderer> {
    id: String,
    content: Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'a>,
    accepts_drops: bool,
}

impl<'a, Message, Theme, Renderer> StaticNodeContent<'a, Message, Theme, Renderer> {
    pub fn new<F>(id: impl Into<String>, content: F) -> Self
    where
        F: Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
    {
        Self {
            id: id.into(),
            content: Box::new(content),
            accepts_drops: false,
        }
    }

    pub fn accepts_drops(mut self) -> Self {
        self.accepts_drops = true;
        self
    }
}

impl<'a, Message, Theme, Renderer> TreeNodeContent<'a, Message, Theme, Renderer>
    for StaticNodeContent<'a, Message, Theme, Renderer>
where 
    Message: 'a,
    Theme: 'a,
    Renderer: 'a
{
    fn view(&self) -> Element<'a, Message, Theme, Renderer> {
        let element = (self.content)();
        println!("🎯 Creating content element for id: {}", self.id);
        element
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn accepts_drops(&self) -> bool {
        self.accepts_drops
    }
}

/// The main tree structure that manages nodes
pub struct TreeManager<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    nodes: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
}

/// Internal node structure
pub struct TreeNodeData<'a, Message, Theme, Renderer> {
    content: Box<dyn TreeNodeContent<'a, Message, Theme, Renderer>>,
    children: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> TreeManager<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create from a vector of node contents
    pub fn from_children<I, C>(children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: TreeNodeContent<'a, Message, Theme, Renderer> + 'static,
    {
        Self {
            nodes: children
                .into_iter()
                .map(|content| TreeNodeData {
                    content: Box::new(content),
                    children: Vec::new(),
                })
                .collect(),
        }
    }

    /// Add a root node
    pub fn add<C>(&mut self, content: C) -> &mut Self
    where
        C: TreeNodeContent<'a, Message, Theme, Renderer> + 'static,
    {
        self.nodes.push(TreeNodeData {
            content: Box::new(content),
            children: Vec::new(),
        });
        self
    }

    /// Add a child to a specific parent
    pub fn add_child<C>(&mut self, parent_id: &str, content: C) -> &mut Self
    where
        C: TreeNodeContent<'a, Message, Theme, Renderer> + 'static,
    {
        if let Some(parent) = self.find_node_mut(parent_id) {
            parent.children.push(TreeNodeData {
                content: Box::new(content),
                children: Vec::new(),
            });
        }
        self
    }

    /// Move a node to a new location
    pub fn move_node(&mut self, node_id: &str, target_id: &str, position: DropPosition) -> bool {
        // Remove the node
        if let Some(node) = self.remove_node_by_id(node_id) {
            // Insert at new location
            match position {
                DropPosition::Into => {
                    if let Some(target) = self.find_node_mut(target_id) {
                        target.children.push(node);
                        return true;
                    }
                }
                DropPosition::Before | DropPosition::After => {
                    return self.insert_sibling(target_id, node, position);
                }
            }
        }
        false
    }

    /// Build the widget
    pub fn view(self) -> TreeWidget<'a, Message, Theme, Renderer> {
        TreeWidget::new(self.nodes)
    }

    // Helper methods
    fn find_node_mut(&mut self, id: &str) -> Option<&mut TreeNodeData<'a, Message, Theme, Renderer>> {
        Self::find_node_mut_helper(&mut self.nodes, id)
    }

    fn find_node_mut_helper<'b>(
        nodes: &'b mut [TreeNodeData<'a, Message, Theme, Renderer>],
        id: &str,
    ) -> Option<&'b mut TreeNodeData<'a, Message, Theme, Renderer>> {
        for node in nodes {
            if node.content.id() == id {
                return Some(node);
            }
            if let Some(found) = Self::find_node_mut_helper(&mut node.children, id) {
                return Some(found);
            }
        }
        None
    }

    fn remove_node_by_id(
        &mut self,
        id: &str,
    ) -> Option<TreeNodeData<'a, Message, Theme, Renderer>> {
        Self::remove_node_helper(&mut self.nodes, id)
    }

    fn remove_node_helper(
        nodes: &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
        id: &str,
    ) -> Option<TreeNodeData<'a, Message, Theme, Renderer>> 
        where 
            Message: Clone + 'a,
            Theme: 'a,
            Renderer: 'a
        {
        for i in 0..nodes.len() {
            if nodes[i].content.id() == id {
                return Some(nodes.remove(i));
            }
        }

        for node in nodes {
            if let Some(removed) = Self::remove_node_helper(&mut node.children, id) {
                return Some(removed);
            }
        }
        None
    }

    fn insert_sibling(
        &mut self,
        target_id: &str,
        new_node: TreeNodeData<'a, Message, Theme, Renderer>,
        position: DropPosition,
    ) -> bool {
        Self::insert_sibling_helper(&mut self.nodes, target_id, new_node, position)
    }

    fn insert_sibling_helper(
        nodes: &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
        target_id: &str,
        new_node: TreeNodeData<'a, Message, Theme, Renderer>,
        position: DropPosition,
    ) -> bool
    where
        Message: Clone + 'a,
        Theme: 'a,
        Renderer: 'a,
    {
        // Check current level first
        for i in 0..nodes.len() {
            if nodes[i].content.id() == target_id {
                let index = match position {
                    DropPosition::Before => i,
                    DropPosition::After => i + 1,
                    _ => return false,
                };
                nodes.insert(index, new_node);
                return true;
            }
        }

        // Store the node temporarily while we search children
        let mut temp_node = Some(new_node);
        
        for i in 0..nodes.len() {
            if let Some(node) = temp_node.take() {
                if Self::insert_sibling_helper(&mut nodes[i].children, target_id, node, position) {
                    return true;
                } else {
                    // Get the node back if it wasn't inserted
                    temp_node = Self::remove_node_helper(&mut nodes[i].children, "temp_unreachable_id");
                }
            }
        }
        
        false
    }
}

/// A tree widget that displays hierarchical data with expand/collapse functionality
#[allow(missing_debug_implementations)]
pub struct TreeWidget<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    nodes: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
    on_select: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_toggle: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_drop: Option<Box<dyn Fn(Vec<String>, String, DropPosition) -> Message + 'a>>,
    spacing: f32,
    indent: f32,
    class: Theme::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

/// State to track expanded nodes and selections
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    pub expanded: HashMap<String, bool>,
    pub selected: HashSet<String>,
    pub focused: Option<String>,
    pub drag_state: Option<DragState>,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub dragged_nodes: Vec<String>,
    pub drag_start_position: Point,
    pub current_position: Point,
    pub drop_target: Option<String>,
    pub drop_position: DropPosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropPosition {
    Before,
    After,
    Into,
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

    pub fn select(&mut self, id: String) {
        self.selected.clear();
        self.selected.insert(id);
    }

    pub fn toggle_select(&mut self, id: String) {
        if self.selected.contains(&id) {
            self.selected.remove(&id);
        } else {
            self.selected.insert(id);
        }
    }

    pub fn add_to_selection(&mut self, id: String) {
        self.selected.insert(id);
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.contains(id)
    }

    pub fn focus(&mut self, id: String) {
        self.focused = Some(id);
    }

    pub fn is_focused(&self, id: &str) -> bool {
        self.focused.as_ref().map_or(false, |s| s == id)
    }

    pub fn start_drag(&mut self, nodes: Vec<String>, start_position: Point) {
        self.drag_state = Some(DragState {
            dragged_nodes: nodes,
            drag_start_position: start_position,
            current_position: start_position,
            drop_target: None,
            drop_position: DropPosition::Into,
        });
    }

    pub fn update_drag_position(&mut self, position: Point) {
        if let Some(ref mut drag_state) = self.drag_state {
            drag_state.current_position = position;
        }
    }

    pub fn set_drop_target(&mut self, target: Option<String>, position: DropPosition) {
        if let Some(ref mut drag_state) = self.drag_state {
            drag_state.drop_target = target;
            drag_state.drop_position = position;
        }
    }

    pub fn end_drag(&mut self) -> Option<DragState> {
        self.drag_state.take()
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn is_being_dragged(&self, id: &str) -> bool {
        self.drag_state
            .as_ref()
            .map_or(false, |drag| drag.dragged_nodes.contains(&String::from(id)))
    }
}

struct State {
    tree_state: TreeState,
    ctrl_pressed: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tree_state: TreeState::new(),
            ctrl_pressed: false,
        }
    }
}

impl<'a, Message, Theme, Renderer> TreeWidget<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Theme: Catalog + 'a,
{
    pub fn new(nodes: Vec<TreeNodeData<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            nodes,
            on_select: None,
            on_toggle: None,
            on_drop: None,
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

    pub fn on_drop<F>(mut self, f: F) -> Self
    where
        F: Fn(Vec<String>, String, DropPosition) -> Message + 'a,
    {
        self.on_drop = Some(Box::new(f));
        self
    }

    fn find_node_by_id<'b>(&'b self, id: &str) -> Option<&'b TreeNodeData<'a, Message, Theme, Renderer>> {
        Self::find_node_in_list(&self.nodes, id)
    }

    fn find_node_in_list<'b>(nodes: &'b [TreeNodeData<'a, Message, Theme, Renderer>], id: &str) -> Option<&'b TreeNodeData<'a, Message, Theme, Renderer>> {
        for node in nodes {
            if node.content.id() == id {
                return Some(node);
            }
            if let Some(found) = Self::find_node_in_list(&node.children, id) {
                return Some(found);
            }
        }
        None
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
    Message: Clone + 'static,
    Theme: Catalog + iced::widget::text::Catalog + 'static,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'static,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        let mut trees = Vec::new();
        for node in &self.nodes {
            self.collect_all_content_trees(node, &mut trees);
        }
        println!("🌲 Tree widget has {} child trees", trees.len());
        trees
    }

    fn diff(&self, tree: &mut Tree) {
        let mut expected_children = Vec::new();
        for node in &self.nodes {
            self.collect_all_content_elements(node, &mut expected_children);
        }
        tree.diff_children(&expected_children);
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
        let mut y_offset = 0.0;
        let mut children = Vec::new();
        let mut tree_index = 0;

        let state = tree.state.downcast_ref::<State>();

        let tree_state = {
            let state = tree.state.downcast_ref::<State>();
            state.tree_state.clone()
        };

        for node in &self.nodes {
            let (child_layout, height) = self.layout_node(
                node,
                &tree_state,
                tree,
                &mut tree_index,
                renderer,
                limits,
                0.0,
                y_offset,
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
        let mut tree_index = 0;

        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                self.draw_node(
                    node,
                    tree,
                    &state.tree_state,
                    &mut tree_index,
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
    ) 
    where 
        Renderer: iced::advanced::Renderer
    {
        let state: &mut State = tree.state.downcast_mut();

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Control) => {
                        state.ctrl_pressed = true;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        let visible_nodes = self.get_visible_node_ids(&state.tree_state);
                        if let Some(current_focused) = &state.tree_state.focused {
                            if let Some(current_index) = visible_nodes.iter().position(|id| id == current_focused) {
                                if current_index > 0 {
                                    let new_focused = visible_nodes[current_index - 1].clone();
                                    state.tree_state.focus(new_focused);
                                }
                                shell.request_redraw();
                            }
                        } else if !visible_nodes.is_empty() {
                            let first_node = visible_nodes[0].clone();
                            state.tree_state.focus(first_node);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        let visible_nodes = self.get_visible_node_ids(&state.tree_state);

                        if let Some(current_focused) = &state.tree_state.focused {
                            if let Some(current_index) = visible_nodes.iter().position(|id| id == current_focused) {
                                if current_index < visible_nodes.len() - 1 {
                                    let new_focused = visible_nodes[current_index + 1].clone();
                                    state.tree_state.focus(new_focused);
                                }
                                shell.request_redraw();
                            }
                        } else if !visible_nodes.is_empty() {
                            let first_node = visible_nodes[0].clone();
                            state.tree_state.focus(first_node);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if let Some(current_focused) = state.tree_state.focused.clone() {
                            // Find the focused node in the tree structure
                            let focused_node = self.find_node_by_id(&current_focused);
                            
                            if let Some(node) = focused_node {
                                if !node.children.is_empty() && state.tree_state.is_expanded(&current_focused) {
                                    // Collapse the node if it has children and is expanded
                                    state.tree_state.toggle(current_focused.clone());
                                    if let Some(on_toggle) = &self.on_toggle {
                                        shell.publish((on_toggle)(current_focused));
                                    }
                                    shell.invalidate_layout();
                                }
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if let Some(current_focused) = state.tree_state.focused.clone() {
                            // Find the focused node in the tree structure
                            let focused_node = self.find_node_by_id(&current_focused);
                            
                            if let Some(node) = focused_node {
                                if !node.children.is_empty() && !state.tree_state.is_expanded(&current_focused) {
                                    // Expand the node if it has children and is collapsed
                                    state.tree_state.toggle(current_focused.clone());
                                    if let Some(on_toggle) = &self.on_toggle {
                                        shell.publish((on_toggle)(current_focused));
                                    }
                                    shell.invalidate_layout();
                                }
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if let Some(current_focused) = state.tree_state.focused.clone() {
                            if modifiers.control() {
                                // Ctrl+Enter: Toggle selection (multi-select)
                                state.tree_state.toggle_select(current_focused.clone());
                            } else {
                                // Enter: Single select the focused node
                                state.tree_state.select(current_focused.clone());
                            }
                            if let Some(on_select) = &self.on_select {
                                shell.publish((on_select)(current_focused));
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Space) => {
                        if let Some(current_focused) = state.tree_state.focused.clone() {
                            if modifiers.control() {
                                // Ctrl+Space: Toggle selection (multi-select)
                                state.tree_state.toggle_select(current_focused.clone());
                            } else {
                                // Space: Single select the focused node
                                state.tree_state.select(current_focused.clone());
                            }
                            if let Some(on_select) = &self.on_select {
                                shell.publish((on_select)(current_focused));
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        // Escape: Clear all selections
                        state.tree_state.clear_selection();
                        if let Some(on_select) = &self.on_select {
                            shell.publish((on_select)("".to_string())); // Signal cleared selection
                        }
                    }
                    keyboard::Key::Character(smol_str) => {
                        if smol_str.as_str() == "a" && modifiers.control() {
                            // Ctrl+A: Select all visible nodes
                            let visible_nodes = self.get_visible_node_ids(&state.tree_state);
                            for node_id in visible_nodes {
                                state.tree_state.add_to_selection(node_id);
                            }
                            if let Some(on_select) = &self.on_select {
                                shell.publish((on_select)("select_all".to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Control) => {
                        state.ctrl_pressed = false;
                    }
                    _ => {}
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                println!("🌳 Tree widget got mouse press at {:?}", cursor.position());
                
                if let Some(cursor_position) = cursor.position() {
                    let mut child_layouts = layout.children();
                    
                    for (i, node) in self.nodes.iter().enumerate() {
                        if let Some(child_layout) = child_layouts.next() {
                            if let Some(content_layout) = child_layout.children().next() {
                                let content_bounds = content_layout.bounds();
                                println!("🎯 Node {} content bounds: {:?}", i, content_bounds);
                                
                                if content_bounds.contains(cursor_position) {
                                    println!("🎯 Click is INSIDE content bounds for node {}", i);
                                    return;
                                } else {
                                    println!("🎯 Click is OUTSIDE content bounds for node {}", i);
                                }
                            }
                        }
                    }
                }
            }

/*             Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {  // attempt #2, somewhat working?
                if let Some(cursor_position) = cursor.position() {
                    let mut child_layouts = layout.children();
                    
                    for node in &self.nodes {
                        if let Some(child_layout) = child_layouts.next() {
                            if let Some(clicked_node_id) = self.get_node_id_at_position(
                                node,
                                child_layout,
                                cursor_position,
                                0.0,
                                &state.tree_state,
                            ) {
                                if state.tree_state.is_selected(&clicked_node_id) {
                                    let selected_nodes: Vec<String> = state.tree_state.selected.iter().cloned().collect();
                                    state.tree_state.start_drag(selected_nodes, cursor_position);
                                } else {
                                    // Only handle tree interaction if click wasn't in content area
                                    self.handle_node_click(
                                        node,
                                        &mut state.tree_state,
                                        cursor_position,
                                        child_layout,
                                        0.0,
                                        state.ctrl_pressed,
                                        shell,
                                    );
                                    shell.invalidate_layout();
                                }
                                break;
                            }
                        }
                    }
                }
            } */

/*             Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position() {
                    let mut child_layouts = layout.children();
                    
                    for node in &self.nodes {
                        if let Some(child_layout) = child_layouts.next() {
                            if let Some(clicked_node_id) = self.get_node_id_at_position(
                                node,
                                child_layout,
                                cursor_position,
                                0.0,
                                &state.tree_state,
                            ) {
                                if state.tree_state.is_selected(&clicked_node_id) {
                                    let selected_nodes: Vec<String> = state.tree_state.selected.iter().cloned().collect();
                                    state.tree_state.start_drag(selected_nodes, cursor_position);
                                } else {
                                    if self.handle_node_click(
                                        node,
                                        &mut state.tree_state,
                                        cursor_position,
                                        child_layout,
                                        0.0,
                                        state.ctrl_pressed,
                                        shell,
                                    ) {
                                        shell.invalidate_layout();
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            } */

            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.tree_state.is_dragging() {
                    state.tree_state.update_drag_position(position.clone());
                    
                    if let Some((target_id, drop_pos)) = self.get_drop_target(layout, position.clone()) {
                        state.tree_state.set_drop_target(Some(target_id), drop_pos);
                    } else {
                        state.tree_state.set_drop_target(None, DropPosition::Into);
                    }
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(drag) = state.tree_state.end_drag() {
                    if let Some(target) = &drag.drop_target {
                        if !drag.dragged_nodes.contains(target) {
                            if let Some(cb) = &self.on_drop {
                                shell.publish(cb(
                                    drag.dragged_nodes.clone(),
                                    target.clone(),
                                    drag.drop_position.clone(),
                                ));
                            }
                        }
                    }
                    shell.invalidate_layout();
                }
            }
            _ => {}
        }
    }

/*     fn mouse_interaction(
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
    } */

   fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(cursor_position) = cursor.position() {
            // First check if we're over tree control areas
            if self.is_over_tree_control_area(layout, cursor_position) {
                return mouse::Interaction::Pointer;
            }
            
            // Then check if we're over content areas and delegate to content
            let mut child_layouts = layout.children();
            let mut tree_index = 0;
            
            for node in &self.nodes {
                if let Some(child_layout) = child_layouts.next() {
                    if let Some(content_layout) = child_layout.children().next() {
                        let content_bounds = content_layout.bounds();
                        
                        if content_bounds.contains(cursor_position) && tree_index < tree.children.len() {
                            // Adjust cursor for content
                            let adjusted_cursor = mouse::Cursor::Available(Point::new(
                                cursor_position.x - content_bounds.x,
                                cursor_position.y - content_bounds.y,
                            ));
                            
                            // Delegate to content widget
                            let content_element = node.content.view();
                            let content_tree = &tree.children[tree_index];
                            
                            return content_element.as_widget().mouse_interaction(
                                content_tree,
                                content_layout,
                                adjusted_cursor,
                                &Rectangle::new(Point::ORIGIN, content_bounds.size()),
                                renderer,
                            );
                        }
                    }
                    
                    tree_index += 1;
                    
                    // Skip children indices
                    if self.should_skip_children_for_mouse_interaction(&node, &tree_index) {
                        self.skip_tree_indices(node, &mut tree_index);
                    }
                }
            }
        }
        
        mouse::Interaction::Idle
    }
}

// Implementation methods for TreeWidget
impl<'a, Message, Theme, Renderer> TreeWidget<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn layout_node(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        tree_state: &TreeState,
        tree: &mut Tree,
        tree_index: &mut usize,
        renderer: &Renderer,
        limits: &Limits,
        indent_level: f32,
        y_offset: f32,
    ) -> (Node, f32) {
        let line_height = 32.0;
        let mut total_height = line_height + self.spacing;
        let mut children = Vec::new();

        let x_offset = indent_level * self.indent;
        let content_x = x_offset + 20.0;
        let content_width = limits.max().width - content_x;

        let content_element = node.content.view();
        let content_layout = if *tree_index < tree.children.len() {
            let content_tree = &mut tree.children[*tree_index];
            let content_limits = Limits::new(Size::ZERO, limits.max())
                .width(Length::Shrink)
                .height(line_height);
            
            content_element.as_widget().layout(content_tree, renderer, &content_limits)
        } else {
            Node::new(Size::new(content_width, line_height))
        };

        let positioned_content = content_layout.move_to(Point::new(content_x, 0.0));
        children.push(positioned_content);
        *tree_index += 1;

        if tree_state.is_expanded(node.content.id()) {
            let mut child_y_offset = line_height + self.spacing;
            
            for child in &node.children {
                let (child_layout, child_height) = self.layout_node(
                    child,
                    tree_state,
                    tree,
                    tree_index,
                    renderer,
                    limits,
                    indent_level + 1.0,
                    child_y_offset,
                );
                children.push(child_layout.move_to(Point::new(0.0, child_y_offset)));
                child_y_offset += child_height;
                total_height += child_height;
            }
        } else {
            self.skip_tree_indices(node, tree_index);
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
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        tree: &Tree,
        tree_state: &TreeState,
        tree_index: &mut usize,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        indent_level: f32,
    ) {
        let bounds = layout.bounds();
        let line_height = 32.0;
        let mut child_layouts = layout.children();

        if let Some(content_layout) = child_layouts.next() {
            let node_id = node.content.id();
            let x_offset = indent_level * self.indent;
            let is_selected = tree_state.is_selected(node_id);
            let is_focused = tree_state.is_focused(node_id);
            let is_expanded = tree_state.is_expanded(node_id);
            let has_children = !node.children.is_empty();
            let appearance = <Theme as Catalog>::style(theme, &self.class);

            // Draw selection background
            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + x_offset + 20.0, bounds.y),
                            Size::new(bounds.width - (x_offset + 20.0), line_height),
                        ),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    appearance.selection_background,
                );
                
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + x_offset + 18.0, bounds.y),
                            Size::new(2.0, line_height),
                        ),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    appearance.selection_border,
                );
            }

            // Draw focus indicator
            if is_focused && !is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + x_offset + 20.0, bounds.y),
                            Size::new(bounds.width - (x_offset + 20.0), line_height),
                        ),
                        border: Border {
                            radius: Radius::new(2.0),
                            color: appearance.focus_border,
                            width: 1.0,
                            ..Border::default()
                        },
                        shadow: iced::Shadow::default(),
                        snap: true,
                    },
                    Color::TRANSPARENT,
                );
            }

            // Draw drag & drop indicators
            if let Some(ref drag_state) = tree_state.drag_state {
                if drag_state.dragged_nodes.contains(&node_id.to_string()) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(bounds.x + x_offset + 20.0, bounds.y),
                                Size::new(bounds.width - (x_offset + 20.0), line_height),
                            ),
                            border: Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: true,
                        },
                        Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                    );
                }
                
                if Some(&node_id.to_string()) == drag_state.drop_target.as_ref() {
                    match drag_state.drop_position {
                        DropPosition::Before => {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        Point::new(bounds.x + x_offset + 20.0, bounds.y - 1.0),
                                        Size::new(bounds.width - (x_offset + 20.0), 2.0),
                                    ),
                                    border: Border::default(),
                                    shadow: iced::Shadow::default(),
                                    snap: true,
                                },
                                appearance.accept_drop_indicator_color,
                            );
                        }
                        DropPosition::After => {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        Point::new(bounds.x + x_offset + 20.0, bounds.y + line_height - 1.0),
                                        Size::new(bounds.width - (x_offset + 20.0), 2.0),
                                    ),
                                    border: Border::default(),
                                    shadow: iced::Shadow::default(),
                                    snap: true,
                                },
                                appearance.accept_drop_indicator_color,
                            );
                        }
                        DropPosition::Into => {
                            let indicator_color = if node.content.accepts_drops() {
                                appearance.accept_drop_indicator_color
                            } else {
                                appearance.deny_drop_indicator_color
                            };

                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        Point::new(bounds.x + x_offset + 20.0, bounds.y),
                                        Size::new(bounds.width - (x_offset + 20.0), line_height),
                                    ),
                                    border: Border {
                                        radius: Radius::new(2.0),
                                        width: 2.0,
                                        color: indicator_color,
                                        ..Border::default()
                                    },
                                    shadow: iced::Shadow::default(),
                                    snap: true,
                                },
                                Color::TRANSPARENT,
                            );
                        }
                    }
                }
            }

            // Draw expand/collapse arrow
            if has_children {
                let arrow_x = bounds.x + x_offset + 4.0;
                let arrow_text = if is_expanded { "▼" } else { "▶" };
                
                renderer.fill_text(
                    iced::advanced::Text {
                        content: arrow_text.to_string(),
                        bounds: Size::new(16.0, line_height),
                        size: iced::Pixels(12.0),
                        font: iced::Font::default(),
                        align_x: iced::advanced::text::Alignment::Left,
                        align_y: Vertical::Center,
                        line_height: iced::advanced::text::LineHeight::default(),
                        shaping: iced::advanced::text::Shaping::Advanced,
                        wrapping: iced::advanced::text::Wrapping::default(),
                    },
                    Point::new(arrow_x, bounds.y + line_height / 2.0),
                    appearance.arrow_color,
                    Rectangle::new(Point::new(arrow_x, bounds.y), Size::new(16.0, line_height)),
                );
            }

            // Draw content
            let content_element = node.content.view();
            if *tree_index < tree.children.len() {
                let content_tree = &tree.children[*tree_index];
                content_element.as_widget().draw(
                    content_tree,
                    renderer,
                    theme,
                    style,
                    content_layout,
                    cursor,
                    viewport,
                );
            }
            *tree_index += 1;

            // Draw children if expanded
            if is_expanded && !node.children.is_empty() {
                let line_x = bounds.x + x_offset + 8.0;
                
                let mut child_infos = Vec::new();
                for child in &node.children {
                    if let Some(child_layout) = child_layouts.next() {
                        let child_bounds = child_layout.bounds();
                        let child_center_y = child_bounds.y + line_height / 2.0;
                        child_infos.push((child, child_layout, child_center_y));
                    }
                }
                
                if let (Some((_, _, first_child_y)), Some((_, _, last_child_y))) = 
                    (child_infos.first(), child_infos.last()) {
                    
                    let line_start_y = bounds.y + line_height * 0.75;
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(line_x - 0.5, line_start_y),
                                Size::new(1.0, *last_child_y - line_start_y + 1.0),
                            ),
                            border: Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: true,
                        },
                        appearance.line_color,
                    );
                }
                
                for (child, child_layout, _) in child_infos {
                    self.draw_node(
                        child,
                        tree,
                        tree_state,
                        tree_index,
                        renderer,
                        theme,
                        style,
                        child_layout,
                        cursor,
                        viewport,
                        indent_level + 1.0,
                    );
                }
            } else {
                self.skip_tree_indices(node, tree_index);
            }
        }
    }

    fn skip_tree_indices(&self, node: &TreeNodeData<'a, Message, Theme, Renderer>, tree_index: &mut usize) {
        for child in &node.children {
            *tree_index += 1;
            self.skip_tree_indices(child, tree_index);
        }
    }

    fn collect_all_content_trees(&self, node: &TreeNodeData<'a, Message, Theme, Renderer>, trees: &mut Vec<Tree>) {
        let element = node.content.view();
        trees.push(Tree::new(&element));
        
        for child in &node.children {
            self.collect_all_content_trees(child, trees);
        }
    }

    fn collect_all_content_elements<'b>(&self, node: &'b TreeNodeData<'a, Message, Theme, Renderer>, elements: &mut Vec<Element<'a, Message, Theme, Renderer>>) {
        elements.push(node.content.view());
        for child in &node.children {
            self.collect_all_content_elements(child, elements);
        }
    }

    fn handle_node_click(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        tree_state: &mut TreeState,
        cursor_position: Point,
        layout: Layout<'_>,
        indent_level: f32,
        ctrl_pressed: bool,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            let x_offset = indent_level * self.indent;
            let arrow_x = bounds.x + x_offset + 4.0;
            let content_start_x = bounds.x + x_offset + 20.0;
            let node_id = node.content.id();
            
            // Check if click is on the arrow
            if !node.children.is_empty() 
                && cursor_position.x >= arrow_x 
                && cursor_position.x < arrow_x + 16.0 {
                
                tree_state.toggle(node_id.to_string());
                if let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(node_id.to_string()));
                }
                return true;
            }
            // Check if click is in the "tree control" area (before content)
            else if cursor_position.x >= bounds.x && cursor_position.x < content_start_x {
                tree_state.focus(node_id.to_string());
                
                if ctrl_pressed {
                    tree_state.toggle_select(node_id.to_string());
                } else {
                    tree_state.select(node_id.to_string());
                }
                
                if let Some(on_select) = &self.on_select {
                    shell.publish((on_select)(node_id.to_string()));
                }
                return true;
            }
            // If click is in content area, don't consume it - let content handle it
            else {
                return false;
            }
        }

        // Check children...
        if tree_state.is_expanded(node.content.id()) {
            let mut child_layouts = layout.children();
            child_layouts.next();
            
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if self.handle_node_click(
                        child,
                        tree_state,
                        cursor_position,
                        child_layout,
                        indent_level + 1.0,
                        ctrl_pressed,
                        shell,
                    ) {
                        return true;
                    }
                }
            }
        }

        false
    }

/*     fn handle_node_click(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        tree_state: &mut TreeState,
        cursor_position: Point,
        layout: Layout<'_>,
        indent_level: f32,
        ctrl_pressed: bool,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            let x_offset = indent_level * self.indent;
            let arrow_x = bounds.x + x_offset + 4.0;
            let node_id = node.content.id();
            
            if !node.children.is_empty() 
                && cursor_position.x >= arrow_x 
                && cursor_position.x < arrow_x + 16.0 {
                
                tree_state.toggle(node_id.to_string());
                if let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(node_id.to_string()));
                }
            } else {
                tree_state.focus(node_id.to_string());
                
                if ctrl_pressed {
                    tree_state.toggle_select(node_id.to_string());
                } else {
                    tree_state.select(node_id.to_string());
                }
                
                if let Some(on_select) = &self.on_select {
                    shell.publish((on_select)(node_id.to_string()));
                }
            }
            return true;
        }

        if tree_state.is_expanded(node.content.id()) {
            let mut child_layouts = layout.children();
            child_layouts.next();
            
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if self.handle_node_click(
                        child,
                        tree_state,
                        cursor_position,
                        child_layout,
                        indent_level + 1.0,
                        ctrl_pressed,
                        shell,
                    ) {
                        return true;
                    }
                }
            }
        }

        false
    } */

    fn skip_all_descendant_indices(&self, node: &TreeNodeData<'a, Message, Theme, Renderer>, tree_index: &mut usize) {
        *tree_index += 1; // For this node's content
        for child in &node.children {
            self.skip_all_descendant_indices(child, tree_index);
        }
    }

    fn should_skip_children_for_mouse_interaction(&self, node: &TreeNodeData<'a, Message, Theme, Renderer>, tree_index: &usize) -> bool {
        // For mouse interaction, we might want to check children even if collapsed
        // since we're just determining cursor appearance
        false
    }

    fn get_node_id_at_position(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
        tree_state: &TreeState,
    ) -> Option<String> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            return Some(node.content.id().to_string());
        }

        if tree_state.is_expanded(node.content.id()) {
            let mut child_layouts = layout.children();
            child_layouts.next();
            
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if let Some(child_id) = self.get_node_id_at_position(
                        child,
                        child_layout,
                        cursor_position,
                        indent_level + 1.0,
                        tree_state,
                    ) {
                        return Some(child_id);
                    }
                }
            }
        }
        
        None
    }

    fn get_drop_target(&self, layout: Layout<'_>, cursor_position: Point) -> Option<(String, DropPosition)> {
        let mut child_layouts = layout.children();
        
        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(result) = self.get_drop_target_recursive(node, child_layout, cursor_position, 0.0) {
                    return Some(result);
                }
            }
        }
        None
    }

    fn get_drop_target_recursive(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> Option<(String, DropPosition)> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            let relative_y = cursor_position.y - bounds.y;
            
            let drop_position = if relative_y < line_height * 0.25 {
                DropPosition::Before
            } else if relative_y > line_height * 0.75 && !node.children.is_empty() {
                DropPosition::Into
            } else if relative_y > line_height * 0.75 {
                DropPosition::After
            } else {
                DropPosition::Into
            };
            
            return Some((node.content.id().to_string(), drop_position));
        }

        let mut child_layouts = layout.children();
        child_layouts.next();
        
        for child in &node.children {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(result) = self.get_drop_target_recursive(
                    child,
                    child_layout,
                    cursor_position,
                    indent_level + 1.0,
                ) {
                    return Some(result);
                }
            }
        }
        
        None
    }

    fn get_visible_node_ids(&self, tree_state: &TreeState) -> Vec<String> {
        let mut visible_ids = Vec::new();
        for node in &self.nodes {
            self.collect_visible_node_ids(node, tree_state, &mut visible_ids);
        }
        visible_ids
    }

    fn collect_visible_node_ids(&self, node: &TreeNodeData<'a, Message, Theme, Renderer>, tree_state: &TreeState, visible_ids: &mut Vec<String>) {
        visible_ids.push(node.content.id().to_string());
        
        if tree_state.is_expanded(node.content.id()) {
            for child in &node.children {
                self.collect_visible_node_ids(child, tree_state, visible_ids);
            }
        }
    }

    fn get_hovered_node(&self, layout: Layout<'_>, cursor_position: Option<Point>) -> Option<&str> {
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
        node: &'b TreeNodeData<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> Option<&'b str> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            return Some(node.content.id());
        }

        let mut child_layouts = layout.children();
        child_layouts.next();
        
        for child in &node.children {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(hovered) = self.get_hovered_node_recursive(
                    child,
                    child_layout,
                    cursor_position,
                    indent_level + 1.0,
                ) {
                    return Some(hovered);
                }
            }
        }

        None
    }

    fn is_over_tree_control_area(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        let mut child_layouts = layout.children();
        
        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                if self.is_over_node_control_area(node, child_layout, cursor_position, 0.0) {
                    return true;
                }
            }
        }
        false
    }

    fn is_over_node_control_area(
        &self,
        node: &TreeNodeData<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> bool {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            let x_offset = indent_level * self.indent;
            let content_start_x = bounds.x + x_offset + 20.0;
            
            // Get actual content bounds
            let content_bounds = if let Some(content_layout) = layout.children().next() {
                content_layout.bounds()
            } else {
                Rectangle::new(Point::new(content_start_x, bounds.y), Size::new(0.0, 0.0))
            };
            
            // Check if cursor is in tree control area (before content or arrow area)
            let arrow_area = Rectangle::new(
                Point::new(bounds.x + x_offset + 4.0, bounds.y),
                Size::new(16.0, line_height)
            );
            
            let selection_area = Rectangle::new(
                Point::new(bounds.x, bounds.y),
                Size::new(content_bounds.x - bounds.x, line_height)
            );
            
            // Return true if over arrow or selection area, but NOT content area
            if arrow_area.contains(cursor_position) || 
            (selection_area.contains(cursor_position) && !content_bounds.contains(cursor_position)) {
                return true;
            }
        }
        
        // Check children if expanded
        let mut child_layouts = layout.children();
        child_layouts.next(); // Skip content layout
        
        for child in &node.children {
            if let Some(child_layout) = child_layouts.next() {
                if self.is_over_node_control_area(child, child_layout, cursor_position, indent_level + 1.0) {
                    return true;
                }
            }
        }
        
        false
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

impl<'a, Message, Theme, Renderer> From<TreeWidget<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'static,  // Changed from 'a to 'static
    Theme: Catalog + iced::widget::text::Catalog + 'static,  // Changed from 'a to 'static
    Renderer: iced::advanced::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
        + 'static,
{
    fn from(tree: TreeWidget<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tree)
    }
}

// Helper function to create a tree node content
pub fn tree_node<'a, Message, Theme, Renderer>(
    id: impl Into<String>,
    content: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
) -> StaticNodeContent<'a, Message, Theme, Renderer> {
    StaticNodeContent::new(id, content)
}