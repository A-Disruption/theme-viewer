use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    }, alignment::Vertical, border::Radius, event, keyboard, mouse, touch, Border, Color, Element, Event, Length, Point, Rectangle, Size, Theme, Vector
};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// A tree widget that displays hierarchical data with expand/collapse functionality
#[allow(missing_debug_implementations)]
pub struct TreeWidget<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    nodes: Vec<TreeNode<'a, Message, Theme, Renderer>>,
    on_select: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_toggle: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_drop: Option<Box<dyn Fn(Vec<String>, String, DropPosition) -> Message + 'a>>,
    spacing: f32,
    indent: f32,
    class: Theme::Class<'a>,
    _renderer: PhantomData<Renderer>,   
}

/// Individual tree node
pub struct TreeNode<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    pub id: String,
    pub content: Element<'a, Message, Theme, Renderer>,
    pub children: Vec<TreeNode<'a, Message, Theme, Renderer>>,
    pub accepts_drops: bool,
}

/// State to track expanded nodes and selections
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    pub expanded: HashMap<String, bool>,
    pub selected: HashSet<String>,
    pub focused: Option<String>,
    pub drag_state: Option<DragState>,
}

/// A collection for managing tree data, similar to Vec or HashMap
#[allow(missing_debug_implementations)]
pub struct TreeData<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    nodes: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
}

/// Individual node in the tree data collection
#[allow(missing_debug_implementations)]
pub struct TreeNodeData<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    pub id: String,
    pub content: Element<'a, Message, Theme, Renderer>,
    pub children: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
    pub accepts_drops: bool,
}

impl<'a, Message, Theme, Renderer> TreeNodeData<'a, Message, Theme, Renderer> {
    pub fn new(
        id: impl Into<String>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            children: Vec::new(),
            accepts_drops: false,
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNodeData<'a, Message, Theme, Renderer>>) -> Self {
        self.children = children;
        self
    }

    pub fn accept_drops(mut self) -> Self {
        self.accepts_drops = true;
        self
    }
}

impl<'a, Message, Theme, Renderer> TreeData<'a, Message, Theme, Renderer> {
    /// Create a new empty tree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    /// Add a root node with widget content
    pub fn add_root(
        &mut self, 
        id: impl Into<String>, 
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> &mut TreeNodeData<'a, Message, Theme, Renderer> {
        let node = TreeNodeData::new(id, content);
        self.nodes.push(node);
        self.nodes.last_mut().unwrap()
    }

    /// Add a child node with widget content
    pub fn add_child<'b>(
        &'b mut self, 
        parent_id: &'b str, 
        id: impl Into<String>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>
    ) -> Option<&'b mut TreeNodeData<'a, Message, Theme, Renderer>> 
    where 
        'b: 'a
    {
        if let Some(parent) = Self::find_node_mut_helper(&mut self.nodes, parent_id) {
            let node = TreeNodeData::new(id, content);
            parent.children.push(node);
            parent.children.last_mut()
        } else {
            None
        }
    }

    /// Move a node to a new location
    pub fn move_node<'b>(&'b mut self, node_id: &'b str, target_id: &'b str, position: DropPosition) -> bool 
    where 
        'b: 'a
    {
        // 1. Remove the node from its current location
        if let Some(node) = Self::remove_node_by_id(&mut self.nodes, node_id) {
            // 2. Insert it at the new location
            match position {
                DropPosition::Into => {
                    if let Some(target) = Self::find_node_mut_helper(&mut self.nodes, target_id) {
                        target.children.push(node);
                        return true;
                    }
                }
                DropPosition::Before | DropPosition::After => {
                    if Self::insert_sibling(&mut self.nodes, target_id, node, position) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Remove a node by ID
    pub fn remove_node(&mut self, node_id: &str) -> Option<TreeNodeData<'a, Message, Theme, Renderer>> {
        Self::remove_node_by_id(&mut self.nodes, node_id)
    }

    /// Find a node by ID (immutable)
    pub fn find_node<'b>(&'b self, node_id: &'b str) -> Option<&'b TreeNodeData<'a, Message, Theme, Renderer>> 
    where 
        'b: 'a
    {
        Self::find_node_ref(&self.nodes, node_id)
    }

    /// Find a node by ID (mutable)
    pub fn find_node_mut<'b>(&'b mut self, node_id: &'b str) -> Option<&'b mut TreeNodeData<'a, Message, Theme, Renderer>> 
    where 
        'b: 'a
    {
        Self::find_node_mut_helper(&mut self.nodes, node_id)
    }

    /// Get all root nodes
    pub fn roots(&self) -> &[TreeNodeData<'a, Message, Theme, Renderer>] {
        &self.nodes
    }

    /// Get all root nodes mutably
    pub fn roots_mut(&mut self) -> &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>> {
        &mut self.nodes
    }

    // Helper methods - static to avoid borrowing issues
    fn find_node_ref(
        nodes: &'a [TreeNodeData<'a, Message, Theme, Renderer>], 
        id: &str
    ) -> Option<&'a TreeNodeData<'a, Message, Theme, Renderer>> {
        for node in nodes {
            if node.id == id {
                return Some(node);
            }
            if let Some(found) = Self::find_node_ref(&node.children, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_node_mut_helper(
        nodes: &'a mut [TreeNodeData<'a, Message, Theme, Renderer>], 
        id: &str
    ) -> Option<&'a mut TreeNodeData<'a, Message, Theme, Renderer>> {
        for node in nodes {
            if node.id == id {
                return Some(node);
            }
            if let Some(found) = Self::find_node_mut_helper(&mut node.children, id) {
                return Some(found);
            }
        }
        None
    }

    fn remove_node_by_id(
        nodes: &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>>, 
        id: &str
    ) -> Option<TreeNodeData<'a, Message, Theme, Renderer>> {
        // Check top-level nodes
        for i in 0..nodes.len() {
            if nodes[i].id == id {
                return Some(nodes.remove(i));
            }
        }

        // Check children recursively
        for node in nodes {
            if let Some(removed) = Self::remove_node_by_id(&mut node.children, id) {
                return Some(removed);
            }
        }

        None
    }

    fn insert_sibling(
        nodes: &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>>, 
        target_id: &str, 
        new_node: TreeNodeData<'a, Message, Theme, Renderer>, 
        position: DropPosition
    ) -> bool {
        println!("Inserting new node at target_id {}, in position {:?}", target_id, position);
        Self::insert_sibling_helper(nodes, target_id, Some(new_node), position).is_some()
    }

    fn insert_sibling_helper(
        nodes: &mut Vec<TreeNodeData<'a, Message, Theme, Renderer>>, 
        target_id: &str, 
        new_node: Option<TreeNodeData<'a, Message, Theme, Renderer>>, 
        position: DropPosition
    ) -> Option<TreeNodeData<'a, Message, Theme, Renderer>> {
        // Find target in current level
        for i in 0..nodes.len() {
            if nodes[i].id == target_id {
                let insert_index = match position {
                    DropPosition::Before => i,
                    DropPosition::After => i + 1,
                    _ => return new_node, // Return the node unused
                };
                if let Some(node) = new_node {
                    nodes.insert(insert_index, node);
                    return None; // Successfully inserted
                }
            }
        }

        // Search in children recursively
        let mut remaining_node = new_node;
        for node in nodes {
            remaining_node = Self::insert_sibling_helper(&mut node.children, target_id, remaining_node, position.clone());
            if remaining_node.is_none() {
                return None; // Successfully inserted
            }
        }

        remaining_node // Not found, return the unused node
    }
}

impl<'a, Message, Theme, Renderer> Default for TreeData<'a, Message, Theme, Renderer> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub dragged_nodes: Vec<String>, // What's being dragged
    pub drag_start_position: Point,
    pub current_position: Point,
    pub drop_target: Option<String>, // Where it would drop
    pub drop_position: DropPosition, // Before, after, or into
}

#[derive(Debug, Clone, PartialEq)]
pub enum DropPosition {
    Before,
    After, 
    Into, // As child
}

impl<'a, Message, Theme, Renderer> TreeNode<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    pub fn new(id: impl Into<String>, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            children: Vec::new(),
            accepts_drops: false,
        }
    }

    pub fn with_children(mut self, children: Vec<TreeNode<'a, Message, Theme, Renderer>>) -> Self {
        self.children = children;
        self
    }

    pub fn push_child(mut self, child: TreeNode<'a, Message, Theme, Renderer>) -> Self {
        self.children.push(child);
        self
    }

    pub fn accept_dropped_nodes(mut self) -> Self {
        self.accepts_drops = true;
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

    pub fn select(&mut self, id: String) {
        self.selected.clear(); // Clear existing selection
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

    pub fn get_focused(&self) -> Option<&String> {
        self.focused.as_ref()
    }

    pub fn start_drag(&mut self, nodes: Vec<String>, start_position: Point) {
        println!("Starting to drag.");
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
        println!("Drag has ended.");
        self.drag_state.take()
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn is_being_dragged(&self, id: &str) -> bool {
        self.drag_state.as_ref()
            .map_or(false, |drag| drag.dragged_nodes.contains(&String::from(id)))
    }
}

#[derive(Debug, Clone)]
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
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    pub fn new(nodes: Vec<TreeNode<'a, Message, Theme, Renderer>>) -> Self {
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

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn on_drop<F>(mut self, f: F) -> Self
    where
        F: Fn(Vec<String>, String, DropPosition) -> Message + 'a,
    {
        self.on_drop = Some(Box::new(f));
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

    fn children(&self) -> Vec<Tree> {
        let mut trees = Vec::new();
        for node in &self.nodes {
            self.collect_all_content_trees(node, &mut trees);
        }
        println!("children() creating {} trees for all content", trees.len());
        trees
    }

    fn diff(&self, tree: &mut Tree) {
        let mut expected_children = Vec::new();
        for node in &self.nodes {
            self.collect_all_content_elements(node, &mut expected_children);
        }
        tree.diff_children(&expected_children);
        println!("diff() called - tree now has {} children", tree.children.len());
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

        // Get state once and clone what we need to avoid borrowing conflicts
        let tree_state = {
            let state = tree.state.downcast_ref::<State>();
            state.tree_state.clone()
        };

        for node in &self.nodes {
            let (child_layout, height) = self.layout_node(
                node, 
                &tree_state,      // Use cloned state
                tree,             // Now we can borrow tree mutably
                &mut tree_index,
                renderer, 
                limits, 
                0.0, 
                y_offset
            );
            children.push(child_layout.move_to(Point::new(0.0, y_offset)));
            y_offset += height;
        }

        println!("layout processed {} tree indices, created {} layouts", tree_index, children.len());

        Node::with_children(
            Size::new(limits.max().width, y_offset),
            children,
        )
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        let state = tree.state.downcast_ref::<State>();
        let mut child_layouts = layout.children();
        let mut tree_index = 0;
        
        println!("draw() has {} trees available", tree.children.len());
        
        for node in &self.nodes {
            if let Some(child_layout) = child_layouts.next() {
                println!("Drawing root node: {} with tree_index: {}", node.id, tree_index);
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
    ) {
        let state = tree.state.downcast_mut::<State>();

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
                                    // Only focus, don't auto-select
                                }
                                shell.request_redraw();
                            }
                        } else if !visible_nodes.is_empty() {
                            // No focus yet, focus first visible node
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
                                    // Only focus, don't auto-select
                                }
                                shell.request_redraw();
                            }
                        } else if !visible_nodes.is_empty() {
                            // No focus yet, focus first visible node
                            let first_node = visible_nodes[0].clone();
                            state.tree_state.focus(first_node);
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if let Some(current_focused) = state.tree_state.focused.clone() {
                            // Find the focused node and collapse it if it has children and is expanded
                            if let Some(node) = self.find_node_by_id(&current_focused) {
                                if !node.children.is_empty() && state.tree_state.is_expanded(&current_focused) {
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
                            // Find the focused node and expand it if it has children and is collapsed
                            if let Some(node) = self.find_node_by_id(&current_focused) {
                                if !node.children.is_empty() && !state.tree_state.is_expanded(&current_focused) {
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
                if let Some(cursor_position) = cursor.position() {
                    let mut child_layouts = layout.children();
                    
                    for node in &self.nodes {
                        if let Some(child_layout) = child_layouts.next() {
                            if let Some(clicked_node_id) = self.get_node_id_at_position(
                                node, 
                                child_layout, 
                                cursor_position, 
                                0.0,
                                &state.tree_state // Pass the tree_state here
                            ) {
                                // Check if clicking on a selected node (potential drag start)
                                if state.tree_state.is_selected(&clicked_node_id) {
                                    let selected_nodes: Vec<String> = state.tree_state.selected.iter().cloned().collect();
                                    state.tree_state.start_drag(selected_nodes, cursor_position);
                                } else {
                                    // Regular click handling
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
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.tree_state.is_dragging() {
                    state.tree_state.update_drag_position(position.clone());
                    
                    // Determine drop target and position
                    if let Some((target_id, drop_pos)) = self.get_drop_target(layout, position.clone()) {
                        state.tree_state.set_drop_target(Some(target_id), drop_pos);
                    } else {
                        state.tree_state.set_drop_target(None, DropPosition::Into);
                    }
                    shell.request_redraw();
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(drag_state) = state.tree_state.end_drag() {
                    if let Some(drop_target) = &drag_state.drop_target {
                        // Don't allow dropping on self or children
                        if !drag_state.dragged_nodes.contains(drop_target) {
                            // Check if the drop is actually allowed
                            let drop_allowed = match drag_state.drop_position {
                                DropPosition::Before | DropPosition::After => {
                                    true // Sibling drops are always allowed
                                }
                                DropPosition::Into => {
                                    // Only allowed if target node accepts drops
                                    self.find_node_by_id(drop_target)
                                        .map_or(false, |node| node.accepts_drops)
                                }
                            };

                            if drop_allowed {
                                if let Some(on_drop) = &self.on_drop {
                                    shell.publish((on_drop)(
                                        drag_state.dragged_nodes,
                                        drop_target.clone(),
                                        drag_state.drop_position
                                    ));
                                }
                            }
                        }
                    }
                    shell.request_redraw();
                } //else { if !state.ctrl_pressed { state.tree_state.clear_selection(); }} // If you click a selection, and don't drag it, and you aren't hold ctrl, clear selection.
            }
            _ => {}
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
        node: &TreeNode<'a, Message, Theme, Renderer>,
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

        // Calculate content positioning
        let x_offset = indent_level * self.indent;
        let content_x = x_offset + 20.0;
        let content_width = limits.max().width - content_x;

        // Layout this node's content
        let content_layout = if *tree_index < tree.children.len() {
            let content_tree = &mut tree.children[*tree_index];
            let content_limits = Limits::new(Size::ZERO, limits.max())
                .width(Length::Shrink)
                .height(line_height);
            
            node.content.as_widget().layout(content_tree, renderer, &content_limits)
        } else {
            Node::new(Size::new(content_width, line_height))
        };

        let positioned_content = content_layout.move_to(Point::new(content_x, 0.0));
        children.push(positioned_content);
        *tree_index += 1;

        // Add children if expanded
        if tree_state.is_expanded(&node.id) {
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
            // Skip tree indices for collapsed children
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
        node: &TreeNode<'a, Message, Theme, Renderer>,
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

        // Get the content layout (should be the first child)
        if let Some(content_layout) = child_layouts.next() {
            let node_bounds = content_layout.bounds();
            let x_offset = indent_level * self.indent;
            let is_selected = tree_state.is_selected(&node.id);
            let is_focused = tree_state.is_focused(&node.id);
            let is_expanded = tree_state.is_expanded(&node.id);
            let has_children = !node.children.is_empty();
            let appearance = <Theme as Catalog>::style(theme, &self.class);

            // Draw selection background with subtle styling
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
                
                // Draw left border for selected item
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

            // Draw focus indicator (subtle outline)
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

            // Draw drag & drop visual feedback
            if let Some(ref drag_state) = tree_state.drag_state {
                // Draw dragged nodes with reduced opacity
                if drag_state.dragged_nodes.contains(&node.id) {
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
                        Color::from_rgba(0.5, 0.5, 0.5, 0.3), // Semi-transparent overlay
                    );
                }
                
                if Some(&node.id) == drag_state.drop_target.as_ref() {
                    match drag_state.drop_position {
                        DropPosition::Before => {
                            // Draw line above node - always allowed (sibling placement)
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
                            // Draw line below node - always allowed (sibling placement)
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
                            // Only check accepts_drops for "Into" - this makes nodes children
                            let indicator_color = if node.accepts_drops {
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
                let is_expanded = tree_state.is_expanded(&node.id);
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

            // Draw this nodes Element content
            if *tree_index < tree.children.len() {
                let content_tree = &tree.children[*tree_index];
                node.content.as_widget().draw(
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

            // Draw children and connecting lines if expanded
            if is_expanded && !node.children.is_empty() {
                let line_x = bounds.x + x_offset + 8.0; // Shifted left from arrow center
                
                // Collect child layouts and their positions
                let mut child_infos = Vec::new();
                for child in &node.children {
                    if let Some(child_layout) = child_layouts.next() {
                        let child_bounds = child_layout.bounds();
                        let child_center_y = child_bounds.y + line_height / 2.0;
                        child_infos.push((child, child_layout, child_center_y));
                    }
                }
                
                // Draw the main vertical line from below parent to last child
                if let (Some((_, _, first_child_y)), Some((_, _, last_child_y))) = 
                    (child_infos.first(), child_infos.last()) {
                    
                    // Start the line a bit below the parent arrow to avoid touching it
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
                
                // Just render the children without horizontal lines
                for (child, child_layout, _) in child_infos {
                    // Draw the child node
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
                // Skip tree indices for collapsed children
                self.skip_tree_indices(node, tree_index);
            }
        }
    }

    fn skip_tree_indices(&self, node: &TreeNode<'a, Message, Theme, Renderer>, tree_index: &mut usize) {
        for child in &node.children {
            *tree_index += 1;
            self.skip_tree_indices(child, tree_index);
        }
    }

    // Collect trees for ALL nodes to maintain consistent indexing
    fn collect_all_content_trees(&self, node: &TreeNode<'a, Message, Theme, Renderer>, trees: &mut Vec<Tree>) {
        // Add tree for this node's content
        trees.push(Tree::new(&node.content));
        
        // Recursively add trees for ALL children's content (regardless of expansion state)
        for child in &node.children {
            self.collect_all_content_trees(child, trees);
        }
    }

    fn collect_all_content_elements<'b>(&self, node: &'b TreeNode<'a, Message, Theme, Renderer>, elements: &mut Vec<&'b Element<'a, Message, Theme, Renderer>>) {
        elements.push(&node.content);
        for child in &node.children {
            self.collect_all_content_elements(child, elements);
        }
    }

    fn handle_node_click(
        &self,
        node: &TreeNode<'a, Message, Theme, Renderer>,
        tree_state: &mut TreeState,
        cursor_position: Point,
        layout: Layout<'_>,
        indent_level: f32,
        ctrl_pressed: bool,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
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
                // Click on node text - handle multi-select
                tree_state.focus(node.id.clone());
                
                if ctrl_pressed {
                    // Ctrl+click: Toggle selection (multi-select)
                    tree_state.toggle_select(node.id.clone());
                } else {
                    // Regular click: Clear selection, Single select
                    //tree_state.clear_selection();
                    tree_state.select(node.id.clone());
                }
                
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

    fn get_node_id_at_position(
        &self,
        node: &TreeNode<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
        tree_state: &TreeState, // Add tree_state parameter
    ) -> Option<String> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        // Check if cursor is over this node
        if cursor_position.y >= bounds.y && cursor_position.y < bounds.y + line_height {
            return Some(node.id.clone());
        }

        // Check children if expanded
        if tree_state.is_expanded(&node.id) {
            let mut child_layouts = layout.children();
            child_layouts.next(); // Skip node layout
            
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if let Some(child_id) = self.get_node_id_at_position(
                        child, 
                        child_layout, 
                        cursor_position, 
                        indent_level + 1.0,
                        tree_state, // Pass tree_state along
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
        node: &TreeNode<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> Option<(String, DropPosition)> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
        // Check if cursor is over this node
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
            
            // Always return the target - let the drawing code decide if it's valid
            return Some((node.id.clone(), drop_position));
        }

        // Check children
        let mut child_layouts = layout.children();
        child_layouts.next(); // Skip node layout
        
        for child in &node.children {
            if let Some(child_layout) = child_layouts.next() {
                if let Some(result) = self.get_drop_target_recursive(
                    child, 
                    child_layout, 
                    cursor_position, 
                    indent_level + 1.0
                ) {
                    return Some(result);
                }
            }
        }
        
        None
    }

    fn calculate_expanded_height(&self, node: &TreeNode<'a, Message, Theme, Renderer>, tree_state: &TreeState) -> f32 {
        let line_height = 32.0;
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

    fn get_hovered_node(&self, layout: Layout<'_>, cursor_position: Option<Point>) -> Option<&TreeNode<'a, Message, Theme, Renderer>,> {
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

    /// Get all visible node IDs in order (for keyboard navigation)
    fn get_visible_node_ids(&self, tree_state: &TreeState) -> Vec<String> {
        let mut visible_ids = Vec::new();
        for node in &self.nodes {
            self.collect_visible_node_ids(node, tree_state, &mut visible_ids);
        }
        visible_ids
    }

    fn collect_visible_node_ids(&self, node: &TreeNode<'a, Message, Theme, Renderer>, tree_state: &TreeState, visible_ids: &mut Vec<String>) {
        visible_ids.push(node.id.clone());
        
        // Only add children if this node is expanded
        if tree_state.is_expanded(&node.id) {
            for child in &node.children {
                self.collect_visible_node_ids(child, tree_state, visible_ids);
            }
        }
    }

    /// Find a node by its ID in the tree
    fn find_node_by_id(&self, id: &str) -> Option<&TreeNode<'a, Message, Theme, Renderer>> {
        for node in &self.nodes {
            if let Some(found) = self.find_node_by_id_recursive(node, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_node_by_id_recursive<'b>(&self, node: &'b TreeNode<'a, Message, Theme, Renderer>, id: &str) -> Option<&'b TreeNode<'a, Message, Theme, Renderer>> {
        if node.id == id {
            return Some(node);
        }
        
        for child in &node.children {
            if let Some(found) = self.find_node_by_id_recursive(child, id) {
                return Some(found);
            }
        }
        
        None
    }

    fn get_hovered_node_recursive<'b>(
        &self,
        node: &'b TreeNode<'a, Message, Theme, Renderer>,
        layout: Layout<'_>,
        cursor_position: Point,
        indent_level: f32,
    ) -> Option<&'b TreeNode<'a, Message, Theme, Renderer>> {
        let bounds = layout.bounds();
        let line_height = 32.0;
        
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

/// Helper function to create the tree widget
pub fn tree<'a, Message, Theme, Renderer>(
    nodes: Vec<TreeNode<'a, Message, Theme, Renderer>>,
) -> TreeWidget<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    TreeWidget::new(nodes)
}

pub fn tree_data<'a, Message, Theme, Renderer>(
    mut tree_data: TreeData<'a, Message, Theme, Renderer>, // Take ownership
) -> TreeWidget<'a, Message, Theme, Renderer>
where 
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Theme: Catalog + 'a,
{
    let nodes = build_tree_nodes_from_owned_data(tree_data.nodes);
    TreeWidget::new(nodes)
}

fn build_tree_nodes_from_owned_data<'a, Message, Theme, Renderer>(
    data_nodes: Vec<TreeNodeData<'a, Message, Theme, Renderer>>,
) -> Vec<TreeNode<'a, Message, Theme, Renderer>>
where 
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Theme: Catalog + 'a,
{
    data_nodes
        .into_iter() // Take ownership
        .map(|data_node| {
            TreeNode::new(data_node.id, data_node.content) // Move the content
                .accept_dropped_nodes()
                .with_children(build_tree_nodes_from_owned_data(data_node.children))
        })
        .collect()
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