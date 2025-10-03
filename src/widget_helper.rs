use iced::{
    alignment::{Horizontal, Vertical}, widget::{
        button, checkbox, column, container, space, pick_list, progress_bar, radio, row, scrollable, slider, text, text_input, toggler, rule, vertical_slider, Space, tooltip, svg, image, pin, stack, mouse_area, combo_box, qr_code, markdown, text_editor,
    }, Alignment, Background, Border, Color, Element, Font, Length, Padding, Shadow, Theme, Vector, ContentFit, Point
};
use std::collections::HashSet;
use uuid::Uuid;
use crate::{widget::generic_overlay::overlay_button, widget_helper::styles::stylefn_builders};
mod controls;
use controls::*;
mod styles;
mod code_generator;
mod views;
use views::type_editor::{self, TypeEditorView};
pub mod type_system;
use type_system::TypeSystem;
pub mod panegrid_dashboard;
use code_generator::{CodeGenerator, build_code_view_with_height};
use widgets::tree::{tree_handle, branch, DropInfo, DropPosition, Branch};
use iced::widget::themer;
use crate::icon;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// Unique identifier for widgets in the hierarchy
#[derive(Debug, Clone)]
pub enum PropertyChange {
    // Common properties
    WidgetName(String),
    Width(Length),
    Height(Length),

    // Draft Properties
    DraftFixedWidth(String),
    DraftFixedHeight(String),
    DraftFillPortionWidth(String),
    DraftFillPortionHeight(String),

    // Padding mode and convenience setters
    PaddingMode(PaddingMode),
    PaddingUniform(f32),       // Sets all sides to same value
    PaddingVertical(f32),      // Sets top and bottom
    PaddingHorizontal(f32),    // Sets left and right
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
    TextLineHeight(text::LineHeight),
    TextWrap(TextWrapping),
    TextShaping(TextShaping),
    TextAlignX(AlignText),
    TextAlignY(AlignmentYOption),
    
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
    SliderHeight(f32),
    SliderWidth(f32),

    // Progress properties
    ProgressValue(f32),
    ProgressMin(f32),
    ProgressMax(f32),
    ProgressLength(Length),     // main axis (width if horizontal, height if vertical)
    ProgressGirth(f32),      // thickness (height if horizontal, width if vertical)
    ProgressVertical(bool),     // orientation
    
    // Toggler properties
    TogglerActive(bool),
    TogglerLabel(String),
    TogglerSize(f32),
    TogglerSpacing(f32),
    
    // PickList properties
    PickListSelected(Option<String>),
    PickListPlaceholder(String),
    PickListOptions(Vec<String>),

    // Rule properties
    Orientation(Orientation),
    RuleThickness(f32),

    // Scrollable properties
    ScrollableDirection(iced::widget::scrollable::Direction),
    ScrollableAnchorX(iced::widget::scrollable::Anchor),
    ScrollableAnchorY(iced::widget::scrollable::Anchor),

    // Image
    ImagePath(String),
    ImageFit(ContentFitChoice),
    // Svg
    SvgPath(String),
    SvgFit(ContentFitChoice),
    // Tooltip
    TooltipText(String),
    TooltipPosition(TooltipPosition),
    TooltipGap(f32),

    // ComboBox
    ComboBoxPlaceholder(String),
    ComboBoxSelected(Option<String>),
    ComboBoxState(Vec<String>),
    ComboBoxUseOnInput(bool),
    ComboBoxUseOnOptionHovered(bool),
    ComboBoxUseOnOpen(bool),
    ComboBoxUseOnClose(bool),
    ComboBoxSize(f32),
    ComboBoxPadding(f32),
    ComboBoxEnumId(Option<Uuid>),
    
    // Markdown
    MarkdownContent(text_editor::Action),
    MarkdownTextSize(f32),
    
    // QR Code
    QRCodeData(String),
    QRCodeCellSize(f32),
    
    // Themer
    ThemerTheme(Option<Theme>),

    //Do Nothing
    Noop
}

// Helper function to apply property changes
pub fn apply_property_change(properties: &mut Properties, change: PropertyChange, type_system: &TypeSystem) {
    match change {
        PropertyChange::Width(value) => {
            properties.width = value;
            properties.draft_fixed_width.clear();
            properties.draft_fill_portion_width.clear();
        }
        
        PropertyChange::Height(value) => {
            properties.height = value;
            properties.draft_fixed_height.clear();
            properties.draft_fill_portion_height.clear();
        }
        PropertyChange::AlignItems(value) => properties.align_items = value,

        PropertyChange::DraftFixedWidth(text) => {
            properties.draft_fixed_width = text.clone();
            if let Ok(px) = text.trim().parse::<f32>() {
                if px >= 0.0 {
                    properties.width = Length::Fixed(px);
                }
            }
        }
        PropertyChange::DraftFixedHeight(text) => {
            properties.draft_fixed_height = text.clone();
            if let Ok(px) = text.trim().parse::<f32>() {
                if px >= 0.0 {
                    properties.height = Length::Fixed(px);
                }
            }
        }
        PropertyChange::DraftFillPortionWidth(text) => {
            properties.draft_fill_portion_width = text.clone();
            if let Ok(p) = text.trim().parse::<u16>() {
                if p >= 1 {
                    properties.width = Length::FillPortion(p);
                }
            }
        }
        PropertyChange::DraftFillPortionHeight(text) => {
            properties.draft_fill_portion_height = text.clone();
            if let Ok(p) = text.trim().parse::<u16>() {
                if p >= 1 {
                    properties.height = Length::FillPortion(p);
                }
            }
        }

        PropertyChange::PaddingMode(mode) => {
            let current = properties.padding;
            properties.padding_mode = mode;

            match mode {
                PaddingMode::Uniform => {
                    properties.padding = Padding::new(current.top);
                }
                PaddingMode::Symmetric => {
                    properties.padding = Padding {
                        top: current.top,
                        right: current.left,
                        bottom: current.top,
                        left: current.left,
                    };
                }
                PaddingMode::Individual => {}
            }
        }
        
        PropertyChange::PaddingUniform(value) => {
            properties.padding_mode = PaddingMode::Uniform;
            properties.padding = Padding::new(value);
        }
        
        PropertyChange::PaddingVertical(value) => {
            properties.padding_mode = PaddingMode::Symmetric;
            properties.padding.top = value;
            properties.padding.bottom = value;
        }
        
        PropertyChange::PaddingHorizontal(value) => {
            properties.padding_mode = PaddingMode::Symmetric;
            properties.padding.left = value;
            properties.padding.right = value;
        }

        PropertyChange::PaddingRight(value) => {
            properties.padding.right = value;
            match properties.padding_mode {
                PaddingMode::Uniform => {
                    properties.padding = Padding::new(value);
                }
                PaddingMode::Symmetric => {
                    properties.padding.left = value;
                }
                PaddingMode::Individual => {}
            }
        }
        
        PropertyChange::PaddingBottom(value) => {
            properties.padding.bottom = value;
            match properties.padding_mode {
                PaddingMode::Uniform => {
                    properties.padding = Padding::new(value);
                }
                PaddingMode::Symmetric => {
                    properties.padding.top = value;
                }
                PaddingMode::Individual => {}
            }
        }
        
        PropertyChange::PaddingLeft(value) => {
            properties.padding.left = value;
            match properties.padding_mode {
                PaddingMode::Uniform => {
                    properties.padding = Padding::new(value);
                }
                PaddingMode::Symmetric => {
                    properties.padding.right = value;
                }
                PaddingMode::Individual => {}
            }
        }

        PropertyChange::PaddingTop(value) => {
            properties.padding.top = value;
            match properties.padding_mode {
                PaddingMode::Uniform => {
                    properties.padding = Padding::new(value);
                }
                PaddingMode::Symmetric => {
                    properties.padding.bottom = value;
                }
                PaddingMode::Individual => {}
            }
        }

        PropertyChange::Spacing(value)          => properties.spacing = value,

        PropertyChange::WidgetName(value) => properties.widget_name = value,

        PropertyChange::BorderWidth(value)  => properties.border_width = value,
        PropertyChange::BorderRadius(value) => properties.border_radius = value,
        PropertyChange::BorderColor(value)  => properties.border_color = value,

        PropertyChange::BackgroundColor(value) => properties.background_color = value,

        PropertyChange::TextContent(value)          => properties.text_content = value,
        PropertyChange::TextSize(value)             => properties.text_size = value,
        PropertyChange::TextColor(value)            => properties.text_color = value,
        PropertyChange::Font(value)                 => properties.font = value,        
        PropertyChange::TextLineHeight(line_height) => properties.line_height = line_height,
        PropertyChange::TextWrap(wrapping)          => properties.wrap = wrapping.to_wrap(),
        PropertyChange::TextShaping(shapping)       => properties.shaping = shapping.to_shaping(),
        PropertyChange::TextAlignX(alignment)       => properties.text_align_x = alignment.to_alignment().into(),
        PropertyChange::TextAlignY(alignment)       => properties.text_align_y = alignment.to_alignment(),

        PropertyChange::ButtonStyle(value) => properties.button_style = value,
        
        // TextInput properties
        PropertyChange::TextInputValue(value)       => properties.text_input_value = value,
        PropertyChange::TextInputPlaceholder(value) => properties.text_input_placeholder = value,
        PropertyChange::TextInputSize(value)        => properties.text_input_size = value,
        PropertyChange::TextInputPadding(value)     => properties.text_input_padding = value,
        PropertyChange::IsSecure(value)             => properties.is_secure = value,
        
        // Checkbox properties
        PropertyChange::CheckboxChecked(value)  => properties.checkbox_checked = value,
        PropertyChange::CheckboxLabel(value)    => properties.checkbox_label = value,
        PropertyChange::CheckboxSize(value)     => properties.checkbox_size = value,
        PropertyChange::CheckboxSpacing(value)  => properties.checkbox_spacing = value,

        // Slider properties
        PropertyChange::SliderValue(value)  => properties.slider_value = value,
        PropertyChange::SliderMin(value)    => properties.slider_min = value,
        PropertyChange::SliderMax(value)    => properties.slider_max = value,
        PropertyChange::SliderStep(value)   => properties.slider_step = value,
        PropertyChange::SliderHeight(value) => properties.slider_height = value,
        PropertyChange::SliderWidth(value)  => properties.slider_width = value,
        
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
        PropertyChange::RadioLabel(value)   => properties.radio_label = value,
        PropertyChange::RadioSize(value)    => properties.radio_size = value,
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
        PropertyChange::ProgressGirth(len)  => properties.progress_girth = len,
        PropertyChange::ProgressVertical(v) => properties.progress_vertical = v,
        
        // Toggler properties
        PropertyChange::TogglerActive(value)    => properties.toggler_active = value,
        PropertyChange::TogglerLabel(value)     => properties.toggler_label = value,
        PropertyChange::TogglerSize(value)      => properties.toggler_size = value,
        PropertyChange::TogglerSpacing(value)   => properties.toggler_spacing = value,
        
        // PickList properties
        PropertyChange::PickListSelected(value)     => properties.picklist_selected = value,
        PropertyChange::PickListPlaceholder(value)  => properties.picklist_placeholder = value,
        PropertyChange::PickListOptions(value)      => properties.picklist_options = value,

        // Rule properties
        PropertyChange::RuleThickness(v)   => properties.rule_thickness  = v,

        //Rule + Space properties
        PropertyChange::Orientation(v) => properties.orientation = v,

        // Scrollable properties
        PropertyChange::ScrollableDirection(value)  => properties.scroll_dir = value,
        PropertyChange::ScrollableAnchorX(value)    => properties.anchor_x = value,
        PropertyChange::ScrollableAnchorY(value)    => properties.anchor_y = value,

        // Image properties
        PropertyChange::ImagePath(v)        => properties.image_path = v,
        PropertyChange::ImageFit(v)         => properties.image_fit = v,

        // Svg properties
        PropertyChange::SvgPath(v)          => properties.svg_path = v,
        PropertyChange::SvgFit(v)           => properties.svg_fit = v,

        // Tooltip properties
        PropertyChange::TooltipText(v)      => properties.tooltip_text = v,
        PropertyChange::TooltipPosition(v)  => properties.tooltip_position = v,
        PropertyChange::TooltipGap(v)       => properties.tooltip_gap = v,

        PropertyChange::ComboBoxSelected(v) => properties.combobox_selected = v,
        PropertyChange::ComboBoxPlaceholder(v) => properties.combobox_placeholder = v,
        PropertyChange::ComboBoxState(v) => {
            properties.combobox_options = v.clone();
            // Recreate state with new options
            properties.combobox_state = combo_box::State::new(v);
        }
        PropertyChange::ComboBoxUseOnInput(v) => properties.combobox_use_on_input = v,
        PropertyChange::ComboBoxUseOnOptionHovered(v) => properties.combobox_use_on_option_hovered = v,
        PropertyChange::ComboBoxUseOnOpen(v) => properties.combobox_use_on_open = v,
        PropertyChange::ComboBoxUseOnClose(v) => properties.combobox_use_on_close = v,
        PropertyChange::ComboBoxSize(v) => properties.combobox_size = v,
        PropertyChange::ComboBoxPadding(v) => properties.combobox_padding = v,
        PropertyChange::ComboBoxEnumId(id) => {
            //Set referenced_enum Id
            properties.referenced_enum = id;

            //Update combo_box state from Enum
            let state = if let Some(ref enum_id) = properties.referenced_enum {
                if let Some(enum_def) = type_system.get_enum(enum_id.clone()) {
                    let variants: Vec<String> = enum_def.variants.iter()
                        .map(|v| v.name.clone())
                        .collect();

                    combo_box::State::new(variants)                  
                } else { combo_box::State::new(vec![])}
            } else { combo_box::State::new(vec![])};

            properties.combobox_state = state;
        }
        
        PropertyChange::MarkdownContent(action) => {
                let is_edit = action.is_edit();

                properties.markdown_source.perform(action);

                if is_edit {
                    properties.markdown_content = markdown::Content::parse(&properties.markdown_source.text()).items().to_vec();
                }
        },
        PropertyChange::MarkdownTextSize(v) => properties.markdown_text_size = v,
        
        PropertyChange::QRCodeData(v) => properties.qrcode_data = v,
        PropertyChange::QRCodeCellSize(v) => properties.qrcode_cell_size = v,
        
        PropertyChange::ThemerTheme(v) => properties.themer_theme = v,

        PropertyChange::Noop => {},
        
        _ => {} // Placeholder for properties not implemented
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash,)]
pub struct WidgetId(pub usize);

/// Central widget hierarchy manager - Simplified to use only IDs
#[derive(Debug, Clone,)]
pub struct WidgetHierarchy {
    root: Widget,
    selected_ids: HashSet<WidgetId>,
    next_id: usize,
    common_properties: Option<CommonProperties>,
}

impl WidgetHierarchy {
    pub fn new(root_type: WidgetType) -> Self {
        let mut selected_ids = HashSet::new();
        selected_ids.insert(WidgetId(0)); // Start with root selected

        Self {
            root: Widget::new(root_type, WidgetId(0)),
            selected_ids,
            next_id: 1,
            common_properties: None
        }
    }
    
    pub fn root(&self) -> &Widget {
        &self.root
    }
    
    pub fn selected_ids(&self) -> &HashSet<WidgetId> {
        &self.selected_ids
    }

    pub fn set_selected_ids(&mut self, ids: HashSet<WidgetId>) {
        // Filter to only valid IDs
        self.selected_ids = ids.into_iter()
            .filter(|id| self.widget_exists(*id))
            .collect();
        
        self.common_properties = Some(self.get_common_properties());
    }
    
    pub fn get_single_selected(&self) -> Option<&Widget> {
        if self.selected_ids.len() == 1 {
            let id = self.selected_ids.iter().next()?;
            self.get_widget_by_id(*id)
        } else {
            None
        }
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
        if !can_have_children(&parent.widget_type) { return false; }

        if parent_id == self.root.id {
            return parent.children.is_empty()
                && matches!(widget_type, WidgetType::Column | WidgetType::Row);
        }

        match parent.widget_type {
            WidgetType::Scrollable => {
                if !parent.children.is_empty() { return false; }
                matches!(widget_type, WidgetType::Column | WidgetType::Row | WidgetType::Container)
            }
            WidgetType::Container => parent.children.is_empty(),
            WidgetType::Tooltip   => parent.children.len() < 2, // <= 2 children
            _ => true,
        }
    } else { false }
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

        // Check if parent is under a scrollable and handle accordingly
        if let Some((_, scroll_dir)) = self.get_scrollable_ancestor_info(parent_id) {
            let should_block_height = match scroll_dir {
                iced::widget::scrollable::Direction::Vertical(_) => true,
                iced::widget::scrollable::Direction::Both { .. } => true,
                iced::widget::scrollable::Direction::Horizontal(_) => false,
            };
            
            let should_block_width = match scroll_dir {
                iced::widget::scrollable::Direction::Horizontal(_) => true,
                iced::widget::scrollable::Direction::Both { .. } => true,
                iced::widget::scrollable::Direction::Vertical(_) => false,
            };
            
            if should_block_height {
                let orig = child.properties.height;
                if matches!(orig, Length::Fill | Length::FillPortion(_)) {
                    child.properties.saved_height_before_scrollable = Some(orig);
                    child.properties.height = Length::Shrink;
                }
            }
            
            if should_block_width {
                let orig = child.properties.width;
                if matches!(orig, Length::Fill | Length::FillPortion(_)) {
                    child.properties.saved_width_before_scrollable = Some(orig);
                    child.properties.width = Length::Shrink;
                }
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
        
        if let Some(parent_id) = self.find_parent_id(id) {
            if let Some(parent) = self.get_widget_by_id_mut(parent_id) {
                parent.children.retain(|child| child.id != id);
                
                // Remove from selection
                self.selected_ids.remove(&id);
                
                // If nothing selected, select parent
                if self.selected_ids.is_empty() {
                    self.selected_ids.insert(parent_id);
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

    pub fn apply_property_change(&mut self, id: WidgetId, change: PropertyChange, type_system: &TypeSystem) {
        // Special handling for scrollable direction changes
        if let PropertyChange::ScrollableDirection(new_dir) = change.clone() {
            if let Some(widget) = self.get_widget_by_id_mut(id) {
                widget.properties.scroll_dir = new_dir;
            }
            // Re-sanitize the subtree with new direction
            self.sanitize_subtree_for_scrollable(id);
            return;
        }
        
        // Handle height changes under scrollable
        if let PropertyChange::Height(h) = change.clone() {
            if let Some((_, scroll_dir)) = self.get_scrollable_ancestor_info(id) {
                let should_block = match scroll_dir {
                    iced::widget::scrollable::Direction::Vertical(_) => true,
                    iced::widget::scrollable::Direction::Both { .. } => true,
                    iced::widget::scrollable::Direction::Horizontal(_) => false,
                };
                
                if should_block && matches!(h, Length::Fill | Length::FillPortion(_)) {
                    if let Some(w) = self.get_widget_by_id_mut(id) {
                        if w.properties.saved_height_before_scrollable.is_none() {
                            w.properties.saved_height_before_scrollable = Some(h);
                        }
                        w.properties.height = Length::Shrink;
                    }
                    return;
                }
            }
        }
        
        // Handle width changes under scrollable
        if let PropertyChange::Width(w) = change.clone() {
            if let Some((_, scroll_dir)) = self.get_scrollable_ancestor_info(id) {
                let should_block = match scroll_dir {
                    iced::widget::scrollable::Direction::Horizontal(_) => true,
                    iced::widget::scrollable::Direction::Both { .. } => true,
                    iced::widget::scrollable::Direction::Vertical(_) => false,
                };
                
                if should_block && matches!(w, Length::Fill | Length::FillPortion(_)) {
                    if let Some(widget) = self.get_widget_by_id_mut(id) {
                        if widget.properties.saved_width_before_scrollable.is_none() {
                            widget.properties.saved_width_before_scrollable = Some(w);
                        }
                        widget.properties.width = Length::Shrink;
                    }
                    return;
                }
            }
        }
        
        if let Some(widget) = self.get_widget_by_id_mut(id) {
            apply_property_change(&mut widget.properties, change, type_system);
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

        if matches!(new_parent_ty, WidgetType::Tooltip) {
            let count = self.get_widget_by_id(new_parent_id).unwrap().children.len();
            if count >= 2 && self.find_parent_id(id) != Some(new_parent_id) {
                return Err("Tooltip can only contain a single child".into());
            }
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
        let node = self.remove_and_return(id).ok_or("Failed to detach node")?;

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
                self.sanitize_subtree_for_scrollable(id);
                return;
            }
        }

        // If we were a Scrollable and swapped back to Container, restore subtree.
        if matches!(old_type, WidgetType::Scrollable) {
            self.restore_subtree_after_scrollable(id);
        }
    }

    // Get the scrollable direction of the nearest scrollable ancestor (if any)
    pub fn get_scrollable_ancestor_info(&self, mut id: WidgetId) -> Option<(WidgetId, iced::widget::scrollable::Direction)> {
        while let Some(parent_id) = self.find_parent_id(id) {
            if let Some(parent) = self.get_widget_by_id(parent_id) {
                if matches!(parent.widget_type, WidgetType::Scrollable) {
                    return Some((parent_id, parent.properties.scroll_dir));
                }
                id = parent_id;
            } else {
                break;
            }
        }
        None
    }

    // Force all descendants of a Scrollable to NOT fill vertically
    fn sanitize_subtree_for_scrollable(&mut self, root_scrollable_id: WidgetId) {
        // Get the scrollable's direction
        let scroll_dir = if let Some(scrollable) = self.get_widget_by_id(root_scrollable_id) {
            scrollable.properties.scroll_dir
        } else {
            return;
        };
        
        fn clamp_descendants(widget: &mut Widget, scroll_dir: iced::widget::scrollable::Direction) {
            let should_block_height = match scroll_dir {
                iced::widget::scrollable::Direction::Vertical(_) => true,
                iced::widget::scrollable::Direction::Both { .. } => true,
                iced::widget::scrollable::Direction::Horizontal(_) => false,
            };
            
            let should_block_width = match scroll_dir {
                iced::widget::scrollable::Direction::Horizontal(_) => true,
                iced::widget::scrollable::Direction::Both { .. } => true,
                iced::widget::scrollable::Direction::Vertical(_) => false,
            };
            
            // Handle height
            if should_block_height {
                match widget.properties.height {
                    Length::Fill | Length::FillPortion(_) => {
                        if widget.properties.saved_height_before_scrollable.is_none() {
                            widget.properties.saved_height_before_scrollable = Some(widget.properties.height);
                        }
                        widget.properties.height = Length::Shrink;
                    }
                    _ => {}
                }
            } else {
                // Restore height if we're not blocking it anymore
                if let Some(h) = widget.properties.saved_height_before_scrollable.take() {
                    widget.properties.height = h;
                }
            }
            
            // Handle width
            if should_block_width {
                match widget.properties.width {
                    Length::Fill | Length::FillPortion(_) => {
                        if widget.properties.saved_width_before_scrollable.is_none() {
                            widget.properties.saved_width_before_scrollable = Some(widget.properties.width);
                        }
                        widget.properties.width = Length::Shrink;
                    }
                    _ => {}
                }
            } else {
                // Restore width if we're not blocking it anymore
                if let Some(w) = widget.properties.saved_width_before_scrollable.take() {
                    widget.properties.width = w;
                }
            }
            
            // Recurse to children
            for child in &mut widget.children {
                clamp_descendants(child, scroll_dir);
            }
        }
        
        if let Some(scrollable) = self.get_widget_by_id_mut(root_scrollable_id) {
            for child in &mut scrollable.children {
                clamp_descendants(child, scroll_dir);
            }
        }
    }

    // Restore any saved heights or widths after leaving a Scrollable subtree, or when a Scrollable Direction is changed.
    fn restore_subtree_after_scrollable(&mut self, root_container_id: WidgetId) {
        fn restore(widget: &mut Widget) {
            // Restore both saved dimensions
            if let Some(h) = widget.properties.saved_height_before_scrollable.take() {
                widget.properties.height = h;
            }
            if let Some(w) = widget.properties.saved_width_before_scrollable.take() {
                widget.properties.width = w;
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

    /// Validates that all selected widgets can be wrapped together
    pub fn validate_wrapping(&self) -> Result<WidgetId, String> {
        // Need at least one widget selected
        if self.selected_ids.is_empty() {
            return Err("No widgets selected".to_string());
        }
        
        // Can't wrap root
        if self.selected_ids.contains(&self.root.id) {
            return Err("Cannot wrap root widget".to_string());
        }
        
        // All selected widgets must share the same parent
        let parent_ids: std::collections::HashSet<_> = self.selected_ids
            .iter()
            .filter_map(|&id| self.find_parent_id(id))
            .collect();
            
        if parent_ids.len() != 1 {
            return Err("Selected widgets must have the same parent".to_string());
        }
        
        let parent_id = *parent_ids.iter().next().unwrap();
        Ok(parent_id)
    }

    /// Wraps selected widgets in a new container
    pub fn wrap_selected_in_container(
        &mut self, 
        container_type: WidgetType
    ) -> Result<WidgetId, String> {
        // Validate before making any changes
        let parent_id = self.validate_wrapping()?;
        
        // Validation for tooltip - takes 2 children
        if container_type == WidgetType::Tooltip && self.selected_ids.len() != 2 {
            return Err("Tooltip takes 2 widgets selected".to_string());
        }

        // Validation for MouseArea - takes 1 child
        if container_type == WidgetType::MouseArea && self.selected_ids.len() != 1 {
            return Err("MouseArea takes 1 widget selected".to_string());
        }
        
        // CRITICAL: Extract data from self before taking mutable borrow
        let selected_ids = self.selected_ids.clone();
        let wrapper_id = WidgetId(self.next_id);
        self.next_id += 1;
        
        let parent = self.get_widget_by_id_mut(parent_id)
            .ok_or("Parent not found")?;
        
        // Find indices of selected widgets in parent's children
        let mut selected_indices: Vec<usize> = parent.children
            .iter()
            .enumerate()
            .filter(|(_, child)| selected_ids.contains(&child.id)) // Use cloned set
            .map(|(i, _)| i)
            .collect();
        
        if selected_indices.is_empty() {
            return Err("No valid widgets to wrap".to_string());
        }
        
        selected_indices.sort_unstable();  // Ensure consistent order
        
        // Extract the selected widgets (in order)
        let first_index = selected_indices[0];
        let mut widgets_to_wrap = Vec::new();
        
        // Remove in reverse order to maintain indices
        for &idx in selected_indices.iter().rev() {
            let widget = parent.children.remove(idx);
            widgets_to_wrap.push(widget);
        }
        widgets_to_wrap.reverse();
        
        let mut wrapper = Widget::new(container_type, wrapper_id);
        wrapper.children = widgets_to_wrap;
        
        parent.children.insert(first_index, wrapper);
        
        self.selected_ids.clear();
        self.selected_ids.insert(wrapper_id);
        
        Ok(wrapper_id)
    }
    
    /// Gets all widgets that are currently selected
    pub fn get_selected_widgets(&self) -> Vec<&Widget> {
        self.selected_ids
            .iter()
            .filter_map(|&id| self.get_widget_by_id(id))
            .collect()
    }
    
    /// Finds properties that are common across all selected widgets
    pub fn get_common_properties(&self) -> CommonProperties {
        let selected = self.get_selected_widgets();
        
        if selected.is_empty() {
            return CommonProperties::default();
        }
        
        // Start with all possible properties as "common"
        // Then eliminate any that aren't shared by ALL selected widgets
        CommonProperties::from_widgets(&selected)
    }
    
    /// Applies a property change to all currently selected widgets
    pub fn apply_property_to_all_selected(
        &mut self, 
        change: PropertyChange,
        type_system: &TypeSystem
    ) {
        // Clone the selected IDs to avoid borrow checker issues
        let selected_ids: Vec<WidgetId> = self.selected_ids.iter().copied().collect();
        
        for widget_id in selected_ids {
            self.apply_property_change(widget_id, change.clone(), type_system);
        }

        match change {
            // Height / Width
            PropertyChange::Height(length) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.uniform_height = Some(length);
                }
            }
            PropertyChange::Width(width) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.uniform_width = Some(width);
                }
            }
            PropertyChange::DraftFixedHeight(height)=> {
                if let Some(prop) = &mut self.common_properties {
                    prop.draft_fixed_height = height;
                }
            }
            PropertyChange::DraftFillPortionHeight(height) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.draft_fill_portion_height = height;
                }
            }
            PropertyChange::DraftFixedWidth(width) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.draft_fixed_width = width;
                }
            }
            PropertyChange::DraftFillPortionWidth(width) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.draft_fill_portion_width = width;
                }
            }

            //Spacing
            PropertyChange::Spacing(spacing) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.uniform_spacing = Some(spacing);
                }
            }

            // Padding
            PropertyChange::PaddingMode(mode) => {
                if let Some(prop) = &mut self.common_properties {
                    prop.uniform_padding_mode = Some(mode);
                }
            }
            PropertyChange::PaddingUniform(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        *padding = Padding::new(v);
                    }
                }
            }
            PropertyChange::PaddingVertical(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.top = v;
                        padding.bottom = v;
                    }
                }
            }
            PropertyChange::PaddingHorizontal(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.left = v;
                        padding.right = v;
                    }
                }
            }
            PropertyChange::PaddingTop(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.top = v;
                    }
                }
            }
            PropertyChange::PaddingBottom(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.bottom = v;
                    }
                }
            }
            PropertyChange::PaddingLeft(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.left = v;
                    }
                }
            }
            PropertyChange::PaddingRight(v) => {
                if let Some(prop) = &mut self.common_properties {
                    if let Some(padding) = &mut prop.uniform_padding {
                        padding.right = v;
                    }
                }
            }
            _ => {}
        }

    }

}

// ============================================================================
// MAIN WIDGET VISUALIZER - Simplified
// ============================================================================

pub struct WidgetVisualizer {
    hierarchy: WidgetHierarchy,
    theme: Theme,
    app_name: String,
    app_window_title: String,
    multiple_windows: bool,
    highlight_selected: bool,
    left_pane: LeftPane,
    right_pane: RightPane,
    custom_themes: stylefn_builders::CustomThemes,
    type_system: TypeSystem,
    type_editor: TypeEditorView,
}

impl Default for WidgetVisualizer {
    fn default() -> Self {
        let hierarchy = WidgetHierarchy::new(WidgetType::Container);
        Self {
            hierarchy,
            theme: Theme::Light,
            app_name: "App".to_string(),
            app_window_title: String::new(),
            multiple_windows: false,
            highlight_selected: true,
            left_pane: LeftPane::Home,
            right_pane: RightPane::Preview,
            custom_themes: stylefn_builders::CustomThemes::new(&Theme::Light),
            type_system: TypeSystem::new(),
            type_editor: TypeEditorView::new(),
        }
    }
}

impl WidgetVisualizer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TreeMove(drop_info) => {
                if let Some(target_external_id) = drop_info.target_id {
                    let target_id = WidgetId(target_external_id);
                    
                    // Convert all dragged external IDs to WidgetIds
                    let dragged_ids: Vec<WidgetId> = drop_info.dragged_ids.iter()
                        .map(|&id| WidgetId(id))
                        .collect();
                    
                    // Process moves one at a time to handle index adjustments properly
                    match drop_info.position {
                        DropPosition::Into => {
                            // Moving into a target - append each item
                            for dragged_id in dragged_ids {
                                if dragged_id != target_id {
                                    // When moving into, get the target's current child count for proper indexing
                                    let target_child_count = self.hierarchy.get_widget_by_id(target_id)
                                        .map(|w| w.children.len())
                                        .unwrap_or(0);
                                    let _ = self.hierarchy.move_widget(
                                        dragged_id, 
                                        target_id, 
                                        target_child_count
                                    );
                                }
                            }
                        }
                        DropPosition::Before | DropPosition::After => {
                            if let Some(parent_id) = self.hierarchy.find_parent_id(target_id) {
                                for dragged_id in dragged_ids {
                                    if dragged_id == target_id {
                                        continue;
                                    }
                                    
                                    // Get fresh target position each time since moves can shift indices
                                    let target_index = self.hierarchy.get_widget_by_id(parent_id)
                                        .and_then(|parent| {
                                            parent.children.iter()
                                                .position(|c| c.id == target_id)
                                        })
                                        .unwrap_or(0);
                                    
                                    let insert_index = match drop_info.position {
                                        DropPosition::Before => target_index,
                                        DropPosition::After => target_index + 1,
                                        _ => target_index,
                                    };
                                    
                                    let _ = self.hierarchy.move_widget(dragged_id, parent_id, insert_index);
                                }
                            }
                        }
                    }
                }
            }

            Message::SelectWidgets(external_ids) => {
                let widget_ids: HashSet<WidgetId> = external_ids.iter()
                    .map(|&id| WidgetId(id))
                    .collect();
                self.hierarchy.set_selected_ids(widget_ids);
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
                self.hierarchy.apply_property_change(id, change.clone(), &self.type_system);

                match self.hierarchy.get_widget_by_id(id) {
                    Some(widget) => { 
                        if widget.widget_type == WidgetType::Space {
                            match change {
                                PropertyChange::Orientation(Orientation::Horizontal) => {
                                    self.hierarchy.apply_property_change(id, PropertyChange::Width(Length::Fill), &self.type_system);
                                    self.hierarchy.apply_property_change(id, PropertyChange::Height(Length::Shrink), &self.type_system);
                                }
                                PropertyChange::Orientation(Orientation::Vertical) => {
                                    self.hierarchy.apply_property_change(id, PropertyChange::Width(Length::Shrink), &self.type_system);
                                    self.hierarchy.apply_property_change(id, PropertyChange::Height(Length::Fill), &self.type_system);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            Message::SwapKind(id) => {
                self.hierarchy.swap_kind(id);
            }

            // Interactive widget messages
            Message::ButtonPressed(_id) => {
                // For preview, we don't need to do anything special
            }
            
            Message::TextInputChanged(id, value) => {
                self.hierarchy.apply_property_change(id, PropertyChange::TextInputValue(value), &self.type_system);
            }
            
            Message::CheckboxToggled(id, checked) => {
                self.hierarchy.apply_property_change(id, PropertyChange::CheckboxChecked(checked), &self.type_system);
            }
            
            Message::RadioSelected(id, index) => {
                self.hierarchy.apply_property_change(id, PropertyChange::RadioSelectedIndex(index), &self.type_system);
            }
            
            Message::SliderChanged(id, value) => {
                self.hierarchy.apply_property_change(id, PropertyChange::SliderValue(value), &self.type_system);
            }
            
            Message::TogglerToggled(id, active) => {
                self.hierarchy.apply_property_change(id, PropertyChange::TogglerActive(active), &self.type_system);
            }
            
            Message::PickListSelected(id, index) => {
                self.hierarchy.apply_property_change(id, PropertyChange::PickListSelected(Some(index)), &self.type_system);
            }

            Message::ComboBoxOnInput(id, value) => {
                let props = &self.hierarchy.get_widget_by_id(id).unwrap().properties;
                if props.combobox_use_on_input {
                    println!("combobox {:?} input text: {}", id, value);
                }
            }
            Message::ComboBoxSelected(id, value) => {
                println!("combobox selected: {:?}", value);
                self.hierarchy.apply_property_change(id, PropertyChange::ComboBoxSelected(Some(value)), &self.type_system);
            }
            Message::ComboBoxOnOpen(id) => {
                let props = &self.hierarchy.get_widget_by_id(id).unwrap().properties;
                if props.combobox_use_on_open {
                    println!("combobox {:?} opened!", id);
                }
            }
            Message::ComboBoxOnClose(id) => {
                let props = &self.hierarchy.get_widget_by_id(id).unwrap().properties;
                if props.combobox_use_on_close {
                    println!("combobox {:?} closed!", id);
                }
            }
            Message::ComboBoxOnOptionHovered(id, options) => {
                let props = &self.hierarchy.get_widget_by_id(id).unwrap().properties;
                if props.combobox_use_on_option_hovered {
                    println!("combobox option hovered: {:?}", options);
                }
            }
            Message::Noop => {
                // Do nothing - for preview-only interactions
            }
            Message::LinkClicked(url) => { println!("url clicked: {}", url) }

            Message::GenerateFullCode => {
                // You could open this in a modal/overlay
                // For now, we'll just log it
                let mut generator = CodeGenerator::new(&self.hierarchy, self.theme.clone(), Some(&self.type_system));
                let tokens = generator.generate_app_code();
                let code = tokens.iter().map(|t| t.text.clone()).collect::<String>();
                println!("Generated Code:\n{}", code);
            }
            
            Message::CopyCode(code) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(code);
                }
            }
            

            Message::ThemeChanged(theme) => {
                self.custom_themes.theme = theme.clone();
                self.theme = theme;
            }

            Message::AppNameChanged(app_name) => {
                self.app_name = app_name;
            }

            Message::ToggleRadioLayout => { // To switch between column/row for radio widget code generation

            }
            Message::OpenHome => {
                // Should Open / Focus the Home Page
                self.left_pane = LeftPane::Home;
            }
            Message::OpenPreview => {
                // Should Open / Focus the Editor Page
                self.right_pane = RightPane::Preview
            }
            Message::OpenCodeView => {
                // Should Open / Focus the 'Full App' code Page
                self.right_pane = RightPane::Code
            }
            Message::OpenThemeEditor => {
                // Should Open / Focus a Theme Editor Page
                self.left_pane = LeftPane::Themes;
            }
            Message::OpenWidgetVisualizerSettings => {
                // Should Open / Focus the Settings Page
                self.left_pane = LeftPane::Settings;
            }

            //Settings
            Message::AppWindowTitleChanged(value) => {
                self.app_window_title = value;
            }
            Message::AppStructName2Changed(value) => {
                self.app_name = value;
            }
            Message::MultipleWindowsToggled(checked) => {
                self.multiple_windows = checked;
            }
            Message::OutlineSelectedWidgetsToggled(b) => {
                self.highlight_selected = b;
            }

            Message::ForwardThemeMessages(msg) => {
                let task = self.custom_themes.update(msg);
                return Action::Run(task.map(Message::ForwardThemeMessages));
            }

            // User Defined Enums
            Message::OpenTypeEditor => {
                self.left_pane = LeftPane::Types;
            }
            Message::TypeEditor(msg) => {
                let task = type_editor::update(msg, &mut self.type_system, &mut self.type_editor)
                    .map(Message::TypeEditor);

                return Action::Run(task);
            }

            Message::WrapSelectedInContainer(container_type) => {
                match self.hierarchy.wrap_selected_in_container(container_type) {
                    Ok(wrapper_id) => {
                        println!("Successfully wrapped widgets in {:?} with id {:?}", 
                                 container_type, wrapper_id);
                    }
                    Err(e) => {
                        println!("Failed to wrap widgets: {}", e);
                        // TODO: Show error to user (could add a status message field)
                    }
                }
            }
            
            Message::BatchPropertyChanged(change) => {
                self.hierarchy.apply_property_to_all_selected(change, &self.type_system);
            }
        }
        
        Action::None
    }
    
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let pane_selection_dock = self.build_pane_selection_dock();
        let left_panel = match self.left_pane {
            LeftPane::Home => self.build_left_panel(),
            LeftPane::Settings => self.build_settings(),
            LeftPane::Themes => self.custom_themes.view().map(Message::ForwardThemeMessages),
            LeftPane::Types => type_editor::view(&self.type_system, &self.type_editor).map(Message::TypeEditor)
        };

        let right_panel = match self.right_pane {
            RightPane::Preview => self.build_preview_panel(),
            RightPane::Code => self.build_full_code_content(),
        };
        
        row![
            pane_selection_dock, 
            left_panel, 
            right_panel
        ].into()
    }
    
    fn build_left_panel<'a>(&'a self) -> Element<'a, Message> {
        let multi_selection_ui = self.build_multi_selection_controls();

        column![
            // Header
            column![
                text("Widget Visualizer").size(24),
                rule::horizontal(5),
            ].spacing(10).align_x(Alignment::Center),
            space::horizontal().height(10),

            // Theme selector
            row![
                space::horizontal(),
                text("Theme").size(18),
                pick_list(
                    Theme::ALL,
                    Some(self.theme.clone()),
                    Message::ThemeChanged,
                ),
                space::horizontal(),
            ].width(Length::Fill).spacing(20),
            space::horizontal().height(10),
            rule::horizontal(5),
            space::horizontal().height(10),
            
            // Widget hierarchy
            column![
                text("Widget Hierarchy").size(18),
                scrollable(
                    self.widget_tree_view()
                ).height(Length::Fill),
            ].spacing(5),

            multi_selection_ui,

           if let Some(selected_widget) = self.hierarchy.get_single_selected() {
                self.build_add_child_controls(selected_widget.id)
           } else {
                self.build_add_child_controls(self.hierarchy.root.id)
           }
           
        ]
        .width(Length::Fixed(400.0))
        .padding(
            Padding {
                top: 10.0,
                right: 5.0,
                left: 5.0,
                bottom: 10.0,
            }
        )
        .into()
    }

    fn build_pane_selection_dock<'a>(&self) -> Element<'a, Message> {
        container(
                column![
                    button(icon::home().center())
                        .width(35)
                        .style(
                            if self.left_pane == LeftPane::Home {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenHome),
                    rule::horizontal(1).style(styles::rule::toolbar_rule),

                    button(icon::global().center())
                        .width(35)
                        .style(
                            if self.left_pane == LeftPane::Settings {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenWidgetVisualizerSettings),
                    rule::horizontal(1).style(styles::rule::toolbar_rule),

                    button(icon::type_icon().center())
                        .width(35)
                        .style(
                            if self.left_pane == LeftPane::Types {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenTypeEditor),
                    rule::horizontal(1).style(styles::rule::toolbar_rule),

                    button(icon::theme().center())
                        .width(35)
                        .style(
                            if self.left_pane == LeftPane::Themes {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenThemeEditor),
                    rule::horizontal(2).style(styles::rule::toolbar_rule),
                    
                    button(icon::preview().center())
                        .width(35)
                        .style(
                            if self.right_pane == RightPane::Preview {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenPreview),
                    rule::horizontal(1).style(styles::rule::toolbar_rule),

                    button(icon::code().center())
                        .width(35)
                        .style(
                            if self.right_pane == RightPane::Code {
                                styles::button::selected_text
                            } else {
                                button::text
                            }
                        )
                        .on_press(Message::OpenCodeView),

                ]
                .spacing(2.5)
                .width(Length::Fixed(45.0))
                .padding(
                    Padding {
                        top: 10.0,
                        right: 5.0,
                        bottom: 5.0,
                        left: 5.0,
                    }
                )
        )
        .into()
    }

    fn widget_tree_view<'a>(&'a self) -> Element<'a, Message> {
        self.build_tree()
    }

    fn build_tree<'a>(&'a self) -> Element<'a, Message> {
        let widget = self.hierarchy.root();
        let overlay_content = self.build_editor_for_widget(widget, widget.id);

        // Determine if this widget can be swapped and the button label
        let swap_label: Option<iced::advanced::widget::Text<'_, Theme, iced::Renderer>> = match widget.widget_type {
            WidgetType::Row        => Some(icon::swap()), 
            WidgetType::Column     => Some(icon::swap()),
            WidgetType::Container  => Some(icon::swap()),
            WidgetType::Scrollable => Some(icon::swap()),
            _ => None,
        };

        // Optional Swap button element
        let swap_button: Option<Element<Message>> = swap_label.map(|label| {
            button(label)
                .on_press(Message::SwapKind(widget.id))
                .style(button::text)
                .into()
        });

        let disabled_delete_button: Element<Message> = { // Don't allow deleting root
                    button(icon::trash())
                        .style(styles::button::cancel)
                        .into()
                };

        //let place_holder = button("  ").style(button::text);

        let mut children = Vec::new();

        for child in &widget.children {
            children.push(self.build_tree_item(child));
        }

        let root = branch(
            row![
                container(text(format!("{}", widget.name))).padding(5),
                space::horizontal(),
                swap_button,

                // Create overlay button with this widget's specific content
                overlay_button(
                    "Edit",
                    format!("Editing {}", widget.name),
                    overlay_content
                )
                .overlay_width(500.0)
                .overlay_height(750.0)
                .style(button::primary),

                disabled_delete_button
                //place_holder

            ].spacing(5)
        ).block_dragging()
        .with_children(children)
        .with_id(widget.id.0);

        let mut tree = tree_handle(
            vec!(root)
        )
        .on_drop(Message::TreeMove)
        .on_select(|selected_ids| Message::SelectWidgets(selected_ids));

        tree = tree.reset_order_state();
    
        tree.into()

    }

    fn build_tree_item(&self, widget: &Widget) -> Branch<'_, Message, Theme, iced::Renderer> {     

        let is_selected = self.hierarchy.selected_ids().contains(&widget.id);
        let selection_count = self.hierarchy.selected_ids().len();   

        let is_first_child_of_root = self.hierarchy.root().children.first()
        .map(|c| c.id == widget.id)
        .unwrap_or(false);

        // Create the overlay content for this specific widget
        let overlay_content = self.build_editor_for_widget(widget, widget.id);
        
        // Determine if this widget can be swapped and the button label
        let swap_label: Option<iced::advanced::widget::Text<'_, Theme, iced::Renderer>> = match widget.widget_type {
            WidgetType::Row        => Some(icon::swap()), 
            WidgetType::Column     => Some(icon::swap()),
            WidgetType::Container  => Some(icon::swap()),
            WidgetType::Scrollable => Some(icon::swap()),
            _ => None,
        };

        // Optional Swap button element
        let swap_button: Option<Element<Message>> = swap_label.map(|label| {
            button(label)
                .on_press(Message::SwapKind(widget.id))
                .style(button::text)
                .into()
        });

        let delete_button: Option<Element<Message>> = if widget.id.0 != 0 { // Don't allow deleting root
                    Some(button(icon::trash())
                        .on_press(Message::DeleteWidget(widget.id))
                        .style(styles::button::cancel)
                        .into())
                } else {
                    None
                };

        let edit_button: Element<Message> = if selection_count == 1 {
            // Original single-widget edit overlay
            Some(overlay_button(
                "Edit",
                format!("Editing {}", widget.name),
                self.build_editor_for_widget(widget, widget.id)
            )
            .overlay_width(500.0)
            .overlay_height(750.0)
            .style(button::primary)).into()
        } else if is_selected {
            // Show indicator that this is in batch selection
            Some(button(text("✓ Selected"))
                .style(button::secondary)).into()
        } else {
            Some(button(text("Edit"))
                .style(button::text)).into()
        };

        let mut children = Vec::new();

        for child in &widget.children {
            children.push(self.build_tree_item(child));
        }

        let branch = match widget.widget_type {
            WidgetType::Row | WidgetType::Column | WidgetType::Container | WidgetType::Scrollable | WidgetType::Tooltip => {

                let content = row![
                        container(text(format!("{}", widget.name))).padding(5),

                        space::horizontal(),

                        swap_button,

                        // Create overlay button with this widget's specific content
                        edit_button,

                        delete_button
                ].spacing(5);

                if !is_first_child_of_root {
                    branch(
                        content
                    ).with_id(widget.id.0)
                    .with_children(children)
                    .accepts_drops()
                } else {
                    branch(
                        content
                    ).with_id(widget.id.0)
                    .with_children(children)
                    .accepts_drops()
                    .block_dragging()
                }

            }
            _ => {
                let content = row![
                        container(text(format!("{}", widget.name))).padding(5),

                        space::horizontal(),

                        swap_button,

                        // Create overlay button with this widget's specific content
                        overlay_button(
                            "Edit",
                            format!("Editing {}", widget.name),
                            overlay_content
                        )
                        .overlay_width(500.0)
                        .overlay_height(750.0)
                        .style(button::primary),

                        delete_button
                ].spacing(5);
                
                if !is_first_child_of_root {
                    branch(
                        content
                    ).with_id(widget.id.0)
                } else { // Block dragging for first child of root
                    branch(
                        content
                    ).with_id(widget.id.0).block_dragging()
                }

            }
        };

        branch
    }
    
    fn build_add_child_controls<'a>(&'a self, parent_id: WidgetId) -> Element<'a, Message> {
        let parent = self.hierarchy.get_widget_by_id(parent_id);
        if parent.is_none() {
            return column![].into();
        }
        let parent = parent.unwrap();
        
        let available_types = if parent_id == self.hierarchy.root().id {
            if self.hierarchy.root().children.is_empty() {
                vec![WidgetType::Column, WidgetType::Row]
            } else {
                vec![]
            }
        } else if parent.widget_type == WidgetType::Scrollable {
            if parent.children.is_empty() {
                vec![WidgetType::Container, WidgetType::Column, WidgetType::Row]
            } else {
                vec![]
            }
        } else if parent.widget_type == WidgetType::Container {
            if parent.children.is_empty() {
                vec![
                    WidgetType::Container,
                    WidgetType::Scrollable,
                    WidgetType::Row,
                    WidgetType::Column,
                    WidgetType::Button,
                    WidgetType::Text,
                    WidgetType::TextInput,
                    WidgetType::Checkbox,
                    WidgetType::Radio,
                    WidgetType::Slider,
                    WidgetType::VerticalSlider,
                    WidgetType::ProgressBar,
                    WidgetType::Toggler,
                    WidgetType::PickList,
                    WidgetType::Space,
                    WidgetType::Rule,
                    WidgetType::Image,
                    WidgetType::Svg,
                    WidgetType::Tooltip,
                    WidgetType::ComboBox,
                    WidgetType::Markdown,
                    WidgetType::MouseArea,
                    WidgetType::Pin,
                    WidgetType::QRCode,
                ]
            } else {
                vec![]
            }
        } else if parent.widget_type == WidgetType::MouseArea {
            if parent.children.is_empty() {
                vec![
                    WidgetType::Container,
                    WidgetType::Scrollable,
                    WidgetType::Row,
                    WidgetType::Column,
                    WidgetType::Button,
                    WidgetType::Text,
                    WidgetType::TextInput,
                    WidgetType::Checkbox,
                    WidgetType::Radio,
                    WidgetType::Slider,
                    WidgetType::VerticalSlider,
                    WidgetType::ProgressBar,
                    WidgetType::Toggler,
                    WidgetType::PickList,
                    WidgetType::Space,
                    WidgetType::Rule,
                    WidgetType::Image,
                    WidgetType::Svg,
                    WidgetType::Tooltip,
                    WidgetType::ComboBox,
                    WidgetType::Markdown,
                    WidgetType::Pin,
                    WidgetType::QRCode,
                ]
            } else {
                vec![]
            }
        } else if parent.widget_type == WidgetType::Tooltip {
            if parent.children.len() < 2 {
                vec![
                    WidgetType::Container,
                    WidgetType::Scrollable,
                    WidgetType::Row,
                    WidgetType::Column,
                    WidgetType::Button,
                    WidgetType::Text,
                    WidgetType::TextInput,
                    WidgetType::Checkbox,
                    WidgetType::Radio,
                    WidgetType::Slider,
                    WidgetType::VerticalSlider,
                    WidgetType::ProgressBar,
                    WidgetType::Toggler,
                    WidgetType::PickList,
                    WidgetType::Image,
                    WidgetType::Svg,
                    WidgetType::ComboBox,
                    WidgetType::Markdown,
                    WidgetType::MouseArea,
                    WidgetType::Pin,
                    WidgetType::QRCode,
                ]
            } else {
                vec![]
            }
        } else if parent.widget_type == WidgetType::Row || parent.widget_type == WidgetType::Column {
            vec![
                WidgetType::Container,
                WidgetType::Scrollable,
                WidgetType::Row,
                WidgetType::Column,
                WidgetType::Button,
                WidgetType::Text,
                WidgetType::TextInput,
                WidgetType::Checkbox,
                WidgetType::Radio,
                WidgetType::Slider,
                WidgetType::VerticalSlider,
                WidgetType::ProgressBar,
                WidgetType::Toggler,
                WidgetType::PickList,
                WidgetType::Space,
                WidgetType::Rule,
                WidgetType::Image,
                WidgetType::Svg,
                WidgetType::Tooltip,
                WidgetType::ComboBox,
                WidgetType::Markdown,
                WidgetType::MouseArea,
                WidgetType::Pin,
                WidgetType::QRCode,
            ]
        } else {
            vec![]
        };
        
        // Use the add_widgets view and map its messages
        crate::widget_helper::views::add_widgets::view(parent_id, &available_types)
            .map(move |msg| match msg {
                crate::widget_helper::views::add_widgets::Message::SelectWidgetType(widget_type) => {
                    Message::AddChild(parent_id, widget_type)
                }
            })
    }

    fn build_preview_panel<'a>(&'a self) -> Element<'a, Message> {
        let widget_preview = self.build_widget_preview(self.hierarchy.root());

        let preview_scoped = themer(
            Some(self.theme.clone()),

            container(widget_preview)
                .width(Length::Fill)
                .height(Length::Fill)
                // Any style closures here will now see the scoped theme
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(theme.palette().background)),
                    border: Border {
                        color: theme.extended_palette().background.strong.color,
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
        )
        // Optional: set default text color / background for this scope:
        .text_color(|th| th.palette().text)
        .background(|th| Background::Color(th.palette().background));

        self.theme.extended_palette().secondary.base.text;
        
        column![
            row![
                tooltip(
                    text("Preview Layout").size(20),
                    text("This represents your app's main content container")
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6))
                        .center(),
                    tooltip::Position::Right
                ),
            ]
            .align_y(Alignment::Center)
            .padding(
                Padding {
                    top: 5.0,
                    right: 10.0,
                    bottom: 0.0,
                    left: 10.0,
                }
            )
            .spacing(20),

            rule::horizontal(5),
            space::horizontal().height(10),

            container(preview_scoped)
            .padding(5)
            .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(theme.extended_palette().background.weak.color)),
                    ..Default::default()
                }), 
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
    
    fn build_widget_preview<'a>(&'a self, widget: &'a Widget) -> Element<'a, Message> {
        let is_selected = self.hierarchy.selected_ids().contains(&widget.id);
        let props = &widget.properties;

        let content = match widget.widget_type {
            WidgetType::Container => {
                let mut content = column![];
                
                if widget.children.is_empty() {
                    content = content.push(text("Container Content"));
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }
                
                let mut container = container(content)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .align_x(match props.align_x {
                        ContainerAlignX::Left => Horizontal::Left,
                        ContainerAlignX::Center => Horizontal::Center,
                        ContainerAlignX::Right => Horizontal::Right,
                    })
                    .align_y(match props.align_y {
                        ContainerAlignY::Top => Vertical::Top,
                        ContainerAlignY::Center => Vertical::Center,
                        ContainerAlignY::Bottom => Vertical::Bottom,
                    });

                // If user sets a style, use that style, otherwise use style from themer
                container = container.style({
                    let bg = props.background_color;
                    let bw = props.border_width;
                    let br = props.border_radius;
                    let bc = props.border_color;
                    let has_shadow = props.has_shadow;
                    let sh_off = props.shadow_offset;
                    let sh_blur = props.shadow_blur;
                    let sh_col  = props.shadow_color;

                    move |_| {
                        let mut st = container::Style::default();

                        if bg.a > 0.0 {
                            st.background = Some(Background::Color(bg));
                        }

                        st.border = Border {
                            color: bc,
                            width: bw,
                            radius: br.into(),
                        };

                        if has_shadow {
                            st.shadow = Shadow {
                                color: sh_col,
                                offset: sh_off,
                                blur_radius: sh_blur,
                            };
                        }

                        st
                    }
                });

                container.into()
            }
            
            WidgetType::Row => {
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

                content.into()
            }
            
            WidgetType::Column => {
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

                content.into()
            }
            
            WidgetType::Button => {
                button(text(&props.text_content))
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
                    })
                    .into()
            }
            
            WidgetType::Text => {
                
                let mut t = text(&props.text_content)
                    .width(props.width)
                    .height(props.height)
                    .size(props.text_size)
                    .font(match props.font { FontType::Default => Font::default(), FontType::Monospace => Font::MONOSPACE });

                let user_color = props.text_color; // Only set the color if a color has been set :D
                t = t.style(move |th: &Theme| {
                    let c = if user_color.a == 0.0 { th.palette().text } else { user_color };
                    text::Style { color: Some(c) }
                });
                t = t.line_height(props.line_height);
                t = t.wrapping(match props.wrap {
                    text::Wrapping::None => text::Wrapping::None,
                    text::Wrapping::Word => text::Wrapping::Word,
                    text::Wrapping::Glyph => text::Wrapping::Glyph,
                    text::Wrapping::WordOrGlyph => text::Wrapping::WordOrGlyph,
                });
                t = t.shaping(match props.shaping {
                    text::Shaping::Basic => text::Shaping::Basic,
                    text::Shaping::Advanced => text::Shaping::Advanced,
                    text::Shaping::Auto => text::Shaping::Auto,
                });
                t = t.align_x(props.text_align_x).align_y(props.text_align_y);

                t.into()
            }

            WidgetType::TextInput => {
                text_input(&props.text_input_placeholder, &props.text_input_value)
                    .on_input(|value| Message::TextInputChanged(widget.id, value))
                    .size(props.text_input_size)
                    .padding(props.text_input_padding)
                    .width(props.width)
                    .secure(props.is_secure)
                    .into()
            }

            WidgetType::Checkbox => {
                checkbox(&props.checkbox_label, props.checkbox_checked)
                    .size(props.checkbox_size)
                    .spacing(props.checkbox_spacing)
                    .width(props.width)
                    .on_toggle(|_| Message::CheckboxToggled(widget.id, !props.checkbox_checked))
                    .into()
            }

            WidgetType::Radio => {
                if !props.radio_options.is_empty() {
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
                    .into()
                } else {
                    text("No radio options").into()
                }
            }

            WidgetType::Slider => {
                slider(props.slider_min..=props.slider_max, props.slider_value, move |value| {
                    Message::SliderChanged(widget.id, value)
                })
                .step(props.slider_step)
                .height(props.slider_height)
                .into()
            }

            WidgetType::VerticalSlider => {
                vertical_slider(props.slider_min..=props.slider_max, props.slider_value, move |value| {
                    Message::SliderChanged(widget.id, value)
                })
                .step(props.slider_step)
                .width(props.slider_width)
                .into()
            }

            WidgetType::ProgressBar => {
                let mut content = progress_bar(props.progress_min..=props.progress_max, props.progress_value)
                    .length(props.progress_length)
                    .girth(props.progress_girth);

                if props.progress_vertical {
                    content = content.vertical();
                }

                content.into()
            }

            WidgetType::Toggler => {
                toggler(props.toggler_active)
                    .on_toggle(|_| Message::TogglerToggled(widget.id, !props.toggler_active))
                    .label(&props.toggler_label)
                    .size(props.toggler_size)
                    .spacing(props.toggler_spacing)
                    .width(props.width)
                    .into()
            }

            WidgetType::PickList => {
                pick_list(
                    props.picklist_options.clone(),
                    props.picklist_selected.clone(),
                    |selected| Message::PickListSelected(widget.id, selected)
                )
                .placeholder(&props.picklist_placeholder)
                .width(props.width)
                .into()
            }

            WidgetType::Scrollable => {
                let mut content = column![];
                
                if widget.children.is_empty() {
                    for i in 1..=10 {
                        content = content.push(text(format!("Scrollable Item {}", i)));
                    }
                } else {
                    for child in &widget.children {
                        content = content.push(self.build_widget_preview(child));
                    }
                }
                
                scrollable(content)
                    .direction(props.scroll_dir)
                    .anchor_x(props.anchor_x)
                    .anchor_y(props.anchor_y)
                    .width(props.width)
                    .height(props.height)
                    .into()
            }

            WidgetType::Space => {
                let s = match props.orientation {
                    Orientation::Horizontal => space::horizontal().width(props.width).height(props.height),
                    Orientation::Vertical => space::vertical().width(props.width).height(props.height),
                };

                if props.show_widget_bounds {
                    container(s)
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color::from_rgba(0.2, 0.6, 1.0, 0.18))),
                            border: Border { color: Color::from_rgb(0.2, 0.6, 1.0), width: 1.0, radius: 2.0.into() },
                            ..Default::default()
                        })
                        .into()
                } else {
                    s.into()
                }
            }

            WidgetType::Rule => {
                match props.orientation {
                    Orientation::Horizontal => rule::horizontal(props.rule_thickness).into(),
                    Orientation::Vertical => rule::vertical(props.rule_thickness).into(),
                }
            }

            WidgetType::Image => {
                let el: Element<_> = if props.image_path.trim().is_empty() {
                    // Placeholder box when no path provided
                    container(text("🖼️ Image (no path)"))
                        .width(props.width).height(props.height)
                        .style(|_| container::Style {
                            border: Border{ color: Color::from_rgb(0.6,0.6,0.6), width: 1.0, radius: 4.0.into() },
                            background: Some(Background::Color(Color::from_rgba(0.5,0.5,0.5,0.05))),
                            ..Default::default()
                        })
                        .into()
                } else {
                    image(image::Handle::from_path(&props.image_path))
                        .content_fit(props.image_fit.into())
                        .width(props.width).height(props.height)
                        .into()
                };
                el
            }

            WidgetType::Svg => {
                let el: Element<_> = if props.svg_path.trim().is_empty() {
                    container(text("🧩 SVG (no path)"))
                        .width(props.width).height(props.height)
                        .style(|_| container::Style {
                            border: Border{ color: Color::from_rgb(0.6,0.6,0.6), width: 1.0, radius: 4.0.into() },
                            background: Some(Background::Color(Color::from_rgba(0.5,0.5,0.5,0.05))),
                            ..Default::default()
                        })
                        .into()
                } else {
                    svg(svg::Handle::from_path(&props.svg_path))
                        .content_fit(props.svg_fit.into())
                        .width(props.width).height(props.height)
                        .into()
                };
                el
            }

            WidgetType::Tooltip => {
                // child[0] = trigger (host), child[1] = popup content
                let host = {
                    let el = widget.children.get(0)
                        .map(|w| self.build_widget_preview(w))
                        .unwrap_or_else(|| text("Tooltip host").into());

                    container(el)
                        .padding(6)
                        .style(|th: &Theme| container::Style {
                            border: Border { color: th.extended_palette().primary.strong.color, width: 1.0, radius: 4.0.into() },
                            ..Default::default()
                        })
                };

                let popup = {
                    let el = widget.children.get(1)
                        .map(|w| self.build_widget_preview(w))
                        .unwrap_or_else(|| text(&props.tooltip_text).size(14).into());

                    container(el)
                        .padding(6)
                        .style(|th: &Theme| container::Style {
                            background: Some(Background::Color(th.extended_palette().background.weak.color)),
                            border: Border { color: th.extended_palette().background.strong.color, width: 1.0, radius: 4.0.into() },
                            ..Default::default()
                        })
                };

                tooltip(host, popup, props.tooltip_position.into())
                    .gap(6)
                    .padding(8)
                    .into()
            }
            
            WidgetType::ComboBox => {
                let id = widget.id;
                let on_selected = move |selected| {
                    Message::PropertyChanged(
                        id,
                        PropertyChange::ComboBoxSelected(Some(selected)),
                    )
                };

                combo_box(
                    &props.combobox_state, 
                    &props.combobox_placeholder,
                    props.combobox_selected.as_ref(), 
                    on_selected
                )
                .on_close(Message::ComboBoxOnClose(id))
                .on_input(move |search| Message::ComboBoxOnInput(id, search))
                .on_open(Message::ComboBoxOnOpen(id))
                .on_option_hovered(move |hovered| Message::ComboBoxOnOptionHovered(id, hovered))
                .into()
            }
            
            WidgetType::Markdown => {
                
                markdown::view(
                    &props.markdown_content,
                    markdown::Settings::with_text_size(
                        props.markdown_text_size,
                        self.theme.clone()
                    )
                ).map(Message::LinkClicked)
            }
            
            WidgetType::MouseArea => {
                let content = if widget.children.is_empty() {
                    container(text("Mouse Area Content"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_| container::Style {
                            border: Border { 
                                color: Color::from_rgba(0.5, 0.5, 0.5, 0.3), 
                                width: 1.0, 
                                radius: 4.0.into() 
                            },
                            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.05))),
                            ..Default::default()
                        })
                        .into()
                } else {
                    self.build_widget_preview(&widget.children[0])
                };
                
                mouse_area(content)
                    .on_press(Message::Noop)
                    .on_release(Message::Noop)
                    .on_enter(Message::Noop)
                    .on_exit(Message::Noop)
                    .into()
            }
            
            WidgetType::QRCode => {
                use iced::widget::qr_code;
                
                match qr_code::Data::new(&props.qrcode_data) {
                    Ok(data) => {
                        // Store the QR data somewhere persistent, or create inline
                        // For preview, we'll show a placeholder
                        container(
                            text(format!("QR: {}", props.qrcode_data))
                                .size(12)
                        )
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(200.0))
                        .style(|_| container::Style {
                            border: Border { 
                                color: Color::from_rgb(0.5, 0.5, 0.5), 
                                width: 1.0, 
                                radius: 4.0.into() 
                            },
                            ..Default::default()
                        })
                        .into()
                    }
                    Err(_) => {
                        text("Invalid QR data").into()
                    }
                }
            }
            
            WidgetType::Stack => {
                let mut layers = Vec::new();
                
                if widget.children.is_empty() {
                    layers.push(
                        container(text("Stack Layer 1"))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center(Length::Fill)
                            .into()
                    );
                    layers.push(
                        container(text("Stack Layer 2").color(Color::from_rgb(1.0, 0.0, 0.0)))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(20)
                            .into()
                    );
                } else {
                    for child in &widget.children {
                        layers.push(self.build_widget_preview(child));
                    }
                }
                
                stack(layers)
                    .width(props.width)
                    .height(props.height)
                    .into()
            }
            
            WidgetType::Themer => {
                let content = if widget.children.is_empty() {
                    container(text("Themed Content"))
                        .padding(10)
                        .into()
                } else {
                    let mut col = column![];
                    for child in &widget.children {
                        col = col.push(self.build_widget_preview(child));
                    }
                    col.into()
                };
                
                if let Some(theme) = &props.themer_theme {
                    themer(Some(theme.clone()), content).into()
                } else {
                    content
                }
            }

            _ => {
                text(format!("{:?} preview", widget.widget_type)).into()
            }
        };

        if is_selected && self.highlight_selected {
            content.explain(self.theme.extended_palette().primary.strong.color)
                .into()
        } else {
            content
        }
    }
    
    fn build_editor_for_widget<'a>(&'a self, widget: &Widget, widget_id: WidgetId) -> Element<'a, Message> {
        let controls_view: Element<Message> = match widget.widget_type {
            WidgetType::Container       => container_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Scrollable      => scrollable_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Row             => row_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Column          => column_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Button          => button_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Text            => text_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::TextInput       => text_input_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Checkbox        => checkbox_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Radio           => radio_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Toggler         => toggler_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::PickList        => picklist_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Slider          => slider_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::VerticalSlider  => vertical_slider_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Rule            => rule_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Space           => space_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::ProgressBar     => progress_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Image           => image_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Svg             => svg_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Tooltip         => tooltip_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::ComboBox        => combobox_controls(&self.hierarchy, widget_id, self.theme.clone(), &self.type_system),
            WidgetType::Markdown        => markdown_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::MouseArea       => mousearea_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::QRCode          => qrcode_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Stack           => stack_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Themer          => themer_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            WidgetType::Pin             => pin_controls(&self.hierarchy, widget_id, self.theme.clone(), Some(&self.type_system)),
            _ => column![text("Editor not implemented for this widget type")].into(),
        };

        column![
            text(format!("Editing: {}", widget.name)).size(20),
            rule::horizontal(5),
            controls_view,
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    /// Builds controls that appear when multiple widgets are selected
    fn build_multi_selection_controls<'a>(&'a self) -> Element<'a, Message> {
        let selected_count = self.hierarchy.selected_ids().len();
        
        if selected_count <= 1 {
            // No multi-selection controls needed
            return column![].into();
        }
        
        // Validate if wrapping is possible
        let can_wrap = self.hierarchy.validate_wrapping().is_ok();
        
        column![
            // Header showing selection count
            text(format!("{} widgets selected", selected_count))
                .size(16),
            
            rule::horizontal(2),
            
            // Wrapping controls
            text("Wrap in:").size(14),
            
            column![
                // Row wrapping button
                button(text("Row"))
                    .on_press_maybe(
                        if can_wrap {
                            Some(Message::WrapSelectedInContainer(WidgetType::Row))
                        } else {
                            None
                        }
                    )
                    .width(Length::Fill),
                
                // Column wrapping button
                button(text("Column"))
                    .on_press_maybe(
                        if can_wrap {
                            Some(Message::WrapSelectedInContainer(WidgetType::Column))
                        } else {
                            None
                        }
                    )
                    .width(Length::Fill),
                
                // MouseArea wrapping button
                button(text("MouseArea"))
                    .on_press_maybe(
                        if can_wrap {
                            Some(Message::WrapSelectedInContainer(WidgetType::MouseArea))
                        } else {
                            None
                        }
                    )
                    .width(Length::Fill),
                
                // Container wrapping (for single widget only)
                button(text("Container"))
                    .on_press_maybe(
                        if can_wrap && selected_count == 1 {
                            Some(Message::WrapSelectedInContainer(WidgetType::Container))
                        } else {
                            None
                        }
                    )
                    .width(Length::Fill),
                
                // Tooltip wrapping (requires exactly 2 widgets)
                button(text("Tooltip (2 widgets only)"))
                    .on_press_maybe(
                        if can_wrap && selected_count == 2 {
                            Some(Message::WrapSelectedInContainer(WidgetType::Tooltip))
                        } else {
                            None
                        }
                    )
                    .width(Length::Fill),
            ]
            .spacing(5),
            
            rule::horizontal(2),
            
            // Batch edit button
            text("Batch Edit:").size(14),
            
            // Create overlay for batch editing
            overlay_button(
                "Edit All Properties",
                format!("Editing {} widgets", selected_count),
                self.build_batch_editor()
            )
            .overlay_width(500.0)
            .overlay_height(750.0)
            .style(button::primary),
            
            // Show validation error if wrapping isn't possible
            if !can_wrap {
                container(
                    text("Selected widgets must have the same parent")
                        .size(12)
                        .color(Color::from_rgb(0.8, 0.3, 0.3))
                )
                .padding(5)
            } else {
                container(space::horizontal())
            }
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
    
    /// Builds the batch property editor overlay
    fn build_batch_editor<'a>(&'a self) -> Element<'a, Message> {
        batch_editor_controls(&self.hierarchy, self.theme.clone())
    }

    fn build_settings<'a>(&'a self) -> Element<'a, Message> {
        container(
            column![
                // Header
                column![
                    text("Global Settings and Defaults").size(24).center(),
                    rule::horizontal(5),
                ].spacing(10).align_x(Alignment::Center),
                Space::new().width(Length::Fill).height(10),

                row![
                    column![
                        text("App Window Title"),
                        text_input("Generated UI", &self.app_window_title)
                            .on_input(Message::AppWindowTitleChanged)
                    ]
                    .spacing(5)
                    .padding(5),
                    column![
                        text("App Struct Name"),
                        text_input("App", &self.app_name)
                            .on_input(Message::AppStructName2Changed)
                    ]
                    .spacing(5)
                    .padding(5),
                ]
                .padding(5),

                column![
                    checkbox("Multi-Windowed App", self.multiple_windows)
                        .on_toggle(Message::MultipleWindowsToggled)
                ],
                column![
                    checkbox("Highlight Selected Widgets", self.highlight_selected)
                        .on_toggle(Message::OutlineSelectedWidgetsToggled)
                ]
            ]
                .spacing(16)
        )
        .width(Length::Fixed(400.0))
        .padding(
            Padding {
                top: 10.0,
                right: 5.0,
                left: 5.0,
                bottom: 10.0,
            }
        )
        .into()

    }

    fn build_full_code_content(&self) -> Element<Message> {
        let mut generator = CodeGenerator::new(&self.hierarchy, self.theme.clone(), Some(&self.type_system));
        generator.set_app_name(self.app_name.clone());
        generator.set_window_title(self.app_window_title.clone());
        let tokens = generator.generate_app_code();
        
        // Create the full code string for copying
        let code_string: String = tokens.iter().map(|t| t.text.clone()).collect();
        
        column![
            // Header with copy button
            row![
                text("Complete Iced Application Code").size(20),
                space::horizontal(),
                tooltip(
                    button(icon::copy())
                        .style(button::text)
                        .on_press(Message::CopyCode(code_string.clone())),
                    text("Copy and paste into your main.rs")
                        .size(12),
                        tooltip::Position::Left
                ),
            ]
            .align_y(Alignment::Center)
            .padding(
                Padding {
                    top: 0.0,
                    right: 10.0,
                    bottom: 0.0,
                    left: 10.0,
                }
            )
            .spacing(20),
            
            rule::horizontal(5),
            Space::new().width(Length::Fill).height(10),
            
            container(
                scrollable(
                    build_code_view_with_height(&tokens, 0.0, self.theme.clone()) // 0.0 height == Length::Fill
                )
                .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(10)
        .padding(10)
        .into()
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

}

// ============================================================================
// MESSAGE TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    // Tree Hierarchy
    TreeMove(DropInfo),

    // Widget Operations
    SelectWidgets(HashSet<usize>),
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
    ComboBoxOnInput(WidgetId, String),
    ComboBoxSelected(WidgetId, String),
    ComboBoxOnOptionHovered(WidgetId, String),
    ComboBoxOnClose(WidgetId),
    ComboBoxOnOpen(WidgetId),
    LinkClicked(markdown::Url),
    Noop,

    // Theme, not sure I'm going to implement this with the theme builder in the same app
    ThemeChanged(Theme),

    // Code generation related messages
    GenerateFullCode,
    CopyCode(String),
    AppNameChanged(String),
    ToggleRadioLayout,

    // Pane Selection Menu
    OpenHome,
    OpenPreview,
    OpenCodeView,
    OpenThemeEditor,
    OpenWidgetVisualizerSettings,

    // Settings
    AppWindowTitleChanged(String),
    AppStructName2Changed(String),
    MultipleWindowsToggled(bool),
    OutlineSelectedWidgetsToggled(bool),

    //Send Messages to Stylefn_Builder
    ForwardThemeMessages(stylefn_builders::Message),

    // Type system messages
    TypeEditor(type_editor::Message),
    OpenTypeEditor,

    // Wrapping operations
    WrapSelectedInContainer(WidgetType),  // Wraps selection in Row/Column/MouseArea/Tooltip
    
    // Batch editing operations  
    BatchPropertyChanged(PropertyChange), // Applies property to all selected widgets
}

pub enum Action {
    Run(iced::Task<Message>),
    None,
}

// ============================================================================
// WIDGET STRUCTURES
// ============================================================================

#[derive(Debug, Clone,)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum WidgetType {
    Container,
    Scrollable,
    Row,
    Column,
    Button,
    Text,
    TextInput,
    Checkbox,
    Radio,
    Slider,
    VerticalSlider,
    ProgressBar,
    Toggler,
    PickList,
    Space,
    Rule,
    Image,
    Svg,
    Tooltip,
    ComboBox,
    Markdown,
    MouseArea,
    QRCode,
    Stack,
    Themer,
    Pin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftPane {
    Home,
    Settings,
    Themes,
    Types,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightPane {
    Preview,
    Code
}

// ============================================================================
// HELPER FUNCTIONS
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
    }
}

fn can_have_children(widget_type: &WidgetType) -> bool {
    matches!(
        widget_type,
        WidgetType::Container | WidgetType::Row | WidgetType::Column | 
        WidgetType::Scrollable | WidgetType::Tooltip | 
        WidgetType::Stack | WidgetType::Themer | WidgetType::MouseArea
    )
}


#[derive(Debug, Clone)]
pub struct Properties {
    pub width: Length,
    pub height: Length,
    pub padding: Padding,

    //draft state for text_inputs
    pub draft_fixed_width: String,
    pub draft_fixed_height: String,
    pub draft_fill_portion_width: String,
    pub draft_fill_portion_height: String,
    pub padding_mode: PaddingMode,
    
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
    pub line_height: text::LineHeight,
    pub wrap: text::Wrapping,
    pub shaping: text::Shaping,
    pub text_align_x: text::Alignment,
    pub text_align_y: iced::alignment::Vertical,
    
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
    pub slider_width: f32,
    pub slider_height: f32,
    
    // Progress properties
    pub progress_value: f32,
    pub progress_min: f32,
    pub progress_max: f32,
    pub progress_length: Length,
    pub progress_girth: f32,
    pub progress_vertical: bool,
    
    // Toggler properties
    pub toggler_active: bool,
    pub toggler_label: String,
    pub toggler_size: f32,
    pub toggler_spacing: f32,
    
    // PickList properties
    pub picklist_selected: Option<String>,
    pub picklist_placeholder: String,
    pub picklist_options: Vec<String>,
    
    // Scrollable properties
    pub scroll_dir: iced::widget::scrollable::Direction,
    pub anchor_x: iced::widget::scrollable::Anchor,
    pub anchor_y: iced::widget::scrollable::Anchor,

    // Rule properties
    pub rule_thickness: f32,

    // Rule + Space properties
    pub orientation: Orientation,

    // Image properties
    pub image_path: String,
    pub image_fit: ContentFitChoice,

    // Svg prroperties
    pub svg_path: String,
    pub svg_fit: ContentFitChoice,

    // Tooltip properties
    pub tooltip_text: String,
    pub tooltip_position: TooltipPosition,
    pub tooltip_gap: f32,

    // ComboBox properties
    pub combobox_state: combo_box::State<String>,
    pub combobox_placeholder: String,
    pub combobox_selected: Option<String>,
    pub combobox_options: Vec<String>,
    pub combobox_size: f32,
    pub combobox_padding: f32,
    pub combobox_use_on_input: bool,
    pub combobox_use_on_option_hovered: bool,
    pub combobox_use_on_open: bool,
    pub combobox_use_on_close: bool,
    pub referenced_enum: Option<Uuid>,
    
    // Markdown properties
    pub markdown_content: Vec<markdown::Item>,
    pub markdown_source: text_editor::Content,
    pub markdown_text_size: f32,

    // QR Code properties
    pub qrcode_data: String,
    pub qrcode_cell_size: f32,

    // Themer properties
    pub themer_theme: Option<Theme>,

    // Pin properties
    pub pin_point: Point,

    pub show_widget_bounds: bool,
    pub widget_name: String,
    pub saved_height_before_scrollable: Option<Length>,
    pub saved_width_before_scrollable: Option<Length>,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            // Common defaults
            width: Length::Fill,
            height: Length::Fill,
            padding: Padding::new(0.0),

            // Draft properties
            draft_fixed_width: String::new(),
            draft_fixed_height: String::new(),
            draft_fill_portion_width: String::new(),
            draft_fill_portion_height: String::new(),
            padding_mode: PaddingMode::Uniform,
            
            // Container defaults
            border_width: 1.0,
            border_radius: 5.0,
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            background_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            has_shadow: false,
            shadow_offset: Vector::new(0.0, 2.0),
            shadow_blur: 5.0,
            shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            
            // Layout defaults
            spacing: 0.0,
            align_items: Alignment::Start,
            align_x: ContainerAlignX::Left,
            align_y: ContainerAlignY::Top,
            
            // Text defaults 
            text_size: 16.0, // should be None
            text_color:  Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            font: FontType::Default,
            line_height: text::LineHeight::default(),
            wrap: text::Wrapping::default(),
            shaping: text::Shaping::default(),
            text_align_x: text::Alignment::default(),
            text_align_y: iced::alignment::Vertical::Top,
            
            // Button defaults
            button_style: ButtonStyleType::Primary,
            
            // TextInput defaults
            text_content: "Sample Text".to_string(),
            text_input_value: String::new(),
            text_input_placeholder: "Enter text...".to_string(),
            text_input_size: 16.0, // should be None
            text_input_padding: 5.0,
            is_secure: false,
            
            // Checkbox defaults
            checkbox_checked: false,
            checkbox_label: "Check me".to_string(),
            checkbox_size: 16.0,
            checkbox_spacing: 8.0,
            
            // Radio defaults
            radio_selected_index: 0,
            radio_options: vec![
                "Option 1".to_string(),
                "Option 2".to_string(), 
                "Option 3".to_string(),
            ],
            radio_label: "Radio Option".to_string(),
            radio_size: radio::Radio::<Theme>::DEFAULT_SIZE,
            radio_spacing: radio::Radio::<Theme>::DEFAULT_SPACING,
            
            // Slider defaults
            slider_value: 50.0,
            slider_min: 0.0,
            slider_max: 100.0,
            slider_step: 1.0,
            slider_height: slider::Slider::<f32, Theme>::DEFAULT_HEIGHT,
            slider_width: vertical_slider::VerticalSlider::<f32, Theme>::DEFAULT_WIDTH,
            
            
            // Progress defaults
            progress_min: 0.0,
            progress_max: 1.0,
            progress_value: 0.5,
            progress_length: Length::Fill,
            progress_girth: progress_bar::ProgressBar::<Theme>::DEFAULT_GIRTH,
            progress_vertical: false,
            
            // Toggler defaults
            toggler_active: false,
            toggler_label: "Toggle me".to_string(),
            toggler_size: toggler::Toggler::<Theme>::DEFAULT_SIZE,
            toggler_spacing: toggler::Toggler::<Theme>::DEFAULT_SIZE / 2.0,
            
            // PickList defaults
            picklist_selected: None,
            picklist_placeholder: String::new(),
            picklist_options: vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ],
            
            // Scrollable defaults
            scroll_dir: iced::widget::scrollable::Direction::default(),
            anchor_x: iced::widget::scrollable::Anchor::default(),
            anchor_y: iced::widget::scrollable::Anchor::default(),

            // Rule defaults
            rule_thickness: 5.0,

            // Rule + Space Orientation
            orientation: Orientation::Horizontal,

            // Image defaults
            image_path: String::new(),
            image_fit: ContentFitChoice::Contain,

            // Svg defaults
            svg_path: String::new(),
            svg_fit: ContentFitChoice::Contain,

            // Tooltip defaults
            tooltip_text: "Tooltip".to_string(),
            tooltip_position: TooltipPosition::Top,
            tooltip_gap: 0.0,

            // ComboBox defaults
            combobox_state: combo_box::State::new(vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ]),

            combobox_selected: None,
            combobox_placeholder: "Type to search...".to_string(),
            combobox_options: vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ],
            combobox_use_on_input: false,
            combobox_use_on_option_hovered: false,
            combobox_use_on_open: false,
            combobox_use_on_close: false,
            combobox_size: 16.0,
            combobox_padding: 5.0,
            referenced_enum: None,

            // Markdown defaults
            markdown_content: Vec::new(),
            markdown_source: text_editor::Content::with_text(""),
            markdown_text_size: 16.0,
            
            // QR Code defaults
            qrcode_data: "https://example.com".to_string(),
            qrcode_cell_size: 4.0,
            
            // Themer defaults
            themer_theme: None,

            //Pin defaults
            pin_point: Point::ORIGIN,

            show_widget_bounds: false,
            widget_name: String::new(),
            saved_height_before_scrollable: None,
            saved_width_before_scrollable: None,
        }
    }
}

impl Properties {
    pub fn for_widget_type(widget_type: WidgetType) -> Self {
        let mut props = Self::default();
        
        // Customize defaults based on widget type [ Match actual iced defaults ]
        match widget_type {
            WidgetType::Container => {
            }
            WidgetType::Scrollable => {
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::Column => {
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::Row => {
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::Button => {
                props.text_content = "Click Me!".to_string();
                props.width = Length::Shrink;
                props.height = Length::Shrink;
                props.padding_mode = PaddingMode::Individual;
                props.padding = Padding { top: 5.0, bottom: 5.0, right: 10.0, left: 10.0 };
            }
            WidgetType::Text => {
                props.text_content = "Sample Text".to_string();
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::TextInput => {
                props.text_input_placeholder = "Enter text...".to_string();
            }
            WidgetType::Checkbox => {
                props.checkbox_label = "Check me".to_string();
                props.width = Length::Shrink;
            }
            WidgetType::Radio => {
                props.radio_options = vec![
                    "Radio Option 1".to_string(),
                    "Radio Option 2".to_string(),
                ];
                props.width = Length::Shrink;
            }
            WidgetType::Toggler => {
                props.toggler_label = "Toggle me".to_string();
                props.width = Length::Shrink;
            }
            WidgetType::PickList => {
                props.padding_mode = PaddingMode::Individual;
                props.padding = Padding { top: 5.0, bottom: 5.0, right: 10.0, left: 10.0 }; // Same as button's padding
                props.width = Length::Shrink;
            }
            WidgetType::Space => {
                props.show_widget_bounds = true;
                props.height = Length::Shrink;
            }
            WidgetType::Image => {
                props.width  = Length::Shrink;
                props.height = Length::Shrink;
                props.show_widget_bounds = true;
            }
            WidgetType::Svg => {
                props.height = Length::Shrink;
                props.show_widget_bounds = true;
            }
            WidgetType::Tooltip => {
                props.width  = Length::Shrink;
                props.height = Length::Shrink;
            }

            WidgetType::Markdown => {
                props.height = Length::Shrink;
            }
            WidgetType::QRCode => {
                props.width = Length::Shrink;
                props.height = Length::Shrink;
            }
            WidgetType::ComboBox => {
                props.combobox_state = combo_box::State::new(props.combobox_options.clone());
            }
            _ => {} // Use defaults for other types
        }
        
        props
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommonProperties {
    // Track which categories of properties are common
    pub has_width_height: bool,
    pub has_padding: bool,
    pub has_spacing: bool,
    pub has_alignment: bool,
    pub has_text_properties: bool,
    pub has_border: bool,
    pub has_background: bool,
    
    // Store the actual values (if all widgets have same value)
    pub uniform_width: Option<Length>,
    pub uniform_height: Option<Length>,
    pub uniform_padding_mode: Option<PaddingMode>,
    pub uniform_padding: Option<Padding>,
    pub uniform_spacing: Option<f32>,
    pub uniform_text_size: Option<f32>,

    pub draft_fixed_width: String,
    pub draft_fixed_height: String,
    pub draft_fill_portion_width: String,
    pub draft_fill_portion_height: String,
}

impl CommonProperties {
    /// Check widgets to find common properties
    pub fn from_widgets(widgets: &[&Widget]) -> Self {
        if widgets.is_empty() {
            return Self::default();
        }
        
        // All widgets have width/height
        let has_width_height = true;
        
        // Check if all widgets have padding (containers do, text doesn't)
        let has_padding = widgets.iter().all(|w| {
            matches!(
                w.widget_type,
                WidgetType::Container | WidgetType::Button | WidgetType::Row | 
                WidgetType::Column | WidgetType::Scrollable
            )
        });
        
        // Check if all widgets have spacing (only Row/Column)
        let has_spacing = widgets.iter().all(|w| {
            matches!(w.widget_type, WidgetType::Row | WidgetType::Column)
        });
        
        // Check if all widgets have text properties
        let has_text_properties = widgets.iter().all(|w| {
            matches!(
                w.widget_type, 
                WidgetType::Text | WidgetType::Button | WidgetType::TextInput
            )
        });
        
        // Check for uniform values
        let uniform_width = Self::get_uniform_property(widgets, |w| w.properties.width);
        let uniform_height = Self::get_uniform_property(widgets, |w| w.properties.height);
        let uniform_padding_mode = Self::get_uniform_property(widgets, |w| w.properties.padding_mode);
        let uniform_padding = Self::get_uniform_property(widgets, |w| w.properties.padding);
        let uniform_spacing = if has_spacing {
            Self::get_uniform_property(widgets, |w| w.properties.spacing)
        } else {
            None
        };
        let uniform_text_size = if has_text_properties {
            Self::get_uniform_property(widgets, |w| w.properties.text_size)
        } else {
            None
        };
        
        Self {
            has_width_height,
            has_padding,
            has_spacing,
            has_alignment: false, // todo
            has_text_properties,
            has_border: false,    // todo
            has_background: false, // todo
            uniform_width,
            uniform_height,
            uniform_padding_mode,
            uniform_padding,
            uniform_spacing,
            uniform_text_size,
            draft_fixed_width: String::new(),
            draft_fixed_height: String::new(),
            draft_fill_portion_width: String::new(),
            draft_fill_portion_height: String::new(),
        }
    }
    
    /// Helper to check if all widgets have the same value for a property
    /// WHY: Generic helper reduces code duplication
    fn get_uniform_property<T, F>(widgets: &[&Widget], getter: F) -> Option<T>
    where
        T: PartialEq + Clone,
        F: Fn(&Widget) -> T,
    {
        if widgets.is_empty() {
            return None;
        }
        
        let first_value = getter(widgets[0]);
        
        if widgets.iter().all(|w| getter(w) == first_value) {
            Some(first_value)
        } else {
            None  // Values differ across widgets
        }
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

impl std::fmt::Display for AlignmentYOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignmentYOption::Top => write!(f, "Top"),
            AlignmentYOption::Center => write!(f, "Center"),
            AlignmentYOption::Bottom => write!(f, "Bottom"),
        }
    }
}

impl std::fmt::Display for TextWrapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextWrapping::None => write!(f, "None"),
            TextWrapping::Word => write!(f, "Word"),
            TextWrapping::Glyph => write!(f, "Glyph"),
            TextWrapping::WordOrGlyph => write!(f, "WordOrGlyph"),
        }
    }
}

impl std::fmt::Display for TextShaping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextShaping::Basic => write!(f, "Basic"),
            TextShaping::Advanced => write!(f, "Advanced"),
            TextShaping::Auto => write!(f, "Auto"),
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Orientation::Horizontal => write!(f, "Horizontal"),
                     Orientation::Vertical   => write!(f, "Vertical"), }
    }
}

impl std::fmt::Display for AlignText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignText::Default => write!(f, "Default"),
            AlignText::Left => write!(f, "Left"),
            AlignText::Center => write!(f, "Center"),
            AlignText::Right => write!(f, "Right"),
            AlignText::Justified => write!(f, "Justified"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,)]
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
impl From<Alignment> for AlignmentOption {
    fn from(a: Alignment) -> Self {
        match a {
            Alignment::Start => Self::Start,
            Alignment::Center => Self::Center,
            Alignment::End => Self::End,
        }
    }
}
impl From<AlignmentOption> for Alignment {
    fn from(c: AlignmentOption) -> Self {
        match c {
            AlignmentOption::Start => Self::Start,
            AlignmentOption::Center => Self::Center,
            AlignmentOption::End => Self::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum AlignmentYOption {
    Top,
    Center,
    Bottom,
}

impl AlignmentYOption {
    // Convert our wrapper TO Iced's Alignment
    fn to_alignment(self) -> iced::alignment::Vertical {
        match self {
            AlignmentYOption::Top => iced::alignment::Vertical::Top,
            AlignmentYOption::Center => iced::alignment::Vertical::Center,
            AlignmentYOption::Bottom => iced::alignment::Vertical::Bottom,
        }
    }
    
    // Convert FROM Iced's Alignment to our wrapper
    fn from_alignment(alignment: iced::alignment::Vertical) -> Self {
        match alignment {
            iced::alignment::Vertical::Top => AlignmentYOption::Top,
            iced::alignment::Vertical::Center => AlignmentYOption::Center,
            iced::alignment::Vertical::Bottom => AlignmentYOption::Bottom,
        }
    }
}
impl From<iced::alignment::Vertical> for AlignmentYOption {
    fn from(v: iced::alignment::Vertical) -> Self {
        match v {
            iced::alignment::Vertical::Top => Self::Top,
            iced::alignment::Vertical::Center => Self::Center,
            iced::alignment::Vertical::Bottom => Self::Bottom,
        }
    }
}
impl From<AlignmentYOption> for iced::alignment::Vertical {
    fn from(c: AlignmentYOption) -> Self {
        match c {
            AlignmentYOption::Top => Self::Top,
            AlignmentYOption::Center => Self::Center,
            AlignmentYOption::Bottom => Self::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum TextWrapping {
    None,
    Word,
    Glyph,
    WordOrGlyph
}

impl TextWrapping {
    fn to_wrap(self) -> text::Wrapping {
        match self {
            TextWrapping::None => text::Wrapping::None,
            TextWrapping::Word => text::Wrapping::Word,
            TextWrapping::Glyph => text::Wrapping::Glyph,
            TextWrapping::WordOrGlyph => text::Wrapping::WordOrGlyph,
        }
    }
    
    fn from_wrap(alignment: text::Wrapping) -> Self {
        match alignment {
            text::Wrapping::None => TextWrapping::None,
            text::Wrapping::Word => TextWrapping::Word,
            text::Wrapping::Glyph => TextWrapping::Glyph,
            text::Wrapping::WordOrGlyph => TextWrapping::WordOrGlyph,
        }
    }
}

impl From<text::Wrapping> for TextWrapping {
    fn from(w: text::Wrapping) -> Self {
        match w {
            text::Wrapping::None => Self::None,
            text::Wrapping::Word => Self::Word,
            text::Wrapping::Glyph => Self::Glyph,
            text::Wrapping::WordOrGlyph => Self::WordOrGlyph,
        }
    }
}
impl From<TextWrapping> for text::Wrapping {
    fn from(c: TextWrapping) -> Self {
        match c {
            TextWrapping::None => Self::None,
            TextWrapping::Word => Self::Word,
            TextWrapping::Glyph => Self::Glyph,
            TextWrapping::WordOrGlyph => Self::WordOrGlyph,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum TextShaping {
    Basic,
    Advanced,
    Auto,
}

impl TextShaping {
    fn to_shaping(self) -> text::Shaping {
        match self {
            TextShaping::Basic => text::Shaping::Basic,
            TextShaping::Advanced => text::Shaping::Advanced,
            TextShaping::Auto => text::Shaping::Auto,
        }
    }
    
    fn from_shaping(alignment: text::Shaping) -> Self {
        match alignment {
            text::Shaping::Basic => TextShaping::Basic,
            text::Shaping::Advanced => TextShaping::Advanced,
            text::Shaping::Auto => TextShaping::Auto,
        }
    }
}
impl From<text::Shaping> for TextShaping {
    fn from(s: text::Shaping) -> Self {
        match s {
            text::Shaping::Basic => Self::Basic,
            text::Shaping::Advanced => Self::Advanced,
            text::Shaping::Auto => Self::Auto,
        }
    }
}
impl From<TextShaping> for text::Shaping {
    fn from(c: TextShaping) -> Self {
        match c {
            TextShaping::Basic => Self::Basic,
            TextShaping::Advanced => Self::Advanced,
            TextShaping::Auto => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum ContainerAlignX { Left, Center, Right }

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum ContainerAlignY { Top, Center, Bottom }

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum RowColumnAlign { Start, Center, End }

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum ButtonStyleType { Primary, Secondary, Success, Danger, Text }

#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum FontType { Default, Monospace }

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum Orientation { Horizontal, Vertical }



#[derive(Debug, Clone, Copy, PartialEq,)]
pub enum AlignText {
    Default,
    Left,
    Center,
    Right,
    Justified,
}

impl AlignText {
    // Convert our wrapper TO Iced's Alignment
    fn to_alignment(self) -> iced::advanced::text::Alignment {
        match self {
            AlignText::Default => iced::advanced::text::Alignment::Default,
            AlignText::Left => iced::advanced::text::Alignment::Left,
            AlignText::Center => iced::advanced::text::Alignment::Center,
            AlignText::Right => iced::advanced::text::Alignment::Right,
            AlignText::Justified => iced::advanced::text::Alignment::Justified,
        }
    }
    
    // Convert FROM Iced's Alignment to our wrapper
    fn from_alignment(alignment: iced::advanced::text::Alignment) -> Self {
        match alignment {
            iced::advanced::text::Alignment::Default => AlignText::Default,
            iced::advanced::text::Alignment::Left => AlignText::Left,
            iced::advanced::text::Alignment::Center => AlignText::Center,
            iced::advanced::text::Alignment::Right => AlignText::Right,
            iced::advanced::text::Alignment::Justified => AlignText::Justified,
        }
    }
}
impl From<iced::advanced::text::Alignment> for AlignText {
    fn from(a: iced::advanced::text::Alignment) -> Self {
        match a {
            iced::advanced::text::Alignment::Default => Self::Default,
            iced::advanced::text::Alignment::Left => Self::Left,
            iced::advanced::text::Alignment::Center => Self::Center,
            iced::advanced::text::Alignment::Right => Self::Right,
            iced::advanced::text::Alignment::Justified => Self::Justified,
        }
    }
}
impl From<AlignText> for iced::advanced::text::Alignment {
    fn from(c: AlignText) -> Self {
        match c {
            AlignText::Default => Self::Default,
            AlignText::Left => Self::Left,
            AlignText::Center => Self::Center,
            AlignText::Right => Self::Right,
            AlignText::Justified => Self::Justified,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum DirChoice { Vertical, Horizontal, Both }
impl std::fmt::Display for DirChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self { DirChoice::Vertical => "Vertical", DirChoice::Horizontal => "Horizontal", DirChoice::Both => "Both" })
    }
}
impl DirChoice {
    fn to_choice(d: iced::widget::scrollable::Direction) -> DirChoice {
        match d {
            iced::widget::scrollable::Direction::Vertical(_) => DirChoice::Vertical,
            iced::widget::scrollable::Direction::Horizontal(_) => DirChoice::Horizontal,
            iced::widget::scrollable::Direction::Both { .. } => DirChoice::Both,
        }
    }
    fn from_choice(c: DirChoice) -> iced::widget::scrollable::Direction {
        match c {
            DirChoice::Vertical   => iced::widget::scrollable::Direction::Vertical(scrollable::Scrollbar::default()),
            DirChoice::Horizontal => iced::widget::scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
            DirChoice::Both       => iced::widget::scrollable::Direction::Both { 
                vertical: scrollable::Scrollbar::default(), 
                horizontal: scrollable::Scrollbar::default() 
            }
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum AnchorChoice { Start, End }
impl std::fmt::Display for AnchorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self { AnchorChoice::Start => "Start", AnchorChoice::End => "End" })
    }
}
impl AnchorChoice {
    fn to_anchor(d: iced::widget::scrollable::Anchor) -> AnchorChoice {
        match d {
            iced::widget::scrollable::Anchor::Start => AnchorChoice::Start,
            iced::widget::scrollable::Anchor::End => AnchorChoice::End,

        }
    }
    fn from_anchor(c: AnchorChoice) -> iced::widget::scrollable::Anchor {
        match c {
            AnchorChoice::Start   => iced::widget::scrollable::Anchor::Start,
            AnchorChoice::End => iced::widget::scrollable::Anchor::End,

        }
    }
}
impl From<iced::widget::scrollable::Anchor> for AnchorChoice {
    fn from(a: iced::widget::scrollable::Anchor) -> Self {
        match a {
            iced::widget::scrollable::Anchor::Start => Self::Start,
            iced::widget::scrollable::Anchor::End => Self::End,

        }
    }
}
impl From<AnchorChoice> for iced::widget::scrollable::Anchor {
    fn from(c: AnchorChoice) -> Self {
        match c {
            AnchorChoice::Start => Self::Start,
            AnchorChoice::End => Self::End,

        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum ContentFitChoice { Contain, Cover, Fill, ScaleDown, None }
impl std::fmt::Display for ContentFitChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ContentFitChoice::*;
        write!(f, "{}", match self { Contain=>"Contain", Cover=>"Cover", Fill=>"Fill", ScaleDown=>"ScaleDown", None=>"None" })
    }
}
impl From<ContentFit> for ContentFitChoice {
    fn from(f: ContentFit) -> Self {
        use ContentFit::*;
        match f { Contain=>Self::Contain, Cover=>Self::Cover, Fill=>Self::Fill, ScaleDown=>Self::ScaleDown, None=>Self::None }
    }
}
impl From<ContentFitChoice> for ContentFit {
    fn from(c: ContentFitChoice) -> Self {
        use ContentFitChoice::*;
        match c { Contain=>ContentFit::Contain, Cover=>ContentFit::Cover, Fill=>ContentFit::Fill, ScaleDown=>ContentFit::ScaleDown, None=>ContentFit::None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq,)]
pub enum TooltipPosition { Top, Bottom, Left, Right, FollowCursor }
impl std::fmt::Display for TooltipPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TooltipPosition::*;
        write!(f, "{}", match self { Top=>"Top", Bottom=>"Bottom", Left=>"Left", Right=>"Right", FollowCursor=>"Follow Cursor" })
    }
}
impl From<TooltipPosition> for tooltip::Position {
    fn from(p: TooltipPosition) -> Self {
        use TooltipPosition::*;
        match p { Top=>tooltip::Position::Top, Bottom=>tooltip::Position::Bottom, Left=>tooltip::Position::Left, Right=>tooltip::Position::Right, FollowCursor=>tooltip::Position::FollowCursor }
    }
}