use iced::{
    widget::{button, column, container, row, text, text_input, scrollable, space},
    Element, Length, Task, Theme, Background, Border, Color
};
use uuid::Uuid;

use crate::icon;
use crate::widget_helper::type_system::*;
use crate::widget_helper::styles::container::*;

// ==================== STATE ====================

#[derive(Debug, Clone)]
pub struct EnumEditorState {
    /// The enum being edited
    pub enum_id: Uuid,
    
    /// Whether this enum's details are expanded
    pub is_expanded: bool,
    
    /// Input field for enum name
    pub name_input: String,
    
    /// Input field for new variant
    pub new_variant_input: String,
    
    /// Any validation errors to display
    pub validation_error: Option<String>,
}

impl EnumEditorState {
    pub fn new(enum_id: Uuid, enum_name: String) -> Self {
        Self {
            enum_id,
            is_expanded: false,
            name_input: enum_name,
            new_variant_input: String::new(),
            validation_error: None,
        }
    }
}

pub struct TypeEditorView {
    /// Reference to the TypeSystem (lives in WidgetVisualizer)
    /// We don't own it, just view it
    
    /// Editor states for each enum
    pub editor_states: Vec<EnumEditorState>,
}

impl TypeEditorView {
    pub fn new() -> Self {
        Self {
            editor_states: Vec::new(),
        }
    }
    
    /// Sync editor states with TypeSystem
    /// Call this whenever TypeSystem changes (after undo/redo, load, etc.)
    pub fn sync_with_type_system(&mut self, type_system: &TypeSystem) {
        // Remove states for deleted enums
        self.editor_states.retain(|state| {
            type_system.get_enum(state.enum_id).is_some()
        });
        
        // Add states for new enums
        for enum_def in type_system.all_enums() {
            if !self.editor_states.iter().any(|s| s.enum_id == enum_def.id) {
                self.editor_states.push(EnumEditorState::new(
                    enum_def.id,
                    enum_def.name.clone(),
                ));
            }
        }
        
        // Update names for existing states
        for state in &mut self.editor_states {
            if let Some(enum_def) = type_system.get_enum(state.enum_id) {
                // Only update if not currently being edited
                if !state.is_expanded {
                    state.name_input = enum_def.name.clone();
                }
            }
        }
    }
}

// ==================== MESSAGES ====================

#[derive(Debug, Clone)]
pub enum Message {
    // Enum operations
    CreateNewEnum,
    DeleteEnum(Uuid),
    RenameEnum { enum_id: Uuid, new_name: String },
    
    // Variant operations
    AddVariant { enum_id: Uuid, name: String },
    RemoveVariant { enum_id: Uuid, variant_id: Uuid },
    UpdateVariant { enum_id: Uuid, variant_id: Uuid, new_name: String },
    
    // UI state
    ToggleExpanded(Uuid),
    EnumNameInputChanged { enum_id: Uuid, value: String },
    NewVariantInputChanged { enum_id: Uuid, value: String },
    SaveEnum(Uuid),
    
    // Undo/Redo
    Undo,
    Redo,
}

// ==================== UPDATE ====================

pub fn update(
    message: Message,
    type_system: &mut TypeSystem,
    editor_view: &mut TypeEditorView,
) -> Task<Message> {
    match message {
        Message::CreateNewEnum => {
            let count = type_system.enum_count() + 1;
            match type_system.add_enum(
                format!("NewEnum{}", count),
                vec!["Variant1".to_string()]
            ) {
                Ok(enum_id) => {
                    editor_view.sync_with_type_system(type_system);
                    // Expand the new enum
                    if let Some(state) = editor_view.editor_states.iter_mut()
                        .find(|s| s.enum_id == enum_id) {
                        state.is_expanded = true;
                    }
                }
                Err(e) => eprintln!("Error creating enum: {}", e),
            }
        }
        
        Message::DeleteEnum(enum_id) => {
            match type_system.remove_enum(enum_id) {
                Ok(()) => {
                    editor_view.sync_with_type_system(type_system);
                }
                Err(e) => {
                    // Show error in the UI
                    if let Some(state) = editor_view.editor_states.iter_mut()
                        .find(|s| s.enum_id == enum_id) {
                        state.validation_error = Some(e);
                    }
                }
            }
        }
        
        Message::RenameEnum { enum_id, new_name } => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                
                match type_system.update_enum_name(enum_id, new_name) {
                    Ok(()) => {
                        state.validation_error = None;
                        state.is_expanded = false; // Collapse after save
                    }
                    Err(e) => {
                        state.validation_error = Some(e);
                    }
                }
            }
        }
        
        Message::AddVariant { enum_id, name } => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                
                match type_system.add_variant(enum_id, name) {
                    Ok(_variant_id) => {
                        state.new_variant_input.clear();
                        state.validation_error = None;
                    }
                    Err(e) => {
                        state.validation_error = Some(e);
                    }
                }
            }
        }
        
        Message::RemoveVariant { enum_id, variant_id } => {
            if let Err(e) = type_system.remove_variant(enum_id, variant_id) {
                if let Some(state) = editor_view.editor_states.iter_mut()
                    .find(|s| s.enum_id == enum_id) {
                    state.validation_error = Some(e);
                }
            }
        }
        
        Message::UpdateVariant { enum_id, variant_id, new_name } => {
            if let Err(e) = type_system.update_variant(enum_id, variant_id, new_name) {
                if let Some(state) = editor_view.editor_states.iter_mut()
                    .find(|s| s.enum_id == enum_id) {
                    state.validation_error = Some(e);
                }
            }
        }
        
        Message::ToggleExpanded(enum_id) => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                state.is_expanded = !state.is_expanded;
                state.validation_error = None;
            }
        }
        
        Message::EnumNameInputChanged { enum_id, value } => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                state.name_input = value;
                state.validation_error = None;
            }
        }
        
        Message::NewVariantInputChanged { enum_id, value } => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                state.new_variant_input = value;
                state.validation_error = None;
            }
        }
        
        Message::SaveEnum(enum_id) => {
            if let Some(state) = editor_view.editor_states.iter_mut()
                .find(|s| s.enum_id == enum_id) {
                
                let new_name = state.name_input.clone();
                match type_system.update_enum_name(enum_id, new_name) {
                    Ok(()) => {
                        state.validation_error = None;
                        state.is_expanded = false;
                    }
                    Err(e) => {
                        state.validation_error = Some(e);
                    }
                }
            }
        }
        
        Message::Undo => {
            if let Err(e) = type_system.undo() {
                eprintln!("Undo failed: {}", e);
            }
            editor_view.sync_with_type_system(type_system);
        }
        
        Message::Redo => {
            if let Err(e) = type_system.redo() {
                eprintln!("Redo failed: {}", e);
            }
            editor_view.sync_with_type_system(type_system);
        }
    }
    
    Task::none()
}

// ==================== VIEW ====================

pub fn view<'a>(
    type_system: &'a TypeSystem,
    editor_view: &'a TypeEditorView,
) -> Element<'a, Message> {
    let mut content = column![
        // Header
        row![
            text("Type Definitions").size(24),
            space::horizontal(),
            // Undo/Redo buttons
            button(text("↶"))
                .on_press_maybe(type_system.can_undo().then_some(Message::Undo))
                .padding(8),
            button(text("↷"))
                .on_press_maybe(type_system.can_redo().then_some(Message::Redo))
                .padding(8),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
        
        text("Define custom enums for your UI components").size(14),
    ]
    .width(400)
    .spacing(10)
    .padding(10);
    
    // List all enums
    for state in &editor_view.editor_states {
        if let Some(enum_def) = type_system.get_enum(state.enum_id) {
            let enum_view = view_single_enum(type_system, enum_def, state);
            content = content.push(enum_view);
        }
    }
    
    // Add new enum button
    content = content.push(
        button(
            row![
                icon::plus().center(),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
        )
        .on_press(Message::CreateNewEnum)
        .style(button::primary)
    );
    
    scrollable(content).into()
}

fn view_single_enum<'a>(
    type_system: &'a TypeSystem,
    enum_def: &'a EnumDef,
    state: &'a EnumEditorState,
) -> Element<'a, Message> {
    let dependents = type_system.get_dependents(enum_def.id);
    let is_in_use = !dependents.is_empty();
    
    if state.is_expanded {
        view_enum_expanded(type_system, enum_def, state, is_in_use, &dependents)
    } else {
        view_enum_collapsed(enum_def, is_in_use, dependents.len())
    }
}

fn view_enum_collapsed<'a>(
    enum_def: &'a EnumDef,
    is_in_use: bool,
    dependent_count: usize,
) -> Element<'a, Message> {
    container(
        row![
            // Expand arrow
            button(icon::collapsed().center())
                .on_press(Message::ToggleExpanded(enum_def.id))
                .style(button::text),
            
            // Enum name
            text(&enum_def.name).size(18),
            
            // Usage indicator
            if is_in_use {
                text(format!("({} widget{})", dependent_count, if dependent_count == 1 { "" } else { "s" }))
                    .size(12)
                    .style(text::secondary)
            } else {
                text("").size(12).style(text::secondary)
            },
            
            space::horizontal(),
            
            // Edit button
            button(icon::edit().center())
                .on_press(Message::ToggleExpanded(enum_def.id))
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
    )
    .padding(10)
    .style(rounded_box)
    .into()
}

fn view_enum_expanded<'a>(
    type_system: &'a TypeSystem,
    enum_def: &'a EnumDef,
    state: &'a EnumEditorState,
    is_in_use: bool,
    dependents: &[String],
) -> Element<'a, Message> {
    let mut content = column![].spacing(10);
    
    // Collapse button and name input
    let header = row![
        button(icon::expanded().center())
            .on_press(Message::ToggleExpanded(enum_def.id))
            .style(button::text),
        
        text("Name:").size(14),
        
        text_input("Enum name...", &state.name_input)
            .on_input(move |value| Message::EnumNameInputChanged {
                enum_id: enum_def.id,
                value,
            })
            .padding(8)
            .width(Length::Fill),
        
        button(icon::trash().center())
            .on_press_maybe(
                if is_in_use {
                    None // Can't delete if in use
                } else {
                    Some(Message::DeleteEnum(enum_def.id))
                }
            )
            .style(if is_in_use { button::secondary } else { button::danger }),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);
    
    content = content.push(header);
    
    // Show usage warning if in use
    if is_in_use {
        let warning = container(
            column![
                text(format!("This enum is used by {} widget(s)", dependents.len()))
                    .size(12),
                text(format!("Widgets: {}", dependents.join(", ")))
                    .size(11),
            ]
            .spacing(5)
        )
        .padding(10)
        .style(warning_box);
        
        content = content.push(warning);
    }
    
    // Variants section
    content = content.push(text("Variants:").size(14));
    
    for variant in &enum_def.variants {
        let variant_row = row![
            text_input("Variant name...", &variant.name)
                .on_input(move |value| Message::UpdateVariant {
                    enum_id: enum_def.id,
                    variant_id: variant.id,
                    new_name: value,
                })
                .padding(8)
                .width(Length::Fill),
            
            button(icon::trash().center())
                .on_press(Message::RemoveVariant {
                    enum_id: enum_def.id,
                    variant_id: variant.id,
                })
                .style(button::danger),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);
        
        content = content.push(variant_row);
    }
    
    // Add new variant
    let add_variant_row = row![
        text_input("New variant...", &state.new_variant_input)
            .on_input(move |value| Message::NewVariantInputChanged {
                enum_id: enum_def.id,
                value,
            })
            .on_submit(Message::AddVariant {
                enum_id: enum_def.id,
                name: state.new_variant_input.clone(),
            })
            .padding(8)
            .width(Length::Fill),
        
        button(icon::plus().center())
            .on_press(Message::AddVariant {
                enum_id: enum_def.id,
                name: state.new_variant_input.clone(),
            })
            .style(button::primary),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);
    
    content = content.push(add_variant_row);
    
    // Validation error
    if let Some(error) = &state.validation_error {
        content = content.push(
            container(
                text(error).size(12)
            )
            .padding(10)
            .style(error_box)
        );
    }
    
    // Save button
    let save_button = button(icon::save().center())
        .on_press(Message::SaveEnum(enum_def.id))
        .style(button::primary);
    
    content = content.push(
        row![space::horizontal(), save_button]
    );
    
    container(content)
        .padding(10)
        .style(rounded_box)
        .into()
}