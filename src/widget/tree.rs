use iced::{
    advanced::{
        layout,
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    }, border::Radius, keyboard, mouse, widget::text::Alignment, window::drag, Border, Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Theme, Transformation, Vector
};
use std::collections::HashSet;


/// Creates a new [`TreeHandle`] with the given root branches.
///
/// Branches can be created using the [`branch()`] function and nested
/// using the `.with_children()` method.
pub fn tree_handle<'a, Message, Theme, Renderer>(
    roots: impl IntoIterator<Item = Branch<'a, Message, Theme, Renderer>>,
) -> TreeHandle<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    TreeHandle::new(roots)
}

/// Creates a new [`Branch`] with the given content element.
///
/// The branch can have children added via `.with_children()`.
pub fn branch<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Branch<'a, Message, Theme, Renderer>
{
    Branch {
        content: content.into(),
        children: Vec::new(),
        align_x: iced::Alignment::Start,
        align_y: iced::Alignment::Center,
        accepts_drops: false,
    }
}

// Default Settings
const LINE_HEIGHT: f32 = 32.0;       // its the row height, are you even reading the const name? smh
const ARROW_X_PAD: f32 = 4.0;       // where the arrow box starts, relative to indent
const ARROW_W: f32 = 16.0;          // arrow font size
const HANDLE_HOVER_W: f32 = 24.0;   // expanded handle width
const HANDLE_STRIPE_W: f32 = 2.0;   // thin base stripe (matches selection strip)
const CONTENT_GAP: f32 = 4.0;       // gap between arrow/handle block and content
const DROP_INDICATOR_HEIGHT: f32 = 3.0; // Height of drop indicator line

#[derive(Debug, Clone)]
pub struct DropInfo{
    pub dragged_ids: Vec<usize>,
    pub target_id: Option<usize>,
    pub position: DropPosition,
}

#[allow(missing_debug_implementations)]
pub struct TreeHandle<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> 
where 
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    branches: Vec<Branch_>,
    branch_content: Vec<Element<'a, Message, Theme, Renderer>>, 
    width: Length, 
    height: Length,
    spacing: f32, 
    indent: f32, 
    padding_x: f32,
    padding_y: f32,
    on_drop: Option<Box<dyn Fn(DropInfo) -> Message + 'a>>,
    class: Theme::Class<'a>,
}

#[derive(Clone, Debug)]
struct Branch_ {
    id: usize,  // Auto-generated unique ID
    parent_id: Option<usize>,  // Track parent for hierarchy
    depth: u16,
    has_children: bool,
    accepts_drops: bool,
    align_x: iced::Alignment,
    align_y: iced::Alignment,
}


// Add metrics to store layout information
struct Metrics {
    branch_heights: Vec<f32>,
    branch_widths: Vec<f32>,
    expanded: HashSet<usize>,
    visible_branches: Vec<bool>,
}

// State for interaction
#[derive(Default)]
struct State {
    selected: HashSet<usize>,  // Multiple selection
    focused: Option<usize>,     // Keyboard focus
    hovered: Option<usize>,     // Mouse hover
    hovered_handle: Option<usize>, // Hovering over drag handle
    drag_state: Option<DragState>,
    branch_order: Option<Vec<usize>>
}

impl<'a, Message, Theme, Renderer> 
    TreeHandle<'a, Message, Theme, Renderer>
where 
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer  + iced::advanced::text::Renderer,
{

    /// Creates a new [`TreeHandle`] from root branches.
    pub fn new<'b>( 
        roots: impl IntoIterator<Item = Branch<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let roots = roots.into_iter();

        let mut width = Length::Fill;
        let mut height = Length::Shrink;

        let mut branches = Vec::new();
        let mut branch_content = Vec::new();
        let mut next_id = 0usize;

        // Flatten the tree structure into arrays
        fn flatten_branch<'a, Message, Theme, Renderer>(
            branch: Branch<'a, Message, Theme, Renderer>,
            parent_id: Option<usize>,
            depth: u16,
            next_id: &mut usize,
            branches: &mut Vec<Branch_>,
            branch_content: &mut Vec<Element<'a, Message, Theme, Renderer>>,
            width: &mut Length,
            height: &mut Length,
        ) where
            Renderer: iced::advanced::Renderer  + iced::advanced::text::Renderer,
        {
            let current_id = *next_id;
            *next_id += 1;
            
            let has_children = !branch.children.is_empty();
            
            // Add the branch metadata
            branches.push(Branch_ {
                id: current_id,
                parent_id,
                depth,
                has_children,
                accepts_drops: branch.accepts_drops,
                align_x: branch.align_x,
                align_y: branch.align_y,
            });
            
            // Add the content and update size hints
            let size_hint = branch.content.as_widget().size_hint();
            *width = width.enclose(size_hint.width);
            *height = height.enclose(size_hint.height);
            branch_content.push(branch.content);
            
            // Recursively add children
            for child in branch.children {
                flatten_branch(
                    child,
                    Some(current_id),
                    depth + 1,
                    next_id,
                    branches,
                    branch_content,
                    width,
                    height,
                );
            }
        }

        // Process all root branches
        for root in roots {
            flatten_branch(
                root,
                None,
                0,
                &mut next_id,
                &mut branches,
                &mut branch_content,
                &mut width,
                &mut height,
            );
        }

        Self {
            branches,
            branch_content,
            width,
            height,
            spacing: 4.0,
            indent: 20.0,
            padding_x: 10.0,
            padding_y: 5.0,
            on_drop: None,
            class: Theme::default(),
        }
    }

    /// Sets the message to emit when a drop occurs
    pub fn on_drop<F>(mut self, f: F) -> Self 
    where
        F: Fn(DropInfo) -> Message + 'a,
    {
        self.on_drop = Some(Box::new(f));
        self
    }

    /// Calculate drop position based on mouse position within a branch
    fn calculate_drop_position(&self, mouse_y: f32, branch_bounds: Rectangle, has_children: bool, expanded: bool) -> DropPosition {
        let relative_y = mouse_y - branch_bounds.y;
        let third_height = branch_bounds.height / 3.0;
        
        if relative_y < third_height {
            DropPosition::Before
        } else if relative_y > branch_bounds.height - third_height {
            DropPosition::After
        } else if has_children && expanded {
            DropPosition::Into
        } else {
            // If no children or collapsed, prefer Before/After
            if relative_y < branch_bounds.height / 2.0 {
                DropPosition::Before
            } else {
                DropPosition::After
            }
        }
    }

    /// Sets the width of the [`Tree`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tree`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the indent of the [`Tree`].
    pub fn indent(mut self, px: f32) -> Self { 
        self.indent = px; self 
    }

    /// Sets the spacing of the [`Tree`].
    pub fn spacing(mut self, px: f32) -> Self { 
        self.spacing = px; self 
    }

    /// Sets the class of the [`Tree`].
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self { 
        self.class = class.into(); self 
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

    /// Helper function to determine if a branch is visible based on parent expansion
    fn is_branch_visible(&self, index: usize, metrics: &Metrics) -> bool {
        if index >= self.branches.len() {
            return false;
        }

        let branch = &self.branches[index];
        
        // Root level items are always visible
        if branch.parent_id.is_none() {
            return true;
        }
        
        // Check if parent is expanded
        if let Some(parent_id) = branch.parent_id {
            // Find parent branch by ID
            if let Some(parent_index) = self.branches.iter().position(|b| b.id == parent_id) {
                // Parent must be visible and expanded
                return metrics.visible_branches[parent_index] && metrics.expanded.contains(&parent_id);
            }
        }
        
        false
    }

    fn get_ordered_indices(&self, state: &State) -> Vec<usize> {
        if let Some(ref order) = state.branch_order {
            let mut indices = Vec::new();
            
            // First add branches in the stored order
            for &id in order {
                if let Some(idx) = self.branches.iter().position(|b| b.id == id) {
                    indices.push(idx);
                }
            }
            
            // Then add any new branches not in the stored order
            for (i, branch) in self.branches.iter().enumerate() {
                if !order.contains(&branch.id) {
                    indices.push(i);
                }
            }
            
            indices
        } else {
            // No stored order yet - use default order
            (0..self.branches.len()).collect()
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TreeHandle<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<(Metrics, State)>()
    }

    fn state(&self) -> widget::tree::State {
        let mut expanded = HashSet::new();
        
        // By default, expand all branches with children
        for branch in &self.branches {
            if branch.has_children {
                expanded.insert(branch.id.clone());
            }
        }
        
        widget::tree::State::new((
            Metrics {
            branch_heights: Vec::new(),
            branch_widths: Vec::new(),
            expanded,
            visible_branches: Vec::new(),
            },
            State::default()
        ))
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.branch_content
            .iter()
            .map(|branch| widget::Tree::new(branch.as_widget()))
            .collect()
    }

    fn diff(&self, state: &mut widget::Tree) {
        state.diff_children(&self.branch_content);
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let (metrics, state) = tree.state.downcast_mut::<(Metrics, State)>();

        // If this is the first layout and we don't have an order yet, save the default order
        if state.branch_order.is_none() {
            state.branch_order = Some(self.branches.iter().map(|b| b.id).collect());
        }

        // Get the ordering to use
        let ordered_indices = self.get_ordered_indices(state);
        let branch_count = self.branches.len();
        
        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        let tree_fluid = self.width.fluid();
        
        // Update visibility based on expansion state
        metrics.visible_branches = vec![false; branch_count];
        for i in 0..branch_count {
            metrics.visible_branches[i] = self.is_branch_visible(i, metrics);
        }
        
        let mut cells = Vec::with_capacity(branch_count);
        cells.resize(branch_count, layout::Node::default());
        
        metrics.branch_heights = vec![0.0; branch_count];
        metrics.branch_widths = vec![0.0; branch_count];

        // Check if we need to add space for drop indicator
        let drop_gap = if state.drag_state.is_some() {
            DROP_INDICATOR_HEIGHT + self.spacing
        } else {
            0.0
        };
        
        // FIRST PASS - Layout non-fluid visible branches
        let mut y = self.padding_y;
        let mut max_content_width = 0.0f32;
        let mut row_fill_factors = vec![0u16; branch_count];
        let mut total_fluid_height = 0.0;
        
        for &i in &ordered_indices {
            if i >= self.branches.len() {
                continue;
            }
            let branch = &self.branches[i];
            let content = &self.branch_content[i];
            let child_state = &mut tree.children[i];

           // For invisible branches, set a default height
            if !metrics.visible_branches[i] {
                cells[i] = layout::Node::new(Size::ZERO);
                metrics.branch_heights[i] = LINE_HEIGHT; // Set default height
                metrics.branch_widths[i] = 0.0;
            }
            
            let size = content.as_widget().size();
            let height_factor = size.height.fill_factor();
            
            // Skip fluid cells for now
            if height_factor != 0 || size.width.is_fill() {
                row_fill_factors[i] = height_factor;
                continue;
            }

            // Add drop gap if this is the drop target
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Before {
                    y += drop_gap;
                }
            }
            
            // Calculate the x position - arrow, then handle, then content
            let indent_x = self.padding_x + (branch.depth as f32 * self.indent);
            let content_x = indent_x + ARROW_W + HANDLE_HOVER_W + CONTENT_GAP;
            
            // Calculate available width for content
            let available_content_width = (available.width - content_x - self.padding_x).max(0.0);
            
            // Create limits for the content
            let content_limits = layout::Limits::new(
                Size::ZERO,
                Size::new(available_content_width, available.height - y),
            );
            
            // Layout the content
            let content_layout = content.as_widget().layout(child_state, renderer, &content_limits);
            let content_size = content_limits.resolve(
                Length::Shrink,
                Length::Shrink,
                content_layout.size(),
            );
            
            // Store metrics
            metrics.branch_heights[i] = content_size.height.max(LINE_HEIGHT);
            metrics.branch_widths[i] = content_size.width;
            
            // Track maximum content width (accounting for indentation)
            let total_width = content_x + content_size.width;
            max_content_width = max_content_width.max(total_width);
            
            // Store the layout node
            cells[i] = content_layout;
            
            y += metrics.branch_heights[i] + self.spacing;

            // Add drop gap after if needed
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::After {
                    y += drop_gap;
                }
            }
        }
        
        // Calculate total non-fluid height
        for (i, &height) in metrics.branch_heights.iter().enumerate() {

            if metrics.visible_branches[i] && row_fill_factors[i] == 0 {
                // Don't count dragged items
                if let Some(ref drag) = state.drag_state {
                    if !drag.dragged_nodes.contains(&self.branches[i].id) {
                        total_fluid_height += height;
                    }
                } else {
                    total_fluid_height += height;
                }
            }
        }
        
        // SECOND PASS - Layout fluid branches
        let total_fill_factor: u16 = row_fill_factors.iter()
            .enumerate()
            .filter(|(i, _)| {

                if !metrics.visible_branches[*i] {
                    return false;
                }
                // Don't count dragged items
                if let Some(ref drag) = state.drag_state {
                    !drag.dragged_nodes.contains(&self.branches[*i].id)
                } else {
                    true
                }
            })
            .map(|(_, &f)| f)
            .sum();
        
        if total_fill_factor > 0 {
            let available_fluid_height = available.height 
                - total_fluid_height 
                - self.padding_y * 2.0
                - self.spacing * metrics.visible_branches.iter().filter(|&&v| v).count().saturating_sub(1) as f32;
            
            let height_unit = available_fluid_height / total_fill_factor as f32;
            
            for &i in &ordered_indices {
                if i >= self.branches.len() {
                    continue;
                }
                let branch = &self.branches[i];
                let content = &self.branch_content[i];
                let child_state = &mut tree.children[i];

                if !metrics.visible_branches[i] || row_fill_factors[i] == 0 {
                    continue;
                }

                // Skip dragged items
                if let Some(ref drag) = state.drag_state {
                    if drag.dragged_nodes.contains(&branch.id) {
                        continue;
                    }
                }
                
                let size = content.as_widget().size();
                
                // Calculate position - consistent with first pass
                let indent_x = self.padding_x + (branch.depth as f32 * self.indent);
                let content_x = indent_x + ARROW_W + HANDLE_HOVER_W + CONTENT_GAP;
                let available_content_width = (available.width - content_x - self.padding_x).max(0.0);
                
                let max_height = if row_fill_factors[i] == 0 {
                    if size.height.is_fill() {
                        metrics.branch_heights[i]
                    } else {
                        (available.height - y).max(0.0)
                    }
                } else {
                    height_unit * row_fill_factors[i] as f32
                };
                
                // Create limits
                let content_limits = layout::Limits::new(
                    Size::ZERO,
                    Size::new(available_content_width, max_height),
                );
                
                // Layout the content
                let content_layout = content.as_widget().layout(child_state, renderer, &content_limits);
                let content_size = content_limits.resolve(
                    if size.width.is_fill() { tree_fluid } else { Length::Shrink },
                    Length::Shrink,
                    content_layout.size(),
                );
                
                // Update metrics
                metrics.branch_heights[i] = metrics.branch_heights[i].max(content_size.height);
                metrics.branch_widths[i] = metrics.branch_widths[i].max(content_size.width);
                cells[i] = content_layout;
                
                // Track maximum width
                let total_width = content_x + content_size.width;
                max_content_width = max_content_width.max(total_width);
            }
        }
        
        // THIRD PASS - Position all visible branches
        y = self.padding_y;

        for &i in &ordered_indices {
            if i >= self.branches.len() {
                continue;
            }
            let branch = &self.branches[i];

            // Skip invisible branches
            if !metrics.visible_branches[i] {
                continue;
            }

            // Skip dragged branches entirely
            if let Some(ref drag) = state.drag_state {
                if drag.dragged_nodes.contains(&branch.id) {
                    continue;
                }
                
                // Add space BEFORE this branch if it's the drop target
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Before {
                    // Add visual space before this branch
                    let first_dragged_idx = drag.dragged_nodes.first()
                        .and_then(|id| self.branches.iter().position(|b| b.id == *id))
                        .unwrap_or(0);
                    if first_dragged_idx < metrics.branch_heights.len() {
                        y += metrics.branch_heights[first_dragged_idx].max(LINE_HEIGHT) + self.spacing;
                    }
                }
            }
            
            let indent_x = self.padding_x + (branch.depth as f32 * self.indent);
            let content_x = indent_x + ARROW_W + HANDLE_HOVER_W + CONTENT_GAP;
            
            // Move the cell to its position
            cells[i].move_to_mut((content_x, y));
            
            // Apply alignment within the branch's bounds
            let Branch_ { align_x, align_y, .. } = branch;
            cells[i].align_mut(
                *align_x,
                *align_y,
                Size::new(metrics.branch_widths[i], metrics.branch_heights[i]),
            );
            
            y += metrics.branch_heights[i] + self.spacing;

            // Add space AFTER this branch if it's the drop target
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::After {
                    let first_dragged_idx = drag.dragged_nodes.first()
                        .and_then(|id| self.branches.iter().position(|b| b.id == *id))
                        .unwrap_or(0);
                    if first_dragged_idx < metrics.branch_heights.len() {
                        y += metrics.branch_heights[first_dragged_idx].max(LINE_HEIGHT) + self.spacing;
                    }
                }
            }
        }
        
        // Calculate final size
        let intrinsic = limits.resolve(
            self.width,
            self.height,
            Size::new(
                max_content_width + self.padding_x,
                y - self.spacing + self.padding_y,
            ),
        );
        
        layout::Node::with_children(intrinsic, cells)
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
        let (metrics, state) = tree.state.downcast_mut::<(Metrics, State)>();

        let ordered_indices = self.get_ordered_indices(state);
        
        // Update all visible children to ensure they get events
        for &i in &ordered_indices {
            if i >= self.branch_content.len() || i >= metrics.visible_branches.len() {
                continue;
            }
            
            if metrics.visible_branches[i] {
                let branch = &mut self.branch_content[i];
                let child_state = &mut tree.children[i];
                let child_layout = layout.children().nth(i).unwrap();
                
                // Skip dragged items
                if let Some(ref drag) = state.drag_state {
                    if !drag.dragged_nodes.contains(&self.branches[i].id) {
                        branch.as_widget_mut().update(
                            child_state, event, child_layout, cursor, renderer, clipboard, shell, viewport,
                        );
                    }
                } else {
                    branch.as_widget_mut().update(
                        child_state, event, child_layout, cursor, renderer, clipboard, shell, viewport,
                    );
                }
            }
        }
        
        // THEN: Handle tree-specific mouse events  
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position() {
                    let bounds = layout.bounds();
                    let mut y = bounds.y + self.padding_y;
                    
                    for &i in &ordered_indices {
                        if i >= self.branches.len() || i >= metrics.visible_branches.len() {
                            continue;
                        }
                        
                        let branch = &self.branches[i];

                        // Confirm bounds is possible, skip invisible bounds
                        if i >= metrics.visible_branches.len() || !metrics.visible_branches[i] {
                            continue;
                        }

                        // Skip dragged branches
                        if let Some(ref drag) = state.drag_state {
                            if drag.dragged_nodes.contains(&branch.id) {
                                continue;
                            }
                        }
                        
                        let indent_x = bounds.x + self.padding_x + (branch.depth as f32 * self.indent);
                        let branch_height = metrics.branch_heights[i];
                        
                        // Check if clicking on arrow (if has children)
                        if branch.has_children {
                            let arrow_bounds = Rectangle {
                                x: indent_x,
                                y,
                                width: ARROW_W,
                                height: branch_height,
                            };
                            
                            if arrow_bounds.contains(position) {
                                // Toggle expansion
                                if metrics.expanded.contains(&branch.id) {
                                    metrics.expanded.remove(&branch.id);
                                } else {
                                    metrics.expanded.insert(branch.id);
                                }
                                shell.invalidate_layout();
                                shell.request_redraw();
                                return;
                            }
                        }

                        // Check if clicking on handle (start drag or select)
                        let handle_x = indent_x + ARROW_W;
                        let handle_bounds = Rectangle {
                            x: handle_x,
                            y,
                            width: HANDLE_HOVER_W,
                            height: branch_height,
                        };

                        if handle_bounds.contains(position) {
                            // Handle click - either start drag or select
                            let modifiers = keyboard::Modifiers::default(); // You'd get this from the event in real implementation
                            
                            // Update selection
                            if modifiers.control() || modifiers.command() {
                                // Toggle selection
                                if state.selected.contains(&branch.id) {
                                    state.selected.remove(&branch.id);
                                } else {
                                    state.selected.insert(branch.id);
                                }
                            } else {
                                // Single selection
                                state.selected.clear();
                                state.selected.insert(branch.id);
                            }
                            
                            // Start drag
                            let dragged = if state.selected.contains(&branch.id) {
                                state.selected.iter().cloned().collect()
                            } else {
                                vec![branch.id]
                            };

                            // Calculate the bounds of the entire branch row
                            let branch_bounds = Rectangle {
                                x: bounds.x,
                                y,
                                width: bounds.width,
                                height: branch_height,
                            };
                            
                            // Calculate offset from click position to branch origin
                            let click_offset = Vector::new(
                                position.x - branch_bounds.x,
                                position.y - branch_bounds.y,
                            );

                            state.drag_state = Some(DragState {
                                dragged_nodes: dragged,
                                drag_start_bounds: branch_bounds,
                                click_offset,
                                drag_offset: Vector::new(0.0, 0.0),
                                current_position: position,
                                drop_target: None,
                                drop_position: DropPosition::Before,
                            });
                            
                            state.focused = Some(branch.id);
                            shell.invalidate_layout();
                            shell.request_redraw();
                            return;
                        }

                        let branch_bounds = Rectangle {
                            x: bounds.x,
                            y,
                            width: bounds.width,
                            height: branch_height,
                        };

                        if branch_bounds.contains(position) {
                            // Select the branch
                            let modifiers = keyboard::Modifiers::default(); // Need to implement
                                
                            if modifiers.control() || modifiers.command() {
                                // Toggle selection
                                if state.selected.contains(&branch.id) {
                                    state.selected.remove(&branch.id);
                                } else {
                                    state.selected.insert(branch.id);
                                }
                            } else {
                                // Single selection
                                state.selected.clear();
                                state.selected.insert(branch.id);
                            }
                                
                            state.focused = Some(branch.id);
                            shell.invalidate_widgets();
                            return;
                        }
                        
                        y += branch_height + self.spacing;
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => { }
        
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Update hover and drag states
                if let Some(position) = cursor.position() {
                    let bounds = layout.bounds();
                    let mut y = bounds.y + self.padding_y;
                    let mut new_hovered = None;
                    let mut new_hovered_handle = None;
                    
                    for &i in &ordered_indices {
                        if i >= self.branches.len() || i >= metrics.visible_branches.len() {
                            continue;
                        }
                        
                        let branch = &self.branches[i];

                        if !metrics.visible_branches[i] {
                            continue;
                        }
                        
                        // Skip dragged items when calculating drop target
                        if let Some(ref mut drag) = state.drag_state {
                            println!("current drag position before: {}", drag.current_position);
                            drag.current_position = position;
                            if drag.dragged_nodes.contains(&branch.id) {
                                continue;
                            }
                            println!("current drag position after: {}", drag.current_position);
                        }
                        
                        let indent_x = bounds.x + self.padding_x + (branch.depth as f32 * self.indent);
                        let branch_height = metrics.branch_heights[i];
                        
                        // Check if hovering over branch
                        let branch_bounds = Rectangle {
                            x: bounds.x,
                            y,
                            width: bounds.width,
                            height: branch_height,
                        };
                        
                        if branch_bounds.contains(position) {
                            state.hovered_handle = Some(branch.id);
                            new_hovered = Some(branch.id);
                            
                            // Check if hovering over handle area
                            let handle_x = indent_x + ARROW_W;
                            let handle_bounds = Rectangle {
                                x: handle_x,
                                y,
                                width: HANDLE_HOVER_W,
                                height: branch_height,
                            };
                            
                            if handle_bounds.contains(position) {
                                new_hovered_handle = Some(branch.id);
                            }
                            
                            // Calculate drop position if dragging
                            if state.drag_state.is_some() {
                                let new_drop_target = Some(branch.id);
                                let new_drop_position = self.calculate_drop_position(
                                    position.y,
                                    branch_bounds,
                                    branch.has_children,
                                    metrics.expanded.contains(&branch.id),
                                );

                                println!("new_drop_target: {:?}", new_drop_target);
                                println!("new_drop_position: {:?}", new_drop_position);

                                // Update drag state
                                if let Some(ref mut drag) = state.drag_state {
                                    drag.current_position = position;
                                    if new_drop_target != drag.drop_target || new_drop_position != drag.drop_position {
                                        drag.drop_target = new_drop_target;
                                        drag.drop_position = new_drop_position;
                                        shell.invalidate_layout();
                                    }
                                }

                            }
                        }
                        
                        y += branch_height + self.spacing;
                    }


                    

                    
                    if new_hovered != state.hovered || new_hovered_handle != state.hovered_handle {
                        let old_hovered_handle = state.hovered_handle;
                        state.hovered = new_hovered;
                        state.hovered_handle = new_hovered_handle;
                        
                        // If handle hover state changed, we need to relayout
                        if old_hovered_handle != new_hovered_handle && state.drag_state.is_none() {
                            shell.invalidate_layout();
                        }
                    }

                    shell.invalidate_widgets();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                // Handle keyboard navigation
                if let Some(focused) = state.focused {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            // Move focus up
                            let visible_indices: Vec<_> = metrics.visible_branches.iter()
                                .enumerate()
                                .filter(|(_, visible)| **visible)
                                .map(|(i, _)| i)
                                .collect();
                            
                            if let Some(current_pos) = visible_indices.iter()
                                .position(|&i| self.branches[i].id == focused) 
                            {
                                if current_pos > 0 {
                                    state.focused = Some(self.branches[visible_indices[current_pos - 1]].id);
                                    shell.invalidate_widgets();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            // Move focus down
                            let visible_indices: Vec<_> = metrics.visible_branches.iter()
                                .enumerate()
                                .filter(|(_, visible)| **visible)
                                .map(|(i, _)| i)
                                .collect();
                            
                            if let Some(current_pos) = visible_indices.iter()
                                .position(|&i| self.branches[i].id == focused) 
                            {
                                if current_pos < visible_indices.len() - 1 {
                                    state.focused = Some(self.branches[visible_indices[current_pos + 1]].id);
                                    shell.invalidate_widgets();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            // Collapse if has children and is expanded
                            if let Some(branch) = self.branches.iter().find(|b| b.id == focused) {
                                if branch.has_children && metrics.expanded.contains(&focused) {
                                    metrics.expanded.remove(&focused);
                                    shell.invalidate_layout();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            // Expand if has children and is collapsed
                            if let Some(branch) = self.branches.iter().find(|b| b.id == focused) {
                                if branch.has_children && !metrics.expanded.contains(&focused) {
                                    metrics.expanded.insert(focused);
                                    shell.invalidate_layout();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::Space) => {
                            // Toggle selection
                            if modifiers.control() || modifiers.command() {
                                if state.selected.contains(&focused) {
                                    state.selected.remove(&focused);
                                } else {
                                    state.selected.insert(focused);
                                }
                            } else {
                                state.selected.clear();
                                state.selected.insert(focused);
                            }
                            shell.invalidate_widgets();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
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
        let bounds = layout.bounds();
        let (metrics, state) = tree.state.downcast_ref::<(Metrics, State)>();
        let ordered_indices = self.get_ordered_indices(state);
        let tree_style = theme.style(&self.class);
        
        let mut y = bounds.y + self.padding_y;
        
        for &i in &ordered_indices {
            if i >= self.branches.len() {
                continue;
            }
            
            let branch = &self.branches[i];

            // Add bounds check
            if i >= metrics.visible_branches.len() || !metrics.visible_branches[i] {
                continue;
            }
            
            // Ensure we have valid metrics
            if i >= metrics.branch_heights.len() {
                continue;
            }

            // Skip dragged branches (they'll be drawn in overlay)
            if let Some(ref drag) = state.drag_state {
                if drag.dragged_nodes.contains(&branch.id) {
                    continue;
                }
                
                // Draw drop indicator before
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Before {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: bounds.x + self.padding_x,
                                y: y - DROP_INDICATOR_HEIGHT / 2.0 - self.spacing / 2.0,
                                width: bounds.width - self.padding_x * 2.0,
                                height: DROP_INDICATOR_HEIGHT,
                            },
                            border: Border::default(),
                            ..Default::default()
                        },
                        tree_style.accept_drop_indicator_color,
                    );
                    y += DROP_INDICATOR_HEIGHT + self.spacing;
                }
            }
            
            let indent_x = bounds.x + self.padding_x + (branch.depth as f32 * self.indent);
            let branch_height = metrics.branch_heights[i];
            
            // Draw selection background
            if state.selected.contains(&branch.id) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            //x: indent_x, // Testing if I like the full row or using the indent
                            x: bounds.x,
                            y,
                            //width: bounds.width - indent_x + bounds.x, // Testing if I like the full row or using the indent
                            width: bounds.width,
                            height: branch_height,
                        },
                        border: Border::default(),
                        ..Default::default()
                    },
                    tree_style.selection_background,
                );
            }

            // Draw drop target highlight for "Into" position
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Into {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: indent_x,
                                y,
                                width: bounds.width - indent_x + bounds.x,
                                height: branch_height,
                            },
                            border: Border {
                                color: tree_style.accept_drop_indicator_color,
                                width: 2.0,
                                radius: Radius::from(2.0),
                            },
                            ..Default::default()
                        },
                        iced::Background::Color(Color::TRANSPARENT),
                    );
                }
            }
            
            // Draw focus border
            if state.focused == Some(branch.id) || state.hovered == Some(branch.id) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            //x: indent_x, // Testing if I like the full row or using the indent
                            x: bounds.x,
                            y,
                            //width: bounds.width - indent_x + bounds.x, // Testing if I like the full row or using the indent
                            width: bounds.width,
                            height: branch_height,
                        },
                        border: Border {
                            color: tree_style.focus_border,
                            width: 2.0,
                            radius: Radius::from(2.0),
                        },
                        ..Default::default()
                    },
                    iced::Background::Color(Color::TRANSPARENT),
                );
            }

            // Draw selected border
            if state.selected.contains(&branch.id) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            //x: indent_x, // Testing if I like the full row or using the indent
                            x: bounds.x,
                            y,
                            //width: bounds.width - indent_x + bounds.x, // Testing if I like the full row or using the indent
                            width: bounds.width,
                            height: branch_height,
                        },
                        border: Border {
                            color: tree_style.selection_border,
                            width: 2.0,
                            radius: Radius::from(2.0),
                        },
                        ..Default::default()
                    },
                    iced::Background::Color(Color::TRANSPARENT),
                );
            }
            
            // Draw expand/collapse arrow if branch has children
            if branch.has_children {
                let arrow_y = y + (branch_height / 2.0);
                
                let arrow = if metrics.expanded.contains(&branch.id) {
                    //"▼"
                    "🠻"
                } else {
                    //"▶"
                    "🠺"
                };
                
                renderer.fill_text(
                    iced::advanced::Text {
                        content: arrow.into(),
                        bounds: Size::new(ARROW_W, branch_height),
                        size: Pixels(24.0),
                        font: iced::Font::default(),
                        align_x: Alignment::Center,
                        align_y: iced::alignment::Vertical::Center,
                        line_height: iced::advanced::text::LineHeight::default(),
                        shaping: iced::advanced::text::Shaping::Advanced,
                        wrapping: iced::advanced::text::Wrapping::default(),
                    },
                    Point::new(indent_x + ARROW_X_PAD, arrow_y),
                    tree_style.arrow_color,
                    *viewport,
                );
            }
            
            // Draw handle/drag area (to the right of arrow)
            let handle_x = indent_x + ARROW_W;
            let handle_width = if state.hovered == Some(branch.id) {
                HANDLE_HOVER_W  // Full width when hovered
            } else {
                HANDLE_STRIPE_W  // Thin stripe when not hovered
            };
            
            let handle_color = if state.hovered_handle == Some(branch.id) {
                // Lighter color when hovered
                Color::from_rgba(
                    tree_style.line_color.r,
                    tree_style.line_color.g,
                    tree_style.line_color.b,
                    0.3,
                )
            } else {
                // Thin stripe color
                tree_style.line_color
            };
            
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: y + 2.0,
                        width: handle_width,
                        height: branch_height - 4.0,
                    },
                    border: Border::default(),
                    ..Default::default()
                },
                handle_color,
            );
            
            y += branch_height + self.spacing;
        }
        
        // Draw branch content
        for (i, ((branch, child_state), child_layout)) in self
            .branch_content
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {

            if metrics.visible_branches[i] {
                // Skip dragged items
                if let Some(ref drag) = state.drag_state {
                    if !drag.dragged_nodes.contains(&self.branches[i].id) {
                        branch.as_widget().draw(
                            child_state, renderer, theme, style, child_layout, cursor, viewport,
                        );
                    }
                } else {
                    branch.as_widget().draw(
                        child_state, renderer, theme, style, child_layout, cursor, viewport,
                    );
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
       let (metrics, state) = tree.state.downcast_ref::<(Metrics, State)>();
       let ordered_indices = self.get_ordered_indices(state);
        
        // Check if hovering over interactive elements
        if let Some(position) = cursor.position() {
            let bounds = layout.bounds();
            let mut y = bounds.y + self.padding_y;
            
            for &i in &ordered_indices {
                if i >= self.branches.len() || i >= metrics.branch_heights.len() {
                    continue;
                }
                
                let branch = &self.branches[i];
                // Skip invisible branches
                if metrics.visible_branches.get(i).copied().unwrap_or(false) == false {
                    continue;
                }
                
                let indent_x = bounds.x + self.padding_x + (branch.depth as f32 * self.indent);
                let branch_height = metrics.branch_heights[i];
                
                // Check arrow
                if branch.has_children {
                    let arrow_bounds = Rectangle {
                        x: indent_x,
                        y,
                        width: ARROW_W,
                        height: branch_height,
                    };
                    
                    if arrow_bounds.contains(position) {
                        return mouse::Interaction::Pointer;
                    }
                }
                
                // Check handle area (to the right of arrow)
                let handle_x = indent_x + ARROW_W;
                let handle_bounds = Rectangle {
                    x: handle_x,
                    y,
                    width: HANDLE_HOVER_W,
                    height: branch_height,
                };
                
                if handle_bounds.contains(position) {
                    return if state.drag_state.is_some() {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Grab
                    };
                }
                
                y += branch_height + self.spacing;
            }
        }
        
        // Check children interactions
        self.branch_content
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
            .filter(|(i, _)| metrics.visible_branches.get(*i).copied().unwrap_or(false))
            .map(|(_, ((branch, child_state), child_layout))| {
                branch.as_widget().mouse_interaction(
                    child_state, child_layout, cursor, viewport, renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        for ((branch, state), layout) in self
            .branch_content
            .iter()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            branch.as_widget().operate(state, layout, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {

        let drag_state_clone = {
            let (_, state) = tree.state.downcast_mut::<(Metrics, State)>();
            state.drag_state.clone()
        };

        // Check if we're dragging something
        if let Some(ref drag_state) = drag_state_clone {
            // Collect the indices of dragged branches
            let dragged_indices: Vec<usize> = self.branches
                .iter()
                .enumerate()
                .filter(|(_, b)| drag_state.dragged_nodes.contains(&b.id))
                .map(|(i, _)| i)
                .collect();

            for (i, ((branch, child_state), child_layout)) in self
                .branch_content
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .enumerate()
            {
                if dragged_indices.contains(&i) {
                    return Some(iced::advanced::overlay::Element::new(Box::new(DragOverlay {
                        tree_handle: self,
                        state: tree,
                        layout: child_layout,
                        tree_layout: layout,
                        viewport: *viewport,
                        dragged_indices,
                        translation,
                    })));
                }
            }

//            println!("[overlay()] returning Some? {}", !dragged_indices.is_empty());
/*             if !dragged_indices.is_empty() {
                
                return Some(iced::advanced::overlay::Element::new(Box::new(DragOverlay {
                    tree_handle: self,
                    state: tree,
                    layout,
                    viewport: *viewport,
                    dragged_indices,
                    translation,               // keep the inherited translation
                })));
            } */
        }
        
        None
        // Otherwise, check children for overlays
/*         iced::advanced::overlay::from_children(
            &mut self.branch_content,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        ) */
    }

}

// Custom overlay for rendering dragged items
struct DragOverlay<'a, 'b, Message, Theme, Renderer>
where 
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer,
{
    tree_handle: &'a mut TreeHandle<'b, Message, Theme, Renderer>,
    state: &'a mut widget::Tree,
    layout: Layout<'a>,
    tree_layout: Layout<'a>,
    viewport: Rectangle,
    dragged_indices: Vec<usize>,
    translation: Vector,
}

impl<'a, Message, Theme, Renderer> iced::advanced::overlay::Overlay<Message, Theme, Renderer> 
    for DragOverlay<'_, '_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {

         let (metrics, state) = self.state.state.downcast_ref::<(Metrics, State)>();
        
        // Calculate position based on current mouse position and click offset
        let position = if let Some(ref drag) = state.drag_state {
            Point::new(
                drag.current_position.x + self.tree_handle.padding_x,
                drag.current_position.y - drag.click_offset.y,
            )
        } else {
            Point::ORIGIN
        };

        // Calculate the actual content width and height
        let (width, height) = if let Some(ref drag) = state.drag_state {
            // Calculate content width based on the dragged items
            let mut max_width = 0.0f32;
            let mut total_height = 0.0f32;
            
            for &i in &self.dragged_indices {
                if i < metrics.branch_widths.len() {
                    // Account for indentation + arrow + handle + content + padding
                    let branch = &self.tree_handle.branches[i];
                    let indent_x = branch.depth as f32 * self.tree_handle.indent;
                    let content_width = indent_x + ARROW_W + HANDLE_HOVER_W + CONTENT_GAP + metrics.branch_widths[i] + self.tree_handle.padding_x;
                    max_width = max_width.max(content_width);
                    
                    total_height += metrics.branch_heights[i].max(LINE_HEIGHT);
                    if i < self.dragged_indices.len() - 1 {
                        total_height += self.tree_handle.spacing;
                    }
                }
            }
            
            (max_width.max(200.0), total_height.max(LINE_HEIGHT))
        } else {
            (300.0, LINE_HEIGHT)
        };        

        layout::Node::new(Size::new(width, height))
            .move_to(position)

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
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(position) = cursor.position() {
                    let (metrics, state) = self.state.state.downcast_mut::<(Metrics, State)>();
                    
                    if let Some(ref mut drag) = state.drag_state {
                        // Update the current position
                        drag.current_position = position;
                        
                        // Find drop target in real-time
                        let tree_bounds = self.tree_layout.bounds();
                        let mut y = tree_bounds.y + self.tree_handle.padding_y;
                        let mut new_drop_target = None;
                        let mut new_drop_position = DropPosition::Before;
                        
                        for (i, branch) in self.tree_handle.branches.iter().enumerate() {
                            // Skip invisible branches
                            if i >= metrics.visible_branches.len() || !metrics.visible_branches[i] {
                                continue;
                            }
                            
                            // Skip dragged branches
                            if drag.dragged_nodes.contains(&branch.id) {
                                continue;
                            }
                            
                            // Account for visual space if this is the current drop target
                            if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Before {
                                // Add the space that would be created
                                let first_dragged_idx = drag.dragged_nodes.first()
                                    .and_then(|id| self.tree_handle.branches.iter().position(|b| b.id == *id))
                                    .unwrap_or(0);
                                if first_dragged_idx < metrics.branch_heights.len() {
                                    y += metrics.branch_heights[first_dragged_idx].max(LINE_HEIGHT) + self.tree_handle.spacing;
                                }
                            }
                            
                            let branch_height = if i < metrics.branch_heights.len() {
                                metrics.branch_heights[i]
                            } else {
                                LINE_HEIGHT
                            };
                            
                            // Create full row bounds
                            let row_bounds = Rectangle {
                                x: tree_bounds.x,
                                y,
                                width: tree_bounds.width,
                                height: branch_height,
                            };
                            
                            if row_bounds.contains(position) {
                                new_drop_target = Some(branch.id);
                                
                                // Calculate drop position
                                let relative_y = position.y - row_bounds.y;
                                let third_height = row_bounds.height / 3.0;
                                
                                new_drop_position = if relative_y < third_height {
                                    DropPosition::Before
                                } else if relative_y > row_bounds.height - third_height {
                                    DropPosition::After
                                } else if branch.has_children && metrics.expanded.contains(&branch.id) {
                                    DropPosition::Into
                                } else {
                                    if relative_y < row_bounds.height / 2.0 {
                                        DropPosition::Before
                                    } else {
                                        DropPosition::After
                                    }
                                };
                                break;
                            }
                            
                            y += branch_height + self.tree_handle.spacing;
                            
                            // Account for visual space after if this is the current drop target
                            if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::After {
                                let first_dragged_idx = drag.dragged_nodes.first()
                                    .and_then(|id| self.tree_handle.branches.iter().position(|b| b.id == *id))
                                    .unwrap_or(0);
                                if first_dragged_idx < metrics.branch_heights.len() {
                                    y += metrics.branch_heights[first_dragged_idx].max(LINE_HEIGHT) + self.tree_handle.spacing;
                                }
                            }
                        }
                        
                        // Update drop target if changed
                        if new_drop_target != drag.drop_target || new_drop_position != drag.drop_position {
                            drag.drop_target = new_drop_target;
                            drag.drop_position = new_drop_position;
                            
                            // Force layout recalculation to show visual space
                            shell.invalidate_layout();
                        }
                        
                        // Always request redraw for smooth dragging
                        shell.request_redraw();
                    }
                }
            }
            
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Get the current drag state info
                let (drop_target, drop_position, dragged_nodes) = {
                    let (_, state) = self.state.state.downcast_ref::<(Metrics, State)>();
                    if let Some(ref drag) = state.drag_state {
                        (drag.drop_target, drag.drop_position.clone(), drag.dragged_nodes.clone())
                    } else {
                        (None, DropPosition::Before, vec![])
                    }
                };
                
                // Handle the drop
                if let Some(target_id) = drop_target {
                    // Perform the reordering
                    self.reorder_with_tree(&dragged_nodes, target_id, &drop_position);
                    
                    // Emit message if needed
                    if let Some(ref on_drop) = self.tree_handle.on_drop {
                        let drop_info = DropInfo {
                            dragged_ids: dragged_nodes,
                            target_id: Some(target_id),
                            position: drop_position,
                        };
                        shell.publish(on_drop(drop_info));
                    }
                }
                
                // Clear drag state
                let (_, state) = self.state.state.downcast_mut::<(Metrics, State)>();
                state.drag_state = None;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            
            Event::Mouse(mouse::Event::CursorLeft) => {
                // Clear drag state if cursor leaves
                let (_, state) = self.state.state.downcast_mut::<(Metrics, State)>();
                state.drag_state = None;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            _ => {}
        }
        
        // Update children (if needed for other events)
        for ((branch, state), layout) in self
            .tree_handle.branch_content
            .iter_mut()
            .zip(&mut self.state.children)
            .zip(layout.children()) 
        {
            branch.as_widget_mut().update(
                state,
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &self.viewport
            );
        }
    }


    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let (metrics, state) = self.state.state.downcast_ref::<(Metrics, State)>();
        let drag_bounds = layout.bounds();
        let tree_style = theme.style(&self.tree_handle.class);
        
        renderer.with_layer(self.viewport, |renderer| {
            // Draw each dragged branch with semi-transparent branch styling
            let mut y_offset = 0.0;
            
            for &i in &self.dragged_indices {
                if let Some(_) = self.tree_handle.branch_content.get(i) {
                    let branch_height = if i < metrics.branch_heights.len() {
                        metrics.branch_heights[i].max(LINE_HEIGHT)
                    } else {
                        LINE_HEIGHT
                    };

                    let branch_depth = self.tree_handle.branches[i].depth;

                    let branch_bounds = Rectangle {
                        x: drag_bounds.x - ( HANDLE_HOVER_W + HANDLE_STRIPE_W + self.tree_handle.spacing + CONTENT_GAP + ARROW_W + (branch_depth as f32 * self.tree_handle.indent) ),
                        y: drag_bounds.y + y_offset,
                        width: state.drag_state.as_ref().unwrap().drag_start_bounds.width,
                        height: branch_height,
                    };

                    // Draw selection background and border with transparency
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: branch_bounds,
                            border: Border {
                                color: Color::from_rgba(
                                    tree_style.selection_border.r,
                                    tree_style.selection_border.g,
                                    tree_style.selection_border.b,
                                    0.9
                                ),
                                width: 2.0,
                                radius: Radius::from(2.0),
                            },
                            ..Default::default()
                        },
                        Color::from_rgba(
                            tree_style.selection_background.r,
                            tree_style.selection_background.g,
                            tree_style.selection_background.b,
                            0.9
                        ),
                    );

                    y_offset += branch_height + self.tree_handle.spacing;
                }
            }
            
            // Calculate translation from original position to drag position
            let translation = Vector::new(
                drag_bounds.x - self.layout.bounds().x,
                drag_bounds.y - self.layout.bounds().y,
            );

            // Create semi-transparent style for dragged content
            let transparent_style = renderer::Style {
                text_color: Color::from_rgba(
                    style.text_color.r,
                    style.text_color.g,
                    style.text_color.b,
                    0.9  // Slightly more opaque for better readability
                ),
            };

            renderer.with_translation(translation, |renderer| {
                // Draw each dragged branch content
                for &i in &self.dragged_indices {
                    if let Some(branch_content) = self.tree_handle.branch_content.get(i) {
                        let branch_tree = &self.state.children[i];
                        
                        branch_content.as_widget().draw(
                            branch_tree,
                            renderer,
                            theme,
                            &transparent_style,
                            self.layout,
                            cursor,
                            &self.layout.bounds()
                        );
                    }
                }
            });
        });
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Grabbing
    }

    fn index(&self) -> f32 {
        f32::MAX
    }
}

impl<'a, 'b, Message, Theme, Renderer> DragOverlay<'a, 'b, Message, Theme, Renderer>
where 
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn reorder_with_tree(
        &mut self,
        dragged_ids: &[usize],
        target_id: usize,
        drop_position: &DropPosition,
    ) {
        // Collect all items to move (dragged + their descendants)
        let mut items_to_move = HashSet::new();
        for &id in dragged_ids {
            self.collect_branch_and_descendants(id, &mut items_to_move);
        }
        
        println!("Dragged IDs: {:?}", dragged_ids);
        println!("Items to move (including descendants): {:?}", items_to_move);
        
        // Get target info before any modifications
        let target_info = self.tree_handle.branches
            .iter()
            .find(|b| b.id == target_id)
            .map(|b| (b.parent_id, b.depth))
            .unwrap_or((None, 0));
        
        // Collect items to remove, preserving their relative order
        let mut removed_items = Vec::new();
        let mut removed_indices = Vec::new();
        
        for (i, branch) in self.tree_handle.branches.iter().enumerate() {
            if items_to_move.contains(&branch.id) {
                removed_indices.push(i);
            }
        }
        
        // Remove in reverse order
        for &idx in removed_indices.iter().rev() {
            removed_items.push((
                self.tree_handle.branches.remove(idx),
                self.tree_handle.branch_content.remove(idx),
                self.state.children.remove(idx),
            ));
        }
        
        removed_items.reverse();
        
        // Determine the new parent and base depth based on drop position
        let (new_parent_id, new_base_depth) = match drop_position {
            DropPosition::Before | DropPosition::After => {
                // When dropping before/after, use the target's parent and depth
                target_info
            }
            DropPosition::Into => {
                // When dropping INTO, the target becomes the parent
                (Some(target_id), target_info.1 + 1)
            }
        };
        
        // Find insertion point
        let insertion_index = match drop_position {
            DropPosition::Before => {
                self.tree_handle.branches
                    .iter()
                    .position(|b| b.id == target_id)
                    .unwrap_or(self.tree_handle.branches.len())
            }
            DropPosition::Into => {
                // Insert right after the target (as first child)
                self.tree_handle.branches
                    .iter()
                    .position(|b| b.id == target_id)
                    .map(|idx| idx + 1)
                    .unwrap_or(self.tree_handle.branches.len())
            }
            DropPosition::After => {
                // Insert after target and all its descendants
                let mut idx = self.tree_handle.branches
                    .iter()
                    .position(|b| b.id == target_id)
                    .unwrap_or(self.tree_handle.branches.len());
                idx += 1;
                
                // Skip past all descendants of target
                while idx < self.tree_handle.branches.len() {
                    if !self.is_descendant_of(self.tree_handle.branches[idx].id, target_id) {
                        break;
                    }
                    idx += 1;
                }
                idx
            }
        };
        
        // Track what the old depth was for the first dragged item
        let old_depth = removed_items
            .iter()
            .find(|(b, _, _)| dragged_ids.contains(&b.id))
            .map(|(b, _, _)| b.depth)
            .unwrap_or(0);
        
        let depth_change = new_base_depth as i32 - old_depth as i32;
        
        // Reinsert with updated hierarchy
        for (i, (mut branch, content, tree)) in removed_items.into_iter().enumerate() {
            // Only update parent_id for directly dragged items
            if dragged_ids.contains(&branch.id) {
                println!("Setting parent_id for {} from {:?} to {:?}", branch.id, branch.parent_id, new_parent_id);
                branch.parent_id = new_parent_id;
                branch.depth = new_base_depth;
            } else {
                // This is a descendant - adjust its depth relative to its parent
                branch.depth = (branch.depth as i32 + depth_change).max(0) as u16;
            }
            
            let insert_at = insertion_index + i;
            if insert_at <= self.tree_handle.branches.len() {
                self.tree_handle.branches.insert(insert_at, branch);
                self.tree_handle.branch_content.insert(insert_at, content);
                self.state.children.insert(insert_at, tree);
            } else {
                self.tree_handle.branches.push(branch);
                self.tree_handle.branch_content.push(content);
                self.state.children.push(tree);
            }
        }
        
        // Save the new order to state
        let new_order: Vec<usize> = self.tree_handle.branches.iter().map(|b| b.id).collect();
        
        let (_, state) = self.state.state.downcast_mut::<(Metrics, State)>();
        state.branch_order = Some(new_order);
        
        self.update_has_children_flags();
    }

    fn collect_branch_and_descendants(&self, branch_id: usize, result: &mut HashSet<usize>) {
        result.insert(branch_id);
        
        // Find all direct children and recursively collect their descendants
        for branch in &self.tree_handle.branches {
            if branch.parent_id == Some(branch_id) {
                self.collect_branch_and_descendants(branch.id, result);
            }
        }
    }

    fn find_insertion_index(&self, target_id: usize, drop_position: &DropPosition) -> usize {
        let target_index = self.tree_handle.branches
            .iter()
            .position(|b| b.id == target_id)
            .unwrap_or(self.tree_handle.branches.len());

        match drop_position {
            DropPosition::Before => target_index,
            DropPosition::Into => target_index + 1,
            DropPosition::After => {
                // Find the end of target's subtree
                let mut idx = target_index + 1;
                while idx < self.tree_handle.branches.len() {
                    if !self.is_descendant_of(self.tree_handle.branches[idx].id, target_id) {
                        break;
                    }
                    idx += 1;
                }
                idx
            }
        }
    }

    fn is_descendant_of(&self, potential_child: usize, potential_ancestor: usize) -> bool {
        let mut current_id = Some(potential_child);
        
        while let Some(id) = current_id {
            if let Some(branch) = self.tree_handle.branches.iter().find(|b| b.id == id) {
                if branch.parent_id == Some(potential_ancestor) {
                    return true;
                }
                current_id = branch.parent_id;
            } else {
                break;
            }
        }
        false
    }

    fn update_has_children_flags(&mut self) {
        // Reset all flags
        for branch in &mut self.tree_handle.branches {
            branch.has_children = false;
        }
        
        // Set flags based on actual parent-child relationships
        let parent_ids: HashSet<usize> = self.tree_handle.branches
            .iter()
            .filter_map(|b| b.parent_id)
            .collect();
            
        for branch in &mut self.tree_handle.branches {
            if parent_ids.contains(&branch.id) {
                branch.has_children = true;
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TreeHandle<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(tree: TreeHandle<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tree)
    }
}


/// A branch in a tree that contains content and can have children.
#[allow(missing_debug_implementations)]
pub struct Branch<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    pub content: Element<'a, Message, Theme, Renderer>,
    pub children: Vec<Branch<'a, Message, Theme, Renderer>>,
    pub align_x: iced::Alignment,
    pub align_y: iced::Alignment,
    pub accepts_drops: bool,
}

impl<'a, Message, Theme, Renderer> 
    Branch<'a, Message, Theme, Renderer> {

        /// Adds children to this branch
        pub fn with_children(mut self, children: Vec<Self>) -> Self {
            self.children = children;
            self
        }

        pub fn accepts_drops(mut self) -> Self {
            self.accepts_drops = true;
            self
        }

        pub fn align_x(
            mut self, 
            alignment: impl Into<iced::Alignment>
        ) -> Self {
            self.align_x = alignment.into();
            self
        }

        pub fn align_y(
            mut self,
            alignment: impl Into<iced::Alignment>
        ) -> Self {
            self.align_y = alignment.into();
            self
        }
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub dragged_nodes: Vec<usize>, // What's being dragged
    pub drag_start_bounds: Rectangle,  // Store the original bounds
    pub click_offset: Vector,          // Offset from click point to item origin
    pub drag_offset: Vector,
    pub current_position: Point,
    pub drop_target: Option<usize>, // Where it would drop
    pub drop_position: DropPosition, // Before, after, or into
}

#[derive(Debug, Clone, PartialEq)]
pub enum DropPosition {
    Before,
    After, 
    Into, // As child
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
                selection_background: palette.background.base.color,
                selection_text: palette.background.base.text,
                selection_border: palette.secondary.base.color,
                focus_border: Color::from_rgba(
                    palette.secondary.base.color.r,
                    palette.secondary.base.color.g,
                    palette.secondary.base.color.b,
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


