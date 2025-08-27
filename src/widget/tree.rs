use iced::{
    advanced::{
        layout,
        renderer,
        text::Renderer as _,
        widget::{self, tree::Tree},
        Clipboard, Layout, Shell, Widget,
    }, border::Radius, keyboard, mouse, widget::text::Alignment, Border, Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector
};
use std::collections::{HashSet, HashMap};


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
    branch_state: Option<Vec<BranchState>>,
}

#[derive(Clone, Debug)]
struct BranchState {
    id: usize,
    parent_id: Option<usize>,
    depth: u16,
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

    // Helper to get ordered indices from saved state
    fn get_ordered_indices(&self, state: &State) -> Vec<usize> {
        if let Some(ref branch_states) = state.branch_state {
            let mut indices = Vec::new();
            
            for bs in branch_states {
                if let Some(idx) = self.branches.iter().position(|b| b.id == bs.id) {
                    indices.push(idx);
                }
            }
            
            // Add any new branches not in saved state
            for (i, branch) in self.branches.iter().enumerate() {
                if !branch_states.iter().any(|bs| bs.id == branch.id) {
                    indices.push(i);
                }
            }
            
            indices
        } else {
            (0..self.branches.len()).collect()
        }
    }
    
    // Helper to get effective branch info (with saved parent_id and depth)
    fn get_branch_info(&self, index: usize, state: &State) -> (usize, Option<usize>, u16) {
        let branch = &self.branches[index];
        
        if let Some(ref branch_states) = state.branch_state {
            if let Some(bs) = branch_states.iter().find(|bs| bs.id == branch.id) {
                return (branch.id, bs.parent_id, bs.depth);
            }
        }
        
        // Fall back to original values
        (branch.id, branch.parent_id, branch.depth)
    }
    
    // Determins if a branch is visible based on drag status [ If branch, or a parent is dragged, hide the branch. ]
    fn is_branch_visible(&self, index: usize, metrics: &Metrics, state: &State) -> bool {
        if index >= self.branches.len() {
            return false;
        }

        let (id, parent_id, _) = self.get_branch_info(index, state);
        
        // Check if this branch itself is being dragged
        if let Some(ref drag) = state.drag_state {
            if drag.dragged_nodes.contains(&id) {
                return false;
            }
            
            // Check if this branch's parent is being dragged
            if let Some(parent_id) = parent_id {
                if drag.dragged_nodes.contains(&parent_id) {
                    return false;
                }
                
                // Also check if any ancestor is being dragged (for deeply nested items)
                let mut current_parent = parent_id;
                while let Some(parent_idx) = self.branches.iter().position(|b| b.id == current_parent) {
                    if drag.dragged_nodes.contains(&current_parent) {
                        return false;
                    }
                    let (_, next_parent, _) = self.get_branch_info(parent_idx, state);
                    if let Some(np) = next_parent {
                        current_parent = np;
                    } else {
                        break;
                    }
                }
            }
        }
        
        // Root level items are always visible (unless being dragged)
        if parent_id.is_none() {
            return true;
        }
        
        // Check if parent is expanded
        if let Some(parent_id) = parent_id {
            // Find parent branch by ID
            if let Some(parent_index) = self.branches.iter().position(|b| b.id == parent_id) {
                // Parent must be visible and expanded
                return self.is_branch_visible(parent_index, metrics, state) 
                    && metrics.expanded.contains(&parent_id);
            }
        }
        
        false
    }

    /// Calculate branch positions without drop indicator spaces for stable hit testing
    fn calculate_stable_positions(&self, metrics: &Metrics, state: &State, bounds: Rectangle) -> Vec<(usize, Rectangle, bool, bool)> {
        let ordered_indices = self.get_ordered_indices(state);
        let mut positions = Vec::new();
        let mut y = bounds.y + self.padding_y;
        
        for &i in &ordered_indices {
            if i >= self.branches.len() || 
               i >= metrics.visible_branches.len() || 
               !metrics.visible_branches[i] {
                continue;
            }
            
            let branch = &self.branches[i];
            
            // Skip dragged branches
            if let Some(ref drag) = state.drag_state {
                if drag.dragged_nodes.contains(&branch.id) {
                    continue;
                }
            }
            
            let branch_height = if i < metrics.branch_heights.len() {
                metrics.branch_heights[i]
            } else {
                LINE_HEIGHT
            };
            
            let branch_bounds = Rectangle {
                x: bounds.x,
                y,
                width: bounds.width,
                height: branch_height,
            };
            
            positions.push((
                branch.id,
                branch_bounds,
                branch.has_children,
                metrics.expanded.contains(&branch.id)
            ));
            
            y += branch_height + self.spacing;
        }
        
        positions
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
        
        // Initialize branch state if not present
        if state.branch_state.is_none() {
            state.branch_state = Some(
                self.branches.iter().map(|b| BranchState {
                    id: b.id,
                    parent_id: b.parent_id,
                    depth: b.depth,
                }).collect()
            );
        }
        
        let ordered_indices = self.get_ordered_indices(state);
        let branch_count = self.branches.len();
        
        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        let tree_fluid = self.width.fluid();
        
        // Update visibility based on expansion state and saved parent relationships
        metrics.visible_branches = vec![false; branch_count];
        for i in 0..branch_count {
            metrics.visible_branches[i] = self.is_branch_visible(i, metrics, state);
        }
        
        let mut cells = Vec::with_capacity(branch_count);
        cells.resize(branch_count, layout::Node::default());
        
        metrics.branch_heights = vec![0.0; branch_count];
        metrics.branch_widths = vec![0.0; branch_count];
        
        // FIRST PASS - Layout non-fluid visible branches
        let mut y = self.padding_y;
        let mut max_content_width = 0.0f32;
        let mut row_fill_factors = vec![0u16; branch_count];
        let mut total_fluid_height = 0.0;
        
        // Process visible branches in order
        for &i in &ordered_indices {
            if i >= self.branches.len() {
                continue;
            }
            
            let content = &self.branch_content[i];
            let child_state = &mut tree.children[i];

           // For invisible branches, set a default height
            if !metrics.visible_branches[i] {
                cells[i] = layout::Node::new(Size::ZERO);
                metrics.branch_heights[i] = LINE_HEIGHT; // Set default height
                metrics.branch_widths[i] = 0.0;
            }
            
            // Get effective depth from saved state
            let (_, _, effective_depth) = self.get_branch_info(i, state);
            
            let size = content.as_widget().size();
            let height_factor = size.height.fill_factor();
            
            // Skip fluid cells for now
            if height_factor != 0 || size.width.is_fill() {
                row_fill_factors[i] = height_factor;
                continue;
            }
            
            // Calculate the x position using effective depth
            let indent_x = self.padding_x + (effective_depth as f32 * self.indent);
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
        }
        
        // Calculate total non-fluid height
        for (i, &height) in metrics.branch_heights.iter().enumerate() {
            if metrics.visible_branches[i] && row_fill_factors[i] == 0 {
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
                if i >= self.branches.len() || !metrics.visible_branches[i] || row_fill_factors[i] == 0 {
                    continue;
                }
                
                let branch = &self.branches[i];
                let content = &self.branch_content[i];
                let child_state = &mut tree.children[i];
                
                // Skip dragged items
                if let Some(ref drag) = state.drag_state {
                    if drag.dragged_nodes.contains(&branch.id) {
                        continue;
                    }
                }
                
                let size = content.as_widget().size();
                
                // Get effective depth from saved state
                let (_, _, effective_depth) = self.get_branch_info(i, state);
                
                // Calculate position using effective depth
                let indent_x = self.padding_x + (effective_depth as f32 * self.indent);
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

        let drop_indicator_space = if state.drag_state.is_some() {
            // Use a consistent space for drop indicators
            LINE_HEIGHT + self.spacing
        } else {
            0.0
        };
        
        for &i in &ordered_indices {
            if i >= self.branches.len() || !metrics.visible_branches[i] {
                continue;
            }
            
            let branch = &self.branches[i];
            
            // Skip dragged branches entirely
            if let Some(ref drag) = state.drag_state {
                if drag.dragged_nodes.contains(&branch.id) {
                    continue;
                }
                
                // Add space BEFORE this branch if it's the drop target
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Before {
                    y += drop_indicator_space;
                }
            }
            
            // Get effective depth from saved state
            let (_, _, effective_depth) = self.get_branch_info(i, state);
            
            let indent_x = self.padding_x + (effective_depth as f32 * self.indent);
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

            // Handle "Into" drop position - add space for first child preview
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::Into {
                    // If the branch is expanded and has children, we need to add space
                    // for the preview that will appear as the first child
                    if metrics.expanded.contains(&branch.id) {
                        // Add space for the preview
                        y += drop_indicator_space;
                        
                        // Push down all children of this branch
                        // Find and adjust positions of immediate children
                        for &child_i in &ordered_indices {
                            if child_i >= self.branches.len() || child_i == i {
                                continue;
                            }
                            
                            let (child_id, child_parent_id, _) = self.get_branch_info(child_i, state);
                            
                            // If this is a child of the drop target, it needs to be pushed down
                            if child_parent_id == Some(branch.id) && !drag.dragged_nodes.contains(&child_id) {
                                // This child and all subsequent visible branches need adjustment
                                // But we'll handle this in the next iteration naturally
                                break;
                            }
                        }
                    }
                }
            }
            
            // Add space AFTER this branch if it's the drop target
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(branch.id) && drag.drop_position == DropPosition::After {
                    y += drop_indicator_space;
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
            if i >= self.branches.len() || i >= metrics.visible_branches.len() || !metrics.visible_branches[i] {
                continue;
            }
            
            if i >= metrics.branch_heights.len() {
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

                                    // Get effective depth from saved state
                        let (_, _, effective_depth) = self.get_branch_info(i, state);

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
                        
                        let indent_x = bounds.x + self.padding_x + (effective_depth as f32 * self.indent);
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
                            shell.request_redraw();
                            return;
                        }
                        
                        y += branch_height + self.spacing;
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => { }
        
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Only handle hover states when NOT dragging
                if state.drag_state.is_none() {
                    if let Some(position) = cursor.position() {
                        let bounds = layout.bounds();
                        
                        // Use stable positions for hit testing
                        let stable_positions = self.calculate_stable_positions(metrics, state, bounds);
                        
                        let mut new_hovered = None;
                        let mut new_hovered_handle = None;
                        
                        for (branch_id, branch_bounds, _, _) in stable_positions {
                            if branch_bounds.contains(position) {
                                new_hovered = Some(branch_id);
                                
                                // Find the branch to get its depth
                                if let Some(branch_idx) = self.branches.iter().position(|b| b.id == branch_id) {
                                    let (_, _, effective_depth) = self.get_branch_info(branch_idx, state);
                                    let indent_x = bounds.x + self.padding_x + (effective_depth as f32 * self.indent);
                                    
                                    // Check if hovering over handle area
                                    let handle_x = indent_x + ARROW_W;
                                    let handle_bounds = Rectangle {
                                        x: handle_x,
                                        y: branch_bounds.y,
                                        width: HANDLE_HOVER_W,
                                        height: branch_bounds.height,
                                    };
                                    
                                    if handle_bounds.contains(position) {
                                        new_hovered_handle = Some(branch_id);
                                    }
                                }
                                break;
                            }
                        }
                        
                        if new_hovered != state.hovered || new_hovered_handle != state.hovered_handle {
                            state.hovered = new_hovered;
                            state.hovered_handle = new_hovered_handle;
                            shell.request_redraw();
                        }
                    }
                }
            }

            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                // Handle keyboard navigation
                if let Some(focused) = state.focused {

                    let ordered_indices = self.get_ordered_indices(state);
                    
                    // Filter to only visible branches in their display order
                    let visible_ordered: Vec<usize> = ordered_indices.iter()
                        .filter(|&&i| i < metrics.visible_branches.len() && metrics.visible_branches[i])
                        .map(|&i| self.branches[i].id)
                        .collect();

                    match key {
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            // Move focus up
                            if let Some(current_pos) = visible_ordered.iter().position(|&id| id == focused) {
                                if current_pos > 0 {
                                    state.focused = Some(visible_ordered[current_pos - 1]);
                                    shell.invalidate_widgets();
                                    shell.request_redraw();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            // Move focus down
                            if let Some(current_pos) = visible_ordered.iter().position(|&id| id == focused) {
                                if current_pos < visible_ordered.len() - 1 {
                                    state.focused = Some(visible_ordered[current_pos + 1]);
                                    shell.invalidate_widgets();
                                    shell.request_redraw();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            // Collapse if has children and is expanded
                            if let Some(branch) = self.branches.iter().find(|b| b.id == focused) {
                                if branch.has_children && metrics.expanded.contains(&focused) {
                                    metrics.expanded.remove(&focused);
                                    shell.invalidate_layout();
                                    shell.request_redraw();
                                }
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            // Expand if has children and is collapsed
                            if let Some(branch) = self.branches.iter().find(|b| b.id == focused) {
                                if branch.has_children && !metrics.expanded.contains(&focused) {
                                    metrics.expanded.insert(focused);
                                    shell.invalidate_layout();
                                    shell.request_redraw();
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
                            shell.request_redraw();
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

        // Helper to draw drop preview
        let draw_drop_preview = |renderer: &mut Renderer, y: f32, depth: u16, width: f32| {
            let preview_indent = bounds.x + self.padding_x + (depth as f32 * self.indent);
            let preview_height = LINE_HEIGHT;
            
            // Draw preview background
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: preview_indent,
                        y,
                        width: width - preview_indent + bounds.x,
                        height: preview_height,
                    },
                    border: Border {
                        color: tree_style.accept_drop_indicator_color,
                        width: 2.0,
                        radius: Radius::from(4.0),
                    },
                    ..Default::default()
                },
                Color::from_rgba(
                    tree_style.accept_drop_indicator_color.r,
                    tree_style.accept_drop_indicator_color.g,
                    tree_style.accept_drop_indicator_color.b,
                    0.1  // Semi-transparent background
                ),
            );
            
            // Draw "ghost" handle and content area
            let handle_x = preview_indent + ARROW_W;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: handle_x,
                        y: y + 2.0,
                        width: HANDLE_STRIPE_W,
                        height: preview_height - 4.0,
                    },
                    border: Border::default(),
                    ..Default::default()
                },
                Color::from_rgba(
                    tree_style.line_color.r,
                    tree_style.line_color.g,
                    tree_style.line_color.b,
                    0.3
                ),
            );
        };

        // Track if we need to adjust for an "Into" preview
        let mut pending_into_adjustment = false;
        
        for &i in &ordered_indices {
            if i >= self.branches.len() || 
            i >= metrics.visible_branches.len() || 
            !metrics.visible_branches[i] ||
            i >= metrics.branch_heights.len() {
                continue;
            }
            
            let branch = &self.branches[i];
            
            // Get effective info from saved state
            let (id, parent_id, effective_depth) = self.get_branch_info(i, state);

            // Skip dragged branches
            if let Some(ref drag) = state.drag_state {
                if drag.dragged_nodes.contains(&id) {
                    continue;
                }
                
                // Draw drop preview BEFORE this branch
                if drag.drop_target == Some(id) && drag.drop_position == DropPosition::Before {
                    let preview_depth = effective_depth;
                    draw_drop_preview(renderer, y, preview_depth, bounds.width);
                    y += LINE_HEIGHT + self.spacing;
                }
            }

            // Check if this branch's parent has an "Into" drop and this is its first child
            if pending_into_adjustment {
                // This is the first child after an "Into" drop preview
                y += LINE_HEIGHT + self.spacing;
                pending_into_adjustment = false;
            }
            
            let indent_x = bounds.x + self.padding_x + (effective_depth as f32 * self.indent);
            let branch_height = metrics.branch_heights[i];

            // Store the actual Y position for this branch
            let branch_y = y;

            // Draw "Into" drop indicator as a highlighted border
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(id) && drag.drop_position == DropPosition::Into {
                    // Only show preview of where it will appear as first child
                    if metrics.expanded.contains(&id) {
                        pending_into_adjustment = true;
                    } else {
                        // If collapsed, show a subtle "into" indicator on the right side
                        let indicator_width = 30.0;
                        let indicator_x = bounds.x + bounds.width - indicator_width - 10.0;
                        
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: indicator_x,
                                    y: y + branch_height / 2.0 - 1.5,
                                    width: indicator_width,
                                    height: 3.0,
                                },
                                border: Border::default(),
                                ..Default::default()
                            },
                            tree_style.accept_drop_indicator_color,
                        );
                        
                        // Draw a small arrow pointing into
                        renderer.fill_text(
                            iced::advanced::Text {
                                content: "→".into(),
                                bounds: Size::new(20.0, branch_height),
                                size: Pixels(16.0),
                                font: iced::Font::default(),
                                align_x: Alignment::Center,
                                align_y: iced::alignment::Vertical::Center,
                                line_height: iced::advanced::text::LineHeight::default(),
                                shaping: iced::advanced::text::Shaping::Advanced,
                                wrapping: iced::advanced::text::Wrapping::default(),
                            },
                            Point::new(indicator_x - 20.0, y + (branch_height / 2.0)),
                            tree_style.accept_drop_indicator_color,
                            *viewport,
                        );
                    }
                }
            }

            // Draw selection background
            if state.selected.contains(&id) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y,
                            width: bounds.width,
                            height: branch_height,
                        },
                        border: Border::default(),
                        ..Default::default()
                    },
                    tree_style.selection_background,
                );
            }
            
            // Draw hover/focus border
            if state.focused == Some(id) || state.hovered == Some(id) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y,
                            width: bounds.width,
                            height: branch_height,
                        },
                        border: Border {
                            color: tree_style.focus_border,
                            width: 1.0,
                            radius: Radius::from(2.0),
                        },
                        ..Default::default()
                    },
                    iced::Background::Color(Color::TRANSPARENT),
                );
            }
            
            // Draw expand/collapse arrow if branch has children
            if branch.has_children {
                let arrow = if metrics.expanded.contains(&id) {
                    "🠻"
                } else {
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
                    Point::new(indent_x + ARROW_X_PAD, y + (branch_height / 2.0)),
                    tree_style.arrow_color,
                    *viewport,
                );
            }
            
            // Draw handle/drag area (to the right of arrow)
            let handle_x = indent_x + ARROW_W;
            let handle_width = HANDLE_STRIPE_W;
            
            let handle_color = if state.hovered_handle == Some(id) {
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
                        y: branch_y + 2.0,
                        width: handle_width,
                        height: branch_height - 4.0,
                    },
                    border: Border::default(),
                    ..Default::default()
                },
                handle_color,
            );
            
            y += branch_height + self.spacing;

            // Draw the "Into" preview if this branch is expanded
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(id) && 
                drag.drop_position == DropPosition::Into && 
                metrics.expanded.contains(&id) {
                    // Draw preview for first child position
                    let child_preview_y = branch_y + branch_height + self.spacing;
                    let child_depth = effective_depth + 1;
                    draw_drop_preview(renderer, child_preview_y, child_depth, bounds.width);
                }
            }

            // Draw drop preview AFTER this branch (and all its children)
            if let Some(ref drag) = state.drag_state {
                if drag.drop_target == Some(id) && drag.drop_position == DropPosition::After {
                    // Check if there are any more visible items after this one
                    let is_last_visible_item = !ordered_indices.iter()
                        .skip_while(|&&j| j != i)
                        .skip(1)
                        .any(|&j| j < self.branches.len() && metrics.visible_branches[j]);
                    
                    // Draw at appropriate depth
                    let preview_depth = if parent_id.is_some() && is_last_visible_item {
                        0  // Root level only if this is a child AND the very last item
                    } else {
                        effective_depth  // Same level as target
                    };
                    
                    draw_drop_preview(renderer, y, preview_depth, bounds.width);
                    y += LINE_HEIGHT + self.spacing;
                }
            }
        
            // Draw branch content
            for (i, ((branch, child_state), child_layout)) in self
                .branch_content
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .enumerate()
            {

                if i < metrics.visible_branches.len() && metrics.visible_branches[i] {
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
                if i >= self.branches.len() || i >= metrics.visible_branches.len() || !metrics.visible_branches[i] {
                    continue;
                }
                
                if i >= metrics.branch_heights.len() {
                    continue;
                }

                // Get effective info from saved state
                let (_id, _parent_id, effective_depth) = self.get_branch_info(i, state);
                
                let branch = &self.branches[i];

                // Skip invisible branches
                if metrics.visible_branches.get(i).copied().unwrap_or(false) == false {
                    continue;
                }
                
                let indent_x = bounds.x + self.padding_x + (effective_depth as f32 * self.indent);
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
        }
        None
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
                    // Get effective depth from saved state
                    let effective_depth = if let Some(ref branch_states) = state.branch_state {
                        branch_states.iter()
                            .find(|bs| self.tree_handle.branches.get(i).map(|b| b.id == bs.id).unwrap_or(false))
                            .map(|bs| bs.depth)
                            .unwrap_or(self.tree_handle.branches[i].depth)
                    } else {
                        self.tree_handle.branches[i].depth
                    };
                    
                    let indent_x = effective_depth as f32 * self.tree_handle.indent;
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
                    let ordered_indices = self.tree_handle.get_ordered_indices(state);
                    
                    // Pre-collect branch info WITHOUT accounting for drop spaces
                    let branch_infos: Vec<_> = ordered_indices.iter()
                        .filter_map(|&i| {
                            if i >= self.tree_handle.branches.len() || 
                            i >= metrics.visible_branches.len() || 
                            !metrics.visible_branches[i] {
                                return None;
                            }
                            
                            let branch = &self.tree_handle.branches[i];
                            let (id, parent_id, depth) = self.tree_handle.get_branch_info(i, state);
                            
                            let branch_height = if i < metrics.branch_heights.len() {
                                metrics.branch_heights[i]
                            } else {
                                LINE_HEIGHT
                            };
                            
                            Some((
                                id,
                                parent_id,
                                depth,
                                branch_height,
                                branch.has_children,
                                metrics.expanded.contains(&id)
                            ))
                        })
                        .collect();
                    
                    if let Some(ref mut drag) = state.drag_state {
                        // Update position
                        drag.current_position = position;
                        
                        let tree_bounds = self.tree_layout.bounds();
                        let mut new_drop_target = drag.drop_target; // Keep current by default
                        let mut new_drop_position = drag.drop_position.clone();
                        
                        // Build STABLE positions (no drop indicator spaces)
                        let mut branch_positions = Vec::new();
                        let mut y = tree_bounds.y + self.tree_handle.padding_y;
                        
                        for (id, parent_id, depth, branch_height, has_children, is_expanded) in &branch_infos {
                            // Skip dragged branches
                            if drag.dragged_nodes.contains(id) {
                                continue;
                            }
                            
                            // DON'T add drop indicator spaces here - keep positions stable
                            branch_positions.push((
                                *id,
                                *parent_id,
                                *depth,
                                y,
                                *branch_height,
                                *has_children,
                                *is_expanded
                            ));
                            
                            y += branch_height + self.tree_handle.spacing;
                        }
                        
                        // Find target using stable positions
                        let mut found_target = false;
                        for (id, parent_id, depth, branch_y, height, has_children, is_expanded) in &branch_positions {
                            let row_bounds = Rectangle {
                                x: tree_bounds.x,
                                y: *branch_y,
                                width: tree_bounds.width,
                                height: *height,
                            };
                            
                            // Expand hit zone slightly to prevent loss during small movements
                            let expanded_bounds = Rectangle {
                                x: row_bounds.x,
                                y: row_bounds.y - 2.0,  // Add a small buffer
                                width: row_bounds.width,
                                height: row_bounds.height + 4.0,
                            };
                            
                            if expanded_bounds.contains(position) {
                                found_target = true;
                                new_drop_target = Some(*id);
                                
                                let relative_y = position.y - row_bounds.y;
                                let height_ratio = relative_y / row_bounds.height;
                                
                                // Simplified logic: No "After" for branches with children
                                if *has_children {
                                    // Only Before or Into for parent nodes
                                    new_drop_position = if height_ratio < 0.3 {
                                        DropPosition::Before
                                    } else {
                                        DropPosition::Into
                                    };
                                } else {
                                    // Before or After for leaf nodes
                                    new_drop_position = if height_ratio < 0.5 {
                                        DropPosition::Before
                                    } else {
                                        DropPosition::After
                                    };
                                }
                                break;
                            }
                        }

                        // Handle end-of-tree drop
                        if !found_target && position.y > tree_bounds.y && !branch_positions.is_empty() {
                            let (last_id, last_parent_id, _, last_y, last_height, _, _) = 
                                branch_positions.last().unwrap();
                            
                            if position.y > last_y + last_height {
                                // Just use the actual last item as the target
                                new_drop_target = Some(*last_id);
                                new_drop_position = DropPosition::After;
                            }
                        }
                        
                        // Only update if actually changed
                        let changed = new_drop_target != drag.drop_target || 
                                    new_drop_position != drag.drop_position;
                        
                        if changed {
                            drag.drop_target = new_drop_target;
                            drag.drop_position = new_drop_position;
                            shell.invalidate_layout();
                        }
                        
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

                    // Get effective depth from saved state
                    let effective_depth = if let Some(ref branch_states) = state.branch_state {
                        branch_states.iter()
                            .find(|bs| self.tree_handle.branches.get(i).map(|b| b.id == bs.id).unwrap_or(false))
                            .map(|bs| bs.depth)
                            .unwrap_or_else(|| self.tree_handle.branches.get(i).map(|b| b.depth).unwrap_or(0))
                    } else {
                        self.tree_handle.branches.get(i).map(|b| b.depth).unwrap_or(0)
                    };

                    let branch_bounds = Rectangle {
                        x: drag_bounds.x - (HANDLE_HOVER_W + HANDLE_STRIPE_W + self.tree_handle.spacing + CONTENT_GAP + ARROW_W + (effective_depth as f32 * self.tree_handle.indent)),
                        y: drag_bounds.y + y_offset,
                        width: state.drag_state.as_ref().unwrap().drag_start_bounds.width,
                        height: branch_height,
                    };
                    println!("branch_bounds: {:?}", branch_bounds);

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
                        
                        println!("self.layout: {:?}", self.layout);
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
        // Get current branch states
        let (_, state) = self.state.state.downcast_ref::<(Metrics, State)>();
        let current_states: Vec<BranchState> = if let Some(ref branch_states) = state.branch_state {
            branch_states.clone()
        } else {
            self.tree_handle.branches.iter().map(|b| BranchState {
                id: b.id,
                parent_id: b.parent_id,
                depth: b.depth,
            }).collect()
        };
        
        // Create a map for quick lookup
        let state_map: HashMap<usize, BranchState> = current_states.iter()
            .map(|bs| (bs.id, bs.clone()))
            .collect();
        
        // Collect all items to move
        let mut items_to_move = HashSet::new();
        for &id in dragged_ids {
            self.collect_branch_and_descendants(id, &mut items_to_move);
        }
        
        // Get target info
        let target_state = state_map.get(&target_id)
            .cloned()
            .unwrap_or_else(|| BranchState {
                id: target_id,
                parent_id: None,
                depth: 0,
            });
        
        // Separate moved and non-moved items
        let mut new_states: Vec<BranchState> = Vec::new();
        let mut removed_states: Vec<BranchState> = Vec::new();
        
        for bs in current_states {
            if items_to_move.contains(&bs.id) {
                removed_states.push(bs);
            } else {
                new_states.push(bs);
            }
        }
        
        // Calculate new parent and depth based on drop position
        let (new_parent_id, new_base_depth) = match drop_position {
            DropPosition::Before => {
                // Same level as target
                (target_state.parent_id, target_state.depth)
            }
            DropPosition::After => {
                // If dropping after a nested item and we want root level
                // This happens when dragging to the bottom of the tree
                // Check if this is meant to be a root-level drop
                let is_last_item = new_states.iter()
                    .rposition(|bs| bs.id == target_id)
                    .map(|idx| idx == new_states.len() - 1 || 
                        // Or it's the last at its level
                        !new_states[idx + 1..].iter().any(|bs| bs.parent_id == target_state.parent_id))
                    .unwrap_or(false);
                
                if is_last_item && target_state.parent_id.is_some() {
                    // Check if we're dropping at the very end - make it root level
                    let has_root_siblings_after = new_states.iter()
                        .skip_while(|bs| bs.id != target_id)
                        .skip(1)
                        .any(|bs| bs.parent_id.is_none());
                    
                    if !has_root_siblings_after {
                        // Drop at root level
                        (None, 0)
                    } else {
                        // Keep at target's level
                        (target_state.parent_id, target_state.depth)
                    }
                } else {
                    // Normal after drop - same level as target
                    (target_state.parent_id, target_state.depth)
                }
            }
            DropPosition::Into => {
                // As child of target
                (Some(target_id), target_state.depth + 1)
            }
        };
        
        // Find insertion point
        let insertion_index = match drop_position {
            DropPosition::Before => {
                new_states.iter().position(|bs| bs.id == target_id)
                    .unwrap_or(new_states.len())
            }
            DropPosition::Into => {
                // Insert as first child
                let parent_pos = new_states.iter().position(|bs| bs.id == target_id)
                    .unwrap_or(new_states.len());
                parent_pos + 1
            }
            DropPosition::After => {
                // Insert after target and all its descendants
                let mut idx = new_states.iter().position(|bs| bs.id == target_id)
                    .map(|i| i + 1)
                    .unwrap_or(new_states.len());
                
                // Skip all descendants
                while idx < new_states.len() {
                    let current = &new_states[idx];
                    if self.is_descendant_of_in_states(current.id, target_id, &new_states) {
                        idx += 1;
                    } else {
                        break;
                    }
                }
                idx
            }
        };
        
        // Calculate depth change
        let old_depth = removed_states.iter()
            .find(|bs| dragged_ids.contains(&bs.id))
            .map(|bs| bs.depth)
            .unwrap_or(0);
        let depth_change = new_base_depth as i32 - old_depth as i32;
        
        // Update and insert moved items
        let mut insert_offset = 0;
        for mut bs in removed_states {
            if dragged_ids.contains(&bs.id) {
                bs.parent_id = new_parent_id;
                bs.depth = new_base_depth;
            } else {
                // Descendant of dragged item
                bs.depth = (bs.depth as i32 + depth_change).max(0) as u16;
            }
            new_states.insert(insertion_index + insert_offset, bs);
            insert_offset += 1;
        }
        
        // Update the state
        let (_, state) = self.state.state.downcast_mut::<(Metrics, State)>();
        state.branch_state = Some(new_states);
        
        // Update has_children flags
        self.update_has_children_flags();
    }

    // Helper to check if an item is a descendant using the states array
    fn is_descendant_of_in_states(&self, potential_child: usize, potential_ancestor: usize, states: &[BranchState]) -> bool {
        let mut current_id = Some(potential_child);
        
        while let Some(id) = current_id {
            if let Some(bs) = states.iter().find(|s| s.id == id) {
                if bs.parent_id == Some(potential_ancestor) {
                    return true;
                }
                current_id = bs.parent_id;
            } else {
                break;
            }
        }
        false
    }

    fn collect_branch_and_descendants(&self, branch_id: usize, result: &mut HashSet<usize>) {
        result.insert(branch_id);
        
        // Use saved state to find children
        let (_, state) = self.state.state.downcast_ref::<(Metrics, State)>();
        
        let children: Vec<usize> = if let Some(ref branch_states) = state.branch_state {
            // Use saved parent relationships
            branch_states.iter()
                .filter(|bs| bs.parent_id == Some(branch_id))
                .map(|bs| bs.id)
                .collect()
        } else {
            // Fall back to original
            self.tree_handle.branches.iter()
                .filter(|b| b.parent_id == Some(branch_id))
                .map(|b| b.id)
                .collect()
        };
        
        for child_id in children {
            self.collect_branch_and_descendants(child_id, result);
        }
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
            
            Style {
                text: palette.background.base.text,
                selection_background: palette.background.weakest.color,
                selection_text: palette.background.base.text,
                selection_border: palette.secondary.base.color,
                focus_border: Color::from_rgba(
                    palette.secondary.base.color.r,
                    palette.secondary.base.color.g,
                    palette.secondary.base.color.b,
                    0.5
                ),
                arrow_color: palette.background.strong.color,
                line_color: palette.primary.weak.color,
                accept_drop_indicator_color: palette.primary.strong.color,
                deny_drop_indicator_color: palette.danger.strong.color,
            }
        })
    }
    
    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}