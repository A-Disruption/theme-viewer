use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, progress_bar, radio, row, scrollable, slider, text, text_input, toggler, vertical_space, Button, Column, Container, Radio, Row, Space, Text, TextInput
    },
    Alignment, Background, Border, Color, Element, Font, Length::{self, FillPortion}, Padding, Shadow,
    Theme, Vector,
};
use std::collections::HashMap;
use crate::widget::generic_overlay::overlay_button;

// ============================================================================
// CORE DATA STRUCTURES - Simplified ID-based approach
// ============================================================================

/// Unique identifier for widgets in the hierarchy
#[derive(Debug, Clone)]
pub enum PropertyChange {
    // Common properties
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
        PropertyChange::ProgressValue(value) => properties.progress_value = value,
        
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
    
    pub fn add_child(&mut self, parent_id: WidgetId, widget_type: WidgetType) -> Result<WidgetId, String> {
        // Check if parent exists and can have children
        if let Some(parent) = self.get_widget_by_id(parent_id) {
            if !can_have_children(&parent.widget_type) {
                return Err(format!("{:?} cannot have children", parent.widget_type));
            }
        } else {
            return Err("Parent widget not found".to_string());
        }
        
        // Create new widget
        let child_id = WidgetId(self.next_id);
        self.next_id += 1;
        let child = Widget::new(widget_type, child_id);
        
        // Add to parent
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
    
    fn find_parent_id(&self, child_id: WidgetId) -> Option<WidgetId> {
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

    pub fn apply_property_change(&mut self, id: WidgetId, change: PropertyChange) {
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
            Message::SelectWidget(id) => {
                self.hierarchy.select_widget(id);
            }
            
            Message::DeleteWidget(id) => {
                let _ = self.hierarchy.delete_widget(id);
            }
            
            Message::AddChild(parent_id, widget_type) => {
                if let Ok(new_id) = self.hierarchy.add_child(parent_id, widget_type) {}
            }
            
            Message::PropertyChanged(id, change) => {
                self.hierarchy.apply_property_change(id, change);
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
        column![
            text("Add Child Widget").size(14),
            pick_list(
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
                ],
                None::<WidgetType>,
                move |widget_type| Message::AddChild(parent_id, widget_type),
            )
        ].spacing(5).into()
    }
    
    fn build_preview_panel(&self) -> Element<Message> {
        let widget_preview = self.build_widget_preview(self.hierarchy.root());
        
        column![
            text("Preview").size(20),
            text("This represents your app's main content container")
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            horizontal_rule(10),
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
                
                container(content)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .style(move |_theme: &Theme| {
                        container::Style {
                            background: Some(Background::Color(props.background_color)),
                            border: Border {
                                color: props.border_color,
                                width: props.border_width,
                                radius: props.border_radius.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .into()
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
                
                // Wrap in container for visualization
                container(content)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .style(move |_theme: &Theme| {
                        if self.show_borders {
                            container::Style {
                                background: Some(Background::Color(Color::from_rgba(0.0, 1.0, 0.0, 0.1))),
                                border: Border {
                                    color: Color::from_rgb(0.0, 0.8, 0.0),
                                    width: 1.0,
                                    radius: 2.0.into(),
                                },
                                ..Default::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    })
                    .into()
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
                
                container(content)
                    .width(props.width)
                    .height(props.height)
                    .padding(props.padding)
                    .style(move |_theme: &Theme| {
                        if self.show_borders {
                            container::Style {
                                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 1.0, 0.1))),
                                border: Border {
                                    color: Color::from_rgb(0.0, 0.0, 0.8),
                                    width: 1.0,
                                    radius: 2.0.into(),
                                },
                                ..Default::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    })
                    .into()
            }
            
            WidgetType::Button => {
                let props = &widget.properties;
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
                let props = &widget.properties;
                text(&props.text_content)
                    .width(props.width)
                    .height(props.height)
                    .size(props.text_size)
                    .color(props.text_color)
                    .font(match props.font {
                        FontType::Default => Font::default(),
                        FontType::Monospace => Font::MONOSPACE,
                    })
                    .into()
            }

            WidgetType::TextInput => {
                let props = &widget.properties;
                let input = text_input(&props.text_input_placeholder, &props.text_input_value)
                    .on_input(|value| Message::TextInputChanged(widget.id, value))
                    .size(props.text_input_size)
                    .padding(props.text_input_padding)
                    .width(props.width)
                    .secure(props.is_secure);
                
                input.into()
            }

            WidgetType::Checkbox => {
                let props = &widget.properties;
                checkbox(&props.checkbox_label, props.checkbox_checked)
                    .size(props.checkbox_size)
                    .spacing(props.checkbox_spacing)
                    .width(props.width)
                    .on_toggle(|_| Message::CheckboxToggled(widget.id, !props.checkbox_checked))
                    .into()
            }

            WidgetType::Radio => {
                let props = &widget.properties;
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
                    .spacing(5)
                    .width(props.width)
                    .height(props.height)
                    .into()
                } else {
                    text("No radio options").into()
                }
            }

            WidgetType::Slider => {
                let props = &widget.properties;

                column![
                    slider(props.slider_min..=props.slider_max, props.slider_value, move |value| {
                        Message::SliderChanged(widget.id, value)
                    })
                        .step(props.slider_step)
                        .width(200),
                    text(format!("{:.1}", props.slider_value)).size(12).center(),
                ]
                .width(props.width)
                .height(props.height)
                .into()
            }

            WidgetType::ProgressBar => {
                let props = &widget.properties;
                progress_bar(0.0..=1.0, props.progress_value)
                    .into()
            }

            WidgetType::Toggler => {
                let props = &widget.properties;
                toggler(props.toggler_active)
                    .on_toggle(|_| Message::TogglerToggled(widget.id, !props.toggler_active))
                    .size(props.toggler_size)
                    .spacing(props.toggler_spacing)
                    .width(props.width)
                    .into()
            }

            WidgetType::PickList => {
                let props = &widget.properties;
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
                
                scrollable(content)
                    .width(props.width)
                    .height(props.height)
                    .into()
            }

            WidgetType::Space => {
                let props = &widget.properties;
                vertical_space()
                    .width(props.width)
                    .height(props.height)
                    .into()
            }

/*             WidgetType::Rule => {
                let props = &widget.properties;
                // Determine if it's horizontal or vertical based on dimensions
                if matches!(props.width, Length::Fill) || 
                   (matches!(props.width, Length::Fixed(w)) if w > 50.0) {
                    // Horizontal rule
                    horizontal_rule(2)
                        .width(props.width)
                        .into()
                } else {
                    // Vertical rule  
                    vertical_space()
                        .width(Length::Fixed(2.0))
                        .height(props.height)
                        .into()
                }
            } */
            
            // Add other widget types as needed...
            _ => {
                text(format!("{:?} preview", widget.widget_type)).into()
            }
        }
    }
    
    fn build_editor_for_widget(&self, widget: &Widget, widget_id: WidgetId) -> Element<Message> {
        let controls = match widget.widget_type {
            WidgetType::Container => self.container_controls(widget_id),
            WidgetType::Row => self.row_controls(widget_id),
            WidgetType::Column => self.column_controls(widget_id),
            WidgetType::Button => self.button_controls(widget_id),
            WidgetType::Text => self.text_controls(widget_id),
            WidgetType::TextInput => self.text_input_controls(widget_id),
            WidgetType::Checkbox => self.checkbox_controls(widget_id),
            WidgetType::Radio => self.radio_controls(widget_id),
            WidgetType::Toggler => self.toggler_controls(widget_id),
            WidgetType::PickList => self.picklist_controls(widget_id),
            // Add other widget types...
            _ => column![text("Editor not implemented for this widget type")].into(),
        };

        column![
            text(format!("Editing: {}", widget.name)).size(20),
            horizontal_rule(10),
            controls,
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
    
    // Element to edit Container properties inside an Overlay
    fn container_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Container Properties").size(16),

             // Alignment controls
            row![
                column![
                    text("Horizontal Align"),
                    pick_list(
                        vec![ContainerAlignX::Left, ContainerAlignX::Center, ContainerAlignX::Right],
                        Some(props.align_x),
                        move |v| Message::PropertyChanged(widget_id, PropertyChange::AlignX(v)),
                    ),
                ]
                .spacing(5)
                .width(FillPortion(1)),
                
                column![
                    text("Vertical Align"),
                    pick_list(
                        vec![ContainerAlignY::Top, ContainerAlignY::Center, ContainerAlignY::Bottom],
                        Some(props.align_y),
                        move |v| Message::PropertyChanged(widget_id, PropertyChange::AlignY(v)),
                    ),
                ]
                .spacing(5)
                .width(FillPortion(1)),
            ].spacing(15),
            
            row![
                // Width control
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(150),
                ]
                .spacing(5)
                .width(FillPortion(1)),
                
                // Height control  
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(150),
                ]
                .spacing(5)
                .width(FillPortion(1)),
            ].spacing(15),


            // Border controls
            text("Border").size(14),
            row![
                column![
                    text("Border Width").size(12),
                    slider(0.0..=15.0, props.border_width, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::BorderWidth(v))
    }               ).step(1.0),
                    text(format!("{:.0}", props.border_width)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                column![
                    text("Border Radius").size(12),
                    slider(0.0..=15.0, props.border_radius, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::BorderRadius(v))
                    }).step(1.0),
                    text(format!("{:.0}", props.border_radius)).size(12).center(),
                ].spacing(5).width(Length::Fill),
/*                 column![
                    text("Border Color").size(12),
                ].spacing(5).width(Length::Fill), */ // will implement Color_picker widget here (maybe)
            ].spacing(15),
            
            // Padding controls
            text("Padding").size(14),
            row![
                column![
                    text("Top").size(12),
                    slider(0.0..=50.0, props.padding.top, move |v|{
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                    }) .step(1.0),
                    text(format!("{:.0}", props.padding.top)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Right").size(12),
                    slider(0.0..=50.0, props.padding.right, move |v|
                        { 
                            Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v)) 
                        }).step(1.0),
                    text(format!("{:.0}", props.padding.right)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15),
            
            row![
                column![
                    text("Bottom").size(12),
                    slider(0.0..=50.0, props.padding.bottom, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.bottom)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Left").size(12),
                    slider(0.0..=50.0, props.padding.left, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.left)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15),

            // Shadow controls
            column![
                checkbox("Enable Shadow", props.has_shadow)
                    .on_toggle( move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::HasShadow(v))
                }),
                
                if props.has_shadow {
                    column![
                        row![
                            column![
                                text("Offset X").size(12),
                                slider(-20.0..=20.0, props.shadow_offset.x, move |v| {
                                    Message::PropertyChanged(widget_id, PropertyChange::ShadowOffsetX(v))
                                })
                                    .step(1.0),
                                text(format!("{:.0}", props.shadow_offset.x)).size(12).center(),
                            ].spacing(5),
                            
                            column![
                                text("Offset Y").size(12),
                                slider(-20.0..=20.0, props.shadow_offset.y, move |v| {
                                    Message::PropertyChanged(widget_id, PropertyChange::ShadowOffsetY(v))
                                })
                                    .step(1.0),
                                text(format!("{:.0}", props.shadow_offset.y)).size(12).center(),
                            ].spacing(5),
                        ].spacing(15),
                        
                        column![
                            text("Blur Radius").size(12),
                            slider(0.0..=50.0, props.shadow_blur, move |v| {
                                Message::PropertyChanged(widget_id, PropertyChange::ShadowBlur(v))
                            })
                                .step(1.0),
                            text(format!("{:.0}", props.shadow_blur)).size(12).center(),
                        ].spacing(5),
                    ].spacing(10)
                } else {
                    column![]
                }
            ].spacing(10),
        ]
        .spacing(15)
        .into()

        
    }
    
    fn row_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Row Properties").size(16),
            
            // Spacing control
            column![
                text("Spacing between items"),
                row![
                    slider(0.0..=50.0, props.spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::Spacing(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.spacing)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Vertical alignment of items in the row
            column![
                text("Vertical Alignment"),
                pick_list(
                    vec![AlignmentOption::Start, AlignmentOption::Center, AlignmentOption::End],
                    Some(AlignmentOption::from_alignment(props.align_items)),
                    move |selected_option| {
                        Message::PropertyChanged(
                            widget_id, 
                            PropertyChange::AlignItems(selected_option.to_alignment())
                        )
                    },
                ).placeholder("Choose alignment"),
            ].spacing(5),
            
            // Width control
            column![
                text("Row Width"),
                row![
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(150),
                    text("(Fill, Shrink, or number for pixels)").size(10).color(Color::from_rgb(0.6, 0.6, 0.6)),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Height control
            column![
                text("Row Height"),
                row![
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(150),
                    text("(Usually Shrink for rows)").size(10).color(Color::from_rgb(0.6, 0.6, 0.6)),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Padding controls
            text("Padding").size(14),
            row![
                column![
                    text("Top").size(12),
                    slider(0.0..=50.0, props.padding.top, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.top)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Right").size(12),
                    slider(0.0..=50.0, props.padding.right, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.right)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15),
            
            row![
                column![
                    text("Bottom").size(12),
                    slider(0.0..=50.0, props.padding.bottom, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.bottom)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Left").size(12),
                    slider(0.0..=50.0, props.padding.left, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.left)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15)
        ].into()            
    }
    
    fn column_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Column Properties").size(16),
            
            // Spacing control
            column![
                text("Spacing between items"),
                row![
                    slider(0.0..=50.0, props.spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::Spacing(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.spacing)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Horizontal alignment of items in the column
            column![
                text("Horizontal Alignment"),
                pick_list(
                    vec![AlignmentOption::Start, AlignmentOption::Center, AlignmentOption::End],
                    Some(AlignmentOption::from_alignment(props.align_items)),
                    move |selected_option| {
                        Message::PropertyChanged(
                            widget_id, 
                            PropertyChange::AlignItems(selected_option.to_alignment())
                        )
                    },
                ).placeholder("Choose alignment"),
            ].spacing(5),
            
            // Width and Height controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(150),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(150),
                ].spacing(5),
            ].spacing(15),
            
            // Padding controls
            text("Padding").size(14),
            row![
                column![
                    text("Top").size(12),
                    slider(0.0..=50.0, props.padding.top, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.top)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Right").size(12),
                    slider(0.0..=50.0, props.padding.right, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.right)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15),
            
            row![
                column![
                    text("Bottom").size(12),
                    slider(0.0..=50.0, props.padding.bottom, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.bottom)).size(12).center(),
                ].spacing(5).width(Length::Fill),
                
                column![
                    text("Left").size(12),
                    slider(0.0..=50.0, props.padding.left, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                    })
                        .step(1.0),
                    text(format!("{:.0}", props.padding.left)).size(12).center(),
                ].spacing(5).width(Length::Fill),
            ].spacing(15),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
    
    fn button_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Button Properties").size(16),
            
            // Button text
            column![
                text("Button Text"),
                text_input("Text", &props.text_content)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextContent(v))
                    })
                    .width(250),
            ].spacing(5),
            
            // Button style
            column![
                text("Button Style"),
                pick_list(
                    vec![
                        ButtonStyleType::Primary,
                        ButtonStyleType::Secondary,
                        ButtonStyleType::Success,
                        ButtonStyleType::Danger,
                        ButtonStyleType::Text,
                    ],
                    Some(props.button_style),
                    move |v| Message::PropertyChanged(widget_id, PropertyChange::ButtonStyle(v)),
                ).width(250),
            ].spacing(5),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
            
            // Padding controls
            text("Padding").size(14),
            row![
                column![
                    text("Top"),
                    slider(0.0..=30.0, props.padding.top, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                    }).step(1.0),
                    text(format!("{:.0}", props.padding.top)).size(12).center(),
                ].spacing(5),
                
                column![
                    text("Right"),
                    slider(0.0..=30.0, props.padding.right, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                    }).step(1.0),
                    text(format!("{:.0}", props.padding.right)).size(12).center(),
                ].spacing(5),
            ].spacing(15),
            
            row![
                column![
                    text("Bottom"),
                    slider(0.0..=30.0, props.padding.bottom, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                    }).step(1.0),
                    text(format!("{:.0}", props.padding.bottom)).size(12).center(),
                ].spacing(5),
                
                column![
                    text("Left"),
                    slider(0.0..=30.0, props.padding.left, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                    }).step(1.0),
                    text(format!("{:.0}", props.padding.left)).size(12).center(),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn text_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Text Properties").size(16),
            
            // Text content
            column![
                text("Text Content"),
                text_input("Content", &props.text_content)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextContent(v))
                    })
                    .width(300),
            ].spacing(5),
            
            // Text size
            column![
                text("Font Size"),
                row![
                    slider(8.0..=72.0, props.text_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextSize(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.text_size)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Font type
            column![
                text("Font"),
                pick_list(
                    vec![FontType::Default, FontType::Monospace],
                    Some(props.font),
                    move |v| Message::PropertyChanged(widget_id, PropertyChange::Font(v)),
                ).width(200),
            ].spacing(5),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn text_input_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Text Input Properties").size(16),
            
            // Placeholder text
            column![
                text("Placeholder Text"),
                text_input("Placeholder", &props.text_input_placeholder)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextInputPlaceholder(v))
                    })
                    .width(300),
            ].spacing(5),
            
            // Font size
            column![
                text("Font Size"),
                row![
                    slider(8.0..=32.0, props.text_input_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextInputSize(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.text_input_size)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Internal padding
            column![
                text("Internal Padding"),
                row![
                    slider(0.0..=30.0, props.text_input_padding, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TextInputPadding(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.text_input_padding)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Secure input toggle
            checkbox("Secure Input (Password)", props.is_secure)
                .on_toggle(move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::IsSecure(v))
                }),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn checkbox_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Checkbox Properties").size(16),
            
            // Label text
            column![
                text("Label Text"),
                text_input("Label", &props.checkbox_label)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::CheckboxLabel(v))
                    })
                    .width(250),
            ].spacing(5),
            
            // Checkbox size
            column![
                text("Checkbox Size"),
                row![
                    slider(12.0..=40.0, props.checkbox_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::CheckboxSize(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.checkbox_size)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Spacing between checkbox and label
            column![
                text("Label Spacing"),
                row![
                    slider(0.0..=30.0, props.checkbox_spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::CheckboxSpacing(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.checkbox_spacing)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Default checked state
            checkbox("Default Checked State", props.checkbox_checked)
                .on_toggle(move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::CheckboxChecked(v))
                }),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn toggler_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Toggler Properties").size(16),
            
            // Label text
            column![
                text("Label Text"),
                text_input("Label", &props.toggler_label)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TogglerLabel(v))
                    })
                    .width(250),
            ].spacing(5),
            
            // Toggler size
            column![
                text("Toggler Size"),
                row![
                    slider(12.0..=40.0, props.toggler_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TogglerSize(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.toggler_size)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Spacing between toggler and label
            column![
                text("Label Spacing"),
                row![
                    slider(0.0..=30.0, props.toggler_spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::TogglerSpacing(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.toggler_spacing)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Default active state
            checkbox("Default Active State", props.toggler_active)
                .on_toggle(move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TogglerActive(v))
                }),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn radio_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Radio Button Properties").size(16),
            
            // Label text
            column![
                text("Label Text"),
                text_input("Label", &props.radio_label)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioLabel(v))
                    })
                    .width(250),
            ].spacing(5),
            
            // Radio button size
            column![
                text("Radio Size"),
                row![
                    slider(12.0..=40.0, props.radio_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSize(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.radio_size)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Spacing between radio and label
            column![
                text("Label Spacing"),
                row![
                    slider(0.0..=30.0, props.radio_spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSpacing(v))
                    })
                        .step(1.0)
                        .width(200),
                    text(format!("{:.0}px", props.radio_spacing)).size(12).width(50),
                ].spacing(10).align_y(Alignment::Center),
            ].spacing(5),
            
            // Options management - similar to picklist
            column![
                text("Radio Options"),
                column(
                    props.radio_options.iter().enumerate().map(|(i, option)| {
                        row![
                            text_input(&format!("Option {}", i + 1), option)
                                .on_input({
                                    let index = i;
                                    let current_options = props.radio_options.clone();
                                    move |v| {
                                        let mut new_options = current_options.clone();
                                        if index < new_options.len() {
                                            new_options[index] = v;
                                        }
                                        Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(new_options))
                                    }
                                })
                                .width(200),
                            button("Remove")
                                .on_press({
                                    let index = i;
                                    let current_options = props.radio_options.clone();
                                    let mut new_options = current_options;
                                    if index < new_options.len() && new_options.len() > 1 {
                                        new_options.remove(index);
                                    }
                                    Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(new_options))
                                })
                                .style(button::danger)
                                .padding(Padding::new(5.0)),
                        ].spacing(10).align_y(Alignment::Center).into()
                    }).collect::<Vec<Element<Message>>>()
                ).spacing(5),
                
                button("Add Option")
                    .on_press({
                        let current_options = props.radio_options.clone();
                        let mut new_options = current_options;
                        new_options.push(format!("Option {}", new_options.len() + 1));
                        Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(new_options))
                    })
                    .style(button::success)
                    .padding(Padding::new(5.0)),
            ].spacing(10),
            
            // Default selected option
            column![
                text("Default Selection"),
                pick_list(
                    props.radio_options.clone(),
                    props.radio_options.get(props.radio_selected_index).cloned(),
                    move |selected_option| {
                        // Find the index of the selected option
                        let current_options = props.radio_options.clone();
                        if let Some(index) = current_options.iter().position(|opt| opt == &selected_option) {
                            Message::PropertyChanged(widget_id, PropertyChange::RadioSelectedIndex(index))
                        } else {
                            Message::PropertyChanged(widget_id, PropertyChange::RadioSelectedIndex(0))
                        }
                    },
                ).width(200),
            ].spacing(5),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
    }
    
    fn picklist_controls(&self, widget_id: WidgetId) -> Element<Message> {
        let widget = self.hierarchy.get_widget_by_id(widget_id).unwrap();
        let props = &widget.properties;
        
        column![
            text("Pick List Properties").size(16),
            
            // Placeholder text
            column![
                text("Placeholder Text"),
                text_input("Placeholder", &props.picklist_placeholder)
                    .on_input(move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::PickListPlaceholder(v))
                    })
                    .width(250),
            ].spacing(5),
            
            // Options management - simplified approach
            column![
                text("Options"),
                column(
                    props.picklist_options.iter().enumerate().map(|(i, option)| {
                        row![
                            text_input(&format!("Option {}", i + 1), option)
                                .on_input({
                                    let index = i;
                                    let current_options = props.picklist_options.clone();
                                    move |v| {
                                        let mut new_options = current_options.clone();
                                        if index < new_options.len() {
                                            new_options[index] = v;
                                        }
                                        Message::PropertyChanged(widget_id, PropertyChange::PickListOptions(new_options))
                                    }
                                })
                                .width(200),
                            button("Remove")
                                .on_press({
                                    let index = i;
                                    let current_options = props.picklist_options.clone();
                                    let mut new_options = current_options;
                                    if index < new_options.len() {
                                        new_options.remove(index);
                                    }
                                    Message::PropertyChanged(widget_id, PropertyChange::PickListOptions(new_options))
                                })
                                .style(button::danger)
                                .padding(Padding::new(5.0)),
                        ].spacing(10).align_y(Alignment::Center).into()
                    }).collect::<Vec<Element<Message>>>()
                ).spacing(5),
                
                button("Add Option")
                    .on_press({
                        let current_options = props.picklist_options.clone();
                        let mut new_options = current_options;
                        new_options.push(format!("Option {}", new_options.len() + 1));
                        Message::PropertyChanged(widget_id, PropertyChange::PickListOptions(new_options))
                    })
                    .style(button::success)
                    .padding(Padding::new(5.0)),
            ].spacing(10),
            
            // Size controls
            row![
                column![
                    text("Width"),
                    text_input("Width", &length_to_string(props.width))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Width(length))
                        })
                        .width(120),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &length_to_string(props.height))
                        .on_input(move |v| {
                            let length = parse_length(&v);
                            Message::PropertyChanged(widget_id, PropertyChange::Height(length))
                        })
                        .width(120),
                ].spacing(5),
            ].spacing(15),
        ]
        .spacing(15)
        .into()
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
    // Widget Operations
    SelectWidget(WidgetId),
    DeleteWidget(WidgetId),
    AddChild(WidgetId, WidgetType),
    PropertyChanged(WidgetId, PropertyChange),

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

fn length_to_string(length: Length) -> String {
    match length {
        Length::Fill => "Fill".to_string(),
        Length::Shrink => "Shrink".to_string(),
        Length::Fixed(pixels) => format!("{}", pixels),
        _ => "Shrink".to_string(),
    }
}

fn can_have_children(widget_type: &WidgetType) -> bool {
    matches!(
        widget_type,
        WidgetType::Container | WidgetType::Row | WidgetType::Column | WidgetType::Scrollable
    )
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
    pub progress_height: f32,
    
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
            progress_value: 0.5,
            progress_height: 10.0,
            
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
        }
    }
}

impl Properties {
    pub fn for_widget_type(widget_type: WidgetType) -> Self {
        let mut props = Self::default();
        
        // Customize defaults based on widget type
        match widget_type {
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
            }
            WidgetType::Checkbox => {
                props.checkbox_label = "Check me".to_string();
            }
            WidgetType::Radio => {
                props.radio_options = vec![
                    "Radio Option 1".to_string(),
                    "Radio Option 2".to_string(),
                ];
            }
            WidgetType::Toggler => {
                props.toggler_label = "Toggle me".to_string();
            }
            WidgetType::PickList => {
                props.picklist_placeholder = "Choose an option...".to_string();
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