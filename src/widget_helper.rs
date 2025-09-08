use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, horizontal_rule, horizontal_space, vertical_rule, pick_list, progress_bar, radio, row, scrollable, slider, text, text_input, toggler, vertical_space, Button, Column, Container, Radio, Row, Space, Text, TextInput
    },
    Alignment, Background, Border, Color, Element, Font, Length::{self, FillPortion}, Padding, Shadow,
    Theme, Vector,
};
use iced::time;
use std::time::Duration;
use std::collections::HashMap;
use crate::widget::generic_overlay::overlay_button;
mod controls;
use controls::*;
use widgets::tree::{tree_handle, branch, DropInfo, DropPosition, Branch};

// ============================================================================
// CORE DATA STRUCTURES - Simplified ID-based approach
// ============================================================================

/// Unique identifier for widgets in the hierarchy
#[derive(Debug, Clone)]
pub enum PropertyChange {
    // Common properties
    WidgetName(String),
    //ShowWidgetBounds(bool),
    ShowWidgetBounds,
    Width(Length),
    Height(Length),
    PaddingTop(f32),
    PaddingRight(f32),
    PaddingBottom(f32),
    PaddingLeft(f32),
    
    // Container properties
    AlignX(ContainerAlignX),
    AlignY(ContainerAlignY),
    BorderWidth(f32),
    BorderRadius(f32),
    BorderColor(Color),
    BackgroundColor(Color),
    HasShadow(bool),
    ShadowOffsetX(f32),
    ShadowOffsetY(f32),
    ShadowBlur(f32),
    ShadowColor(Color),
    
    // Layout properties
    Spacing(f32),
    AlignItems(Alignment),
    
    // Text properties
    TextContent(String),
    TextSize(f32),
    TextColor(Color),
    Font(FontType),
    
    // Button properties
    ButtonStyle(ButtonStyleType),

    // TextInput properties
    TextInputValue(String),
    TextInputPlaceholder(String),
    TextInputSize(f32),
    TextInputPadding(f32),
    IsSecure(bool),
    
    // Checkbox properties
    CheckboxChecked(bool),
    CheckboxLabel(String),
    CheckboxSize(f32),
    CheckboxSpacing(f32),
    
    // Radio properties
    RadioSelectedIndex(usize),
    RadioOptions(Vec<String>),
    RadioLabel(String),
    RadioSize(f32),
    RadioSpacing(f32),

    // Slider properties
    SliderValue(f32),
    SliderMin(f32),
    SliderMax(f32),
    SliderStep(f32),

    // Progress properties
    ProgressValue(f32),
    ProgressMin(f32),
    ProgressMax(f32),
    ProgressLength(Length),     // main axis (width if horizontal, height if vertical)
    ProgressGirth(Length),      // thickness (height if horizontal, width if vertical)
    ProgressVertical(bool),     // orientation
    
    // Toggler properties
    TogglerActive(bool),
    TogglerLabel(String),
    TogglerSize(f32),
    TogglerSpacing(f32),
    
    // PickList properties
    PickListSelectedIndex(Option<usize>),
    PickListSelected(Option<String>),
    PickListPlaceholder(String),
    PickListOptions(Vec<String>),

    // Rule properties
    RuleOrientation(RuleOrientation),
    RuleThickness(f32),
}

// Helper function to apply property changes
pub fn apply_property_change(properties: &mut Properties, change: PropertyChange) {
    match change {
        PropertyChange::Width(value) => properties.width = value,
        PropertyChange::Height(value) => properties.height = value,
        PropertyChange::AlignItems(value) => properties.align_items = value,

        PropertyChange::PaddingTop(value) => properties.padding.top = value,
        PropertyChange::PaddingRight(value) => properties.padding.right = value,
        PropertyChange::PaddingBottom(value) => properties.padding.bottom = value,
        PropertyChange::PaddingLeft(value) => properties.padding.left = value,
        PropertyChange::Spacing(value) => properties.spacing = value,

        PropertyChange::WidgetName(value) => properties.widget_name = value,
        //PropertyChange::ShowWidgetBounds(value) => properties.show_widget_bounds = value,
        PropertyChange::ShowWidgetBounds => properties.show_widget_bounds = !properties.show_widget_bounds,

        PropertyChange::BorderWidth(value) => properties.border_width = value,
        PropertyChange::BorderRadius(value) => properties.border_radius = value,
        PropertyChange::BorderColor(value) => properties.border_color = value,

        PropertyChange::BackgroundColor(value) => properties.background_color = value,

        PropertyChange::TextContent(value) => properties.text_content = value,
        PropertyChange::TextSize(value) => properties.text_size = value,
        PropertyChange::TextColor(value) => properties.text_color = value,
        PropertyChange::Font(value) => properties.font = value,

        PropertyChange::ButtonStyle(value) => properties.button_style = value,
        
        // TextInput properties
        PropertyChange::TextInputValue(value) => properties.text_input_value = value,
        PropertyChange::TextInputPlaceholder(value) => properties.text_input_placeholder = value,
        PropertyChange::TextInputSize(value) => properties.text_input_size = value,
        PropertyChange::TextInputPadding(value) => properties.text_input_padding = value,
        PropertyChange::IsSecure(value) => properties.is_secure = value,
        
        // Checkbox properties
        PropertyChange::CheckboxChecked(value) => properties.checkbox_checked = value,
        PropertyChange::CheckboxLabel(value) => properties.checkbox_label = value,
        PropertyChange::CheckboxSize(value) => properties.checkbox_size = value,
        PropertyChange::CheckboxSpacing(value) => properties.checkbox_spacing = value,

        // Slider properties
        PropertyChange::SliderValue(value) => properties.slider_value = value,
        PropertyChange::SliderMin(value) => properties.slider_min = value,
        PropertyChange::SliderMax(value) => properties.slider_max = value,
        PropertyChange::SliderStep(value) => properties.slider_step = value,
        
        // Radio properties
        PropertyChange::RadioSelectedIndex(value) => {
            if value < properties.radio_options.len() {
                properties.radio_selected_index = value;
            }
        },
        PropertyChange::RadioOptions(value) => {
            properties.radio_options = value;
            // Reset selection if it's out of bounds
            if properties.radio_selected_index >= properties.radio_options.len() {
                properties.radio_selected_index = 0;
            }
        },
        PropertyChange::RadioLabel(value) => properties.radio_label = value,
        PropertyChange::RadioSize(value) => properties.radio_size = value,
        PropertyChange::RadioSpacing(value) => properties.radio_spacing = value,

        // Progress properties
        PropertyChange::ProgressValue(v) => {
            let lo = properties.progress_min.min(properties.progress_max);
            let hi = properties.progress_min.max(properties.progress_max);
            properties.progress_value = v.clamp(lo, hi);
        }
        PropertyChange::ProgressMin(v) => {
            properties.progress_min = v;
            let lo = properties.progress_min.min(properties.progress_max);
            let hi = properties.progress_min.max(properties.progress_max);
            properties.progress_value = properties.progress_value.clamp(lo, hi);
        }
        PropertyChange::ProgressMax(v) => {
            properties.progress_max = v;
            let lo = properties.progress_min.min(properties.progress_max);
            let hi = properties.progress_min.max(properties.progress_max);
            properties.progress_value = properties.progress_value.clamp(lo, hi);
        }
        PropertyChange::ProgressLength(len) => properties.progress_length = len,
        PropertyChange::ProgressGirth(len) => properties.progress_girth = len,
        PropertyChange::ProgressVertical(v) => properties.progress_vertical = v,
        
        // Toggler properties
        PropertyChange::TogglerActive(value) => properties.toggler_active = value,
        PropertyChange::TogglerLabel(value) => properties.toggler_label = value,
        PropertyChange::TogglerSize(value) => properties.toggler_size = value,
        PropertyChange::TogglerSpacing(value) => properties.toggler_spacing = value,
        
        // PickList properties
        PropertyChange::PickListSelectedIndex(value) => properties.picklist_selected_index = value,
        PropertyChange::PickListSelected(value) => properties.picklist_selected = value,
        PropertyChange::PickListPlaceholder(value) => properties.picklist_placeholder = value,
        PropertyChange::PickListOptions(value) => properties.picklist_options = value,

        // Rule properties
        PropertyChange::RuleOrientation(v) => properties.rule_orientation = v,
        PropertyChange::RuleThickness(v)   => properties.rule_thickness  = v,
        
        _ => {} // Placeholder for properties not implemented
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub usize);

/// Central widget hierarchy manager - Simplified to use only IDs
#[derive(Debug, Clone)]
pub struct WidgetHierarchy {
    root: Widget,
    selected_id: Option<WidgetId>,
    next_id: usize,
}

impl WidgetHierarchy {
    pub fn new(root_type: WidgetType) -> Self {
        Self {
            root: Widget::new(root_type, WidgetId(0)),
            selected_id: Some(WidgetId(0)), // Start with root selected
            next_id: 1,
        }
    }
    
    pub fn root(&self) -> &Widget {
        &self.root
    }

    pub fn can_add_to_root(&self, widget_type: WidgetType) -> bool {
        if self.root.children.is_empty() {
            // Root can only have Column or Row as first child
            matches!(widget_type, WidgetType::Column | WidgetType::Row)
        } else {
            // Root already has a child, can't add more
            false
        }
    }
    
    pub fn selected_id(&self) -> Option<WidgetId> {
        self.selected_id
    }
    
    pub fn select_widget(&mut self, id: WidgetId) {
        if self.widget_exists(id) {
            self.selected_id = Some(id);
        }
    }
    
    pub fn get_selected(&self) -> Option<&Widget> {
        self.selected_id.and_then(|id| self.get_widget_by_id(id))
    }
    
    pub fn get_widget_by_id(&self, id: WidgetId) -> Option<&Widget> {
        fn find_widget(widget: &Widget, target_id: WidgetId) -> Option<&Widget> {
            if widget.id == target_id {
                return Some(widget);
            }
            for child in &widget.children {
                if let Some(found) = find_widget(child, target_id) {
                    return Some(found);
                }
            }
            None
        }
        find_widget(&self.root, id)
    }
    
    pub fn get_widget_by_id_mut(&mut self, id: WidgetId) -> Option<&mut Widget> {
        fn find_widget_mut(widget: &mut Widget, target_id: WidgetId) -> Option<&mut Widget> {
            if widget.id == target_id {
                return Some(widget);
            }
            for child in &mut widget.children {
                if let Some(found) = find_widget_mut(child, target_id) {
                    return Some(found);
                }
            }
            None
        }
        find_widget_mut(&mut self.root, id)
    }
    
    pub fn widget_exists(&self, id: WidgetId) -> bool {
        self.get_widget_by_id(id).is_some()
    }

    pub fn can_add_child(&self, parent_id: WidgetId, widget_type: WidgetType) -> bool {
        if let Some(parent) = self.get_widget_by_id(parent_id) {
            // Check if parent can have children
            if !can_have_children(&parent.widget_type) {
                return false;
            }
            
            // Special constraint for root container
            if parent_id == self.root.id {
                if parent.children.is_empty() {
                    // Root can only have Column or Row as first child
                    matches!(widget_type, WidgetType::Column | WidgetType::Row)
                } else {
                    // Root already has a child, can't add more
                    false
                }
            } else {
                // Non-root containers can add any compatible child
                true
            }
        } else {
            false
        }
    }
    
    pub fn add_child(&mut self, parent_id: WidgetId, widget_type: WidgetType) -> Result<WidgetId, String> {
        if !self.can_add_child(parent_id, widget_type) {
            if parent_id == self.root.id {
                if self.root.children.is_empty() {
                    return Err("Root container can only have Column or Row as its first child".to_string());
                } else {
                    return Err("Root container can only have one child".to_string());
                }
            } else {
                return Err(format!("Cannot add {:?} to this parent", widget_type));
            }
        }

        let child_id = WidgetId(self.next_id);
        self.next_id += 1;
        let mut child = Widget::new(widget_type, child_id);

        let parent_is_under_scrollable =
            matches!(self.get_widget_by_id(parent_id).map(|p| p.widget_type), Some(WidgetType::Scrollable))
            || self.has_scrollable_ancestor(parent_id);

        if parent_is_under_scrollable {
            let orig = child.properties.height;
            if matches!(orig, Length::Fill | Length::FillPortion(_)) {
                child.properties.saved_height_before_scrollable = Some(orig);
                child.properties.height = Length::Shrink;
            }
        }

        if let Some(parent) = self.get_widget_by_id_mut(parent_id) {
            parent.children.push(child);
            Ok(child_id)
        } else {
            Err("Parent widget not found".to_string())
        }
    }
    
    pub fn delete_widget(&mut self, id: WidgetId) -> Result<(), String> {
        if id == self.root.id {
            return Err("Cannot delete root widget".to_string());
        }
        
        // Find parent and remove child
        if let Some(parent_id) = self.find_parent_id(id) {
            if let Some(parent) = self.get_widget_by_id_mut(parent_id) {
                parent.children.retain(|child| child.id != id);
                
                // If we deleted the selected widget, select the parent
                if self.selected_id == Some(id) {
                    self.selected_id = Some(parent_id);
                }
                
                Ok(())
            } else {
                Err("Parent widget not found".to_string())
            }
        } else {
            Err("Cannot find parent of widget".to_string())
        }
    }
    
    pub fn find_parent_id(&self, child_id: WidgetId) -> Option<WidgetId> {
        fn find_parent(widget: &Widget, target_id: WidgetId) -> Option<WidgetId> {
            for child in &widget.children {
                if child.id == target_id {
                    return Some(widget.id);
                }
                if let Some(parent_id) = find_parent(child, target_id) {
                    return Some(parent_id);
                }
            }
            None
        }
        find_parent(&self.root, child_id)
    }

    pub fn apply_property_change(&mut self, id: WidgetId, mut change: PropertyChange) {
        if let PropertyChange::Height(h) = change.clone() {
            if self.has_scrollable_ancestor(id) {
                if matches!(h, Length::Fill | Length::FillPortion(_)) {
                    if let Some(w) = self.get_widget_by_id_mut(id) {
                        if w.properties.saved_height_before_scrollable.is_none() {
                            w.properties.saved_height_before_scrollable = Some(h);
                        }
                        w.properties.height = Length::Shrink; // clamp
                    }
                    return;
                }
            }
        }

        if let Some(widget) = self.get_widget_by_id_mut(id) {
            apply_property_change(&mut widget.properties, change);
        }
    }
    
    pub fn change_widget_type(&mut self, id: WidgetId, new_type: WidgetType) {
        if let Some(widget) = self.get_widget_by_id_mut(id) {
            widget.widget_type = new_type;
            widget.properties = Properties::for_widget_type(new_type);
            widget.name = format!("{:?}", new_type);
            
            // Clear children if new type can't have them
            if !can_have_children(&new_type) {
                widget.children.clear();
            }
        }
    }

    pub fn move_widget(
        &mut self,
        id: WidgetId,
        new_parent_id: WidgetId,
        mut new_index: usize,
    ) -> Result<(), String> {
        if id == self.root.id {
            return Err("Cannot move root widget".into());
        }
        if !self.widget_exists(id) {
            return Err("Widget to move not found".into());
        }
        if !self.widget_exists(new_parent_id) {
            return Err("New parent not found".into());
        }

        // Prevent cycles: cannot move a node into its own subtree
        if self.is_descendant(id, new_parent_id) {
            return Err("Cannot move a widget into its own descendant".into());
        }

        // Parent capability checks
        let new_parent_ty = self.get_widget_by_id(new_parent_id).unwrap().widget_type;
        if !can_have_children(&new_parent_ty) {
            return Err(format!("{new_parent_ty:?} cannot have children"));
        }

        // Root container constraints
        if new_parent_id == self.root.id {
            // Only Column/Row allowed under root
            let moving_ty = self.get_widget_by_id(id).unwrap().widget_type;
            if !matches!(moving_ty, WidgetType::Column | WidgetType::Row) {
                return Err("Root can only contain Column or Row".into());
            }
            // Root can have only one child (unless we're reordering the same one)
            let root_children = &self.root.children;
            let already_under_root = self.find_parent_id(id) == Some(self.root.id);
            if !already_under_root && !root_children.is_empty() {
                return Err("Root container can only have one child".into());
            }
            // Clamp index for root (0 or existing 0)
            new_index = 0;
        }

        // Detach node from current parent
        let old_parent_id = self.find_parent_id(id).ok_or("Old parent not found")?;
        let mut node = self.remove_and_return(id).ok_or("Failed to detach node")?;

        // If moving within the same parent and we removed a lower index, fix target index
        if old_parent_id == new_parent_id {
            let siblings_len = self.get_widget_by_id(new_parent_id).unwrap().children.len();
            // After removal, children length decreased by 1. Clamp index accordingly.
            new_index = new_index.min(siblings_len);
        } else {
            let siblings_len = self.get_widget_by_id(new_parent_id).unwrap().children.len();
            new_index = new_index.min(siblings_len);
        }

        // Insert into new parent
        let parent = self.get_widget_by_id_mut(new_parent_id).ok_or("New parent not found")?;
        parent.children.insert(new_index, node);

        // Keep selection reasonable
        self.selected_id = Some(id);
        Ok(())
    }

    fn is_descendant(&self, ancestor: WidgetId, candidate: WidgetId) -> bool {
        fn walk(w: &Widget, anc: WidgetId, cand: WidgetId) -> bool {
            if w.id == anc {
                return contains(&w.children, cand);
            }
            for c in &w.children {
                if walk(c, anc, cand) {
                    return true;
                }
            }
            false
        }
        fn contains(children: &[Widget], id: WidgetId) -> bool {
            for c in children {
                if c.id == id { return true; }
                if contains(&c.children, id) { return true; }
            }
            false
        }
        walk(&self.root, ancestor, candidate)
    }

    /// Remove a node from the tree and return it.
    fn remove_and_return(&mut self, id: WidgetId) -> Option<Widget> {
        fn take_from(parent: &mut Widget, id: WidgetId) -> Option<Widget> {
            if let Some(pos) = parent.children.iter().position(|c| c.id == id) {
                return Some(parent.children.remove(pos));
            }
            for c in &mut parent.children {
                if let Some(found) = take_from(c, id) {
                    return Some(found);
                }
            }
            None
        }
        if id == self.root.id { return None; }
        take_from(&mut self.root, id)
    }

    /// Toggle Row<->Column and Container<->Scrollable without resetting props/children
    pub fn swap_kind(&mut self, id: WidgetId) {
        let old_type;
        {
            let w = match self.get_widget_by_id(id) { Some(w) => w, None => return };
            old_type = w.widget_type;
        }

        if let Some(w) = self.get_widget_by_id_mut(id) {
            let new_type = match w.widget_type {
                WidgetType::Row        => WidgetType::Column,
                WidgetType::Column     => WidgetType::Row,
                WidgetType::Container  => WidgetType::Scrollable,
                WidgetType::Scrollable => WidgetType::Container,
                _ => w.widget_type,
            };

            if new_type != w.widget_type {
                w.widget_type = new_type;
                w.name = format!("{:?}", w.widget_type);
            }
        }

        // If we just became a Scrollable, clamp subtree.
        if let Some(w) = self.get_widget_by_id(id) {
            if matches!(w.widget_type, WidgetType::Scrollable) {
                drop(w);
                self.sanitize_subtree_for_scrollable(id);
                return;
            }
        }

        // If we were a Scrollable and swapped back to Container, restore subtree.
        if matches!(old_type, WidgetType::Scrollable) {
            self.restore_subtree_after_scrollable(id);
        }
    }

    // Is there a Scrollable anywhere above this node?
    pub fn has_scrollable_ancestor(&self, mut id: WidgetId) -> bool {
        while let Some(parent_id) = self.find_parent_id(id) {
            if let Some(parent) = self.get_widget_by_id(parent_id) {
                if matches!(parent.widget_type, WidgetType::Scrollable) {
                    return true;
                }
                id = parent_id;
            } else {
                break;
            }
        }
        false
    }

    // Force all descendants of a Scrollable to NOT fill vertically
    fn sanitize_subtree_for_scrollable(&mut self, root_scrollable_id: WidgetId) {
        fn clamp_descendants(widget: &mut Widget) {
            match widget.properties.height {
                Length::Fill | Length::FillPortion(_) => {
                    if widget.properties.saved_height_before_scrollable.is_none() {
                        widget.properties.saved_height_before_scrollable = Some(widget.properties.height);
                    }
                    widget.properties.height = Length::Shrink;
                }
                _ => {}
            }
            for child in &mut widget.children {
                clamp_descendants(child);
            }
        }
        if let Some(scrollable) = self.get_widget_by_id_mut(root_scrollable_id) {
            for child in &mut scrollable.children {
                clamp_descendants(child);
            }
        }
    }

    // Restore any saved heights after leaving a Scrollable subtree
    fn restore_subtree_after_scrollable(&mut self, root_container_id: WidgetId) {
        fn restore(widget: &mut Widget) {
            if let Some(h) = widget.properties.saved_height_before_scrollable.take() {
                widget.properties.height = h;
            }
            for child in &mut widget.children {
                restore(child);
            }
        }
        if let Some(container) = self.get_widget_by_id_mut(root_container_id) {
            for child in &mut container.children {
                restore(child);
            }
        }
    }

}

// ============================================================================
// MAIN WIDGET VISUALIZER - Simplified
// ============================================================================

pub struct WidgetVisualizer {
    // Core state - simplified to single hierarchy
    hierarchy: WidgetHierarchy,
    
    // UI state
    show_padding: bool,
    show_spacing: bool,
    show_borders: bool,
    theme: Theme,
}

impl Default for WidgetVisualizer {
    fn default() -> Self {
        let hierarchy = WidgetHierarchy::new(WidgetType::Container);
        Self {
            hierarchy,
            show_padding: true,
            show_spacing: true,
            show_borders: true,
            theme: Theme::Light,
        }
    }
}

impl WidgetVisualizer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TreeAction => {
            
            }

            Message::TreeMove { id, new_parent, new_index } => {
                let _ = self.hierarchy.move_widget(id, new_parent, new_index);
            }

            Message::TreeDrop { dragged_id, target_id, position } => {
                match position {
                    DropPosition::Into => {
                        // Drop into target - simple case
                        let _ = self.hierarchy.move_widget(dragged_id, target_id, 0);
                    }
                    DropPosition::Before | DropPosition::After => {
                        // Need target's parent and position
                        if let Some(parent_id) = self.hierarchy.find_parent_id(target_id) {
                            if let Some(parent) = self.hierarchy.get_widget_by_id(parent_id) {
                                let target_index = parent.children.iter()
                                    .position(|c| c.id == target_id)
                                    .unwrap_or(0);
                                
                                let new_index = match position {
                                    DropPosition::After => target_index + 1,
                                    _ => target_index,
                                };
                                
                                let _ = self.hierarchy.move_widget(dragged_id, parent_id, new_index);
                            }
                        }
                    }
                }
            }

            Message::SelectWidget(id) => {
                self.hierarchy.select_widget(id);
            }
            
            Message::DeleteWidget(id) => {
                let _ = self.hierarchy.delete_widget(id);
            }
            
            Message::AddChild(parent_id, widget_type) => {
                println!("Adding {:?} to parent {:?}", widget_type, parent_id);
                if let Ok(new_id) = self.hierarchy.add_child(parent_id, widget_type) {
                    println!("Successfully added with id {:?}", new_id);
                    // Debug print the tree
                    self.debug_print_widget(&self.hierarchy.root(), 0);
                } else {
                    println!("Failed to add child");
                }
            }
            
            Message::PropertyChanged(id, change) => {
                self.hierarchy.apply_property_change(id, change);
            }

            Message::SwapKind(id) => {
                self.hierarchy.swap_kind(id);
            }

            // Interactive widget messages
            Message::ButtonPressed(_id) => {
                // For preview, we don't need to do anything special
            }
            
            Message::TextInputChanged(id, value) => {
                self.hierarchy.apply_property_change(id, PropertyChange::TextInputValue(value));
            }
            
            Message::CheckboxToggled(id, checked) => {
                self.hierarchy.apply_property_change(id, PropertyChange::CheckboxChecked(checked));
            }
            
            Message::RadioSelected(id, index) => {
                self.hierarchy.apply_property_change(id, PropertyChange::RadioSelectedIndex(index));
            }
            
            Message::SliderChanged(id, value) => {
                self.hierarchy.apply_property_change(id, PropertyChange::SliderValue(value));
            }
            
            Message::TogglerToggled(id, active) => {
                self.hierarchy.apply_property_change(id, PropertyChange::TogglerActive(active));
            }
            
            Message::PickListSelected(id, index) => {
                self.hierarchy.apply_property_change(id, PropertyChange::PickListSelected(Some(index)));
            }
            
            // Visual helpers
            Message::ShowPaddingToggled(show) => {
                self.show_padding = show;
            }
            
            Message::ShowSpacingToggled(show) => {
                self.show_spacing = show;
            }
            
            Message::ShowBordersToggled(show) => {
                self.show_borders = show;
            }

            Message::Explain(id) => {
                // self.hierarchy.apply_property_change(id, PropertyChange::ShowWidgetBounds(true));
                self.hierarchy.apply_property_change(id, PropertyChange::ShowWidgetBounds);
            }
            Message::ExplainTimeout(id) => { // Need to add Subscription to time to implement
                //self.hierarchy.apply_property_change(id, PropertyChange::ShowWidgetBounds(false));
            }
            
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
        }
        
        Action::None
    }
    
    pub fn view(&self) -> Element<Message> {
        let left_panel = self.build_left_panel();
        let right_panel = self.build_preview_panel();
        
        row![left_panel, right_panel].into()
    }
    
    fn build_left_panel(&self) -> Element<Message> {
        column![
            // Header
            column![
                text("Widget Visualizer").size(24),
                horizontal_rule(5),
            ].spacing(10),
            Space::new(Length::Fill, 10),

            // Theme selector
            column![
                text("Theme").size(18),
                pick_list(
                    Theme::ALL,
                    Some(self.theme.clone()),
                    Message::ThemeChanged,
                ),
            ].spacing(5),
            
            // Widget hierarchy
            column![
                text("Widget Hierarchy").size(16),
                self.widget_tree_view(),
            ].spacing(5),
            
            // Add child controls
            if let Some(selected_id) = self.hierarchy.selected_id() {
                if let Some(selected_widget) = self.hierarchy.get_widget_by_id(selected_id) {
                    if can_have_children(&selected_widget.widget_type) {
                        self.build_add_child_controls(selected_id)
                    } else {
                        Element::from(column![])
                    }
                } else {
                    Element::from(column![])
                }
            } else {
                Element::from(column![])
            }
        ]
        .width(Length::Fixed(400.0))
        .padding(10)
        .into()
    }


     fn widget_tree_view(&self) -> Element<Message> {
         self.build_tree_item(self.hierarchy.root(), 0, self.hierarchy.selected_id())
     }

    fn build_tree_item(&self, widget: &Widget, depth: usize, selected_id: Option<WidgetId>) -> Element<Message> {
        let indent = "  ".repeat(depth);
        let is_selected = Some(widget.id) == selected_id;
        
        // Create the overlay content for this specific widget
        let overlay_content = self.build_editor_for_widget(widget, widget.id);
        
        // Determine if this widget can be swapped and the button label
        let swap_label: Option<&'static str> = match widget.widget_type {
            WidgetType::Row        => Some("Swap to Column"),
            WidgetType::Column     => Some("Swap to Row"),
            WidgetType::Container  => Some("Make Scrollable"),
            WidgetType::Scrollable => Some("Make Container"),
            _ => None,
        };

        // Optional Swap button element
        let swap_btn: Option<Element<Message>> = swap_label.map(|label| {
            button(label)
                .on_press(Message::SwapKind(widget.id))
                .style(button::secondary)
                .into()
        });

        let mut items = vec![
            row![
                button(text(format!("{}{}", indent, widget.name)))
                    .on_press(Message::SelectWidget(widget.id))
                    .style(if is_selected { 
                        button::primary 
                    } else { 
                        button::secondary 
                    }),
                horizontal_space(),
                if let Some(b) = swap_btn { b } else { horizontal_space().into() },

                // Create overlay button with this widget's specific content
                overlay_button(
                    "Edit",
                    format!("Editing {}", widget.name),
                    overlay_content
                )
                .overlay_width(500.0)
                .overlay_height(750.0)
                .style(button::success),
                if widget.id.0 != 0 { // Don't allow deleting root
                    button("×")
                        .on_press(Message::DeleteWidget(widget.id))
                        .style(button::danger)
                        .into()
                } else {
                    Element::from(horizontal_space())
                }
            ].spacing(5).into()
        ];
        
        // Add children
        for child in &widget.children {
            items.push(self.build_tree_item(child, depth + 1, selected_id));
        }
        
        column(items).spacing(2).into()
    }
    
    fn build_add_child_controls(&self, parent_id: WidgetId) -> Element<Message> {
        let available_types = if parent_id == self.hierarchy.root().id {
            // Root container constraints
            if self.hierarchy.root().children.is_empty() {
                // Root is empty, can only add Column or Row
                vec![WidgetType::Column, WidgetType::Row]
            } else {
                // Root already has a child, can't add more
                vec![]
            }
        } else {
            // Regular containers can have all widget types
            vec![
                WidgetType::Container,
                WidgetType::Row,
                WidgetType::Column,
                WidgetType::Button,
                WidgetType::Text,
                WidgetType::TextInput,
                WidgetType::Checkbox,
                WidgetType::Radio,
                WidgetType::Slider,
                WidgetType::ProgressBar,
                WidgetType::Toggler,
                WidgetType::PickList,
                WidgetType::Space,
                WidgetType::Rule,
            ]
        };
        
        if available_types.is_empty() {
            column![
                text("Add Child Widget").size(14),
                text("Root container can only have one child").size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
            ].spacing(5).into()
        } else {
            column![
                text("Add Child Widget").size(14),
                pick_list(
                    available_types,
                    None::<WidgetType>,
                    move |widget_type| Message::AddChild(parent_id, widget_type),
                )
            ].spacing(5).into()
        }
    }
    
    fn build_preview_panel(&self) -> Element<Message> {
        let widget_preview = self.build_widget_preview(self.hierarchy.root());
        
        column![
            text("Preview").size(20),
            text("This represents your app's main content container")
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            horizontal_rule(5),
            container(widget_preview)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|theme: &Theme| {
                    container::Style {
                        background: Some(Background::Color(Color::WHITE)),
                        border: Border {
                            color: theme.extended_palette().background.strong.color,
                            width: 2.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .padding(10),
        ]
        .spacing(10)
        .padding(40)
        .into()
    }
    
    fn build_widget_preview<'a>(&'a self, widget: &'a Widget) -> Element<'a, Message> {
        match widget.widget_type {
            WidgetType::Container => {
                let props = &widget.properties;
                let mut content = column![];
                
                if widget.children.is_empty() {
                    content = content.push(text("Container Content"));
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }
                
                return self.with_explain_overlay(content.into(), props);
            }
            
            WidgetType::Row => {
                let props = &widget.properties;
                let mut content = row![]
                    .spacing(props.spacing)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .align_y(props.align_items);
                
                if widget.children.is_empty() {
                    content = content.push(text("Row Item 1"));
                    content = content.push(text("Row Item 2"));
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }
                
                return self.with_explain_overlay(content.into(), props);
            }
            
            WidgetType::Column => {
                let props = &widget.properties;
                let mut content = column![]
                    .spacing(props.spacing)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .align_x(props.align_items);
                
                if widget.children.is_empty() {
                    content = content.push(text("Column Item 1"));
                    content = content.push(text("Column Item 2"));
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }

                return self.with_explain_overlay(content.into(), props);
            }
            
            WidgetType::Button => {
                let props = &widget.properties;
                let content = button(text(&props.text_content))
                    .on_press(Message::ButtonPressed(widget.id))
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .style(match props.button_style {
                        ButtonStyleType::Primary => button::primary,
                        ButtonStyleType::Secondary => button::secondary,
                        ButtonStyleType::Success => button::success,
                        ButtonStyleType::Danger => button::danger,
                        ButtonStyleType::Text => button::text,
                    });
                
                return self.with_explain_overlay(content.into(), props);
            }
            
            WidgetType::Text => {
                let props = &widget.properties;
                let content = text(&props.text_content)
                    .width(props.width)
                    .height(props.height)
                    .size(props.text_size)
                    .color(props.text_color)
                    .font(match props.font {
                        FontType::Default => Font::default(),
                        FontType::Monospace => Font::MONOSPACE,
                    });

                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::TextInput => {
                let props = &widget.properties;
                let content = text_input(&props.text_input_placeholder, &props.text_input_value)
                    .on_input(|value| Message::TextInputChanged(widget.id, value))
                    .size(props.text_input_size)
                    .padding(props.text_input_padding)
                    .width(props.width)
                    .secure(props.is_secure);
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Checkbox => {
                let props = &widget.properties;
                let content = checkbox(&props.checkbox_label, props.checkbox_checked)
                    .size(props.checkbox_size)
                    .spacing(props.checkbox_spacing)
                    .width(props.width)
                    .on_toggle(|_| Message::CheckboxToggled(widget.id, !props.checkbox_checked));
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Radio => {
                let props = &widget.properties;
                let content: Element<_> = if !props.radio_options.is_empty() {
                    column(
                        props.radio_options.iter().enumerate().map(|(i, option)| {
                            radio(
                                option,
                                i,
                                Some(props.radio_selected_index),
                                move |selected_index| Message::RadioSelected(widget.id, selected_index)
                            )
                            .size(props.radio_size)
                            .spacing(props.radio_spacing)
                            .into()
                        }).collect::<Vec<Element<Message>>>()
                    )
                    .spacing(5)
                    .width(props.width)
                    .height(props.height)
                    .into()
                } else {
                    text("No radio options").into()
                };

                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Slider => {
                let props = &widget.properties;

                let content = column![
                    slider(props.slider_min..=props.slider_max, props.slider_value, move |value| {
                        Message::SliderChanged(widget.id, value)
                    })
                        .step(props.slider_step)
                        .width(200),
                    text(format!("{:.1}", props.slider_value)).size(12).center(),
                ]
                .width(props.width)
                .height(props.height);
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::ProgressBar => {
                let props = &widget.properties;

                let mut content = progress_bar(props.progress_min..=props.progress_max, props.progress_value)
                    .length(props.progress_length)
                    .girth(props.progress_girth);

                if props.progress_vertical {
                    content = content.vertical();
                }

                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Toggler => {
                let props = &widget.properties;
                let content = toggler(props.toggler_active)
                    .on_toggle(|_| Message::TogglerToggled(widget.id, !props.toggler_active))
                    .size(props.toggler_size)
                    .spacing(props.toggler_spacing)
                    .width(props.width);
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::PickList => {
                let props = &widget.properties;
                let content = pick_list(
                    props.picklist_options.clone(),
                    props.picklist_selected.clone(),
                    |selected| Message::PickListSelected(widget.id, selected)
                )
                .placeholder(&props.picklist_placeholder)
                .width(props.width);
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Scrollable => {
                let props = &widget.properties;
                let mut content = column![];
                
                if widget.children.is_empty() {
                    // Add some sample content for preview
                    for i in 1..=10 {
                        content = content.push(text(format!("Scrollable Item {}", i)));
                    }
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }
                
                let content = scrollable(content)
                    .width(props.width)
                    .height(props.height);
                
                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Space => {
                let props = &widget.properties;
                let s = vertical_space().width(props.width).height(props.height);

                let content: Element<_> = if props.show_widget_bounds {
                    container(s)
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color::from_rgba(0.2, 0.6, 1.0, 0.18))),
                            border: Border { color: Color::from_rgb(0.2, 0.6, 1.0), width: 1.0, radius: 2.0.into() },
                            ..Default::default()
                        })
                        .into()
                } else {
                    s.into()
                };

                return self.with_explain_overlay(content.into(), props);
            }

            WidgetType::Rule => {
                let props = &widget.properties;

                let content: Element<_> = match props.rule_orientation {
                    RuleOrientation::Horizontal => {
                        horizontal_rule(props.rule_thickness).into()
                    }
                    RuleOrientation::Vertical => {
                        vertical_rule(props.rule_thickness).into()
                    }
                };

                return self.with_explain_overlay(content.into(), props);
            }
            
            _ => {
                text(format!("{:?} preview", widget.widget_type)).into()
            }
        }
    }

    fn build_editor_for_widget_by_id(&self, widget_id: WidgetId) -> Element<Message> {
        if let Some(widget) = self.hierarchy.get_widget_by_id(widget_id) {
            self.build_editor_for_widget(widget, widget_id)
        } else {
            text("Widget not found").into()
        }
    }
    
    fn build_editor_for_widget(&self, widget: &Widget, widget_id: WidgetId) -> Element<Message> {
        let controls_view: Element<Message> = match widget.widget_type {
            WidgetType::Container  => container_controls(&self.hierarchy, widget_id),
            WidgetType::Row       => row_controls(&self.hierarchy, widget_id),
            WidgetType::Column    => column_controls(&self.hierarchy, widget_id),
            WidgetType::Button    => button_controls(&self.hierarchy, widget_id),
            WidgetType::Text      => text_controls(&self.hierarchy, widget_id),
            WidgetType::TextInput => text_input_controls(&self.hierarchy, widget_id),
            WidgetType::Checkbox  => checkbox_controls(&self.hierarchy, widget_id),
            WidgetType::Radio     => radio_controls(&self.hierarchy, widget_id),
            WidgetType::Toggler   => toggler_controls(&self.hierarchy, widget_id),
            WidgetType::PickList  => picklist_controls(&self.hierarchy, widget_id),
            WidgetType::Slider     => slider_controls(&self.hierarchy, widget_id),
            WidgetType::Rule       => rule_controls(&self.hierarchy, widget_id),
            WidgetType::Scrollable => scrollable_controls(&self.hierarchy, widget_id),
            WidgetType::Space => space_controls(&self.hierarchy, widget_id),
            WidgetType::ProgressBar => progress_controls(&self.hierarchy, widget_id),
            _ => column![text("Editor not implemented for this widget type")].into(),
        };

        let explain_btn = button("Explain")
            .on_press(Message::Explain(widget_id))
            .style(button::secondary);

        column![
            row![
                text(format!("Editing: {}", widget.name)).size(20),
                horizontal_space(),
                explain_btn,
            ],
            horizontal_rule(5),
            controls_view,
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn with_explain_overlay<'a>(
        &self,
        inner: Element<'a, Message>,
        props: &Properties,
    ) -> Element<'a, Message> {
        if props.show_widget_bounds {
            container(inner)
                .padding(4)
                .style(|theme: &Theme| {
                    let c = theme.extended_palette().primary.strong.color;
                    container::Style {
                        background: Some(Background::Color(Color::from_rgba(c.r, c.g, c.b, 0.06))),
                        border: Border { color: c, width: 2.0, radius: 6.0.into() },
                        ..Default::default()
                    }
                })
                .into()
        } else {
            inner
        }
    }

    fn debug_print_widget(&self, widget: &Widget, depth: usize) {
        println!("{}- {:?} (id: {:?}, children: {})", 
            "  ".repeat(depth), 
            widget.widget_type, 
            widget.id,
            widget.children.len()
        );
        for child in &widget.children {
            self.debug_print_widget(child, depth + 1);
        }
    }

    pub fn theme(&self, theme: Theme) -> Theme {
        theme
    }
}

// ============================================================================
// MESSAGE TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    // Tree Hierarchy
    TreeAction,
    TreeMove { 
        id: WidgetId, 
        new_parent: WidgetId, 
        new_index: usize 
    },
    TreeDrop {
        dragged_id: WidgetId,
        target_id: WidgetId,
        position: DropPosition,
    },

    // Widget Operations
    SelectWidget(WidgetId),
    DeleteWidget(WidgetId),
    AddChild(WidgetId, WidgetType),
    PropertyChanged(WidgetId, PropertyChange),
    SwapKind(WidgetId),

    // Interactive widget messages
    ButtonPressed(WidgetId),
    TextInputChanged(WidgetId, String),
    CheckboxToggled(WidgetId, bool),
    RadioSelected(WidgetId, usize),
    SliderChanged(WidgetId, f32),
    TogglerToggled(WidgetId, bool),
    PickListSelected(WidgetId, String),

    // Visual Helpers
    ShowPaddingToggled(bool),
    ShowSpacingToggled(bool),
    ShowBordersToggled(bool),
    Explain(WidgetId),
    ExplainTimeout(WidgetId),

    // Theme, not sure I'm going to implement this with the theme builder in the same app
    ThemeChanged(Theme),
}

pub enum Action {
    Run(iced::Task<Message>),
    None,
}

// ============================================================================
// WIDGET STRUCTURES - Keep as-is, these are good
// ============================================================================

#[derive(Debug, Clone)]
pub struct Widget {
    pub id: WidgetId,
    pub widget_type: WidgetType,
    pub name: String,
    pub properties: Properties,
    pub children: Vec<Widget>,
}

impl Widget {
    fn new(widget_type: WidgetType, id: WidgetId) -> Self {
        Self {
            id,
            widget_type,
            name: format!("{:?}", widget_type),
            properties: Properties::for_widget_type(widget_type),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Container,
    Row,
    Column,
    Button,
    Text,
    TextInput,
    Checkbox,
    Radio,
    Slider,
    ProgressBar,
    Toggler,
    PickList,
    Scrollable,
    Space,
    Rule,
}

// ============================================================================
// HELPER FUNCTIONS - Keep these, they're useful
// ============================================================================

fn parse_length(value: &str) -> Length {
    match value.to_lowercase().as_str() {
        "fill" => Length::Fill,
        "shrink" => Length::Shrink,
        _ => {
            if let Ok(pixels) = value.parse::<f32>() {
                Length::Fixed(pixels)
            } else if value.ends_with("px") {
                if let Ok(pixels) = value[..value.len()-2].parse::<f32>() {
                    Length::Fixed(pixels)
                } else {
                    Length::Shrink
                }
            } else {
                Length::Shrink
            }
        }
    }
}

pub fn length_to_string(length: Length) -> String {
    match length {
        Length::Fill => "Fill".to_string(),
        Length::Shrink => "Shrink".to_string(),
        Length::Fixed(pixels) => format!("{}", pixels),
        Length::FillPortion(p) => format!("FillPortion({p})"),
        _ => "Shrink".to_string(),
    }
}

fn can_have_children(widget_type: &WidgetType) -> bool {
    matches!(
        widget_type,
        WidgetType::Container | WidgetType::Row | WidgetType::Column | WidgetType::Scrollable
    )
}

// Helper function to get widget type icon for tree display
pub fn get_widget_icon(widget_type: WidgetType) -> &'static str {
    match widget_type {
        WidgetType::Container => "📦",
        WidgetType::Row => "↔️",
        WidgetType::Column => "↕️",
        WidgetType::Button => "🔘",
        WidgetType::Text => "📝",
        WidgetType::TextInput => "📝",
        WidgetType::Checkbox => "☑️",
        WidgetType::Radio => "🔘",
        WidgetType::Slider => "🎛️",
        WidgetType::ProgressBar => "📊",
        WidgetType::Toggler => "🔀",
        WidgetType::PickList => "📋",
        WidgetType::Scrollable => "📜",
        WidgetType::Space => "⬜",
        WidgetType::Rule => "➖",
    }
}

#[derive(Debug, Clone)]
pub struct Properties {
    pub width: Length,
    pub height: Length,
    pub padding: Padding,
    
    // Container properties
    pub align_x: ContainerAlignX,
    pub align_y: ContainerAlignY,
    pub border_width: f32,
    pub border_radius: f32,
    pub border_color: Color,
    pub background_color: Color,
    pub has_shadow: bool,
    pub shadow_offset: Vector,
    pub shadow_blur: f32,
    pub shadow_color: Color,
    
    // Layout properties (Row/Column)
    pub spacing: f32,
    pub align_items: Alignment,
    
    // Text properties
    pub text_content: String,
    pub text_size: f32,
    pub text_color: Color,
    pub font: FontType,
    
    // Button properties
    pub button_style: ButtonStyleType,
    
    // TextInput properties
    pub text_input_value: String,
    pub text_input_placeholder: String,
    pub text_input_size: f32,
    pub text_input_padding: f32,
    pub is_secure: bool,
    
    // Checkbox properties
    pub checkbox_checked: bool,
    pub checkbox_label: String,
    pub checkbox_size: f32,
    pub checkbox_spacing: f32,
    
    // Radio properties
    pub radio_selected_index: usize,
    pub radio_options: Vec<String>,
    pub radio_label: String,
    pub radio_size: f32,
    pub radio_spacing: f32,
    
    // Slider properties
    pub slider_value: f32,
    pub slider_min: f32,
    pub slider_max: f32,
    pub slider_step: f32,
    
    // Progress properties
    pub progress_value: f32,
    pub progress_min: f32,
    pub progress_max: f32,
    pub progress_length: Length,
    pub progress_girth: Length,
    pub progress_vertical: bool,
    
    // Toggler properties
    pub toggler_active: bool,
    pub toggler_label: String,
    pub toggler_size: f32,
    pub toggler_spacing: f32,
    
    // PickList properties
    pub picklist_selected_index: Option<usize>,
    pub picklist_selected: Option<String>,
    pub picklist_placeholder: String,
    pub picklist_options: Vec<String>,
    
    // Scrollable properties
    pub scrollable_width: f32,
    pub scrollable_height: f32,

    // Rule properties
    pub rule_orientation: RuleOrientation,
    pub rule_thickness: f32,

    pub show_widget_bounds: bool,
    pub widget_name: String,
    pub saved_height_before_scrollable: Option<Length>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            // Common defaults
            width: Length::Fill,
            height: Length::Fill,
            padding: Padding::new(5.0),
            
            // Container defaults
            border_width: 1.0,
            border_radius: 5.0,
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            background_color: Color::from_rgba(0.9, 0.9, 0.9, 1.0),
            has_shadow: false,
            shadow_offset: Vector::new(0.0, 2.0),
            shadow_blur: 5.0,
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            
            // Layout defaults
            spacing: 10.0,
            align_items: Alignment::Start,
            align_x: ContainerAlignX::Center,
            align_y: ContainerAlignY::Center,
            
            // Text defaults 
            text_size: 16.0,
            text_color: Color::BLACK,
            font: FontType::Default,
            
            // Button defaults
            button_style: ButtonStyleType::Primary,
            
            // TextInput defaults
            text_content: "Sample Text".to_string(),
            text_input_value: String::new(),
            text_input_placeholder: "Enter text...".to_string(),
            text_input_size: 16.0,
            text_input_padding: 10.0,
            is_secure: false,
            
            // Checkbox defaults
            checkbox_checked: false,
            checkbox_label: "Check me".to_string(),
            checkbox_size: 20.0,
            checkbox_spacing: 10.0,
            
            // Radio defaults
            radio_selected_index: 0,
            radio_options: vec![
                "Option 1".to_string(),
                "Option 2".to_string(), 
                "Option 3".to_string(),
            ],
            radio_label: "Radio Option".to_string(),
            radio_size: 20.0,
            radio_spacing: 10.0,
            
            // Slider defaults
            slider_value: 50.0,
            slider_min: 0.0,
            slider_max: 100.0,
            slider_step: 1.0,
            
            // Progress defaults
            progress_min: 0.0,
            progress_max: 1.0,
            progress_value: 0.5,
            progress_length: Length::Fill,          // spans available width
            progress_girth: Length::Fixed(10.0),    // 10px tall
            progress_vertical: false,               // horizontal by default
            
            // Toggler defaults
            toggler_active: false,
            toggler_label: "Toggle me".to_string(),
            toggler_size: 20.0,
            toggler_spacing: 10.0,
            
            // PickList defaults
            picklist_selected_index: None,
            picklist_selected: None,
            picklist_placeholder: "Choose an option...".to_string(),
            picklist_options: vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ],
            
            // Scrollable defaults
            scrollable_width: 300.0,
            scrollable_height: 200.0,

            // Rule defaults
            rule_orientation: RuleOrientation::Horizontal,
            rule_thickness: 5.0,

            show_widget_bounds: false,
            widget_name: String::new(),
            saved_height_before_scrollable: None,
        }
    }
}

impl Properties {
    pub fn for_widget_type(widget_type: WidgetType) -> Self {
        let mut props = Self::default();
        
        // Customize defaults based on widget type
        match widget_type {
            WidgetType::Column => {
                props.show_widget_bounds = true;
            }
            WidgetType::Row => {
                props.show_widget_bounds = true;
            }
            WidgetType::Button => {
                props.text_content = "Click Me!".to_string();
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::Text => {
                props.text_content = "Sample Text".to_string();
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::TextInput => {
                props.text_input_placeholder = "Enter text...".to_string();
                props.height = Length::Shrink;
            }
            WidgetType::Checkbox => {
                props.checkbox_label = "Check me".to_string();
                props.height = Length::Shrink;
                props.width = Length::Shrink;
            }
            WidgetType::Radio => {
                props.radio_options = vec![
                    "Radio Option 1".to_string(),
                    "Radio Option 2".to_string(),
                ];
                props.height = Length::Shrink;
                props.width = Length::Shrink;
            }
            WidgetType::Toggler => {
                props.toggler_label = "Toggle me".to_string();
                props.height = Length::Shrink;
                props.width = Length::Shrink;
            }
            WidgetType::PickList => {
                props.picklist_placeholder = "Choose an option...".to_string();
                props.height = Length::Shrink;
                props.width = Length::Shrink;
            }
            WidgetType::Slider => {
                props.height = Length::Shrink;
                props.width = Length::Shrink;
            }
            _ => {} // Use defaults for other types
        }
        
        props
    }
}

// Display implementations
impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for ContainerAlignX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for ContainerAlignY {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for RowColumnAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for ButtonStyleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for FontType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for AlignmentOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignmentOption::Start => write!(f, "Start"),
            AlignmentOption::Center => write!(f, "Center"),
            AlignmentOption::End => write!(f, "End"),
        }
    }
}

impl std::fmt::Display for RuleOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { RuleOrientation::Horizontal => write!(f, "Horizontal"),
                     RuleOrientation::Vertical   => write!(f, "Vertical"), }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignmentOption {
    Start,
    Center,
    End,
}

impl AlignmentOption {
    // Convert our wrapper TO Iced's Alignment
    fn to_alignment(self) -> Alignment {
        match self {
            AlignmentOption::Start => Alignment::Start,
            AlignmentOption::Center => Alignment::Center,
            AlignmentOption::End => Alignment::End,
        }
    }
    
    // Convert FROM Iced's Alignment to our wrapper
    fn from_alignment(alignment: Alignment) -> Self {
        match alignment {
            Alignment::Start => AlignmentOption::Start,
            Alignment::Center => AlignmentOption::Center,
            Alignment::End => AlignmentOption::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerAlignX { Left, Center, Right }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerAlignY { Top, Center, Bottom }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowColumnAlign { Start, Center, End }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonStyleType { Primary, Secondary, Success, Danger, Text }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontType { Default, Monospace }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOrientation { Horizontal, Vertical }