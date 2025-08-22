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

const ROW_HEIGHT: f32 = 32.0;       // its the row height, are you even reading the const name? smh
const ARROW_X_PAD: f32 = 4.0;       // where the arrow box starts, relative to indent
const ARROW_W: f32 = 16.0;          // arrow font size
const HANDLE_BASE_W: f32 = 8.0;     // collapsed handle width
const HANDLE_HOVER_W: f32 = 24.0;   // expanded handle width
const HANDLE_STRIPE_W: f32 = 2.0;   // thin base stripe (matches selection strip)
const CONTENT_GAP: f32 = 4.0;       // gap between arrow/handle block and content

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
    pub hovered: Option<String>,
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

//        println!("layout processed {} tree indices, created {} layouts", tree_index, children.len());

        Node::with_children(
            Size::new(limits.max().width, y_offset),
            children,
        )
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &renderer::Style, layout: Layout<'_>, cursor: mouse::Cursor, viewport: &Rectangle) {
        let state = tree.state.downcast_ref::<State>();
        let mut child_layouts = layout.children();
        let mut tree_index = 0;
        
//        println!("draw() has {} trees available", tree.children.len());
        
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
//    debugln!("TREE.update: event = {:?}", event);

    // Use the tree state to track which nodes are expanded
    let tree_state = {
        let state = tree.state.downcast_ref::<State>();
        state.tree_state.clone()
    };
    
    let mut rows = layout.children();
    let mut tree_index = 0;

    for i in 0..self.nodes.len() {
        if let Some(row_layout) = rows.next() {
            // Try to update this node
            if update_node_recursive(
                &mut self.nodes[i],
                &tree_state,
                tree,
                &mut tree_index,
                event,
                row_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
                0.0, // indent level
            ) {
                return; // Event was handled, stop processing
            }
        }
    }


/* 
        match event {
            // ---------------- Keyboard ----------------
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Control) => {
                        tree.state.downcast_mut::<State>().ctrl_pressed = true;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        let state = tree.state.downcast_mut::<State>();
                        let visible = self.get_visible_node_ids(&state.tree_state);
                        if let Some(f) = &state.tree_state.focused {
                            if let Some(i) = visible.iter().position(|id| id == f) {
                                if i > 0 {
                                    state.tree_state.focus(visible[i - 1].clone());
                                    shell.request_redraw();
                                }
                            }
                        } else if !visible.is_empty() {
                            state.tree_state.focus(visible[0].clone());
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        let state = tree.state.downcast_mut::<State>();
                        let visible = self.get_visible_node_ids(&state.tree_state);
                        if let Some(f) = &state.tree_state.focused {
                            if let Some(i) = visible.iter().position(|id| id == f) {
                                if i + 1 < visible.len() {
                                    state.tree_state.focus(visible[i + 1].clone());
                                    shell.request_redraw();
                                }
                            }
                        } else if !visible.is_empty() {
                            state.tree_state.focus(visible[0].clone());
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        let state = tree.state.downcast_mut::<State>();
                        if let Some(id) = state.tree_state.focused.clone() {
                            if let Some(node) = self.find_node_by_id(&id) {
                                if !node.children.is_empty() && state.tree_state.is_expanded(&id) {
                                    state.tree_state.toggle(id.clone());
                                    if let Some(cb) = &self.on_toggle { shell.publish(cb(id)); }
                                    shell.invalidate_layout();
                                }
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        let state = tree.state.downcast_mut::<State>();
                        if let Some(id) = state.tree_state.focused.clone() {
                            if let Some(node) = self.find_node_by_id(&id) {
                                if !node.children.is_empty() && !state.tree_state.is_expanded(&id) {
                                    state.tree_state.toggle(id.clone());
                                    if let Some(cb) = &self.on_toggle { shell.publish(cb(id)); }
                                    shell.invalidate_layout();
                                }
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter)
                    | keyboard::Key::Named(keyboard::key::Named::Space) => {
                        let state = tree.state.downcast_mut::<State>();
                        if let Some(id) = state.tree_state.focused.clone() {
                            if modifiers.control() {
                                state.tree_state.toggle_select(id.clone());
                            } else {
                                state.tree_state.select(id.clone());
                            }
                            if let Some(cb) = &self.on_select { shell.publish(cb(id)); }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        let state = tree.state.downcast_mut::<State>();
                        state.tree_state.clear_selection();
                        if let Some(cb) = &self.on_select { shell.publish(cb("".into())); }
                    }
                    keyboard::Key::Character(s) if s.as_str() == "a" && modifiers.control() => {
                        let state = tree.state.downcast_mut::<State>();
                        for id in self.get_visible_node_ids(&state.tree_state) {
                            state.tree_state.add_to_selection(id);
                        }
                        if let Some(cb) = &self.on_select { shell.publish(cb("select_all".into())); }
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                if let keyboard::Key::Named(keyboard::key::Named::Control) = key {
                    tree.state.downcast_mut::<State>().ctrl_pressed = false;
                }
            }

            // ---------------- Mouse Press ----------------
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor.position() {
                    if let Some((path, row)) = hit_row_path(&self.nodes, layout, p) {
                        // Resolve id + child info w/o long borrows
                        let (node_id, has_kids) = {
                            let mut node = &self.nodes[path[0]];
                            for &i in &path[1..] { node = &node.children[i]; }
                            (node.id.clone(), !node.children.is_empty())
                        };

                        debugln!("Clicked NodeID {}, Node has kids: {}", node_id, has_kids);

                        // Commit hover so handle/content rect widens immediately
                        {
                            let mut st = tree.state.downcast_mut::<State>();
                            if st.tree_state.hovered.as_deref() != Some(&node_id) {
                                st.tree_state.hovered = Some(node_id.clone());
                                shell.invalidate_layout();
                            }
                        }

                        // Row hit-test (Arrow / Handle / Content)
                        let row_bounds = row.bounds();
                        let hit = hit_test_row_static(
                            self.indent,
                            Some(&node_id),
                            &node_id,
                            row_bounds,
                            p,
                            path.len() as f32 - 1.0,
                            has_kids,
                        );

                        match hit {
                            HitRegion::Arrow => {
                                if has_kids {
                                    { tree.state.downcast_mut::<State>().tree_state.toggle(node_id.clone()); }
                                    if let Some(cb) = self.on_toggle.as_deref() { shell.publish(cb(node_id)); }
                                    shell.invalidate_layout();
                                }
                                return;
                            }

                            HitRegion::Handle => {
                                // start dragging this node (or current selection if it already includes it)
                                let (already, selected): (bool, Vec<String>) = {
                                    let st = tree.state.downcast_ref::<State>();
                                    (st.tree_state.is_selected(&node_id), st.tree_state.selected.iter().cloned().collect())
                                };
                                let mut st = tree.state.downcast_mut::<State>();
                                st.tree_state.start_drag(if already && !selected.is_empty() { selected } else { vec![node_id.clone()] }, p);
                                shell.request_redraw();
                                return;
                            }

                            HitRegion::Content => {
                                // Forward *press* into the child widget if inside the computed content rect
                                let mut sub = row.children();
                                if let Some(content_layout) = sub.next() {
                                    let content_rect = content_rect_for_row(
                                        self.indent,
                                        path.len() as f32 - 1.0,
                                        Some(&node_id),
                                        &node_id,
                                        row_bounds,
                                    );
                                    if content_rect.contains(p) {
                                        let flat = flattened_index_of_path::<Message, Theme, Renderer>(&self.nodes, &path);
                                        if let Some(content_tree) = tree.children.get_mut(flat) {
                                            let adjusted = mouse::Cursor::Available(Point::new(
                                                p.x - content_rect.x,
                                                p.y - content_rect.y,
                                            ));
                                            // Forward the *real* ButtonPressed to the button content
                                            let _ = {
                                                let mut node_mut = &mut self.nodes[path[0]];
                                                for &i in &path[1..] { node_mut = &mut node_mut.children[i]; }
                                                node_mut.content.as_widget_mut().update(
                                                    content_tree,
                                                    event,
                                                    content_layout,
                                                    adjusted,
                                                    renderer,
                                                    clipboard,
                                                    shell,
                                                    &Rectangle::new(Point::ORIGIN, content_rect.size()),
                                                )
                                            };
                                            return;
                                        }
                                    }
                                }
                            }

                            HitRegion::None => { /* fall through to your row-select logic if desired */ }
                        }
                    }
                }
            }

            // ---------------- CursorMoved ----------------
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position() {
                    debugln!("CursorMoved @ ({:.1}, {:.1})", cursor_position.x, cursor_position.y);

                    // If dragging → only update drag visuals / drop target
                    if tree.state.downcast_ref::<State>().tree_state.is_dragging() {
                        let mut st = tree.state.downcast_mut::<State>();
                        st.tree_state.update_drag_position(cursor_position);
                        if let Some((target_id, drop_pos)) = self.get_drop_target(layout, cursor_position) {
                            st.tree_state.set_drop_target(Some(target_id), drop_pos);
                        } else {
                            st.tree_state.set_drop_target(None, DropPosition::Into);
                        }
                        shell.request_redraw();
                        shell.invalidate_layout();
                        return;
                    }

                    let mut rows = layout.children();
                    let mut new_hover: Option<String> = None;
                    let mut tree_index = 0usize;

                    for i in 0..self.nodes.len() {
                        if let Some(row_layout) = rows.next() {
                            let row_bounds = row_layout.bounds();

                            // Hover only if inside row band
                            if cursor_position.y >= row_bounds.y && cursor_position.y < row_bounds.y + ROW_HEIGHT {
                                new_hover = Some(self.nodes[i].id.clone());

                                // Forward CursorMoved to content + return if inside content rect
                                let mut sub = row_layout.children();
                                if let (Some(content_layout), Some(content_tree)) =
                                    (sub.next(), tree.children.get_mut(tree_index))
                                {
                                    let hovered_id = new_hover.as_deref();
                                    debugln!("Hovered ID: {:?}", hovered_id);
                                    let content_rect = content_rect_for_row(
                                        self.indent, 0.0, hovered_id, &self.nodes[i].id, row_bounds,
                                    );

                                    if content_rect.contains(cursor_position) {
                                        let adjusted_cursor = mouse::Cursor::Available(Point::new(
                                            cursor_position.x - content_rect.x,
                                            cursor_position.y - content_rect.y,
                                        ));
                                        let _ = self.nodes[i].content.as_widget_mut().update(
                                            content_tree,
                                            event, // real CursorMoved
                                            content_layout,
                                            adjusted_cursor,
                                            renderer,
                                            clipboard,
                                            shell,
                                            &content_rect,
                                        );
                                    }
                                }
                            }

                            // descend only if expanded (so children can claim hover)
                            let expanded = tree.state.downcast_ref::<State>().tree_state.is_expanded(&self.nodes[i].id);
                            let my_idx = tree_index;
                            tree_index += 1;

                            if expanded {
                                cursor_moved_descend_static(
                                    self.indent,
                                    &mut new_hover,
                                    &mut self.nodes[i],
                                    tree,
                                    cursor_position,
                                    row_layout,
                                    1.0,
                                    my_idx,
                                    cursor,
                                    renderer,
                                    clipboard,
                                    shell,
                                    event,
                                );
                            }

                            skip_indices_static(&self.nodes[i], &mut tree_index);
                        }
                    }

/*                     debugln!(
                        "CursorMoved finished with tree_index = {}, total children = {}",
                        tree_index,
                        tree.children.len()
                    ); */

                    // commit hover (re-layout for handle width change)
                    let state = tree.state.downcast_mut::<State>();
                    if state.tree_state.hovered != new_hover {
                        state.tree_state.hovered = new_hover;
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                }
            }

            // ---------------- Mouse Release ----------------
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // DnD finishes first (keep your existing DnD block here if you have it)
                {
                    let st = tree.state.downcast_ref::<State>();
                    if st.tree_state.is_dragging() {
                        let mut st = tree.state.downcast_mut::<State>();
                        if let Some(drag) = st.tree_state.end_drag() {
                            if let Some(tgt) = &drag.drop_target {
                                if !drag.dragged_nodes.contains(tgt) {
                                    let ok = match drag.drop_position {
                                        DropPosition::Before | DropPosition::After => true,
                                        DropPosition::Into =>
                                            self.find_node_by_id(tgt).map_or(false, |n| n.accepts_drops),
                                    };
                                    if ok {
                                        if let Some(on_drop) = &self.on_drop {
                                            shell.publish((on_drop)(drag.dragged_nodes, tgt.clone(), drag.drop_position));
                                        }
                                    }
                                }
                            }
                            shell.request_redraw();
                        }
                        return;
                    }
                }

                if let Some(p) = cursor.position() {
                    if let Some((path, row)) = hit_row_path(&self.nodes, layout, p) {
                        let node_id = {
                            let mut node = &self.nodes[path[0]];
                            for &i in &path[1..] { node = &node.children[i]; }
                            node.id.clone()
                        };

                        let row_bounds = row.bounds();
                        let content_rect = content_rect_for_row(
                            self.indent,
                            path.len() as f32 - 1.0,
                            Some(&node_id),
                            &node_id,
                            row_bounds,
                        );

                        if content_rect.contains(p) {
                            let mut sub = row.children();
                            if let Some(content_layout) = sub.next() {
                                let flat = flattened_index_of_path::<Message, Theme, Renderer>(&self.nodes, &path);
                                if let Some(content_tree) = tree.children.get_mut(flat) {
                                    let adjusted = mouse::Cursor::Available(Point::new(
                                        p.x - content_rect.x,
                                        p.y - content_rect.y,
                                    ));
                                    let _ = {
                                        let mut node_mut = &mut self.nodes[path[0]];
                                        for &i in &path[1..] { node_mut = &mut node_mut.children[i]; }
                                        node_mut.content.as_widget_mut().update(
                                            content_tree,
                                            event, // the *real* ButtonReleased
                                            content_layout,
                                            adjusted,
                                            renderer,
                                            clipboard,
                                            shell,
                                            &Rectangle::new(Point::ORIGIN, content_rect.size()),
                                        )
                                    };
                                    return;
                                }
                            }
                        }
                    }
                } 
            }

            _ => {}
        }*/
    }

    fn mouse_interaction(
        &self, 
        tree: &Tree, 
        layout: Layout<'_>, 
        cursor: mouse::Cursor,
        viewport: &Rectangle, 
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let tree_state = {
            let state = tree.state.downcast_ref::<State>();
            state.tree_state.clone()
        };
        
        let mut rows = layout.children();
        let mut tree_index = 0;

        for i in 0..self.nodes.len() {
            if let Some(row_layout) = rows.next() {
                if let Some(interaction) = self.mouse_interaction_node_recursive(
                    &self.nodes[i],
                    &tree_state,
                    tree,
                    &mut tree_index,
                    row_layout,
                    cursor,
                    viewport,
                    renderer,
                    0.0, // indent level
                ) {
                    return interaction;
                }
            }
        }

        mouse::Interaction::default()


/*         let hovered_id = {
            let st = tree.state.downcast_ref::<State>();
            st.tree_state.hovered.clone()
        };

        if let Some(p) = cursor.position() {
            let mut rows = layout.children();
            let mut tree_index = 0usize;

            for node in &self.nodes {
                if let Some(row) = rows.next() {
                    let row_bounds = row.bounds();

                    // only hovered row forwards to content
                    if hovered_id.as_deref() == Some(&node.id) {
                        let content_rect = content_rect_for_row(
                            self.indent, 0.0, hovered_id.as_deref(), &node.id, row_bounds,
                        );
                        if content_rect.contains(p) {
                            let mut sub = row.children();
                            if let (Some(content_layout), Some(content_tree)) =
                                (sub.next(), tree.children.get(tree_index))
                            {
                                let cb = content_layout.bounds();
                                let adjusted = mouse::Cursor::Available(Point::new(
                                    p.x - cb.x,
                                    p.y - cb.y,
                                ));
                                return node.content.as_widget().mouse_interaction(
                                    content_tree,
                                    content_layout,
                                    adjusted,
                                    &Rectangle::new(Point::ORIGIN, cb.size()),
                                    renderer,
                                );
                            }
                        }
                    }

                    // descend if expanded
                    let my_idx = tree_index;
                    tree_index += 1;

                    let expanded = {
                        let st = tree.state.downcast_ref::<State>();
                        st.tree_state.is_expanded(&node.id)
                    };

                    if expanded {
                        if let Some(inter) = mouse_interaction_descend_static(
                            self.indent,
                            hovered_id.as_deref(),
                            node,
                            tree,
                            row,
                            1.0,
                            my_idx,
                            cursor,
                            viewport,
                            renderer,
                        ) { return inter; }
                    } else {
                        skip_indices_static(node, &mut tree_index);
                    }
                }
            }
        }
        mouse::Interaction::Pointer */
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitRegion { Handle, Arrow, Content, None }

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
        let line_height = ROW_HEIGHT;
        let handle_w = handle_width_for(&node.id, tree_state.hovered.as_deref());
        let (_, _, content_x) = row_x_offsets_for(self.indent, indent_level, handle_w);
        let mut total_height = line_height + self.spacing;
        let mut children = Vec::new();

        // Layout this node's content
        let content_layout = if *tree_index < tree.children.len() {
            let content_tree = &mut tree.children[*tree_index];
            let content_limits = Limits::new(Size::ZERO, limits.max())
                .width(Length::Shrink)
                .height(line_height);

            node.content.as_widget().layout(content_tree, renderer, &content_limits)
        } else {
            Node::new(Size::new(
                (limits.max().width - content_x).max(0.0),
                line_height,
            ))
        };

        children.push(content_layout.move_to(Point::new(content_x, 0.0)));
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
        let line_height = ROW_HEIGHT;
        let mut child_layouts = layout.children();

        // Get the content layout (should be the first child)
        if let Some(content_layout) = child_layouts.next() {
            let x_offset = indent_level * self.indent;
            let is_selected = tree_state.is_selected(&node.id);
            let is_focused = tree_state.is_focused(&node.id);
            let is_expanded = tree_state.is_expanded(&node.id);
            let has_children = !node.children.is_empty();
            let appearance = <Theme as Catalog>::style(theme, &self.class);

            let node_bounds = content_layout.bounds();
            let adjusted_cursor = cursor.position()
                .map(|local_point| mouse::Cursor::Available(Point::new(local_point.x - node_bounds.x, local_point.y - node_bounds.y)))
                .unwrap_or(mouse::Cursor::Unavailable);

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

            // Draw handle bar (expand when hovered)
            let handle_w = handle_width_for(&node.id, tree_state.hovered.as_deref());
            let (arrow_x, handle_x, _content_x) = row_x_offsets_for(self.indent, indent_level, handle_w);

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(bounds.x + handle_x, bounds.y),
                        Size::new(handle_w.max(HANDLE_STRIPE_W), line_height),
                    ),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                if tree_state.hovered.as_deref() == Some(&node.id) {
                    Color::from_rgba(appearance.focus_border.r, appearance.focus_border.g, appearance.focus_border.b, 0.25)
                } else {
                    Color::from_rgba(appearance.focus_border.r, appearance.focus_border.g, appearance.focus_border.b, 0.10)
                }
            );

            // Draw expand/collapse arrow (if any)
            if has_children {
                renderer.fill_text(
                    iced::advanced::Text {
                        content: if is_expanded { "▼".into() } else { "▶".into() },
                        bounds: Size::new(ARROW_W, line_height),
                        size: iced::Pixels(12.0),
                        font: iced::Font::default(),
                        align_x: iced::advanced::text::Alignment::Left,
                        align_y: Vertical::Center,
                        line_height: iced::advanced::text::LineHeight::default(),
                        shaping: iced::advanced::text::Shaping::Advanced,
                        wrapping: iced::advanced::text::Wrapping::default(),
                    },
                    Point::new(bounds.x + arrow_x, bounds.y + line_height / 2.0),
                    appearance.arrow_color,
                    Rectangle::new(Point::new(bounds.x + arrow_x, bounds.y), Size::new(ARROW_W, line_height)),
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
                    adjusted_cursor,
                    &node_bounds,
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
                        &child_layout.bounds(),
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

    fn hover_at_point<'l>(
        &self,
        layout: Layout<'l>,
        p: Point,
    ) -> Option<(usize /*root idx*/, Layout<'l>)> {
        let mut rows = layout.children();
        for i in 0..self.nodes.len() {
            if let Some(row) = rows.next() {
                let b = row.bounds();
                if p.y >= b.y && p.y < b.y + ROW_HEIGHT {
                    return Some((i, row));
                }
            }
        }
        None
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

    fn mouse_interaction_node_recursive(
        &self,
        node: &TreeNode<'a, Message, Theme, Renderer>,
        tree_state: &TreeState,
        tree: &Tree,
        tree_index: &mut usize,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
        indent_level: f32,
    ) -> Option<mouse::Interaction> {
        let mut child_layouts = layout.children();
        
        // Check this node's content
        if let Some(content_layout) = child_layouts.next() {
            let cb = content_layout.bounds();

            if cursor.is_over(cb) {
                if let Some(content_tree) = tree.children.get(*tree_index) {
                    let adjusted = cursor.position()
                        .map(|p| mouse::Cursor::Available(Point::new(p.x - cb.x, p.y - cb.y)))
                        .unwrap_or(mouse::Cursor::Unavailable);

                    println!("Cursor is over content bounds! Mouse_Interaction node {} at tree_index {}", node.id, *tree_index);
                    println!("Content bounds: {:?}", cb);
                    println!("Cursor available in mouse interaction {:?}", adjusted.position());

                    let interaction = node.content.as_widget().mouse_interaction(
                        content_tree,
                        content_layout,
                        adjusted,
                        &Rectangle::new(Point::ORIGIN, cb.size()),
                        renderer,
                    );
                    
                    if interaction != mouse::Interaction::default() {
                        return Some(interaction);
                    }
                }
            }
        }
        
        *tree_index += 1;

        // Check children if expanded
        if tree_state.is_expanded(&node.id) {
            for child in &node.children {
                if let Some(child_layout) = child_layouts.next() {
                    if let Some(interaction) = self.mouse_interaction_node_recursive(
                        child,
                        tree_state,
                        tree,
                        tree_index,
                        child_layout,
                        cursor,
                        viewport,
                        renderer,
                        indent_level + 1.0,
                    ) {
                        return Some(interaction);
                    }
                }
            }
        } else {
            // Skip tree indices for collapsed children
            self.skip_tree_indices_for_node(node, tree_index);
        }

        None
    }

    fn skip_tree_indices_for_node(&self, node: &TreeNode<'a, Message, Theme, Renderer>, tree_index: &mut usize) {
        for child in &node.children {
            *tree_index += 1;
            self.skip_tree_indices_for_node(child, tree_index);
        }
    }
}

fn content_rect_for_row(
    indent: f32,
    indent_level: f32,
    hovered_id: Option<&str>,
    node_id: &str,
    row_bounds: Rectangle,
) -> Rectangle {
    let handle_w = handle_width_for(node_id, hovered_id);
    let (_, _, content_x) = row_x_offsets_for(indent, indent_level, handle_w);
    Rectangle {
        x: row_bounds.x + content_x,
        y: row_bounds.y,
        width: (row_bounds.width - content_x).max(0.0),
        height: ROW_HEIGHT,
    }
}


fn handle_width_for(id: &str, hovered: Option<&str>) -> f32 {
    if hovered == Some(id) { HANDLE_HOVER_W } else { HANDLE_STRIPE_W }
}

fn row_x_offsets_for(indent: f32, indent_level: f32, handle_w: f32) -> (f32, f32, f32) {
    let x_offset = indent_level * indent;
    let arrow_x  = x_offset + ARROW_X_PAD;                               // arrow box origin
    let stripe_x = x_offset + ARROW_X_PAD + ARROW_W - HANDLE_STRIPE_W;   // thin stripe sits just right of arrow
    let handle_x = stripe_x;                                             // handle expands to the right from the thin stripe
    let content_x = stripe_x + HANDLE_STRIPE_W + handle_w + CONTENT_GAP; // push content to the right of expanded handle
    (arrow_x, handle_x, content_x)
}

fn hit_test_row_static(
    indent: f32,
    hovered: Option<&str>,
    node_id: &str,
    bounds: Rectangle,
    cursor: Point,
    indent_level: f32,
    has_children: bool,
) -> HitRegion {
    if cursor.y < bounds.y || cursor.y >= bounds.y + ROW_HEIGHT {
        return HitRegion::None;
    }

    let handle_w = handle_width_for(node_id, hovered);
    let (arrow_x, handle_x, content_x) = row_x_offsets_for(indent, indent_level, handle_w);

    let arrow_bounds = Rectangle::new(Point::new(bounds.x + arrow_x, bounds.y), Size::new(ARROW_W, ROW_HEIGHT));
    let handle_bounds = Rectangle::new(Point::new(bounds.x + handle_x, bounds.y), Size::new(handle_w.max(HANDLE_STRIPE_W), ROW_HEIGHT));
    let content_bounds = Rectangle::new(Point::new(bounds.x + content_x, bounds.y), Size::new(bounds.width - content_x, ROW_HEIGHT));

    // Always give priority to arrow clicks
    if has_children && arrow_bounds.contains(cursor) { return HitRegion::Arrow; }
    if handle_bounds.contains(cursor) { return HitRegion::Handle; }
    if content_bounds.contains(cursor) { return HitRegion::Content; }
    HitRegion::None
}

fn skip_indices_static<'a, Message, Theme, Renderer>(
    node: &TreeNode<'a, Message, Theme, Renderer>,
    idx: &mut usize,
) {
    for child in &node.children {
        *idx += 1;
        skip_indices_static(child, idx);
    }
}

fn button_press_descend_static<'a, Message, Theme, Renderer>(
    indent: f32,
    hovered: Option<&str>,
    node: &mut TreeNode<'a, Message, Theme, Renderer>,
    tree: &mut widget::tree::Tree,
    cursor_position: Point,
    row_layout: Layout<'_>,
    indent_level: f32,
    _ctrl: bool,
    start_index: usize,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    cursor: mouse::Cursor,
    on_select: Option<&(dyn Fn(String) -> Message)>,
    on_toggle: Option<&(dyn Fn(String) -> Message)>,
) -> bool
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    let mut child_layouts = row_layout.children();
    child_layouts.next(); // skip parent's content layout

    let mut idx = start_index + 1;

    for c in 0..node.children.len() {
        if let Some(child_row) = child_layouts.next() {
            let (child_id, child_has_kids) = {
                let child = &node.children[c];
                (child.id.clone(), !child.children.is_empty())
            };

            let bounds = child_row.bounds();

            // row-band hit test
            let hit = hit_test_row_static(
                indent,
                hovered,
                &child_id,
                bounds,
                cursor_position,
                indent_level,
                child_has_kids,
            );

            match hit {
                HitRegion::Handle => {
                    let list = {
                        let st = tree.state.downcast_ref::<State>();
                        if st.tree_state.is_selected(&child_id) {
                            st.tree_state.selected.iter().cloned().collect()
                        } else {
                            vec![child_id.clone()]
                        }
                    };
                    tree.state.downcast_mut::<State>().tree_state.start_drag(list, cursor_position);
                    shell.request_redraw();
                    return true;
                }
                HitRegion::Arrow => {
                    if child_has_kids {
                        { tree.state.downcast_mut::<State>().tree_state.toggle(child_id.clone()); }
                        if let Some(cb) = on_toggle { shell.publish(cb(child_id)); }
                        shell.invalidate_layout();
                        return true;
                    }
                }
                HitRegion::Content => {
                    if let Some(content_tree) = tree.children.get_mut(idx) {
                        let mut sub = child_row.children();
                        if let Some(content_layout) = sub.next() {
                            let cb = content_layout.bounds();
                            let adjusted = cursor.position()
                                .map(|p| mouse::Cursor::Available(Point::new(p.x - cb.x, p.y - cb.y)))
                                .unwrap_or(mouse::Cursor::Unavailable);
                            let child_mut = &mut node.children[c];
                            let _ = child_mut.content.as_widget_mut().update(
                                content_tree,
                                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                                content_layout,
                                adjusted,
                                renderer,
                                clipboard,
                                shell,
                                &Rectangle::new(Point::ORIGIN, cb.size()),
                            );
                        }
                    }
                    return true; // stop here — don't bubble or start DnD
                }
                HitRegion::None => {}
            }

            // recurse if expanded
            let is_expanded = {
                let st = tree.state.downcast_ref::<State>();
                st.tree_state.is_expanded(&child_id)
            };
            if is_expanded {
                let child_mut = &mut node.children[c];
                if button_press_descend_static(
                    indent,
                    hovered,
                    child_mut,
                    tree,
                    cursor_position,
                    child_row,
                    indent_level + 1.0,
                    _ctrl,
                    idx,
                    renderer,
                    clipboard,
                    shell,
                    cursor,
                    on_select,
                    on_toggle,
                ) { return true; }
            }

            idx += 1;
            skip_indices_static(&node.children[c], &mut idx);
        }
    }
    false
}

fn button_release_descend_static<'a, Message, Theme, Renderer>(
    node: &mut TreeNode<'a, Message, Theme, Renderer>,
    tree: &mut widget::tree::Tree,
    row_layout: Layout<'_>,
    start_index: usize,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    event: &Event, // ButtonReleased
) -> bool
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    let mut layouts = row_layout.children();
    layouts.next(); // skip parent content

    let mut idx = start_index + 1;

    if let Some(p) = cursor.position() {
        for c in 0..node.children.len() {
            if let Some(child_row) = layouts.next() {
                let b = child_row.bounds();

                if p.y >= b.y && p.y < b.y + ROW_HEIGHT {
                    let mut sub = child_row.children();
                    if let (Some(content_layout), Some(content_tree)) =
                        (sub.next(), tree.children.get_mut(idx))
                    {
                        let cb = content_layout.bounds();
                        let adjusted = mouse::Cursor::Available(Point::new(p.x - cb.x, p.y - cb.y));

                        let _ = node.children[c].content.as_widget_mut().update(
                            content_tree,
                            event, // real release
                            content_layout,
                            adjusted,
                            renderer,
                            clipboard,
                            shell,
                            &Rectangle::new(Point::ORIGIN, cb.size()),
                        );
                        return true;
                    }
                }

                // recurse if expanded
                let is_expanded = {
                    let st = tree.state.downcast_ref::<State>();
                    st.tree_state.is_expanded(&node.children[c].id)
                };

                let my_idx = idx;
                idx += 1;

                if is_expanded {
                    if button_release_descend_static(
                        &mut node.children[c],
                        tree,
                        child_row,
                        my_idx,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        event,
                    ) { return true; }
                } else {
                    skip_indices_static(&node.children[c], &mut idx);
                }
            }
        }
    }
    false
}


fn cursor_moved_descend_static<'a, Message, Theme, Renderer>(
    indent: f32,
    hovered: &mut Option<String>,
    node: &mut TreeNode<'a, Message, Theme, Renderer>,
    tree: &mut widget::tree::Tree,
    cursor_position: Point,
    row_layout: Layout<'_>,
    indent_level: f32,
    start_index: usize,
    _cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    event: &Event,
)
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    let mut child_layouts = row_layout.children();
    child_layouts.next(); // skip parent content

    let mut idx = start_index + 1;

    for c in 0..node.children.len() {
        if let Some(child_row) = child_layouts.next() {
            let row_bounds = child_row.bounds();
            let child_id = node.children[c].id.clone();

            // hover per row band
            if cursor_position.y >= row_bounds.y && cursor_position.y < row_bounds.y + ROW_HEIGHT {
                *hovered = Some(child_id.clone());

                // Only the hovered row’s content receives CursorMoved
                if let Some(content_tree) = tree.children.get_mut(idx) {
                    let mut sub = child_row.children();
                    if let Some(content_layout) = sub.next() {
                        let content_rect = content_rect_for_row(
                            indent,
                            indent_level,
                            hovered.as_deref(),
                            &child_id,
                            row_bounds,
                        );
                        if content_rect.contains(cursor_position) {
                            let adjusted = mouse::Cursor::Available(Point::new(
                                cursor_position.x - content_rect.x,
                                cursor_position.y - content_rect.y,
                            ));
                            let _ = node.children[c].content.as_widget_mut().update(
                                content_tree,
                                event,
                                content_layout,
                                adjusted,
                                renderer,
                                clipboard,
                                shell,
                                &Rectangle::new(Point::ORIGIN, content_rect.size()),
                            );
                        }
                    }
                }
            }

            let my_idx = idx;
            idx += 1;

            // Recurse only if expanded
            let is_expanded = {
                let st = tree.state.downcast_ref::<State>();
                st.tree_state.is_expanded(&child_id)
            };

            if is_expanded {
                cursor_moved_descend_static(
                    indent,
                    hovered,
                    &mut node.children[c],
                    tree,
                    cursor_position,
                    child_row,
                    indent_level + 1.0,
                    my_idx,
                    _cursor,
                    renderer,
                    clipboard,
                    shell,
                    event,
                );
            } else {
                skip_indices_static(&node.children[c], &mut idx);
            }
        }
    }
}

fn mouse_interaction_descend_static<'a, Message, Theme, Renderer>(
    indent: f32,
    hovered: Option<&str>,
    node: &TreeNode<'a, Message, Theme, Renderer>,
    tree: &widget::tree::Tree,
    row_layout: Layout<'_>,
    indent_level: f32,
    start_index: usize,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    renderer: &Renderer,
) -> Option<mouse::Interaction>
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    let mut child_layouts = row_layout.children();
    child_layouts.next(); // skip parent content

    let mut idx = start_index + 1;

    if let Some(p) = cursor.position() {
        for child in &node.children {
            if let Some(child_row) = child_layouts.next() {
                let bounds = child_row.bounds();

                let hit = hit_test_row_static(
                    indent,
                    hovered,
                    &child.id,
                    bounds,
                    p,
                    indent_level,
                    !child.children.is_empty(),
                );

                match hit {
                    HitRegion::Handle => return Some(mouse::Interaction::Grab),
                    HitRegion::Content => {
                        let mut sub = child_row.children();
                        if let (Some(content_layout), Some(content_tree)) =
                            (sub.next(), tree.children.get(idx))
                        {
                            let inter = child.content.as_widget().mouse_interaction(
                                content_tree,
                                content_layout,
                                cursor,
                                viewport,
                                renderer,
                            );
                            return Some(inter);
                        }
                    }
                    HitRegion::Arrow | HitRegion::None => {}
                }

                let this_child_idx = idx;
                idx += 1;

                let st = tree.state.downcast_ref::<State>();
                if st.tree_state.is_expanded(&child.id) {
                    if let Some(inter) = mouse_interaction_descend_static(
                        indent,
                        hovered,
                        child,
                        tree,
                        child_row,
                        indent_level + 1.0,
                        this_child_idx,
                        cursor,
                        viewport,
                        renderer,
                    ) {
                        return Some(inter);
                    }
                } else {
                    skip_indices_static(child, &mut idx);
                }
            }
        }
    }
    None
}

fn update_node_recursive<'a, Message, Theme, Renderer>(
    node: &mut TreeNode<'a, Message, Theme, Renderer>,
    tree_state: &TreeState,
    tree: &mut Tree,
    tree_index: &mut usize,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
    indent_level: f32,
) -> bool 
where
    Message: Clone + 'a,
    Theme: Catalog + iced::widget::text::Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    let mut child_layouts = layout.children();
    
    // First layout should be this node's content
    if let Some(content_layout) = child_layouts.next() {
        let cb = content_layout.bounds();
        
        // Only forward events to this node if the cursor is over it OR if it's a non-mouse event
        let should_forward = match event {
            Event::Mouse(_) => cursor.is_over(cb),
            _ => true, // Forward non-mouse events to all nodes
        };

        if should_forward {
            if let Some(content_tree) = tree.children.get_mut(*tree_index) {
                let adjusted = cursor.position()
                    .map(|p| mouse::Cursor::Available(Point::new(p.x - cb.x, p.y - cb.y)))
                    .unwrap_or(mouse::Cursor::Unavailable);

                println!("Forwarding event {:?} to node {} content (cursor over: {})", 
                    event, node.id, cursor.is_over(cb));

                node.content.as_widget_mut().update(
                    content_tree,
                    event,
                    content_layout,
                    adjusted,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle::new(Point::ORIGIN, cb.size()),
                );

                println!("After forwarding to node {} content", node.id);
                
                // If this was a mouse event and cursor is over this content, consider it handled
/*                 if let Event::Mouse(_) = event {
                    if cursor.is_over(cb) {
                        println!("Event handled by node {} content", node.id);
                        return true;
                    }
                } */
            }
        }
    }
    
    *tree_index += 1;

    // If this node is expanded, check its children
    if tree_state.is_expanded(&node.id) {
        for child in &mut node.children {
            if let Some(child_layout) = child_layouts.next() {
                if update_node_recursive(
                    child,
                    tree_state,
                    tree,
                    tree_index,
                    event,
                    child_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                    indent_level + 1.0,
                ) {
                    // return true; // Event was handled by child
                }
            }
        }
    } else {
        // Skip tree indices for collapsed children
        skip_tree_indices_for_node(node, tree_index);
    }

    false // Event not handled
}

fn skip_tree_indices_for_node<'a, Message, Theme, Renderer>(
    node: &TreeNode<'a, Message, Theme, Renderer>, 
    tree_index: &mut usize
) {
    for child in &node.children {
        *tree_index += 1;
        skip_tree_indices_for_node(child, tree_index);
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

fn subtree_len<'a, Message, Theme, Renderer>(node: &TreeNode<'a, Message, Theme, Renderer>) -> usize {
    1 + node.children.iter().map(|c| subtree_len::<Message, Theme, Renderer>(c)).sum::<usize>()
}

fn tree_index_for_row<'a, Message, Theme, Renderer>(
    root_idx: usize,
    roots: &[TreeNode<'a, Message, Theme, Renderer>],
) -> usize {
    let mut idx = 0;
    for r in 0..root_idx {
        idx += subtree_len::<Message, Theme, Renderer>(&roots[r]);
    }
    idx
}

// Map a node path (root→child→grandchild) into the flattened `tree.children` index
fn flattened_index_of_path<'a, Message, Theme, Renderer>(
    roots: &[TreeNode<'a, Message, Theme, Renderer>],
    path: &[usize],
) -> usize {
    let mut idx = 0;
    if path.is_empty() { return idx; }

    // previous root subtrees
    for r in 0..path[0] {
        idx += subtree_len::<Message, Theme, Renderer>(&roots[r]);
    }

    // walk down the path, adding previous siblings at each depth
    let mut node = &roots[path[0]];
    idx += 1; // include the root itself
    for depth in 1..path.len() {
        for s in 0..path[depth] {
            idx += subtree_len::<Message, Theme, Renderer>(&node.children[s]);
        }
        node = &node.children[path[depth]];
        idx += 1;
    }
    idx - 1 // we advanced one too far for the target's content slot
}

// Find the row (at any depth) under `p`, returning its *path* and that row's Layout
fn hit_row_path<'l, 'a, Message, Theme, Renderer>(
    nodes: &[TreeNode<'a, Message, Theme, Renderer>],
    layout: Layout<'l>,
    p: Point,
) -> Option<(Vec<usize>, Layout<'l>)> {
    fn go<'l, 'a, Message, Theme, Renderer>(
        nodes: &[TreeNode<'a, Message, Theme, Renderer>],
        rows: &mut dyn Iterator<Item = Layout<'l>>,
        p: Point,
        out_path: &mut Vec<usize>,
    ) -> Option<Layout<'l>> {
        for (i, node) in nodes.iter().enumerate() {
            if let Some(row) = rows.next() {
                let b = row.bounds();
                if p.y >= b.y && p.y < b.y + ROW_HEIGHT {
                    out_path.push(i);
                    return Some(row);
                }
                // descend into this node's children
                out_path.push(i);
                let mut child_rows = row.children();
                child_rows.next(); // skip content layout, the rest are child rows
                if let Some(hit) = go::<Message, Theme, Renderer>(&node.children, &mut child_rows, p, out_path) {
                    return Some(hit);
                }
                out_path.pop();
            }
        }
        None
    }

    let mut rows = layout.children();
    let mut path = Vec::new();
    go::<Message, Theme, Renderer>(nodes, &mut rows, p, &mut path).map(|row| (path, row))
}

// 2) Hit test which *root row* the point is over and return its Layout
fn hover_at_point<'l, 'a, Message, Theme, Renderer>(
    nodes: &[TreeNode<'a, Message, Theme, Renderer>],
    layout: Layout<'l>,
    p: Point,
) -> Option<(usize, Layout<'l>)> {
    let mut rows = layout.children();
    for (i, _node) in nodes.iter().enumerate() {
        if let Some(row) = rows.next() {
            let b = row.bounds();
            if p.y >= b.y && p.y < b.y + ROW_HEIGHT { return Some((i, row)); }
        }
    }
    None
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