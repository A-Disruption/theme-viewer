use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, combo_box, container, horizontal_rule, horizontal_space,
        pick_list, progress_bar, radio, row, scrollable, slider, text, text_editor,
        text_input, toggler, tooltip, vertical_rule, vertical_space, Button, Checkbox,
        Column, Container, Row, Scrollable, Slider, Space, Text, TextInput, Toggler,
    },
    Alignment, Background, Border, Color, Element, Font, Length, Padding, Pixels, Shadow,
    Size, Theme, Vector,
};
use std::collections::HashMap;
use crate::widget::generic_overlay::overlay_button;

#[derive(Debug, Clone)]
pub enum Message {
    // Widget selection
    SelectWidget(WidgetType),
    
    // Common properties
    WidthChanged(String),
    HeightChanged(String),
    PaddingTopChanged(f32),
    PaddingRightChanged(f32),
    PaddingBottomChanged(f32),
    PaddingLeftChanged(f32),
    
    // Container properties
    ContainerAlignXChanged(ContainerAlignX),
    ContainerAlignYChanged(ContainerAlignY),
    ContainerBorderWidthChanged(f32),
    ContainerBorderRadiusChanged(f32),
    ContainerBorderColorChanged(Color),
    ContainerBackgroundColorChanged(Color),
    ContainerShadowToggled(bool),
    ContainerShadowOffsetXChanged(f32),
    ContainerShadowOffsetYChanged(f32),
    ContainerShadowBlurChanged(f32),
    ContainerShadowColorChanged(Color),
    
    // Row/Column properties
    SpacingChanged(f32),
    AlignItemsChanged(RowColumnAlign),
    
    // Button properties
    ButtonTextChanged(String),
    ButtonStyleChanged(ButtonStyleType),
    
    // Text properties
    TextContentChanged(String),
    TextSizeChanged(f32),
    TextColorChanged(Color),
    TextFontChanged(FontType),
    
    // Text Input properties
    TextInputValueChanged(String),
    TextInputPlaceholderChanged(String),
    TextInputSizeChanged(f32),
    TextInputPaddingChanged(f32),
    TextInputSecureToggled(bool),
    
    // Checkbox properties
    CheckboxToggled(bool),
    CheckboxLabelChanged(String),
    CheckboxSizeChanged(f32),
    CheckboxSpacingChanged(f32),
    
    // Radio properties
    RadioSelected(RadioOption),
    RadioLabelChanged(String),
    RadioSizeChanged(f32),
    RadioSpacingChanged(f32),
    
    // Slider properties
    SliderValueChanged(f32),
    SliderMinChanged(String),
    SliderMaxChanged(String),
    SliderStepChanged(String),
    
    // Progress Bar properties
    ProgressValueChanged(f32),
    ProgressHeightChanged(f32),
    
    // Toggler properties
    TogglerToggled(bool),
    TogglerLabelChanged(String),
    TogglerSizeChanged(f32),
    TogglerSpacingChanged(f32),
    
    // Pick List properties
    PickListSelected(String),
    PickListPlaceholderChanged(String),
    
    // Scrollable properties
    ScrollableWidthChanged(f32),
    ScrollableHeightChanged(f32),
    
    // Visual helpers
    ShowPaddingToggled(bool),
    ShowSpacingToggled(bool),
    ShowBordersToggled(bool),
    
    // Nesting
    AddChildWidget(WidgetType),
    RemoveChildWidget(usize),
    SelectChildWidget(usize),
    
    // Theme
    ThemeChanged(ThemeType),
}

pub enum Action {
    Run(iced::Task<Message>),
    None,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerAlignX {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerAlignY {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowColumnAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonStyleType {
    Primary,
    Secondary,
    Success,
    Danger,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontType {
    Default,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioOption {
    Option1,
    Option2,
    Option3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeType {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
}

#[derive(Debug, Clone)]
struct WidgetProperties {
    // Common properties
    width: Length,
    height: Length,
    padding: Padding,
    
    // Widget-specific properties
    container: ContainerProperties,
    row_column: RowColumnProperties,
    button: ButtonProperties,
    text: TextProperties,
    text_input: TextInputProperties,
    checkbox: CheckboxProperties,
    radio: RadioProperties,
    slider: SliderProperties,
    progress: ProgressProperties,
    toggler: TogglerProperties,
    pick_list: PickListProperties,
    scrollable: ScrollableProperties,
}

#[derive(Debug, Clone)]
struct ContainerProperties {
    align_x: ContainerAlignX,
    align_y: ContainerAlignY,
    border_width: f32,
    border_radius: f32,
    border_color: Color,
    background_color: Color,
    has_shadow: bool,
    shadow_offset: Vector,
    shadow_blur: f32,
    shadow_color: Color,
}

#[derive(Debug, Clone)]
struct RowColumnProperties {
    spacing: f32,
    align_items: RowColumnAlign,
}

#[derive(Debug, Clone)]
struct ButtonProperties {
    text: String,
    style: ButtonStyleType,
}

#[derive(Debug, Clone)]
struct TextProperties {
    content: String,
    size: f32,
    color: Color,
    font: FontType,
}

#[derive(Debug, Clone)]
struct TextInputProperties {
    value: String,
    placeholder: String,
    size: f32,
    padding: f32,
    is_secure: bool,
}

#[derive(Debug, Clone)]
struct CheckboxProperties {
    is_checked: bool,
    label: String,
    size: f32,
    spacing: f32,
}

#[derive(Debug, Clone)]
struct RadioProperties {
    selected: RadioOption,
    label: String,
    size: f32,
    spacing: f32,
}

#[derive(Debug, Clone)]
struct SliderProperties {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
}

#[derive(Debug, Clone)]
struct ProgressProperties {
    value: f32,
    height: f32,
}

#[derive(Debug, Clone)]
struct TogglerProperties {
    is_active: bool,
    label: String,
    size: f32,
    spacing: f32,
}

#[derive(Debug, Clone)]
struct PickListProperties {
    selected: Option<String>,
    placeholder: String,
    options: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScrollableProperties {
    width: f32,
    height: f32,
}

pub struct WidgetVisualizer {
    selected_widget: WidgetType,
    properties: WidgetProperties,
    
    // Visual helpers
    show_padding: bool,
    show_spacing: bool,
    show_borders: bool,
    
    // Nested widgets for containers
    child_widgets: Vec<(WidgetType, WidgetProperties)>,
    selected_widget_type: Option<WidgetType>,
    selected_child: Option<usize>,
    
    // Theme
    theme: ThemeType,
    
    // Input values
    width_input: String,
    height_input: String,
    slider_min_input: String,
    slider_max_input: String,
    slider_step_input: String,

    //checkbox in overlay
    overlay_checkbox: bool,
}

impl Default for WidgetVisualizer {
    fn default() -> Self {
        Self {
            selected_widget: WidgetType::Container,
            properties: WidgetProperties::default(),
            show_padding: true,
            show_spacing: true,
            show_borders: true,
            child_widgets: vec![],
            selected_widget_type: None,
            selected_child: None,
            theme: ThemeType::Light,
            width_input: "Fill".to_string(),
            height_input: "Shrink".to_string(),
            slider_min_input: "0".to_string(),
            slider_max_input: "100".to_string(),
            slider_step_input: "1".to_string(),
            overlay_checkbox: false,
        }
    }
}

impl Default for WidgetProperties {
    fn default() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            container: ContainerProperties {
                align_x: ContainerAlignX::Center,
                align_y: ContainerAlignY::Center,
                border_width: 1.0,
                border_radius: 5.0,
                border_color: Color::from_rgb(0.5, 0.5, 0.5),
                background_color: Color::from_rgba(0.9, 0.9, 0.9, 1.0),
                has_shadow: false,
                shadow_offset: Vector::new(0.0, 2.0),
                shadow_blur: 5.0,
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            },
            row_column: RowColumnProperties {
                spacing: 10.0,
                align_items: RowColumnAlign::Start,
            },
            button: ButtonProperties {
                text: "Click Me!".to_string(),
                style: ButtonStyleType::Primary,
            },
            text: TextProperties {
                content: "Sample Text".to_string(),
                size: 16.0,
                color: Color::BLACK,
                font: FontType::Default,
            },
            text_input: TextInputProperties {
                value: String::new(),
                placeholder: "Enter text...".to_string(),
                size: 16.0,
                padding: 10.0,
                is_secure: false,
            },
            checkbox: CheckboxProperties {
                is_checked: false,
                label: "Check me".to_string(),
                size: 20.0,
                spacing: 10.0,
            },
            radio: RadioProperties {
                selected: RadioOption::Option1,
                label: "Radio Option".to_string(),
                size: 20.0,
                spacing: 10.0,
            },
            slider: SliderProperties {
                value: 50.0,
                min: 0.0,
                max: 100.0,
                step: 1.0,
            },
            progress: ProgressProperties {
                value: 0.5,
                height: 10.0,
            },
            toggler: TogglerProperties {
                is_active: false,
                label: "Toggle me".to_string(),
                size: 20.0,
                spacing: 10.0,
            },
            pick_list: PickListProperties {
                selected: None,
                placeholder: "Choose an option...".to_string(),
                options: vec![
                    "Option 1".to_string(),
                    "Option 2".to_string(),
                    "Option 3".to_string(),
                ],
            },
            scrollable: ScrollableProperties {
                width: 300.0,
                height: 200.0,
            },
        }
    }
}

impl WidgetVisualizer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::SelectWidget(widget_type) => {
                self.selected_widget = widget_type;
                self.child_widgets.clear();
                self.selected_child = None;
            }
            
            // Common properties
            Message::WidthChanged(value) => {
                self.width_input = value.clone();
                self.properties.width = parse_length(&value);
            }
            Message::HeightChanged(value) => {
                self.height_input = value.clone();
                self.properties.height = parse_length(&value);
            }
            Message::PaddingTopChanged(value) => {
                self.properties.padding = Padding {
                    top: value,
                    right: self.properties.padding.right,
                    bottom: self.properties.padding.bottom,
                    left: self.properties.padding.left,
                };
            }
            Message::PaddingRightChanged(value) => {
                self.properties.padding = Padding {
                    top: self.properties.padding.top,
                    right: value,
                    bottom: self.properties.padding.bottom,
                    left: self.properties.padding.left,
                };
            }
            Message::PaddingBottomChanged(value) => {
                self.properties.padding = Padding {
                    top: self.properties.padding.top,
                    right: self.properties.padding.right,
                    bottom: value,
                    left: self.properties.padding.left,
                };
            }
            Message::PaddingLeftChanged(value) => {
                self.properties.padding = Padding {
                    top: self.properties.padding.top,
                    right: self.properties.padding.right,
                    bottom: self.properties.padding.bottom,
                    left: value,
                };
            }
            
            // Container properties
            Message::ContainerAlignXChanged(align) => {
                self.properties.container.align_x = align;
            }
            Message::ContainerAlignYChanged(align) => {
                self.properties.container.align_y = align;
            }
            Message::ContainerBorderWidthChanged(width) => {
                self.properties.container.border_width = width;
            }
            Message::ContainerBorderRadiusChanged(radius) => {
                self.properties.container.border_radius = radius;
            }
            Message::ContainerBorderColorChanged(color) => {
                self.properties.container.border_color = color;
            }
            Message::ContainerBackgroundColorChanged(color) => {
                self.properties.container.background_color = color;
            }
            Message::ContainerShadowToggled(enabled) => {
                self.properties.container.has_shadow = enabled;
            }
            Message::ContainerShadowOffsetXChanged(x) => {
                self.properties.container.shadow_offset.x = x;
            }
            Message::ContainerShadowOffsetYChanged(y) => {
                self.properties.container.shadow_offset.y = y;
            }
            Message::ContainerShadowBlurChanged(blur) => {
                self.properties.container.shadow_blur = blur;
            }
            Message::ContainerShadowColorChanged(color) => {
                self.properties.container.shadow_color = color;
            }
            
            // Row/Column properties
            Message::SpacingChanged(spacing) => {
                self.properties.row_column.spacing = spacing;
            }
            Message::AlignItemsChanged(align) => {
                self.properties.row_column.align_items = align;
            }
            
            // Button properties
            Message::ButtonTextChanged(text) => {
                self.properties.button.text = text;
            }
            Message::ButtonStyleChanged(style) => {
                self.properties.button.style = style;
            }
            
            // Text properties
            Message::TextContentChanged(content) => {
                self.properties.text.content = content;
            }
            Message::TextSizeChanged(size) => {
                self.properties.text.size = size;
            }
            Message::TextColorChanged(color) => {
                self.properties.text.color = color;
            }
            Message::TextFontChanged(font) => {
                self.properties.text.font = font;
            }
            
            // Text Input properties
            Message::TextInputValueChanged(value) => {
                self.properties.text_input.value = value;
            }
            Message::TextInputPlaceholderChanged(placeholder) => {
                self.properties.text_input.placeholder = placeholder;
            }
            Message::TextInputSizeChanged(size) => {
                self.properties.text_input.size = size;
            }
            Message::TextInputPaddingChanged(padding) => {
                self.properties.text_input.padding = padding;
            }
            Message::TextInputSecureToggled(secure) => {
                self.properties.text_input.is_secure = secure;
            }
            
            // Checkbox properties
            Message::CheckboxToggled(checked) => {
                self.properties.checkbox.is_checked = checked;
            }
            Message::CheckboxLabelChanged(label) => {
                self.properties.checkbox.label = label;
            }
            Message::CheckboxSizeChanged(size) => {
                self.properties.checkbox.size = size;
            }
            Message::CheckboxSpacingChanged(spacing) => {
                self.properties.checkbox.spacing = spacing;
            }
            
            // Radio properties
            Message::RadioSelected(option) => {
                self.properties.radio.selected = option;
            }
            Message::RadioLabelChanged(label) => {
                self.properties.radio.label = label;
            }
            Message::RadioSizeChanged(size) => {
                self.properties.radio.size = size;
            }
            Message::RadioSpacingChanged(spacing) => {
                self.properties.radio.spacing = spacing;
            }
            
            // Slider properties
            Message::SliderValueChanged(value) => {
                self.properties.slider.value = value;
            }
            Message::SliderMinChanged(min) => {
                self.slider_min_input = min.clone();
                if let Ok(value) = min.parse::<f32>() {
                    self.properties.slider.min = value;
                }
            }
            Message::SliderMaxChanged(max) => {
                self.slider_max_input = max.clone();
                if let Ok(value) = max.parse::<f32>() {
                    self.properties.slider.max = value;
                }
            }
            Message::SliderStepChanged(step) => {
                self.slider_step_input = step.clone();
                if let Ok(value) = step.parse::<f32>() {
                    self.properties.slider.step = value;
                }
            }
            
            // Progress Bar properties
            Message::ProgressValueChanged(value) => {
                self.properties.progress.value = value;
            }
            Message::ProgressHeightChanged(height) => {
                self.properties.progress.height = height;
            }
            
            // Toggler properties
            Message::TogglerToggled(active) => {
                self.properties.toggler.is_active = active;
            }
            Message::TogglerLabelChanged(label) => {
                self.properties.toggler.label = label;
            }
            Message::TogglerSizeChanged(size) => {
                self.properties.toggler.size = size;
            }
            Message::TogglerSpacingChanged(spacing) => {
                self.properties.toggler.spacing = spacing;
            }
            
            // Pick List properties
            Message::PickListSelected(option) => {
                self.properties.pick_list.selected = Some(option);
            }
            Message::PickListPlaceholderChanged(placeholder) => {
                self.properties.pick_list.placeholder = placeholder;
            }
            
            // Scrollable properties
            Message::ScrollableWidthChanged(width) => {
                self.properties.scrollable.width = width;
            }
            Message::ScrollableHeightChanged(height) => {
                self.properties.scrollable.height = height;
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
            
            // Nesting
            Message::AddChildWidget(widget_type) => {
                if matches!(self.selected_widget, WidgetType::Container | WidgetType::Row | WidgetType::Column | WidgetType::Scrollable) {
                    self.child_widgets.push((widget_type, WidgetProperties::default()));
                }
            }
            Message::RemoveChildWidget(index) => {
                if index < self.child_widgets.len() {
                    self.child_widgets.remove(index);
                    self.selected_child = None;
                }
            }
            Message::SelectChildWidget(index) => {
                self.selected_child = Some(index);
            }
            
            // Theme
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
        }
        Action::None
    }
    
    pub fn view(&self) -> Element<Message> {

        let overlay_content = column![
            Space::with_height(100),
            text("Testing Generic Overlay"),
            checkbox("Enable Feature", self.overlay_checkbox).on_toggle(Message::TextInputSecureToggled),
        ].spacing(20);

        let test_overlay_button = overlay_button(
            "Open Dialog", 
            "Test Overlay", 
            overlay_content
        )
        .overlay_width(500.0)
        .overlay_height(400.0);


        let left_panel = self.build_controls_panel();
        let right_panel = self.build_preview_panel();
        
        row![
            test_overlay_button.width(50).height(50),
            container(left_panel)
                .width(Length::Fixed(400.0))
                .height(Length::Fill)
                .style(|theme: &Theme| {
                    container::Style {
                        background: Some(Background::Color(
                            theme.extended_palette().background.weak.color
                        )),
                        border: Border {
                            color: theme.extended_palette().background.strong.color,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .padding(20),
            
            container(right_panel)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(40)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .into()
    }
    
    fn build_controls_panel(&self) -> Element<Message> {
        let mut controls = column![
            text("Widget Visualizer").size(24),
            horizontal_rule(10),
        ]
        .spacing(10);
        
        // Theme selector
        controls = controls.push(
            column![
                text("Theme").size(18),
                pick_list(
                    vec![
                        ThemeType::Light,
                        ThemeType::Dark,
                        ThemeType::Dracula,
                        ThemeType::Nord,
                        ThemeType::SolarizedLight,
                        ThemeType::SolarizedDark,
                    ],
                    Some(self.theme),
                    Message::ThemeChanged,
                ),
            ]
            .spacing(5)
        );
        
        // Widget selector
        let widget_types = vec![
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
            WidgetType::Scrollable,
            WidgetType::Space,
            WidgetType::Rule,
        ];
        
        controls = controls.push(
            column![
                text("Widget Type").size(18),
                pick_list(
                    widget_types,
                    Some(self.selected_widget),
                    Message::SelectWidget,
                ),
            ]
            .spacing(5)
        );
        
        // Visual helpers
        controls = controls.push(
            column![
                text("Visual Helpers").size(18),
                checkbox("Show Padding", self.show_padding)
                    .on_toggle(Message::ShowPaddingToggled),
                checkbox("Show Spacing", self.show_spacing)
                    .on_toggle(Message::ShowSpacingToggled),
                checkbox("Show Borders", self.show_borders)
                    .on_toggle(Message::ShowBordersToggled),
            ]
            .spacing(5)
        );
        
        controls = controls.push(horizontal_rule(10));
        
        // Common properties
        controls = controls.push(self.build_common_properties());
        
        // Widget-specific properties
        controls = controls.push(self.build_widget_specific_properties());
        
        // Nesting controls for containers
        if matches!(self.selected_widget, WidgetType::Container | WidgetType::Row | WidgetType::Column | WidgetType::Scrollable) {
            controls = controls.push(horizontal_rule(10));
            controls = controls.push(self.build_nesting_controls());
        }
        
        scrollable(controls).into()
    }
    
    fn build_common_properties(&self) -> Element<Message> {
        column![
            text("Common Properties").size(16),
            
            // Width & Height
            row![
                column![
                    text("Width"),
                    text_input("Width", &self.width_input)
                        .on_input(Message::WidthChanged)
                        .size(14),
                ].spacing(5),
                
                column![
                    text("Height"),
                    text_input("Height", &self.height_input)
                        .on_input(Message::HeightChanged)
                        .size(14),
                ].spacing(5),
            ].spacing(10),
            
            // Padding
            text("Padding"),
            row![
                column![
                    text("Top").size(12),
                    slider(0.0..=50.0, self.properties.padding.top, Message::PaddingTopChanged)
                        .step(1.0),
                    text(format!("{:.0}", self.properties.padding.top)).size(12),
                ].spacing(5),
                
                column![
                    text("Right").size(12),
                    slider(0.0..=50.0, self.properties.padding.right, Message::PaddingRightChanged)
                        .step(1.0),
                    text(format!("{:.0}", self.properties.padding.right)).size(12),
                ].spacing(5),
            ].spacing(10),
            
            row![
                column![
                    text("Bottom").size(12),
                    slider(0.0..=50.0, self.properties.padding.bottom, Message::PaddingBottomChanged)
                        .step(1.0),
                    text(format!("{:.0}", self.properties.padding.bottom)).size(12),
                ].spacing(5),
                
                column![
                    text("Left").size(12),
                    slider(0.0..=50.0, self.properties.padding.left, Message::PaddingLeftChanged)
                        .step(1.0),
                    text(format!("{:.0}", self.properties.padding.left)).size(12),
                ].spacing(5),
            ].spacing(10),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_widget_specific_properties(&self) -> Element<Message> {
        match self.selected_widget {
            WidgetType::Container => self.build_container_properties(),
            WidgetType::Row | WidgetType::Column => self.build_row_column_properties(),
            WidgetType::Button => self.build_button_properties(),
            WidgetType::Text => self.build_text_properties(),
            WidgetType::TextInput => self.build_text_input_properties(),
            WidgetType::Checkbox => self.build_checkbox_properties(),
            WidgetType::Radio => self.build_radio_properties(),
            WidgetType::Slider => self.build_slider_properties(),
            WidgetType::ProgressBar => self.build_progress_properties(),
            WidgetType::Toggler => self.build_toggler_properties(),
            WidgetType::PickList => self.build_pick_list_properties(),
            WidgetType::Scrollable => self.build_scrollable_properties(),
            _ => column![].into(),
        }
    }
    
    fn build_container_properties(&self) -> Element<Message> {
        column![
            text("Container Properties").size(16),
            
            // Alignment
            row![
                column![
                    text("Horizontal Align"),
                    pick_list(
                        vec![ContainerAlignX::Left, ContainerAlignX::Center, ContainerAlignX::Right],
                        Some(self.properties.container.align_x),
                        Message::ContainerAlignXChanged,
                    ),
                ].spacing(5),
                
                column![
                    text("Vertical Align"),
                    pick_list(
                        vec![ContainerAlignY::Top, ContainerAlignY::Center, ContainerAlignY::Bottom],
                        Some(self.properties.container.align_y),
                        Message::ContainerAlignYChanged,
                    ),
                ].spacing(5),
            ].spacing(10),
            
            // Border
            text("Border"),
            row![
                column![
                    text("Width").size(12),
                    slider(0.0..=10.0, self.properties.container.border_width, Message::ContainerBorderWidthChanged)
                        .step(0.5),
                ].spacing(5),
                
                column![
                    text("Radius").size(12),
                    slider(0.0..=50.0, self.properties.container.border_radius, Message::ContainerBorderRadiusChanged)
                        .step(1.0),
                ].spacing(5),
            ].spacing(10),
            
            // Shadow
            checkbox("Enable Shadow", self.properties.container.has_shadow)
                .on_toggle(Message::ContainerShadowToggled),
            
            if self.properties.container.has_shadow {
                column![
                    row![
                        column![
                            text("Offset X").size(12),
                            slider(-20.0..=20.0, self.properties.container.shadow_offset.x, Message::ContainerShadowOffsetXChanged)
                                .step(1.0),
                        ].spacing(5),
                        
                        column![
                            text("Offset Y").size(12),
                            slider(-20.0..=20.0, self.properties.container.shadow_offset.y, Message::ContainerShadowOffsetYChanged)
                                .step(1.0),
                        ].spacing(5),
                    ].spacing(10),
                    
                    column![
                        text("Blur Radius").size(12),
                        slider(0.0..=50.0, self.properties.container.shadow_blur, Message::ContainerShadowBlurChanged)
                            .step(1.0),
                    ].spacing(5),
                ].spacing(10)
            } else {
                column![].into()
            },
        ]
        .spacing(10)
        .into()
    }
    
    fn build_row_column_properties(&self) -> Element<Message> {
        column![
            text(format!("{:?} Properties", self.selected_widget)).size(16),
            
            column![
                text("Spacing"),
                slider(0.0..=50.0, self.properties.row_column.spacing, Message::SpacingChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.row_column.spacing)).size(12),
            ].spacing(5),
            
            column![
                text("Align Items"),
                pick_list(
                    vec![RowColumnAlign::Start, RowColumnAlign::Center, RowColumnAlign::End],
                    Some(self.properties.row_column.align_items),
                    Message::AlignItemsChanged,
                ),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_button_properties(&self) -> Element<Message> {
        column![
            text("Button Properties").size(16),
            
            column![
                text("Text"),
                text_input("Button text", &self.properties.button.text)
                    .on_input(Message::ButtonTextChanged),
            ].spacing(5),
            
            column![
                text("Style"),
                pick_list(
                    vec![
                        ButtonStyleType::Primary,
                        ButtonStyleType::Secondary,
                        ButtonStyleType::Success,
                        ButtonStyleType::Danger,
                        ButtonStyleType::Text,
                    ],
                    Some(self.properties.button.style),
                    Message::ButtonStyleChanged,
                ),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_text_properties(&self) -> Element<Message> {
        column![
            text("Text Properties").size(16),
            
            column![
                text("Content"),
                text_input("Text content", &self.properties.text.content)
                    .on_input(Message::TextContentChanged),
            ].spacing(5),
            
            column![
                text("Size"),
                slider(8.0..=72.0, self.properties.text.size, Message::TextSizeChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.text.size)).size(12),
            ].spacing(5),
            
            column![
                text("Font"),
                pick_list(
                    vec![FontType::Default, FontType::Monospace],
                    Some(self.properties.text.font),
                    Message::TextFontChanged,
                ),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_text_input_properties(&self) -> Element<Message> {
        column![
            text("Text Input Properties").size(16),
            
            column![
                text("Placeholder"),
                text_input("Placeholder", &self.properties.text_input.placeholder)
                    .on_input(Message::TextInputPlaceholderChanged),
            ].spacing(5),
            
            column![
                text("Font Size"),
                slider(8.0..=32.0, self.properties.text_input.size, Message::TextInputSizeChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.text_input.size)).size(12),
            ].spacing(5),
            
            column![
                text("Padding"),
                slider(0.0..=30.0, self.properties.text_input.padding, Message::TextInputPaddingChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.text_input.padding)).size(12),
            ].spacing(5),
            
            checkbox("Secure Input", self.properties.text_input.is_secure)
                .on_toggle(Message::TextInputSecureToggled),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_checkbox_properties(&self) -> Element<Message> {
        column![
            text("Checkbox Properties").size(16),
            
            column![
                text("Label"),
                text_input("Label", &self.properties.checkbox.label)
                    .on_input(Message::CheckboxLabelChanged),
            ].spacing(5),
            
            column![
                text("Size"),
                slider(12.0..=40.0, self.properties.checkbox.size, Message::CheckboxSizeChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.checkbox.size)).size(12),
            ].spacing(5),
            
            column![
                text("Spacing"),
                slider(0.0..=30.0, self.properties.checkbox.spacing, Message::CheckboxSpacingChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.checkbox.spacing)).size(12),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_radio_properties(&self) -> Element<Message> {
        column![
            text("Radio Properties").size(16),
            
            column![
                text("Size"),
                slider(12.0..=40.0, self.properties.radio.size, Message::RadioSizeChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.radio.size)).size(12),
            ].spacing(5),
            
            column![
                text("Spacing"),
                slider(0.0..=30.0, self.properties.radio.spacing, Message::RadioSpacingChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.radio.spacing)).size(12),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_slider_properties(&self) -> Element<Message> {
        column![
            text("Slider Properties").size(16),
            
            row![
                column![
                    text("Min"),
                    text_input("Min", &self.slider_min_input)
                        .on_input(Message::SliderMinChanged)
                        .size(14),
                ].spacing(5),
                
                column![
                    text("Max"),
                    text_input("Max", &self.slider_max_input)
                        .on_input(Message::SliderMaxChanged)
                        .size(14),
                ].spacing(5),
                
                column![
                    text("Step"),
                    text_input("Step", &self.slider_step_input)
                        .on_input(Message::SliderStepChanged)
                        .size(14),
                ].spacing(5),
            ].spacing(10),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_progress_properties(&self) -> Element<Message> {
        column![
            text("Progress Bar Properties").size(16),
            
            column![
                text("Height"),
                slider(2.0..=50.0, self.properties.progress.height, Message::ProgressHeightChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.progress.height)).size(12),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_toggler_properties(&self) -> Element<Message> {
        column![
            text("Toggler Properties").size(16),
            
            column![
                text("Label"),
                text_input("Label", &self.properties.toggler.label)
                    .on_input(Message::TogglerLabelChanged),
            ].spacing(5),
            
            column![
                text("Size"),
                slider(12.0..=40.0, self.properties.toggler.size, Message::TogglerSizeChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.toggler.size)).size(12),
            ].spacing(5),
            
            column![
                text("Spacing"),
                slider(0.0..=30.0, self.properties.toggler.spacing, Message::TogglerSpacingChanged)
                    .step(1.0),
                text(format!("{:.0}", self.properties.toggler.spacing)).size(12),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_pick_list_properties(&self) -> Element<Message> {
        column![
            text("Pick List Properties").size(16),
            
            column![
                text("Placeholder"),
                text_input("Placeholder", &self.properties.pick_list.placeholder)
                    .on_input(Message::PickListPlaceholderChanged),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_scrollable_properties(&self) -> Element<Message> {
        column![
            text("Scrollable Properties").size(16),
            
            column![
                text("Width"),
                slider(100.0..=800.0, self.properties.scrollable.width, Message::ScrollableWidthChanged)
                    .step(10.0),
                text(format!("{:.0}", self.properties.scrollable.width)).size(12),
            ].spacing(5),
            
            column![
                text("Height"),
                slider(100.0..=600.0, self.properties.scrollable.height, Message::ScrollableHeightChanged)
                    .step(10.0),
                text(format!("{:.0}", self.properties.scrollable.height)).size(12),
            ].spacing(5),
        ]
        .spacing(10)
        .into()
    }
    
    fn build_nesting_controls(&self) -> Element<Message> {
        let mut controls = column![
            text("Child Widgets").size(16),
        ]
        .spacing(10);
        
        // Add child widget button
        let add_child_types = vec![
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
        ];
        
        controls = controls.push(
            row![
                text("Add:"),
                pick_list(
                    add_child_types,
                    self.selected_widget_type,
                    Message::AddChildWidget,
                ),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        );
        
        // List current children
        if !self.child_widgets.is_empty() {
            controls = controls.push(text("Children:").size(14));
            
            for (index, (widget_type, _)) in self.child_widgets.iter().enumerate() {
                let is_selected = self.selected_child == Some(index);
                
                controls = controls.push(
                    row![
                        button(text(format!("{:?} #{}", widget_type, index + 1)))
                            .on_press(Message::SelectChildWidget(index))
                            .style(if is_selected {
                                button::primary
                            } else {
                                button::secondary
                            }),
                        button("×")
                            .on_press(Message::RemoveChildWidget(index))
                            .style(button::danger),
                    ]
                    .spacing(5)
                );
            }
        }
        
        controls.into()
    }
    
    fn build_preview_panel(&self) -> Element<Message> {
        let mut preview = self.build_widget_preview(&self.selected_widget, &self.properties);
        
        // Apply visual helpers
        if self.show_padding && self.properties.padding.top > 0.0 {
            preview = container(preview)
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Background::Color(
                            Color::from_rgba(0.2, 0.8, 0.2, 0.2)
                        )),
                        ..Default::default()
                    }
                })
                .padding(0)
                .into();
        }
        
        if self.show_borders {
            preview = container(preview)
                .style(|_theme: &Theme| {
                    container::Style {
                        border: Border {
                            color: Color::from_rgba(1.0, 0.0, 0.0, 0.5),
                            width: 2.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .into();
        }
        
        column![
            text("Preview").size(20),
            horizontal_rule(10),
            container(preview)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .spacing(20)
        .into()
    }
    
    fn build_widget_preview<'a>(&'a self, widget_type: &'a WidgetType, properties: &'a WidgetProperties) -> Element<'a, Message> {
        match widget_type {
            WidgetType::Container => {
                let mut content = column![
                    text("Container Content"),
                ];
                
                // Add child widgets
                for (child_type, child_props) in &self.child_widgets {
                    content = content.push(self.build_widget_preview(child_type, child_props));
                }
                
                let mut style = container(content)
                    .width(properties.width)
                    .height(properties.height)
                    .padding(properties.padding);
                
                style = style.style(move |_theme: &Theme| {
                    let mut style = container::Style {
                        background: Some(Background::Color(properties.container.background_color)),
                        border: Border {
                            color: properties.container.border_color,
                            width: properties.container.border_width,
                            radius: properties.container.border_radius.into(),
                        },
                        ..Default::default()
                    };
                    
                    if properties.container.has_shadow {
                        style.shadow = Shadow {
                            color: properties.container.shadow_color,
                            offset: properties.container.shadow_offset,
                            blur_radius: properties.container.shadow_blur,
                        };
                    }
                    
                    style
                });
                
                // Apply alignment
                style = match properties.container.align_x {
                    ContainerAlignX::Left => style.align_x(Horizontal::Left),
                    ContainerAlignX::Center => style.center_x(Length::Fill),
                    ContainerAlignX::Right => style.align_x(Horizontal::Right),
                };
                
                style = match properties.container.align_y {
                    ContainerAlignY::Top => style.align_y(Vertical::Top),
                    ContainerAlignY::Center => style.center_y(Length::Fill),
                    ContainerAlignY::Bottom => style.align_y(Vertical::Bottom),
                };
                
                style.into()
            }
            
            WidgetType::Row => {
                let mut content = row![]
                    .spacing(properties.row_column.spacing)
                    .width(properties.width)
                    .height(properties.height)
                    .padding(properties.padding);
                
                if self.child_widgets.is_empty() {
                    content = content.push(text("Row Item 1"));
                    content = content.push(text("Row Item 2"));
                    content = content.push(text("Row Item 3"));
                } else {
                    for (child_type, child_props) in &self.child_widgets {
                        content = content.push(self.build_widget_preview(child_type, child_props));
                    }
                }
                
                // Apply alignment
                content = match properties.row_column.align_items {
                    RowColumnAlign::Start => content.align_y(Alignment::Start),
                    RowColumnAlign::Center => content.align_y(Alignment::Center),
                    RowColumnAlign::End => content.align_y(Alignment::End),
                };
                
                if self.show_spacing && properties.row_column.spacing > 0.0 {
                    container(content)
                        .style(|_: &Theme| {
                            container::Style {
                                background: Some(Background::Color(
                                    Color::from_rgba(0.2, 0.2, 0.8, 0.2)
                                )),
                                ..Default::default()
                            }
                        })
                        .into()
                } else {
                    content.into()
                }
            }
            
            WidgetType::Column => {
                let mut content = column![]
                    .spacing(properties.row_column.spacing)
                    .width(properties.width)
                    .height(properties.height)
                    .padding(properties.padding);
                
                if self.child_widgets.is_empty() {
                    content = content.push(text("Column Item 1"));
                    content = content.push(text("Column Item 2"));
                    content = content.push(text("Column Item 3"));
                } else {
                    for (child_type, child_props) in &self.child_widgets {
                        content = content.push(self.build_widget_preview(child_type, child_props));
                    }
                }
                
                // Apply alignment
                content = match properties.row_column.align_items {
                    RowColumnAlign::Start => content.align_x(Alignment::Start),
                    RowColumnAlign::Center => content.align_x(Alignment::Center),
                    RowColumnAlign::End => content.align_x(Alignment::End),
                };
                
                if self.show_spacing && properties.row_column.spacing > 0.0 {
                    container(content)
                        .style(|_: &Theme| {
                            container::Style {
                                background: Some(Background::Color(
                                    Color::from_rgba(0.2, 0.2, 0.8, 0.2)
                                )),
                                ..Default::default()
                            }
                        })
                        .into()
                } else {
                    content.into()
                }
            }
            
            WidgetType::Button => {
                let mut btn = button(text(&properties.button.text))
                    .padding(properties.padding);
                
                btn = match properties.button.style {
                    ButtonStyleType::Primary => btn.style(button::primary),
                    ButtonStyleType::Secondary => btn.style(button::secondary),
                    ButtonStyleType::Success => btn.style(button::success),
                    ButtonStyleType::Danger => btn.style(button::danger),
                    ButtonStyleType::Text => btn.style(button::text),
                };
                
                btn.into()
            }
            
            WidgetType::Text => {
                let mut txt = text(&properties.text.content)
                    .size(properties.text.size)
                    .color(properties.text.color);
                
                txt = match properties.text.font {
                    FontType::Default => txt.font(Font::default()),
                    FontType::Monospace => txt.font(Font::MONOSPACE),
                };
                
                txt.into()
            }
            
            WidgetType::TextInput => {
                let mut input = text_input(&properties.text_input.placeholder, &properties.text_input.value)
                    .on_input(Message::TextInputValueChanged)
                    .padding(properties.text_input.padding)
                    .size(properties.text_input.size)
                    .width(properties.width);
                
                if properties.text_input.is_secure {
                    input = input.secure(true);
                }
                
                input.into()
            }
            
            WidgetType::Checkbox => {
                checkbox(&properties.checkbox.label, properties.checkbox.is_checked)
                    .on_toggle(Message::CheckboxToggled)
                    .size(properties.checkbox.size)
                    .spacing(properties.checkbox.spacing)
                    .into()
            }
            
            WidgetType::Radio => {
                column![
                    radio(
                        "Option 1",
                        RadioOption::Option1,
                        Some(properties.radio.selected),
                        Message::RadioSelected,
                    )
                    .size(properties.radio.size)
                    .spacing(properties.radio.spacing),
                    
                    radio(
                        "Option 2",
                        RadioOption::Option2,
                        Some(properties.radio.selected),
                        Message::RadioSelected,
                    )
                    .size(properties.radio.size)
                    .spacing(properties.radio.spacing),
                    
                    radio(
                        "Option 3",
                        RadioOption::Option3,
                        Some(properties.radio.selected),
                        Message::RadioSelected,
                    )
                    .size(properties.radio.size)
                    .spacing(properties.radio.spacing),
                ]
                .spacing(10)
                .into()
            }
            
            WidgetType::Slider => {
                column![
                    slider(
                        properties.slider.min..=properties.slider.max,
                        properties.slider.value,
                        Message::SliderValueChanged,
                    )
                    .step(properties.slider.step)
                    .width(properties.width),
                    
                    text(format!("Value: {:.1}", properties.slider.value)).size(14),
                ]
                .spacing(10)
                .into()
            }
            
            WidgetType::ProgressBar => {
                progress_bar(0.0..=1.0, properties.progress.value)
                    .girth(properties.progress.height)
                    .length(properties.width)
                    .into()
            }
            
            WidgetType::Toggler => {
                toggler(properties.toggler.is_active)
                    .on_toggle(Message::TogglerToggled)
                    .label(&properties.toggler.label)
                    .size(properties.toggler.size)
                    .spacing(properties.toggler.spacing)
                    .into()
            }
            
            WidgetType::PickList => {
                pick_list(
                    &properties.pick_list.options[..],
                    properties.pick_list.selected.as_ref(),
                    Message::PickListSelected,
                )
                .placeholder(&properties.pick_list.placeholder)
                .width(properties.width)
                .padding(properties.padding)
                .into()
            }
            
            WidgetType::Scrollable => {
                let mut content = column![];
                
                if self.child_widgets.is_empty() {
                    for i in 0..20 {
                        content = content.push(text(format!("Scrollable Item {}", i + 1)));
                    }
                } else {
                    for (child_type, child_props) in &self.child_widgets {
                        content = content.push(self.build_widget_preview(child_type, child_props));
                    }
                }
                
                scrollable(
                    container(content)
                        .width(Length::Fill)
                        .padding(10)
                )
                .width(Length::Fixed(properties.scrollable.width))
                .height(Length::Fixed(properties.scrollable.height))
                .into()
            }
            
            WidgetType::Space => {
                match (properties.width, properties.height) {
                    (Length::Fixed(w), Length::Fixed(h)) => {
                        Space::new(Length::Fixed(w), Length::Fixed(h))
                    }
                    (Length::Fixed(w), _) => Space::with_width(w),
                    (_, Length::Fixed(h)) => Space::with_height(h),
                    _ => Space::new(properties.width, properties.height),
                }
                .into()
            }
            
            WidgetType::Rule => {
                if matches!(properties.width, Length::Fixed(_)) {
                    vertical_rule(10)
                } else {
                    horizontal_rule(10)
                }
                .into()
            }
        }
    }
    
    pub fn theme(&self) -> Theme {
        match self.theme {
            ThemeType::Light => Theme::Light,
            ThemeType::Dark => Theme::Dark,
            ThemeType::Dracula => Theme::Dracula,
            ThemeType::Nord => Theme::Nord,
            ThemeType::SolarizedLight => Theme::SolarizedLight,
            ThemeType::SolarizedDark => Theme::SolarizedDark,
        }
    }
}

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

impl std::fmt::Display for ThemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for RadioOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}