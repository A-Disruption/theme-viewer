use iced::{Color, Element, Length, Padding, widget::{column, container, space::horizontal, row, scrollable, text}, Background, Border, Theme};
use crate::widget_helper::*;
use crate::widget_helper::type_system::EnumDef;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Keyword,      // let, fn, impl, use, pub, enum, match, move
    Type,         // Element, Message, Length, Color, etc.
    Function,     // function names
    String,       // string literals
    Number,       // numeric literals
    Comment,      // comments
    Operator,     // =, ->, ::, .
    Identifier,   // variable names
    Macro,        // column!, row!, etc.
    Plain,        // everything else
}

impl TokenType {
    pub fn color(&self) -> Color {
        match self {
            TokenType::Keyword => Color::from_rgb8(86, 156, 214),     // Blue
            TokenType::Type => Color::from_rgb8(78, 201, 176),        // Teal
            TokenType::Function => Color::from_rgb8(220, 220, 170),   // Light yellow
            TokenType::String => Color::from_rgb8(206, 145, 120),     // Orange
            TokenType::Number => Color::from_rgb8(181, 206, 168),     // Light green
            TokenType::Comment => Color::from_rgb8(106, 153, 85),     // Green
            TokenType::Operator => Color::from_rgb8(212, 212, 212),   // Light gray
            TokenType::Identifier => Color::from_rgb8(156, 220, 254), // Light blue
            TokenType::Macro => Color::from_rgb8(197, 134, 192),      // Purple
            TokenType::Plain => Color::from_rgb8(212, 212, 212),      // Light gray
        }
    }
}

impl TokenType {
    pub fn color_for_theme(&self, theme: &Theme) -> Color {
        let palette = theme.extended_palette();

        match theme {
            Theme::Light => match self {
                TokenType::Keyword => Color::from_rgb8(0, 0, 255),         // Blue (like VSCode)
                TokenType::Type => Color::from_rgb8(0, 128, 128),          // Teal
                TokenType::Function => Color::from_rgb8(121, 94, 38),      // Brown/yellow
                TokenType::String => Color::from_rgb8(163, 21, 21),        // Dark red
                TokenType::Number => Color::from_rgb8(9, 134, 88),         // Green
                TokenType::Comment => Color::from_rgb8(0, 128, 0),         // Green
                TokenType::Operator => Color::from_rgb8(0, 0, 0),          // Black
                TokenType::Identifier => Color::from_rgb8(0, 16, 128),     // Dark blue
                TokenType::Macro => Color::from_rgb8(175, 0, 219),         // Purple
                TokenType::Plain => Color::from_rgb8(0, 0, 0),             // Black
            },
            Theme::Dark => match self {
                TokenType::Keyword => Color::from_rgb8(86, 156, 214),      // Blue
                TokenType::Type => Color::from_rgb8(78, 201, 176),        // Teal/cyan
                TokenType::Function => Color::from_rgb8(220, 220, 170),    // Light yellow
                TokenType::String => Color::from_rgb8(206, 145, 120),      // Orange/salmon
                TokenType::Number => Color::from_rgb8(181, 206, 168),      // Light green
                TokenType::Comment => Color::from_rgb8(106, 153, 85),      // Green
                TokenType::Operator => Color::from_rgb8(212, 212, 212),    // Light gray
                TokenType::Identifier => Color::from_rgb8(156, 220, 254),  // Light blue
                TokenType::Macro => Color::from_rgb8(197, 134, 192),       // Purple
                TokenType::Plain => Color::from_rgb8(212, 212, 212),       // Light gray
            },
            _ => {
                // Default/custom theme colors
                match self {
                    TokenType::Keyword => palette.danger.base.color,
                    TokenType::Type => palette.primary.strong.color,
                    TokenType::Function => palette.warning.weak.color,
                    TokenType::String => palette.warning.base.color,
                    TokenType::Number => palette.primary.weak.color,
                    TokenType::Comment => palette.success.weak.color,
                    TokenType::Operator => palette.danger.weak.color,
                    TokenType::Identifier => palette.primary.base.color,
                    TokenType::Macro => palette.success.base.color,
                    TokenType::Plain => palette.secondary.base.color,
                }
            }
        }
    }
}

/// A highlighted token in the code
#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
}

/// Helper struct for building token streams with proper syntax highlighting
pub struct TokenBuilder {
    tokens: Vec<Token>,
    indent_level: usize,
}

impl TokenBuilder {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            indent_level: 0,
        }
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn set_indent(&mut self, level: usize) {
        self.indent_level = level;
    }

    pub fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn add_keyword(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Keyword,
        });
    }

    pub fn add_type(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Type,
        });
    }

    pub fn add_function(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Function,
        });
    }

    pub fn add_number(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Number,
        });
    }

    pub fn add_plain(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Plain,
        });
    }

    pub fn add_operator(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Operator,
        });
    }

    pub fn add_identifier(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Identifier,
        });
    }

    pub fn add_string(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::String,
        });
    }

    pub fn add_comment(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Comment,
        });
    }

    pub fn add_macro(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Macro,
        });
    }

    pub fn add_newline(&mut self) {
        self.tokens.push(Token {
            text: "\n".to_string(),
            token_type: TokenType::Plain,
        });
    }

    pub fn add_indent(&mut self) {
        self.tokens.push(Token {
            text: "    ".repeat(self.indent_level),
            token_type: TokenType::Plain,
        });
    }

    pub fn add_space(&mut self) {
        self.tokens.push(Token {
            text: " ".to_string(),
            token_type: TokenType::Plain,
        });
    }

    // Helper methods for common patterns
    pub fn add_color(&mut self, color: Color) {
        self.add_type("Color");
        self.add_operator("::");
        self.add_function("from_rgba");
        self.add_plain("(");
        self.add_number(&format!("{:.1}", color.r));
        self.add_plain(", ");
        self.add_number(&format!("{:.1}", color.g));
        self.add_plain(", ");
        self.add_number(&format!("{:.1}", color.b));
        self.add_plain(", ");
        self.add_number(&format!("{:.1}", color.a));
        self.add_plain(")");
    }

    pub fn add_color_hex(&mut self, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        
        if color.a < 1.0 {
            let a = (color.a * 255.0) as u8;
            self.add_type("Color");
            self.add_operator("::");
            self.add_function("from_rgba8");
            self.add_plain("(");
            self.add_number(&format!("0x{:02X}", r));
            self.add_plain(", ");
            self.add_number(&format!("0x{:02X}", g));
            self.add_plain(", ");
            self.add_number(&format!("0x{:02X}", b));
            self.add_plain(", ");
            self.add_number(&format!("0x{:02X}", a));
            self.add_plain(")");
        } else {
            self.add_type("Color");
            self.add_operator("::");
            self.add_function("from_rgb8");
            self.add_plain("(");
            self.add_number(&format!("0x{:02X}", r));
            self.add_plain(", ");
            self.add_number(&format!("0x{:02X}", g));
            self.add_plain(", ");
            self.add_number(&format!("0x{:02X}", b));
            self.add_plain(")");
        }
    }

    pub fn add_field(&mut self, name: &str, value_fn: impl FnOnce(&mut Self)) {
        self.add_indent();
        self.add_plain(name);
        self.add_operator(":");
        self.add_space();
        value_fn(self);
        self.add_plain(",");
        self.add_newline();
    }

    pub fn add_struct(&mut self, name: &str, fields_fn: impl FnOnce(&mut Self)) {
        self.add_type(name);
        self.add_space();
        self.add_plain("{");
        self.add_newline();
        self.increase_indent();
        fields_fn(self);
        self.decrease_indent();
        self.add_indent();
        self.add_plain("}");
    }
}

/// Code generator for creating Iced code from widget hierarchy
pub struct CodeGenerator<'a> {
    hierarchy: &'a WidgetHierarchy,
    indent_level: usize,
    tokens: Vec<Token>,
    app_name: String,
    app_window_title: String,
    widget_counts: HashMap<String, usize>,  // Track duplicate widgets
    used_widgets: HashSet<&'static str>,  // Track which widgets are used for the impl code gen
    widget_names: HashMap<WidgetId, String>,
    type_system: Option<&'a TypeSystem>,
    theme: Theme,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(hierarchy: &'a WidgetHierarchy, theme: Theme, type_system: Option<&'a TypeSystem>) -> Self {
        Self {
            hierarchy,
            indent_level: 0,
            tokens: Vec::new(),
            app_name: "App".to_string(),
            app_window_title: "App Window".to_string(),
            widget_counts: HashMap::new(),
            used_widgets: HashSet::new(),
            widget_names: HashMap::new(),
            type_system: type_system,
            theme,
        }
    }

    /// Set App name for code generation
    pub fn set_app_name(&mut self, name: String) {
        self.app_name = if name.trim().is_empty() { 
            "App".to_string() 
        } else { 
            // Ensure it's a valid Rust identifier
            name.chars()
                .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                .collect::<String>()
        };
    }

    /// Set Window Title for code generation
    pub fn set_window_title(&mut self, name: String) {
        self.app_window_title = if name.trim().is_empty() { 
            "App".to_string() 
        } else { 
            // Ensure it's a valid Rust identifier
            name.chars()
                .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                .collect::<String>()
        };
    }

    /// Generate code for a specific widget
    pub fn generate_widget_code(&mut self, widget_id: WidgetId) -> Vec<Token> {
        self.tokens.clear();
        self.indent_level = 0;
        
        if let Some(widget) = self.hierarchy.get_widget_by_id(widget_id) {
            self.generate_widget_creation(widget, false);
        }
        
        self.tokens.clone()
    }

    fn generate_enum_definitions(&mut self) {
        if self.type_system.is_none() { return }
        for enum_def in self.type_system.unwrap().enums.values() {
            self.generate_enum_code(enum_def);
            self.add_newline();
            self.add_newline();
        }
    }

    fn generate_enum_code(&mut self, enum_def: &EnumDef) {
        self.add_comment(&format!("// {} enum", enum_def.name));
        self.add_newline();
        self.add_plain("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
        self.add_newline();
        self.add_keyword("pub enum");
        self.add_plain(" ");
        self.add_type(&enum_def.name);
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        for variant in &enum_def.variants {
            self.add_indent();
            self.add_plain(&variant.name);
            self.add_plain(",");
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_plain("}");
        self.add_newline();
        self.add_newline();
        
        // Generate Display impl
        self.generate_enum_display_impl(enum_def);
        self.add_newline();
        
        // Generate ALL constant for combo_box
        self.generate_enum_all_const(enum_def);
    }

    fn generate_enum_display_impl(&mut self, enum_def: &EnumDef) {
        self.add_keyword("impl");
        self.add_plain(" std::fmt::Display ");
        self.add_keyword("for");
        self.add_plain(" ");
        self.add_type(&enum_def.name);
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("fmt");
        self.add_plain("(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_keyword("match");
        self.add_plain(" self {");
        self.add_newline();
        self.indent_level += 1;
        
        for variant in &enum_def.variants {
            self.add_indent();
            self.add_type(&enum_def.name);
            self.add_operator("::");
            self.add_plain(&variant.name);
            self.add_plain(" => write!(f, ");
            self.add_string(&format!("\"{}\"", &variant.name));
            self.add_plain("),");
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_plain("}");
    }

    fn generate_enum_all_const(&mut self, enum_def: &EnumDef) {
        self.add_keyword("impl");
        self.add_plain(" ");
        self.add_type(&enum_def.name);
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_keyword("pub const");
        self.add_plain(" ALL: &'static [Self] = &[");
        self.add_newline();
        self.indent_level += 1;
        
        for variant in &enum_def.variants {
            self.add_indent();
            self.add_plain("Self::");
            self.add_plain(&variant.name);
            self.add_plain(",");
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("];");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_plain("}");
    }

    /// Generate complete application code
    pub fn generate_app_code(&mut self) -> Vec<Token> {
        self.tokens.clear();
        self.indent_level = 0;
        self.used_widgets.clear();
        
        // CRITICAL: Generate all widget names ONCE at the beginning
        self.generate_all_widget_names();
        
        // First pass: collect all used widgets
        self.collect_used_widgets(&self.hierarchy.root().clone());
        
        // Generate imports
        self.generate_imports();
        self.add_newline();
        self.add_newline();

        // Generate enum definitions
        self.generate_enum_definitions();
        self.add_newline();
        
        // Generate Message enum
        self.generate_message_enum();
        self.add_newline();
        self.add_newline();
        
        // Generate App struct
        self.generate_app_struct();
        self.add_newline();
        self.add_newline();
        
        // Generate impl block (now just impl App, not impl Application for App)
        self.generate_impl_block();
        self.add_newline();
        self.add_newline();
        
        // Generate main function with new iced API
        self.generate_main_function();
        
        self.tokens.clone()
    }


    // Generate unique name for duplicate widgets
    fn get_unique_widget_name(&mut self, widget: &Widget) -> String {
        // If widget has a custom name, use it
        if !widget.properties.widget_name.trim().is_empty() {
            return self.sanitize_name(&widget.properties.widget_name);
        }
        
        // Otherwise, use widget type as base name
        let base_name = match widget.widget_type {
            WidgetType::Button => "button",
            WidgetType::Text => "text",
            WidgetType::TextInput => "text_input",
            WidgetType::Checkbox => "checkbox",
            WidgetType::Radio => "radio",
            WidgetType::Slider => "slider",
            WidgetType::VerticalSlider => "vertical_slider",
            WidgetType::ProgressBar => "progress_bar",
            WidgetType::Toggler => "toggler",
            WidgetType::PickList => "pick_list",
            _ => return format!("{:?}", widget.widget_type).to_lowercase(), // Fallback to type name
        }.to_string();
        
        // Add number if there are duplicates
        let type_key = format!("{:?}", widget.widget_type).to_lowercase();
        let count = self.widget_counts.entry(type_key).or_insert(0);
        *count += 1;
        
        if *count > 1 {
            format!("{}_{}", base_name, count)
        } else {
            base_name
        }
    }

    fn generate_new_method(&mut self) {
        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("new");
        self.add_plain("() ");
        self.add_operator("->");
        self.add_plain(" (");
        self.add_keyword("Self");
        self.add_plain( ", " );
        self.add_number("iced");
        self.add_operator("::");
        self.add_type("Task");
        self.add_plain("<");
        self.add_type("Message");
        self.add_plain( ">) {" );
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_plain("(");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_keyword("Self");
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        // Initialize state fields
        self.widget_counts.clear();
        self.generate_state_initializers(&self.hierarchy.root().clone());
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("},");
        self.add_newline();
        
        self.add_indent();
        self.add_number("iced");
        self.add_operator("::");
        self.add_type("Task");
        self.add_operator("::");
        self.add_plain("none()");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain(")");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
    }

    fn generate_state_initializers(&mut self, widget: &Widget) {
        let name = self.get_widget_name(widget.id);
        let props = &widget.properties;
        
        match widget.widget_type {
            WidgetType::TextInput => {
                self.add_indent();
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" String::new(),");
                self.add_newline();
            }
            WidgetType::Checkbox => {
                self.add_indent();
                self.add_identifier(&format!("{}_checked", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_keyword(if props.checkbox_checked { "true" } else { "false" });
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Radio => {
                self.add_indent();
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_number(&format!("{}", props.radio_selected_index));
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Slider | WidgetType::VerticalSlider => {
                self.add_indent();
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_number(&format!("{:.1}", props.slider_value));
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Toggler => {
                self.add_indent();
                self.add_identifier(&format!("{}_active", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_keyword(if props.toggler_active { "true" } else { "false" });
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::PickList => {
                self.add_indent();
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" None,");
                self.add_newline();
            }
            WidgetType::ComboBox => {
                if self.type_system.is_none() { return }
                // Get the enum definition and initialize properly
                if let Some(ref enum_id) = props.referenced_enum {
                    if let Some(enum_def) = self.type_system.unwrap().get_enum(enum_id.clone()) {
                        self.add_indent();
                        self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                        self.add_operator(":");
                        self.add_plain(" ");
                        self.add_type(&enum_def.name);
                        self.add_operator("::");
                        self.add_plain(&enum_def.variants[0].name);
                        self.add_plain(",");
                        self.add_newline();
                        
                        // Initialize state with all variants
                        self.add_indent();
                        self.add_identifier(&format!("{}_state", to_snake_case(&name)));
                        self.add_operator(":");
                        self.add_plain(" ");
                        self.add_type("combo_box::State");
                        self.add_operator("::");
                        self.add_function("new");
                        self.add_plain("(");
                        self.add_type(&enum_def.name);
                        self.add_operator("::");
                        self.add_plain("ALL.to_vec()");
                        self.add_plain("),");
                        self.add_newline();  
                    }
                } else {
                    self.add_indent();
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                    self.add_operator(":");
                    self.add_plain(" String::new(),");
                    self.add_newline();
                    
                    self.add_indent();
                    self.add_identifier(&format!("{}_state", to_snake_case(&name)));
                    self.add_operator(":");
                    self.add_plain(" ");
                    self.add_type("combo_box::State");
                    self.add_operator("::");
                    self.add_function("new");
                    self.add_plain("(vec![");
                    for (i, option) in props.combobox_options.iter().enumerate() {
                        self.add_string(&format!("\"{}\"", option));
                        self.add_operator(".");
                        self.add_function("to_string");
                        self.add_plain("()");
                        if i < props.combobox_options.len() - 1 {
                            self.add_plain(", ");
                        }
                    }
                    self.add_plain("]),");
                    self.add_newline();
                }
            }
            _ => {}
        }
        
        for child in &widget.children {
            self.generate_state_initializers(child);
        }
    }

    fn generate_title_method(&mut self) {
        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("title");
        self.add_plain("(");
        self.add_operator("&");
        self.add_keyword("self");
        self.add_plain(") ");
        self.add_operator("->");
        self.add_plain(" ");
        self.add_type("String");
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_type("String");
        self.add_operator("::");
        self.add_function("from");
        self.add_plain("(");
        self.add_string(&format!("\"{}\"", self.app_window_title));
        self.add_plain(")");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
    }

    fn generate_theme_method(&mut self, theme: Theme) {
        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("theme");
        self.add_plain("(");
        self.add_operator("&");
        self.add_keyword("self");
        self.add_plain(") ");
        self.add_operator("->");
        self.add_plain(" ");
        self.add_type("Theme");
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_type("Theme");
        self.add_operator("::");
        // Use the actual theme passed in
        match theme {
            Theme::Light => self.add_plain("Light"),
            Theme::Dark => self.add_plain("Dark"),
            Theme::Dracula => self.add_plain("Dracula"),
            Theme::Nord => self.add_plain("Nord"),
            Theme::SolarizedLight => self.add_plain("SolarizedLight"),
            Theme::SolarizedDark => self.add_plain("SolarizedDark"),
            Theme::GruvboxLight => self.add_plain("GruvboxLight"),
            Theme::GruvboxDark => self.add_plain("GruvboxDark"),
            Theme::CatppuccinLatte => self.add_plain("CatppuccinLatte"),
            Theme::CatppuccinFrappe => self.add_plain("CatppuccinFrappe"),
            Theme::CatppuccinMacchiato => self.add_plain("CatppuccinMacchiato"),
            Theme::CatppuccinMocha => self.add_plain("CatppuccinMocha"),
            Theme::TokyoNight => self.add_plain("TokyoNight"),
            Theme::TokyoNightStorm => self.add_plain("TokyoNightStorm"),
            Theme::TokyoNightLight => self.add_plain("TokyoNightLight"),
            Theme::KanagawaWave => self.add_plain("KanagawaWave"),
            Theme::KanagawaDragon => self.add_plain("KanagawaDragon"),
            Theme::KanagawaLotus => self.add_plain("KanagawaLotus"),
            Theme::Moonfly => self.add_plain("Moonfly"),
            Theme::Nightfly => self.add_plain("Nightfly"),
            Theme::Oxocarbon => self.add_plain("Oxocarbon"),
            Theme::Ferra => self.add_plain("Ferra"),
            _ => self.add_plain("Dark"), // Default fallback for custom themes
        }
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
    }

    // Updated impl block generation
    fn generate_impl_block(&mut self) {
        self.add_keyword("impl");
        self.add_plain(" ");
        self.add_type(&self.app_name.clone());
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        // Generate new method
        self.generate_new_method();
        self.add_newline();
        self.add_newline();
        
        // Generate title method
        self.generate_title_method();
        self.add_newline();
        self.add_newline();
        
        // Generate theme method  
        self.generate_theme_method(self.theme.clone());
        self.add_newline();
        self.add_newline();
        
        // Generate update method
        self.generate_update_method();
        self.add_newline();
        self.add_newline();
        
        // Generate view method
        self.generate_view_method();
        
        self.indent_level -= 1;
        self.add_newline();
        self.add_plain("}");
    }

    fn generate_update_method(&mut self) {
        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("update");
        self.add_plain("(");
        self.add_operator("&");
        self.add_keyword("mut");
        self.add_plain(" ");
        self.add_keyword("self");
        self.add_plain(", ");
        self.add_identifier("message");
        self.add_operator(":");
        self.add_plain(" ");
        self.add_type("Message");
        self.add_plain(") {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_keyword("match");
        self.add_plain(" ");
        self.add_identifier("message");
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        // Generate match arms for each message
        self.generate_update_match_arms(&self.hierarchy.root().clone());
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
        self.add_newline();
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
    }

    fn generate_main_function(&mut self) {
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("main");
        self.add_plain("() ");
        self.add_operator("->");
        self.add_plain(" iced::Result {");
        self.add_newline();
        self.indent_level += 1;
        
        self.add_indent();
        self.add_plain("iced::application(");
        self.add_type(&self.app_name.clone());
        self.add_operator("::");
        self.add_plain("new, ");
        self.add_type(&self.app_name.clone());
        self.add_operator("::");
        self.add_plain("update, ");
        self.add_type(&self.app_name.clone());
        self.add_operator("::");
        self.add_plain("view)");
        self.add_newline();
        
        self.indent_level += 1;
        self.add_indent();
        self.add_operator(".");
        self.add_function("theme");
        self.add_plain("(");
        self.add_type(&self.app_name.clone());
        self.add_operator("::");
        self.add_plain("theme)");
        self.add_newline();
        
        self.add_indent();
        self.add_operator(".");
        self.add_function("title");
        self.add_plain("(");
        self.add_type(&self.app_name.clone());
        self.add_operator("::");
        self.add_plain("title)");
        self.add_newline();
        
        self.add_indent();
        self.add_operator(".");
        self.add_function("run");
        self.add_plain("()");
        self.add_newline();
        
        self.indent_level -= 2;
        self.add_plain("}");
    }

    fn generate_imports(&mut self) {
        // Scan the entire hierarchy
        let mut tracker = ImportTracker::new();
        tracker.scan_widget(&self.hierarchy.root().clone());
        
        self.add_keyword("use");
        self.add_number(" iced::");
        self.add_plain("{");
        self.add_newline();
        self.indent_level += 1;
        
        // Core types - build list
        let mut core_imports = Vec::new();
        
        if tracker.uses_length {
            core_imports.push("Length");
        }
        if tracker.uses_alignment {
            core_imports.push("Alignment");
        }
        if tracker.uses_color {
            core_imports.push("Color");
        }
        if tracker.uses_padding {
            core_imports.push("Padding");
        }
        if tracker.uses_font {
            core_imports.push("Font");
        }
        if tracker.uses_border {
            core_imports.push("Border");
        }
        if tracker.uses_shadow {
            core_imports.push("Shadow");
        }
        if tracker.uses_background {
            core_imports.push("Background");
        }
        if tracker.uses_vector {
            core_imports.push("Vector");
        }
        if tracker.uses_point {
            core_imports.push("Point");
        }
        
        // Element, Theme, and Task are always needed
        core_imports.push("Element");
        core_imports.push("Theme");
        core_imports.push("Task");

        self.add_indent();
        core_imports.into_iter().for_each(|import| {
            self.add_type(import);
            self.add_plain(",");
        });
        self.add_newline();
        
/*         if !core_imports.is_empty() {
            self.add_indent();
            self.add_plain(&core_imports.join(", "));
            self.add_plain(",");
            self.add_newline();
        } */
        
        // Widget imports
        if !tracker.used_widgets.is_empty() {
            self.add_indent();
            self.add_number("widget");
            self.add_operator("::");
            self.add_plain("{");
            let mut widgets: Vec<_> = tracker.used_widgets.iter().map(|s| *s).collect();
            widgets.sort();
            self.add_plain(&widgets.join(", "));
            self.add_plain("},");
            self.add_newline();
        }
        
        // Mouse module - only if MouseArea is used
        if tracker.uses_mouse {
            self.add_indent();
            self.add_plain("mouse");
            
            let mut mouse_items = Vec::new();
            if tracker.uses_mouse_interaction {
                mouse_items.push("Interaction");
            }
            if tracker.uses_mouse_scroll_delta {
                mouse_items.push("ScrollDelta");
            }
            
            if !mouse_items.is_empty() {
                self.add_plain("::{");
                self.add_plain(&mouse_items.join(", "));
                self.add_plain("}");
            }
            self.add_plain(",");
            self.add_newline();
        }
        
        // Text module - only if text properties are used
        if tracker.uses_text_line_height || tracker.uses_text_wrapping || 
        tracker.uses_text_shaping || tracker.uses_text_alignment {
            self.add_indent();
            self.add_plain("widget::text");
            
            let mut text_items = Vec::new();
            if tracker.uses_text_line_height {
                text_items.push("LineHeight");
            }
            if tracker.uses_text_wrapping {
                text_items.push("Wrapping");
            }
            if tracker.uses_text_shaping {
                text_items.push("Shaping");
            }
            if tracker.uses_text_alignment {
                text_items.push("Alignment as TextAlignment");
            }
            
            if !text_items.is_empty() {
                self.add_plain("::{");
                self.add_plain(&text_items.join(", "));
                self.add_plain("}");
            }
            self.add_plain(",");
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_plain("};");
        self.add_newline();
    }

    // Collect which widgets are actually used
    fn collect_used_widgets(&mut self, widget: &Widget) {
        match widget.widget_type {
            WidgetType::Container => self.used_widgets.insert("container"),
            WidgetType::Row => self.used_widgets.insert("row"),
            WidgetType::Column => self.used_widgets.insert("column"),
            WidgetType::Button => self.used_widgets.insert("button"),
            WidgetType::Text => self.used_widgets.insert("text"),
            WidgetType::TextInput => self.used_widgets.insert("text_input"),
            WidgetType::Checkbox => self.used_widgets.insert("checkbox"),
            WidgetType::Radio => self.used_widgets.insert("radio"),
            WidgetType::Slider => self.used_widgets.insert("slider"),
            WidgetType::VerticalSlider => self.used_widgets.insert("vertical_slider"),
            WidgetType::ProgressBar => self.used_widgets.insert("progress_bar"),
            WidgetType::Toggler => self.used_widgets.insert("toggler"),
            WidgetType::PickList => self.used_widgets.insert("pick_list"),
            WidgetType::Scrollable => self.used_widgets.insert("scrollable"),
            WidgetType::Space => self.used_widgets.insert("space"),
            WidgetType::Rule => self.used_widgets.insert("rule"),
            WidgetType::Image => self.used_widgets.insert("image"),
            WidgetType::Svg => self.used_widgets.insert("svg"),
            WidgetType::Tooltip => self.used_widgets.insert("tooltip"),
            WidgetType::ComboBox => self.used_widgets.insert("combo_box"),
            WidgetType::Markdown => self.used_widgets.insert("markdown"),
            WidgetType::MouseArea => self.used_widgets.insert("mouse_area"),
            WidgetType::QRCode => self.used_widgets.insert("qr_code"),
            WidgetType::Stack => self.used_widgets.insert("stack"),
            WidgetType::Themer => self.used_widgets.insert("themer"),
            WidgetType::Pin => self.used_widgets.insert("pin"),
        };
        
        for child in &widget.children {
            self.collect_used_widgets(child);
        }
    }

    fn generate_message_enum(&mut self) {
        self.add_comment("// Application messages");
        self.add_newline();
        self.add_plain("#[derive(Debug, Clone)]");
        self.add_newline();
        self.add_keyword("pub enum");
        self.add_plain(" ");
        self.add_type("Message");
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        // Collect all interactive widgets and generate message variants
        self.generate_message_variants(&self.hierarchy.root().clone());
        
        self.indent_level -= 1;
        self.add_plain("}");
    }

    fn generate_app_struct(&mut self) {
        self.add_comment("// Application state");
        self.add_newline();
        self.add_keyword("struct");
        self.add_plain(" ");
        self.add_type(&self.app_name.clone());
        self.add_plain(" {");
        self.add_newline();
        self.indent_level += 1;
        
        // Generate state fields for interactive widgets
        self.generate_state_fields(&self.hierarchy.root().clone());
        
        self.indent_level -= 1;
        self.add_plain("}");
    }

    fn generate_message_variants(&mut self, widget: &Widget) {
        let name = self.get_widget_name(widget.id);
        
        match widget.widget_type {
            WidgetType::Button => {
                self.add_indent();
                self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::TextInput => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                self.add_indent();
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("String");
                self.add_plain("),");
                self.add_newline();
                
                if props.text_input_on_submit {
                    self.add_indent();
                    self.add_plain(&format!("{}Submitted", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                
                if props.text_input_on_paste {
                    self.add_indent();
                    self.add_plain(&format!("{}Pasted", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_type("String");
                    self.add_plain("),");
                    self.add_newline();
                }
            }
            WidgetType::Checkbox => {
                self.add_indent();
                self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("bool");
                self.add_plain("),");
                self.add_newline();
            }
            WidgetType::Radio => {
                self.add_indent();
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("usize");
                self.add_plain("),");
                self.add_newline();
            }
            WidgetType::Slider | WidgetType::VerticalSlider => {
                self.add_indent();
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("f32");
                self.add_plain("),");
                self.add_newline();
            }
            WidgetType::Toggler => {
                self.add_indent();
                self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("bool");
                self.add_plain("),");
                self.add_newline();
            }
            WidgetType::PickList => {
                self.add_indent();
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("String");
                self.add_plain("),");
                self.add_newline();
            }
            WidgetType::ComboBox => {
                let props = &widget.properties;
                
                // Determine the type parameter based on whether enum is used
                let type_name = if let Some(ref enum_id) = props.referenced_enum {
                    if let Some(enum_def) = self.type_system.unwrap().get_enum(enum_id.clone()) {
                        enum_def.name.clone()
                    } else {
                        "String".to_string()
                    }
                } else {
                    "String".to_string()
                };
                
                // Always generate Selected message
                self.add_indent();
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type(&type_name);
                self.add_plain("),");
                self.add_newline();
                
                // Conditionally generate on_input
                if props.combobox_use_on_input {
                    self.add_indent();
                    self.add_plain(&format!("{}OnInput", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_type("String");
                    self.add_plain("),");
                    self.add_newline();
                }
                
                // Conditionally generate on_option_hovered
                if props.combobox_use_on_option_hovered {
                    self.add_indent();
                    self.add_plain(&format!("{}OnOptionHovered", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_type(&type_name);
                    self.add_plain("),");
                    self.add_newline();
                }
                
                // Conditionally generate on_open
                if props.combobox_use_on_open {
                    self.add_indent();
                    self.add_plain(&format!("{}OnOpen", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                
                // Conditionally generate on_close
                if props.combobox_use_on_close {
                    self.add_indent();
                    self.add_plain(&format!("{}OnClose", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
            }
            WidgetType::MouseArea => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                if props.mousearea_on_press {
                    self.add_indent();
                    self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_release {
                    self.add_indent();
                    self.add_plain(&format!("{}Released", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_double_click {
                    self.add_indent();
                    self.add_plain(&format!("{}DoubleClicked", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_right_press {
                    self.add_indent();
                    self.add_plain(&format!("{}RightPressed", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_right_release {
                    self.add_indent();
                    self.add_plain(&format!("{}RightReleased", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_middle_press {
                    self.add_indent();
                    self.add_plain(&format!("{}MiddlePressed", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_middle_release {
                    self.add_indent();
                    self.add_plain(&format!("{}MiddleReleased", to_pascal_case(&name)));
                    self.add_plain(",");
                    self.add_newline();
                }
                if props.mousearea_on_scroll {
                    self.add_indent();
                    self.add_plain(&format!("{}Scrolled", to_pascal_case(&name)));
                    self.add_plain("(mouse::ScrollDelta),");
                    self.add_newline();
                }
                if props.mousearea_on_enter {
                    self.add_indent();
                    self.add_plain(&format!("{}Entered", to_pascal_case(&name)));
                    self.add_plain("(Point),");
                    self.add_newline();
                }
                if props.mousearea_on_move {
                    self.add_indent();
                    self.add_plain(&format!("{}Moved", to_pascal_case(&name)));
                    self.add_plain("(Point),");
                    self.add_newline();
                }
                if props.mousearea_on_exit {
                    self.add_indent();
                    self.add_plain(&format!("{}Exited", to_pascal_case(&name)));
                    self.add_plain("(Point),");
                    self.add_newline();
                }
            }
            _ => {}
        }
        
        for child in &widget.children {
            self.generate_message_variants(child);
        }
    }

    fn generate_state_fields(&mut self, widget: &Widget) {
        let name = self.get_widget_name(widget.id);
        let props = &widget.properties;
        
        match widget.widget_type {
            WidgetType::TextInput => {
                self.add_indent();
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("String");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Checkbox => {
                self.add_indent();
                self.add_identifier(&format!("{}_checked", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("bool");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Radio => {
                self.add_indent();
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("usize");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Slider | WidgetType::VerticalSlider => {
                self.add_indent();
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("f32");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::Toggler => {
                self.add_indent();
                self.add_identifier(&format!("{}_active", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("bool");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::PickList => {
                self.add_indent();
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("Option");
                self.add_operator("<");
                self.add_type("String");
                self.add_operator(">");
                self.add_plain(",");
                self.add_newline();
            }
            WidgetType::ComboBox => {
                if let Some(ref enum_id) = props.referenced_enum {
                    if let Some(enum_def) = self.type_system.unwrap().get_enum(enum_id.clone()) {
                        // Enum-based combo box
                        self.add_indent();
                        self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                        self.add_operator(":");
                        self.add_plain(" ");
                        self.add_type(&enum_def.name);
                        self.add_plain(",");
                        self.add_newline();
                        
                        self.add_indent();
                        self.add_identifier(&format!("{}_state", to_snake_case(&name)));
                        self.add_operator(":");
                        self.add_plain(" ");
                        self.add_type("combo_box::State");
                        self.add_operator("<");
                        self.add_type(&enum_def.name);
                        self.add_operator(">");
                        self.add_plain(",");
                        self.add_newline();
                        return;
                    }
                }
                
                // String-based combo box (existing code)
                self.add_indent();
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("String");
                self.add_plain(",");
                self.add_newline();
                
                self.add_indent();
                self.add_identifier(&format!("{}_state", to_snake_case(&name)));
                self.add_operator(":");
                self.add_plain(" ");
                self.add_type("combo_box::State");
                self.add_operator("<");
                self.add_type("String");
                self.add_operator(">");
                self.add_plain(",");
                self.add_newline();
            }
            _ => {}
        }
        
        for child in &widget.children {
            self.generate_state_fields(child);
        }
    }

    fn generate_update_match_arms(&mut self, widget: &Widget) {
        let name = self.get_widget_name(widget.id);
        let props = &widget.properties;
        
        match widget.widget_type {
            WidgetType::Button => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
                self.add_plain(" ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_comment("// Handle button press");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::TextInput => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                // Always generate the Changed handler
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("value");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("value");
                self.add_plain(";");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
                
                // Conditionally generate on_submit handler
                if props.text_input_on_submit {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Submitted", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle text input submission (Enter key pressed)");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment(&format!("// Current value: self.{}_value", to_snake_case(&name)));
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Conditionally generate on_paste handler
                if props.text_input_on_paste {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Pasted", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_identifier("pasted_text");
                    self.add_plain(") ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle text being pasted");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("// pasted_text contains the pasted string");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("// Note: on_input will also fire with the new combined value");
                    self.add_newline();
                    self.add_indent();
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=");
                    self.add_plain(" ");
                    self.add_identifier("pasted_text");
                    self.add_plain(";");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
            }
            WidgetType::Checkbox => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("checked");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_checked", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("checked");
                self.add_plain(";");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::Radio => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("index");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("index");
                self.add_plain(";");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::Slider | WidgetType::VerticalSlider => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("value");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("value");
                self.add_plain(";");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::Toggler => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("active");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_active", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("active");
                self.add_plain(";");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::PickList => {
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("value");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" Some(");
                self.add_identifier("value");
                self.add_plain(");");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
            }
            WidgetType::ComboBox => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                // Always generate Selected handler with helpful example
                self.add_indent();
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_identifier("value");
                self.add_plain(") ");
                self.add_operator("=>");
                self.add_plain(" {");
                self.add_newline();
                self.indent_level += 1;
                
                // Add helpful println
                self.add_indent();
                self.add_macro("println!");
                self.add_plain("(");
                self.add_string(&format!("\"{} selected: {{:?}}\"", name));
                self.add_plain(", ");
                self.add_identifier("value");
                self.add_plain(");");
                self.add_newline();
                
                // Update state
                self.add_indent();
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                self.add_plain(" ");
                self.add_operator("=");
                self.add_plain(" ");
                self.add_identifier("value");
                self.add_plain(";");
                self.add_newline();
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
                
                // Conditionally generate on_input handler with example
                if props.combobox_use_on_input {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}OnInput", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_identifier("text");
                    self.add_plain(") ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    
                    self.add_indent();
                    self.add_macro("println!");
                    self.add_plain("(");
                    self.add_string(&format!("\"{} input text: {{}}\"", name));
                    self.add_plain(", ");
                    self.add_identifier("text");
                    self.add_plain(");");
                    self.add_newline();
                    
                    self.add_indent();
                    self.add_comment("// You can filter options, update state, etc.");
                    self.add_newline();
                    
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Conditionally generate on_option_hovered handler with example
                if props.combobox_use_on_option_hovered {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}OnOptionHovered", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_identifier("option");
                    self.add_plain(") ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    
                    self.add_indent();
                    self.add_macro("println!");
                    self.add_plain("(");
                    self.add_string(&format!("\"{} option hovered: {{:?}}\"", name));
                    self.add_plain(", ");
                    self.add_identifier("option");
                    self.add_plain(");");
                    self.add_newline();
                    
                    self.add_indent();
                    self.add_comment("// Preview the hovered option, update UI, etc.");
                    self.add_newline();
                    
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Conditionally generate on_open handler with example
                if props.combobox_use_on_open {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}OnOpen", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    
                    self.add_indent();
                    self.add_macro("println!");
                    self.add_plain("(");
                    self.add_string(&format!("\"{} opened!\"", name));
                    self.add_plain(");");
                    self.add_newline();
                    
                    self.add_indent();
                    self.add_comment("// Refresh data, log analytics, etc.");
                    self.add_newline();
                    
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Conditionally generate on_close handler with example
                if props.combobox_use_on_close {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}OnClose", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    
                    self.add_indent();
                    self.add_macro("println!");
                    self.add_plain("(");
                    self.add_string(&format!("\"{} closed!\"", name));
                    self.add_plain(");");
                    self.add_newline();
                    
                    self.add_indent();
                    self.add_comment("// Save user choice, validate selection, etc.");
                    self.add_newline();
                    
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
            }
            WidgetType::MouseArea => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                // Left button press
                if props.mousearea_on_press {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle left mouse button press");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Left button release
                if props.mousearea_on_release {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Released", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle left mouse button release");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Double click
                if props.mousearea_on_double_click {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}DoubleClicked", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle double click");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("// Note: on_press and on_release will also fire");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Right button press
                if props.mousearea_on_right_press {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}RightPressed", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle right mouse button press");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Right button release
                if props.mousearea_on_right_release {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}RightReleased", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle right mouse button release");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Middle button press
                if props.mousearea_on_middle_press {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}MiddlePressed", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle middle mouse button press");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Middle button release
                if props.mousearea_on_middle_release {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}MiddleReleased", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle middle mouse button release");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Scroll with delta parameter
                if props.mousearea_on_scroll {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Scrolled", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_identifier("delta");
                    self.add_plain(") ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle scroll event");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("// delta is mouse::ScrollDelta enum:");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("//   Lines { x: f32, y: f32 } - scroll in lines");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("//   Pixels { x: f32, y: f32 } - scroll in pixels");
                    self.add_newline();
                    self.add_indent();
                    self.add_keyword("match");
                    self.add_plain(" ");
                    self.add_identifier("delta");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_plain("mouse::ScrollDelta::Lines { ");
                    self.add_identifier("x");
                    self.add_plain(", ");
                    self.add_identifier("y");
                    self.add_plain(" } ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle line-based scrolling");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                    self.add_indent();
                    self.add_plain("mouse::ScrollDelta::Pixels { ");
                    self.add_identifier("x");
                    self.add_plain(", ");
                    self.add_identifier("y");
                    self.add_plain(" } ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle pixel-based scrolling");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Mouse enter
                if props.mousearea_on_enter {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Entered", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle mouse entering the area");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Mouse move with point parameter
                if props.mousearea_on_move {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Moved", to_pascal_case(&name)));
                    self.add_plain("(");
                    self.add_identifier("point");
                    self.add_plain(") ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle mouse movement within the area");
                    self.add_newline();
                    self.add_indent();
                    self.add_comment("// point is Point { x: f32, y: f32 } relative to the widget's bounds");
                    self.add_newline();
                    self.add_indent();
                    self.add_keyword("let");
                    self.add_plain(" ");
                    self.add_identifier("x");
                    self.add_plain(" ");
                    self.add_operator("=");
                    self.add_plain(" ");
                    self.add_identifier("point");
                    self.add_operator(".");
                    self.add_identifier("x");
                    self.add_plain(";");
                    self.add_newline();
                    self.add_indent();
                    self.add_keyword("let");
                    self.add_plain(" ");
                    self.add_identifier("y");
                    self.add_plain(" ");
                    self.add_operator("=");
                    self.add_plain(" ");
                    self.add_identifier("point");
                    self.add_operator(".");
                    self.add_identifier("y");
                    self.add_plain(";");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }
                
                // Mouse exit
                if props.mousearea_on_exit {
                    self.add_indent();
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Exited", to_pascal_case(&name)));
                    self.add_plain(" ");
                    self.add_operator("=>");
                    self.add_plain(" {");
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_comment("// Handle mouse leaving the area");
                    self.add_newline();
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("}");
                    self.add_newline();
                }

                self.add_plain(",");

            }
            _ => {}
        }
        
        for child in &widget.children {
            self.generate_update_match_arms(child);
        }
    }

    fn generate_view_method(&mut self) {
        self.widget_counts.clear();

        self.add_indent();
        self.add_keyword("fn");
        self.add_plain(" ");
        self.add_function("view");
        self.add_plain("<");
        self.add_operator("'a");
        self.add_plain(">");
        self.add_plain("(");
        self.add_operator("&'a ");
        self.add_type("self");
        self.add_plain(")");
        self.add_operator(" -> ");
        self.add_type("Element");
        self.add_plain("<");
        self.add_operator("'a");
        self.add_plain(", ");
        self.add_operator("Message");
        self.add_plain("> {");
        self.add_newline();
        self.indent_level += 1;

        let root = self.hierarchy.root();

        if root.children.is_empty() {
            self.add_indent();
            self.add_function("container");
            self.add_plain("(");
            self.add_function("text");
            self.add_plain("(");
            self.add_string("\"Empty\"");
            self.add_plain("))");
        } else {
            self.generate_widget_creation(root, true);
        }

        self.add_indent();
        self.add_operator(".");
        self.add_function("into");
        self.add_plain("()");
        self.add_newline();

        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("}");
    }

    fn generate_widget_creation(&mut self, widget: &Widget, use_self: bool) {
        let props = &widget.properties;
        
        match widget.widget_type {
            WidgetType::Container => {
                self.add_indent();
                self.add_function("container");
                self.add_plain("(");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Container Content\"");
                    self.add_plain(")");
                } else {
                    for child in &widget.children {
                        self.generate_widget_creation(child, use_self);
                    }
                }
                
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                self.generate_container_properties(props);
                self.add_newline();
            }
            WidgetType::Row => {
                self.add_indent();
                self.add_macro("row!");
                self.add_plain("[");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Row Item\"");
                    self.add_plain(")");
                } else {
                    for (i, child) in widget.children.iter().enumerate() {
                        self.generate_widget_creation(child, use_self);
                        if i < widget.children.len() - 1 {
                            self.add_plain(",");
                        }
                        self.add_newline();
                    }
                }

                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("]");
                
                // Generate row properties
                self.generate_layout_properties(props, true);
                
                // NEW: If wrapping, add .wrap() and wrapping properties
                if props.is_wrapping_row {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("wrap");
                    self.add_plain("()");
                    
                    // Vertical spacing
                    if let Some(v_spacing) = props.wrapping_vertical_spacing {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("vertical_spacing");
                        self.add_plain("(");
                        self.add_number(&format!("{:.1}", v_spacing));
                        self.add_plain(")");
                    }
                    
                    // Horizontal alignment (only if not Left/default)
                    if !matches!(props.wrapping_align_x, ContainerAlignX::Left) {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("align_x");
                        self.add_plain("(");
                        match props.wrapping_align_x {
                            ContainerAlignX::Left => self.add_type("Alignment::Start"),
                            ContainerAlignX::Center => self.add_type("Alignment::Center"),
                            ContainerAlignX::Right => self.add_type("Alignment::End"),
                        }
                        self.add_plain(")");
                    }
                }
            }
            WidgetType::Column => {
                self.add_indent();
                self.add_macro("column!");
                self.add_plain("[");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Column Item\"");
                    self.add_plain(")");
                } else {
                    for (i, child) in widget.children.iter().enumerate() {
                        self.generate_widget_creation(child, use_self);
                        if i < widget.children.len() - 1 {
                            self.add_plain(",");
                        }
                        self.add_newline();
                    }
                }
                
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("]");
                self.generate_layout_properties(props, false);
            }
            WidgetType::Button => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("button");
                self.add_plain("(");
//                self.add_function("text");
//                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.text_content));
//                self.add_plain(")");
                self.add_plain(")");
                self.generate_button_properties(widget, props);
            }
            WidgetType::Text => {
                self.add_indent();
                self.add_function("text");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.text_content));
                self.add_plain(")");
                self.generate_text_properties(props);
            }
            WidgetType::TextInput => {
                let name = self.get_widget_name(widget.id);
                let props = &widget.properties;
                
                self.add_indent();
                self.add_function("text_input");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.text_input_placeholder));
                self.add_plain(", ");
                if use_self {
                    self.add_operator("&");
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                } else {
                    self.add_string("\"\"");
                }
                self.add_plain(")");
                self.indent_level += 1;
                
                // Always add on_input
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("on_input");
                self.add_plain("(");
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain(")");
                
                // Conditionally add on_submit
                if props.text_input_on_submit {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_submit");
                    self.add_plain("(");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Submitted", to_pascal_case(&name)));
                    self.add_plain(")");
                }
                
                // Conditionally add on_paste
                if props.text_input_on_paste {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_paste");
                    self.add_plain("(");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Pasted", to_pascal_case(&name)));
                    self.add_plain(")");
                }
                
                // Add secure if enabled
                if props.is_secure {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("secure");
                    self.add_plain("(");
                    self.add_plain("true");
                    self.add_plain(")");
                }
                
                // Add font if not default
                if props.text_input_font != FontType::Default {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("font");
                    self.add_plain("(");
                    match props.text_input_font {
                        FontType::Monospace => {
                            self.add_type("Font");
                            self.add_operator("::");
                            self.add_plain("MONOSPACE");
                        }
                        _ => {
                            self.add_type("Font");
                            self.add_operator("::");
                            self.add_plain("default()");
                        }
                    }
                    self.add_plain(")");
                }
                
                // Add size
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("size");
                self.add_plain("(");
                self.add_plain(&format!("{}", props.text_input_size));
                self.add_plain(")");
                
                // Add padding
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("padding");
                self.add_plain("(");
                self.add_plain(&format!("{}", props.text_input_padding));
                self.add_plain(")");
                
                // Add line_height if not default
                if props.text_input_line_height != text::LineHeight::default() {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("line_height");
                    self.add_plain("(");
                    // Generate line_height value based on type
                    match props.text_input_line_height {
                        text::LineHeight::Absolute(pixels) => {
                            self.add_plain(&format!("{}", pixels.0));
                        }
                        text::LineHeight::Relative(factor) => {
                            self.add_plain("text::LineHeight::Relative(");
                            self.add_plain(&format!("{}", factor));
                            self.add_plain(")");
                        }
                    }
                    self.add_plain(")");
                }
                
                // Add alignment if not left
                if props.text_input_alignment != ContainerAlignX::Left {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("align_x");
                    self.add_plain("(");
                    //self.add_plain("alignment::Horizontal::");
                    self.add_plain("Alignment::");
                    match props.text_input_alignment {
                        ContainerAlignX::Left => self.add_plain("Start"),
                        ContainerAlignX::Center => self.add_plain("Center"),
                        ContainerAlignX::Right => self.add_plain("End"),
                    }
                    self.add_plain(")");
                }
                
                // Add width
                if !matches!(props.width, Length::Fill) {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("width");
                    self.add_plain("(");
                    self.add_length(props.width);
                    self.add_plain(")");
                }
                
                self.indent_level -= 1;
            }
            WidgetType::Checkbox => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("checkbox");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.checkbox_label));
                self.add_plain(", ");
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_checked", to_snake_case(&name)));
                } else {
                    self.add_keyword(if props.checkbox_checked { "true" } else { "false" });
                }
                self.add_plain(")");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("on_toggle");
                self.add_plain("(");
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_plain(")");
                self.generate_checkbox_properties(props);
                self.indent_level -= 1;
            }
            WidgetType::Radio => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_macro("column!");
                self.add_plain("[");
                self.add_newline();
                self.indent_level += 1;
                
                for (i, option) in props.radio_options.iter().enumerate() {
                    self.add_indent();
                    self.add_function("radio");
                    self.add_plain("(");
                    self.add_string(&format!("\"{}\"", option));
                    self.add_plain(", ");
                    self.add_number(&format!("{}", i));
                    self.add_plain(", ");
                    if use_self {
                        self.add_plain("Some(");
                        self.add_keyword("self");
                        self.add_operator(".");
                        self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                        self.add_plain(")");
                    } else {
                        self.add_plain("Some(");
                        self.add_number(&format!("{}", props.radio_selected_index));
                        self.add_plain(")");
                    }
                    self.add_plain(", ");
                    if use_self {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                    } else {
                        self.add_operator("|");
                        self.add_identifier("_");
                        self.add_operator("|");
                        self.add_plain(" ");
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain("Noop");
                    }
                    self.add_plain(")");
                    if props.radio_size != 16.0 {
                        self.add_newline();
                        self.indent_level += 1;
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("size");
                        self.add_plain("(");
                        self.add_number(&format!("{}", props.radio_size));
                        self.add_plain(")");
                        self.indent_level -= 1;
                    }
                    if props.width != Length::Shrink {
                        self.add_newline();
                        self.indent_level += 1;
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("width");
                        self.add_plain("(");
                        self.add_length(props.width);
                        self.add_plain(")");
                        self.indent_level -= 1;
                    }
                    if i < props.radio_options.len() - 1 {
                        self.add_plain(",");
                    }
                    self.add_newline();
                }
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("]");
            }
            WidgetType::Slider => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("slider");
                self.add_plain("(");
                self.add_number(&format!("{:.1}", props.slider_min));
                self.add_operator("..=");
                self.add_number(&format!("{:.1}", props.slider_max));
                self.add_plain(", ");
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                } else {
                    self.add_number(&format!("{}", props.slider_value));
                }
                self.add_plain(", ");
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_plain(")");
                self.generate_slider_properties(props);
            }
            WidgetType::VerticalSlider => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("vertical_slider");
                self.add_plain("(");
                self.add_number(&format!("{:.1}", props.slider_min));
                self.add_operator("..=");
                self.add_number(&format!("{:.1}", props.slider_max));
                self.add_plain(", ");
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                } else {
                    self.add_number(&format!("{}", props.slider_value));
                }
                self.add_plain(", ");
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_plain(")");
                self.generate_vertical_slider_properties(props);
            }
            WidgetType::ProgressBar => {
                self.add_indent();
                self.add_function("progress_bar");
                self.add_plain("(");
                self.add_number(&format!("{:.1}", props.progress_min));
                self.add_operator("..=");
                self.add_number(&format!("{:.1}", props.progress_max));
                self.add_plain(", ");
                self.add_number(&format!("{:.2}", props.progress_value));
                self.add_plain(")");
                self.generate_progress_properties(props);
            }
            WidgetType::Toggler => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("toggler");
                self.add_plain("(");
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_active", to_snake_case(&name)));
                } else {
                    self.add_keyword(if props.toggler_active { "true" } else { "false" });
                }
                self.add_plain(")");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("on_toggle");
                self.add_plain("(");
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Toggled", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_plain(")");
                self.generate_toggler_properties(props);
                self.indent_level -= 1;
            }
            WidgetType::PickList => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("pick_list");
                self.add_plain("(");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_plain("vec![");
                for (i, option) in props.picklist_options.iter().enumerate() {
                    self.add_string(&format!("\"{}\"", option));
                    self.add_operator(".");
                    self.add_function("to_string");
                    self.add_plain("()");
                    if i < props.picklist_options.len() - 1 {
                        self.add_plain(", ");
                    }
                }
                self.add_plain("],");
                self.add_newline();
                self.add_indent();
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                    self.add_operator(".");
                    self.add_function("clone");
                    self.add_plain("()");
                } else if let Some(ref selected) = props.picklist_selected {
                    self.add_plain("Some(");
                    self.add_string(&format!("\"{}\"", selected));
                    self.add_operator(".");
                    self.add_function("to_string");
                    self.add_plain("())");
                } else {
                    self.add_plain("None");
                }
                self.add_plain(",");
                self.add_newline();
                self.add_indent();
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                self.generate_picklist_properties(props);
            }
            WidgetType::Scrollable => {
                self.add_indent();
                self.add_function("scrollable");
                self.add_plain("(");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_macro("column!");
                    self.add_plain("[");
                    self.add_newline();
                    self.indent_level += 1;
                    for i in 1..=10 {
                        self.add_indent();
                        self.add_function("text");
                        self.add_plain("(");
                        self.add_string(&format!("\"Scrollable Item {}\"", i));
                        self.add_plain(")");
                        if i < 10 {
                            self.add_plain(",");
                        }
                        self.add_newline();
                    }
                    self.indent_level -= 1;
                    self.add_indent();
                    self.add_plain("]");
                } else {
                    for child in &widget.children {
                        self.generate_widget_creation(child, use_self);
                    }
                }
                
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                self.generate_scrollable_properties(props);
            }
            WidgetType::Space => {
                self.add_indent();
                match props.orientation {
                    Orientation::Horizontal => {
                        self.add_function("space::horizontal");
                    }
                    Orientation::Vertical => {
                        self.add_function("space::vertical");
                    }
                }
                self.add_plain("()");
                self.generate_space_properties(props);
            }
            WidgetType::Rule => {
                self.add_indent();
                match props.orientation {
                    Orientation::Horizontal => {
                        self.add_function("rule::horizontal");
                    }
                    Orientation::Vertical => {
                        self.add_function("rule::vertical");
                    }
                }
                self.add_plain("(");
                self.add_number(&format!("{}", props.rule_thickness));
                self.add_plain(")");
            }
            WidgetType::Image => {
                self.add_indent();
                self.add_function("image");
                self.add_plain("(");
                if props.image_path.is_empty() {
                    self.add_string("\"path/to/image.png\"");
                } else {
                    self.add_plain("r");
                    self.add_string(&format!("\"{}\"", props.image_path));
                }
                self.add_plain(")");
                self.generate_image_properties(props);
            }
            WidgetType::Svg => {
                self.add_indent();
                self.add_function("svg");
                self.add_plain("(svg::Handle::from_path(");
                if props.svg_path.is_empty() {
                    self.add_string("\"path/to/icon.svg\"");
                } else {
                    self.add_plain("r");
                    self.add_string(&format!("\"{}\"", props.svg_path));
                }
                self.add_plain("))");
                self.generate_svg_properties(props);
            }
            WidgetType::Tooltip => {
                self.add_indent();
                self.add_function("tooltip");
                self.add_plain("(");
                self.add_newline();
                self.indent_level += 1;
                
                // First child (host)
                if let Some(host) = widget.children.get(0) {
                    self.generate_widget_creation(host, use_self);
                } else {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Hover me\"");
                    self.add_plain(")");
                }
                self.add_plain(",");
                self.add_newline();
                
                // Second child (tooltip content) or text
                if let Some(content) = widget.children.get(1) {
                    self.generate_widget_creation(content, use_self);
                } else {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string(&format!("\"{}\"", props.tooltip_text));
                    self.add_plain(")");
                }
                self.add_plain(",");
                self.add_newline();
                
                // Position
                self.add_indent();
                self.add_plain("tooltip::Position::");
                match props.tooltip_position {
                    TooltipPosition::Top => self.add_plain("Top"),
                    TooltipPosition::Bottom => self.add_plain("Bottom"),
                    TooltipPosition::Left => self.add_plain("Left"),
                    TooltipPosition::Right => self.add_plain("Right"),
                    TooltipPosition::FollowCursor => self.add_plain("FollowCursor"),
                }
                
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
            }
            WidgetType::ComboBox => {
                let name = self.get_widget_name(widget.id);
                self.add_indent();
                self.add_function("combo_box");
                self.add_plain("(");
                self.add_newline();
                self.indent_level += 1;
                
                // State reference
                self.add_indent();
                self.add_operator("&");
                if use_self {
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_state", to_snake_case(&name)));
                } else {
                    self.add_plain("state");
                }
                self.add_plain(",");
                self.add_newline();
                
                // Placeholder
                self.add_indent();
                self.add_string(&format!("\"{}\"", props.combobox_placeholder));
                self.add_plain(",");
                self.add_newline();
                
                // Current value
                self.add_indent();
                if use_self {
                    self.add_plain("Some(");
                    self.add_operator("&");
                    self.add_keyword("self");
                    self.add_operator(".");
                    self.add_identifier(&format!("{}_value", to_snake_case(&name)));
                    self.add_plain(")");
                } else {
                    if let Some(ref val) = props.combobox_selected {
                        self.add_plain("Some(");
                        self.add_string(&format!("\"{}\"", val));
                        self.add_plain(")");
                    } else {
                        self.add_plain("None");
                    }
                }
                self.add_plain(",");
                self.add_newline();
                
                // On option selected (always present)
                self.add_indent();
                if use_self {
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
                } else {
                    self.add_operator("|");
                    self.add_identifier("_");
                    self.add_operator("|");
                    self.add_plain(" ");
                    self.add_type("Message");
                    self.add_operator("::");
                    self.add_plain("Noop");
                }
                self.add_newline();
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                
                // Now add optional methods
                if props.combobox_use_on_input {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_input");
                    self.add_plain("(");
                    if use_self {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain(&format!("{}OnInput", to_pascal_case(&name)));
                    } else {
                        self.add_operator("|");
                        self.add_identifier("_");
                        self.add_operator("|");
                        self.add_plain(" ");
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain("Noop");
                    }
                    self.add_plain(")");
                }
                
                if props.combobox_use_on_option_hovered {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_option_hovered");
                    self.add_plain("(");
                    if use_self {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain(&format!("{}OnOptionHovered", to_pascal_case(&name)));
                    } else {
                        self.add_operator("|");
                        self.add_identifier("_");
                        self.add_operator("|");
                        self.add_plain(" ");
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain("Noop");
                    }
                    self.add_plain(")");
                }
                
                if props.combobox_use_on_open {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_open");
                    self.add_plain("(");
                    if use_self {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain(&format!("{}OnOpen", to_pascal_case(&name)));
                    } else {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain("Noop");
                    }
                    self.add_plain(")");
                }
                
                if props.combobox_use_on_close {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_close");
                    self.add_plain("(");
                    if use_self {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain(&format!("{}OnClose", to_pascal_case(&name)));
                    } else {
                        self.add_type("Message");
                        self.add_operator("::");
                        self.add_plain("Noop");
                    }
                    self.add_plain(")");
                }
                
                self.generate_combobox_properties(props);
            }
            
            WidgetType::Markdown => {
/*                self.add_indent();
                self.add_keyword("let");
                self.add_plain(" items = ");
                self.add_function("markdown::parse");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.markdown_source.replace("\"", "\\\"")));
                self.add_plain(");");
                self.add_newline();
                self.add_indent();
                self.add_function("markdown::view");
                self.add_plain("(");
                self.add_operator("&");
                self.add_plain("items, ");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_type("markdown::Settings");
                self.add_plain(" {");
                self.add_newline();
                 self.indent_level += 1;
                
                self.add_indent();
                self.add_plain("link_color: Some(");
                self.add_color(props.markdown_link_color);
                self.add_plain("),");
                self.add_newline();
                
                self.add_indent();
                self.add_plain("link_underline: ");
                self.add_keyword(if props.markdown_link_underline { "true" } else { "false" });
                self.add_plain(",");
                self.add_newline();
                
                self.add_indent();
                self.add_plain("code_color: Some(");
                self.add_color(props.markdown_code_color);
                self.add_plain("),");
                self.add_newline();
                
                self.add_indent();
                self.add_plain("block_spacing: ");
                self.add_number(&format!("{}", props.markdown_block_spacing));
                self.add_plain(",");
                self.add_newline();
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("}");
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                self.generate_markdown_properties(props);
*/
            }
            
            WidgetType::MouseArea => {
                self.add_indent();
                self.add_function("mouse_area");
                self.add_plain("(");
                self.add_newline();
                
                // Generate child
                if !widget.children.is_empty() {
                    self.generate_widget_creation(&widget.children[0], use_self);
                }
                
                self.add_plain(")");
                self.indent_level += 1;
                
                let name = self.get_widget_name(widget.id);
                
                // Conditionally add event handlers
                if props.mousearea_on_press {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_press");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_release {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_press");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}Released", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_double_click {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_double_click");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}DoubleClicked", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_right_press {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_right_press");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}RightPressed", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_right_release {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_right_release");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}RightReleased", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_middle_press {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_middle_press");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}MiddlePressed", to_pascal_case(&name)));
                    self.add_plain(")");
                }

                if props.mousearea_on_middle_release {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_middle_release");
                    self.add_plain("(Message::");
                    self.add_plain(&format!("{}MiddleReleased", to_pascal_case(&name)));
                    self.add_plain(")");
                }
                
                if props.mousearea_on_scroll {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_scroll");
                    self.add_plain("(|delta| Message::");
                    self.add_plain(&format!("{}Scrolled", to_pascal_case(&name)));
                    self.add_plain("(delta))");
                }

                if props.mousearea_on_enter {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_enter");
                    self.add_plain("(|point| Message::");
                    self.add_plain(&format!("{}Entered", to_pascal_case(&name)));
                    self.add_plain("(point))");
                }
                
                if props.mousearea_on_move {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_move");
                    self.add_plain("(|point| Message::");
                    self.add_plain(&format!("{}Moved", to_pascal_case(&name)));
                    self.add_plain("(point))");
                }

                if props.mousearea_on_exit {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("on_exit");
                    self.add_plain("(|point| Message::");
                    self.add_plain(&format!("{}Exited", to_pascal_case(&name)));
                    self.add_plain("(point))");
                }
                
                if let Some(interaction) = props.mousearea_interaction {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("interaction");
                    self.add_plain("(Interaction::");
                    self.add_plain(&format!("{:?}", interaction));
                    self.add_plain(")");
                }
                
                // Add layout properties
                //self.generate_layout_properties(props, false);
                self.indent_level -= 1;
            }
            
            WidgetType::QRCode => {
                self.add_indent();
                self.add_type("qr_code::QRCode");
                self.add_operator("::");
                self.add_function("new");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.qrcode_data));
                self.add_plain(")");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("cell_size");
                self.add_plain("(");
                self.add_number(&format!("{}", props.qrcode_cell_size));
                self.add_plain(")");
                self.generate_qrcode_properties(props);
                self.indent_level -= 1;
            }
            
            WidgetType::Stack => {
                self.add_indent();
                self.add_function("stack");
                self.add_plain("(vec![");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Layer 1\"");
                    self.add_plain(").into(),");
                    self.add_newline();
                    self.add_indent();
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Layer 2\"");
                    self.add_plain(").into(),");
                } else {
                    for (i, child) in widget.children.iter().enumerate() {
                        self.generate_widget_creation(child, use_self);
                        self.add_operator(".");
                        self.add_function("into");
                        self.add_plain("()");
                        if i < widget.children.len() - 1 {
                            self.add_plain(",");
                        }
                        self.add_newline();
                    }
                }
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("])");
                self.generate_stack_properties(props);
            }
            
            WidgetType::Themer => {
                self.add_indent();
                self.add_function("themer");
                self.add_plain("(");
                if let Some(theme) = &props.themer_theme {
                    self.add_plain("Some(");
                    self.add_type("Theme");
                    self.add_operator("::");
                    match theme {
                        Theme::Light => self.add_plain("Light"),
                        Theme::Dark => self.add_plain("Dark"),
                        Theme::Dracula => self.add_plain("Dracula"),
                        Theme::Nord => self.add_plain("Nord"),
                        _ => self.add_plain("Dark"),
                    }
                    self.add_plain(")");
                } else {
                    self.add_plain("None");
                }
                self.add_plain(", ");
                self.add_newline();
                self.indent_level += 1;
                
                if widget.children.is_empty() {
                    self.add_indent();
                    self.add_function("container");
                    self.add_plain("(");
                    self.add_function("text");
                    self.add_plain("(");
                    self.add_string("\"Themed content\"");
                    self.add_plain("))");
                } else {
                    for child in &widget.children {
                        self.generate_widget_creation(child, use_self);
                    }
                }
                
                self.add_newline();
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain(")");
                self.generate_themer_properties(props);
            } 
            WidgetType::Pin => {
            //    todo!("implement code gen for pin");
            }
        }
    }

    fn collect_widget_names(&mut self, widget: &Widget) {
        // Generate unique name for this widget
        let mut derp = String::new();
        let base_name = if !widget.properties.widget_name.trim().is_empty() {
            self.sanitize_name(&widget.properties.widget_name)
        } else {
            match widget.widget_type {
                WidgetType::Button => "button",
                WidgetType::Text => "text",
                WidgetType::TextInput => "text_input",
                WidgetType::Checkbox => "checkbox",
                WidgetType::Radio => "radio",
                WidgetType::Slider => "slider",
                WidgetType::VerticalSlider => "vertical_slider",
                WidgetType::ProgressBar => "progress_bar",
                WidgetType::Toggler => "toggler",
                WidgetType::PickList => "pick_list",
                _ => {
                    derp = format!("{:?}", widget.widget_type).to_lowercase();

                    &derp
                }
            }.to_string()
        };

        // Get count for this widget type
        let type_key = format!("{:?}", widget.widget_type).to_lowercase();
        let count = self.widget_counts.entry(type_key).or_insert(0);
        *count += 1;

        let final_name = if *count > 1 {
            format!("{}_{}", base_name, count)
        } else {
            base_name
        };

        self.widget_names.insert(widget.id, final_name);

        // Process children
        for child in &widget.children {
            self.collect_widget_names(child);
        }
    }

    fn get_widget_name(&self, widget_id: WidgetId) -> String {
        self.widget_names.get(&widget_id)
            .cloned()
            .unwrap_or_else(|| "widget".to_string())
    }

    fn generate_radio_widget(&mut self, widget: &Widget, use_self: bool, use_column: bool) {
        let props = &widget.properties;
        let name = self.get_unique_widget_name(widget);
        
        self.add_indent();
        if use_column {
            self.add_macro("column!");
        } else {
            self.add_macro("row!");
        }
        self.add_plain("[");
        self.add_newline();
        self.indent_level += 1;
        
        for (i, option) in props.radio_options.iter().enumerate() {
            self.add_indent();
            self.add_function("radio");
            self.add_plain("(");
            self.add_newline();
            self.indent_level += 1;
            
            // Each parameter on its own line for readability
            self.add_indent();
            self.add_string(&format!("\"{}\"", option));
            self.add_plain(",");
            self.add_newline();
            
            self.add_indent();
            self.add_number(&format!("{}", i));
            self.add_plain(",");
            self.add_newline();
            
            self.add_indent();
            if use_self {
                self.add_plain("Some(");
                self.add_keyword("self");
                self.add_operator(".");
                self.add_identifier(&format!("{}_selected", to_snake_case(&name)));
                self.add_plain(")");
            } else {
                self.add_plain("Some(");
                self.add_number(&format!("{}", props.radio_selected_index));
                self.add_plain(")");
            }
            self.add_plain(",");
            self.add_newline();
            
            self.add_indent();
            if use_self {
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain(&format!("{}Selected", to_pascal_case(&name)));
            } else {
                self.add_operator("|");
                self.add_identifier("_");
                self.add_operator("|");
                self.add_plain(" ");
                self.add_type("Message");
                self.add_operator("::");
                self.add_plain("Noop");
            }
            self.add_newline();
            
            self.indent_level -= 1;
            self.add_indent();
            self.add_plain(")");
            
            if props.radio_size != 20.0 {
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("size");
                self.add_plain("(");
                self.add_number(&format!("{}", props.radio_size));
                self.add_plain(")");
                self.indent_level -= 1;
            }
            
            if i < props.radio_options.len() - 1 {
                self.add_plain(",");
            }
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("]");
    }

    fn generate_container_properties(&mut self, props: &Properties) {
        // Widget ID
        if let Some(ref id) = props.widget_id {
            if !id.is_empty() {
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("id");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", id));
                self.add_plain(")");
            }
        }

        // Sizing
        match props.container_sizing_mode {
            ContainerSizingMode::Manual => {
                // Width
                if !matches!(props.width, Length::Fill) {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("width");
                    self.add_plain("(");
                    self.add_length(props.width);
                    self.add_plain(")");
                }
                
                // Height
                if !matches!(props.height, Length::Fill) {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("height");
                    self.add_plain("(");
                    self.add_length(props.height);
                    self.add_plain(")");
                }
                
                // Alignment
                match props.align_x {
                    ContainerAlignX::Left => {},
                    ContainerAlignX::Center => {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("align_x");
                        self.add_plain("(");
                        self.add_type("Alignment::Center");
                        self.add_plain(")");
                    }
                    ContainerAlignX::Right => {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("align_x");
                        self.add_plain("(");
                        self.add_type("Alignment::End");
                        self.add_plain(")");
                    }
                }
                match props.align_y {
                    ContainerAlignY::Top => {},
                    ContainerAlignY::Center => {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("align_y");
                        self.add_plain("(");
                        self.add_type("Alignment::Center");
                        self.add_plain(")");
                    }
                    ContainerAlignY::Bottom => {
                        self.add_newline();
                        self.add_indent();
                        self.add_operator(".");
                        self.add_function("align_y");
                        self.add_plain("(");
                        self.add_type("Alignment::End");
                        self.add_plain(")");
                    }
                }
            }
            ContainerSizingMode::CenterX => {
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("center_x");
                self.add_plain("(");
                self.add_length(props.container_center_length);
                self.add_plain(")");
            }
            ContainerSizingMode::CenterY => {
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("center_y");
                self.add_plain("(");
                self.add_length(props.container_center_length);
                self.add_plain(")");
            }
            ContainerSizingMode::Center => {
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("center");
                self.add_plain("(");
                self.add_length(props.container_center_length);
                self.add_plain(")");
            }
        }

        // Max width
        if let Some(max_w) = props.max_width {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("max_width");
            self.add_plain("(");
            self.add_number(&format!("{:.1}", max_w));
            self.add_plain(")");
        }

        // Max Height
        if let Some(max_h) = props.max_height {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("max_height");
            self.add_plain("(");
            self.add_number(&format!("{:.1}", max_h));
            self.add_plain(")");
        }
        
        // Padding
        if props.padding != Padding::ZERO {
            self.generate_padding(&props.padding, props.padding_mode);
        }

        // Clip
        if props.clip {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("clip");
            self.add_plain("(");
            self.add_keyword("true");
            self.add_plain(")");
        }
    }

    fn generate_layout_properties(&mut self, props: &Properties, is_row: bool) {
        // Spacing
        if props.spacing != 0.0 {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("spacing");
            self.add_plain("(");
            self.add_number(&format!("{}", props.spacing));
            self.add_plain(")");
        }

        // Alignment
        if !matches!(props.align_items, Alignment::Start) {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            
            if is_row {
                // Row aligns children vertically
                self.add_function("align_y");
                self.add_plain("(");
                match props.align_items {
                    Alignment::Start => self.add_type("Alignment::Start"),
                    Alignment::Center => self.add_type("Alignment::Center"),
                    Alignment::End => self.add_type("Alignment::End"),
                }
            } else {
                // Column aligns children horizontally
                self.add_function("align_x");
                self.add_plain("(");
                match props.align_items {
                    Alignment::Start => self.add_type("Alignment::Start"),
                    Alignment::Center => self.add_type("Alignment::Center"),
                    Alignment::End => self.add_type("Alignment::End"),
                }
            }
            self.add_plain(")");
        }
        
        // Width
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
        }

        // Max_Width for Column only
        if !is_row {
            if let Some(max_w) = props.max_width {
                self.add_newline();
                self.add_indent();
                self.add_operator(".");
                self.add_function("max_width");
                self.add_plain("(");
                self.add_number(&format!("{:.1}", max_w));
                self.add_plain(")");
            }
        }
        
        // Height
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
        }

        // Padding
        if props.padding != Padding::ZERO {
            self.generate_padding(&props.padding, props.padding_mode);
        }

        // Clip
        if props.clip {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("clip");
            self.add_plain("(");
            self.add_keyword("true");
            self.add_plain(")");
        }
    }

    fn generate_button_properties(&mut self, widget: &Widget, props: &Properties) {
        let mut name = props.widget_name.clone();
        if name.len() == 0 {
            name = self.get_widget_name(widget.id);
        }

        if props.button_on_press_enabled {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("on_press");
            self.add_plain("(");
            self.add_type("Message");
            self.add_operator("::");
            self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
            self.add_plain(")");
            self.indent_level -= 1;
        }

        if props.button_on_press_with_enabled {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("on_press_with");
            self.add_plain("(|| ");
            self.add_type("Message");
            self.add_operator("::");
            self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
            self.add_plain(")");
            self.indent_level -= 1;
        }

        if props.button_on_press_maybe_enabled {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("on_press_maybe");
            self.add_plain("(Some(");
            self.add_type("Message");
            self.add_operator("::");
            self.add_plain(&format!("{}Pressed", to_pascal_case(&name)));
            self.add_plain("))");
            self.indent_level -= 1;
        }


        // Clip
        if props.clip {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("clip");
            self.add_plain("(");
            self.add_keyword("true");
            self.add_plain(")");
            self.indent_level -= 1;
        }

        // Style - only add if not Primary (default)
        match props.button_style {
            ButtonStyleType::Secondary => {
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("style");
                self.add_plain("(button::secondary)");
                self.indent_level -= 1;
            }
            ButtonStyleType::Success => {
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("style");
                self.add_plain("(button::success)");
                self.indent_level -= 1;
            }
            ButtonStyleType::Danger => {
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("style");
                self.add_plain("(button::danger)");
                self.indent_level -= 1;
            }
            ButtonStyleType::Text => {
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("style");
                self.add_plain("(button::text)");
                self.indent_level -= 1;
            }
            ButtonStyleType::Primary => {} // Default, don't add
        }
        
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }

        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }

        // Padding
        if props.padding != (Padding { top: 5.0, bottom: 5.0, right: 10.0, left: 10.0 }) {
            self.generate_padding(&props.padding, props.padding_mode);
        }
    }

    fn generate_text_properties(&mut self, props: &Properties) {
        if props.text_size != 16.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.text_size));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }

        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_text_input_properties(&mut self, props: &Properties) {
        if props.is_secure {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("secure");
            self.add_plain("(");
            self.add_keyword("true");
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if props.text_input_size != 16.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.text_input_size));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.width, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }
    
    fn generate_checkbox_properties(&mut self, props: &Properties) {
        if props.checkbox_size != 16.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.checkbox_size));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if props.checkbox_spacing != 8.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("spacing");
            self.add_plain("(");
            self.add_number(&format!("{}", props.checkbox_spacing));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }
    
    fn generate_slider_properties(&mut self, props: &Properties) {
        if props.slider_step != 1.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("step");
            self.add_plain("(");
            self.add_number(&format!("{}", props.slider_step));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // For horizontal slider, only height can be set
        if !matches!(props.slider_height, iced::widget::slider::Slider::<f32, Theme>::DEFAULT_HEIGHT) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.slider_height.into());
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_vertical_slider_properties(&mut self, props: &Properties) {
        if props.slider_step != 1.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("step");
            self.add_plain("(");
            self.add_number(&format!("{}", props.slider_step));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // For vertical slider, width can be set as a Length
        if !matches!(props.slider_width, iced::widget::vertical_slider::VerticalSlider::<f32, Theme>::DEFAULT_WIDTH) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.slider_width.into());
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }
    
    fn generate_progress_properties(&mut self, props: &Properties) {
        if !matches!(props.progress_length, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("length");
            self.add_plain("(");
            self.add_length(props.progress_length);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if props.progress_girth != iced::widget::progress_bar::ProgressBar::<Theme>::DEFAULT_GIRTH {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("girth");
            self.add_plain("(");
            self.add_number(&format!("{}", props.progress_girth));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if props.progress_vertical {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("vertical");
            self.add_plain("()");
            self.indent_level -= 1;
        }
    }
    
    fn generate_toggler_properties(&mut self, props: &Properties) {
        if props.toggler_size != iced::widget::toggler::Toggler::<Theme>::DEFAULT_SIZE {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.toggler_size));
            self.add_plain(")");
        }
        
        if props.toggler_spacing != iced::widget::toggler::Toggler::<Theme>::DEFAULT_SIZE / 2.0 {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("spacing");
            self.add_plain("(");
            self.add_number(&format!("{}", props.toggler_spacing));
            self.add_plain(")");
        }
        
        if !props.toggler_label.is_empty() {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("label");
            self.add_plain("(");
            self.add_string(&format!("\"{}\"", props.toggler_label));
            self.add_plain(")");
        }

        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
        }
    }
    
    fn generate_picklist_properties(&mut self, props: &Properties) {
        if !props.picklist_placeholder.is_empty() && props.picklist_placeholder != "Choose an option..." {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("placeholder");
            self.add_plain("(");
            self.add_string(&format!("\"{}\"", props.picklist_placeholder));
            self.add_plain(")");
        }
        
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
        }

        if props.padding != (Padding { top: 5.0, bottom: 5.0, right: 10.0, left: 10.0 }) {
            self.generate_padding(&props.padding, props.padding_mode);
        }
    }
    
    fn generate_scrollable_properties(&mut self, props: &Properties) {
        // Direction
        let is_default_dir = matches!(
            props.scroll_dir,
            iced::widget::scrollable::Direction::Vertical(_)
        );
        
        if !is_default_dir {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("direction");
            self.add_plain("(");
            match props.scroll_dir {
                iced::widget::scrollable::Direction::Horizontal(_) => {
                    self.add_plain("scrollable::Direction::Horizontal(scrollable::Scrollbar::default())");
                }
                iced::widget::scrollable::Direction::Both { .. } => {
                    self.add_plain("scrollable::Direction::Both { vertical: scrollable::Scrollbar::default(), horizontal: scrollable::Scrollbar::default() }");
                }
                _ => {}
            }
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Width
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Height
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }
    
    fn generate_space_properties(&mut self, props: &Properties) {
        match props.orientation {
            Orientation::Horizontal => {
                if !matches!(props.width, Length::Fill) {
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("width");
                    self.add_plain("(");
                    self.add_length(props.width);
                    self.add_plain(")");
                    self.indent_level -= 1;
                }
                
                if !matches!(props.height, Length::Shrink) {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("height");
                    self.add_plain("(");
                    self.add_length(props.height);
                    self.add_plain(")");
                }
            }
            Orientation::Vertical => {
                if !matches!(props.width, Length::Shrink) {
                    self.add_newline();
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("width");
                    self.add_plain("(");
                    self.add_length(props.width);
                    self.add_plain(")");
                }
                
                if !matches!(props.height, Length::Fill) {
                    self.add_newline();
                    self.indent_level += 1;
                    self.add_indent();
                    self.add_operator(".");
                    self.add_function("height");
                    self.add_plain("(");
                    self.add_length(props.height);
                    self.add_plain(")");
                    self.indent_level -= 1;
                }
            }
        }

    }
    
    fn generate_image_properties(&mut self, props: &Properties) {
        // Content fit
        if !matches!(props.image_fit, ContentFitChoice::Contain) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("content_fit");
            self.add_plain("(ContentFit::");
            match props.image_fit {
                ContentFitChoice::Cover => self.add_plain("Cover"),
                ContentFitChoice::Fill => self.add_plain("Fill"),
                ContentFitChoice::ScaleDown => self.add_plain("ScaleDown"),
                ContentFitChoice::None => self.add_plain("None"),
                _ => self.add_plain("Contain"),
            }
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Width
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Height
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }
    
    fn generate_svg_properties(&mut self, props: &Properties) {
        // Content fit
        if !matches!(props.svg_fit, ContentFitChoice::Contain) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("content_fit");
            self.add_plain("(ContentFit::");
            match props.svg_fit {
                ContentFitChoice::Cover => self.add_plain("Cover"),
                ContentFitChoice::Fill => self.add_plain("Fill"),
                ContentFitChoice::ScaleDown => self.add_plain("ScaleDown"),
                ContentFitChoice::None => self.add_plain("None"),
                _ => self.add_plain("Contain"),
            }
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Width
        if !matches!(props.width, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        // Height
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_combobox_properties(&mut self, props: &Properties) {
        if props.combobox_size != 16.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.combobox_size));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.width, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_markdown_properties(&mut self, props: &Properties) {
        if !matches!(props.width, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_qrcode_properties(&mut self, props: &Properties) {
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_stack_properties(&mut self, props: &Properties) {
        if !matches!(props.width, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.height, Length::Fill) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_themer_properties(&mut self, props: &Properties) {
        if !matches!(props.width, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("width");
            self.add_plain("(");
            self.add_length(props.width);
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !matches!(props.height, Length::Shrink) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("height");
            self.add_plain("(");
            self.add_length(props.height);
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_padding(&mut self, padding: &Padding, padding_mode: PaddingMode) {
        self.add_newline();
        self.add_indent();
        self.add_operator(".");
        self.add_function("padding");
        self.add_plain("(");
        
        match padding_mode {
            PaddingMode::Uniform => {
                // .padding(10.0)
                self.add_number(&format!("{:.1}", padding.top));
            }
            PaddingMode::Symmetric => {
                // .padding([vertical, horizontal])
                self.add_plain("[");
                self.add_number(&format!("{:.1}", padding.top));
                self.add_plain(", ");
                self.add_number(&format!("{:.1}", padding.left));
                self.add_plain("]");
            }
            PaddingMode::Individual => {
                // .padding(Padding { top, right, bottom, left })
                self.add_type("Padding");
                self.add_plain(" { ");
                self.add_plain("top: ");
                self.add_number(&format!("{:.1}", padding.top));
                self.add_plain(", ");
                self.add_plain("right: ");
                self.add_number(&format!("{:.1}", padding.right));
                self.add_plain(", ");
                self.add_plain("bottom: ");
                self.add_number(&format!("{:.1}", padding.bottom));
                self.add_plain(", ");
                self.add_plain("left: ");
                self.add_number(&format!("{:.1}", padding.left));
                self.add_plain(" }");
            }
        }
        
        self.add_plain(")");
    }

    fn add_length(&mut self, length: Length) {
        match length {
            Length::Fill => {
                self.add_type("Length");
                self.add_operator("::");
                self.add_plain("Fill");
            }
            Length::Shrink => {
                self.add_type("Length");
                self.add_operator("::");
                self.add_plain("Shrink");
            }
            Length::Fixed(px) => {
                self.add_type("Length");
                self.add_operator("::");
                self.add_plain("Fixed");
                self.add_plain("(");
                self.add_number(&format!("{:.1}", px));
                self.add_plain(")");
            }
            Length::FillPortion(p) => {
                self.add_type("Length");
                self.add_operator("::");
                self.add_plain("FillPortion");
                self.add_plain("(");
                self.add_number(&format!("{}", p));
                self.add_plain(")");
            }
            _ => {
                self.add_type("Length");
                self.add_operator("::");
                self.add_plain("Shrink");
            }
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return "widget".to_string();
        }
        
        // Replace spaces and special characters with underscores
        let sanitized = trimmed
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>()
            .to_lowercase();
        
        // Ensure it starts with a letter or underscore
        if sanitized.chars().next().map_or(false, |c| c.is_numeric()) {
            format!("_{}", sanitized)
        } else if sanitized.is_empty() {
            "widget".to_string()
        } else {
            sanitized
        }
    }

    fn generate_all_widget_names(&mut self) {
        self.widget_counts.clear();
        self.widget_names.clear();
        self.collect_widget_names(&self.hierarchy.root().clone());
    }

    fn add_color(&mut self, color: Color) {
        self.add_type("Color");
        self.add_operator("::");
        self.add_function("from_rgba");
        self.add_plain("(");
        self.add_number(&format!("{:.3}", color.r));
        self.add_plain(", ");
        self.add_number(&format!("{:.3}", color.g));
        self.add_plain(", ");
        self.add_number(&format!("{:.3}", color.b));
        self.add_plain(", ");
        self.add_number(&format!("{:.3}", color.a));
        self.add_plain(")");
    }

    // Token helper methods
    fn add_keyword(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Keyword,
        });
    }

    fn add_type(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Type,
        });
    }

    fn add_function(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Function,
        });
    }

    fn add_string(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::String,
        });
    }

    fn add_number(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Number,
        });
    }

    fn add_comment(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Comment,
        });
    }

    fn add_operator(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Operator,
        });
    }

    fn add_identifier(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Identifier,
        });
    }

    fn add_macro(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Macro,
        });
    }

    fn add_plain(&mut self, text: &str) {
        self.tokens.push(Token {
            text: text.to_string(),
            token_type: TokenType::Plain,
        });
    }

    fn add_newline(&mut self) {
        self.tokens.push(Token {
            text: "\n".to_string(),
            token_type: TokenType::Plain,
        });
    }

    fn add_indent(&mut self) {
        self.tokens.push(Token {
            text: "    ".repeat(self.indent_level),
            token_type: TokenType::Plain,
        });
    }
}

// Helper functions
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    s.to_lowercase().replace(' ', "_")
}

pub fn build_code_view_with_height<'a>(
    tokens: &[Token], 
    height: f32,
    theme: Theme
) -> Element<'a, crate::widget_helper::Message> {
    // Group tokens by lines
    let mut lines: Vec<Vec<Token>> = vec![vec![]];
    
    for token in tokens {
        if token.text.contains('\n') {
            // Handle tokens that contain newlines
            let parts: Vec<&str> = token.text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    lines.last_mut().unwrap().push(Token {
                        text: part.to_string(),
                        token_type: token.token_type,
                    });
                }
                // Add new line for all but the last part
                if i < parts.len() - 1 {
                    lines.push(vec![]);
                }
            }
        } else {
            lines.last_mut().unwrap().push(token.clone());
        }
    }

    let bg_color = match theme {
        Theme::Light => Color::from_rgb8(248, 248, 248),  // Very light gray
        Theme::Dark => Color::from_rgb8(30, 30, 30),       // Dark gray
        _ => Color::from_rgb8(40, 40, 40),                 // Default dark
    };

    let border_color = match theme {
        Theme::Light => Color::from_rgb8(200, 200, 200),   // Light gray border
        Theme::Dark => Color::from_rgb8(60, 60, 60),        // Dark gray border
        _ => Color::from_rgb8(80, 80, 80),
    };
    
    // Build the content as a column of rows
    let content = column(
        lines.into_iter().map(|line| {
            if line.is_empty() {
                row![text(" ").size(14).font(iced::Font::MONOSPACE)].into()
            } else {
                row(
                    line.into_iter().map(|token| {
                        text(token.text)
                            .size(14)
                            .font(iced::Font::MONOSPACE)
                            .color(token.token_type.color_for_theme(&theme))
                            .into()
                    }).collect::<Vec<Element<'a, crate::widget_helper::Message>>>()
                ).into()
            }
        }).collect::<Vec<Element<'a, crate::widget_helper::Message>>>()
    )
    .spacing(2);
    
    container(
        scrollable(
            container(content)
                .width(Length::Fill)
                .padding(15)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(
            if height == 0.0 {
                Length::Fill
            }
            else {
                Length::Fixed(height)
            }
        )
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Build a syntax-highlighted code view
pub fn build_code_view<'a>(tokens: &[Token], theme: Theme) -> Element<'a, crate::widget_helper::Message> {
    build_code_view_with_height(tokens, 300.0, theme)
}

/// Build a syntax-highlighted code view - generic so I can use it outside of widget_helper::Messages
pub fn build_code_view_with_height_generic<'a, Message: 'a>(
    tokens: &[Token], 
    height: f32,
    theme: Theme
) -> Element<'a, Message> {
    // Group tokens by lines
    let mut lines: Vec<Vec<Token>> = vec![vec![]];
    
    for token in tokens {
        if token.text.contains('\n') {
            let parts: Vec<&str> = token.text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if !part.is_empty() {
                    lines.last_mut().unwrap().push(Token {
                        text: part.to_string(),
                        token_type: token.token_type,
                    });
                }
                if i < parts.len() - 1 {
                    lines.push(vec![]);
                }
            }
        } else {
            lines.last_mut().unwrap().push(token.clone());
        }
    }

    let bg_color = match theme {
        Theme::Light => Color::from_rgb8(248, 248, 248),
        Theme::Dark => Color::from_rgb8(30, 30, 30),
        _ => Color::from_rgb8(40, 40, 40),
    };

    let border_color = match theme {
        Theme::Light => Color::from_rgb8(200, 200, 200),
        Theme::Dark => Color::from_rgb8(60, 60, 60),
        _ => Color::from_rgb8(80, 80, 80),
    };
    
    let content = column(
        lines.into_iter().map(|line| {
            if line.is_empty() {
                row![text(" ").size(14).font(iced::Font::MONOSPACE)].into()
            } else {
                row(
                    line.into_iter().map(|token| {
                        text(token.text)
                            .size(14)
                            .font(iced::Font::MONOSPACE)
                            .color(token.token_type.color_for_theme(&theme))
                            .into()
                    }).collect::<Vec<Element<'a, Message>>>()
                ).into()
            }
        }).collect::<Vec<Element<'a, Message>>>()
    )
    .spacing(2);
    
    container(
        scrollable(
            container(content)
                .width(Length::Fill)
                .padding(15)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(
            if height == 0.0 {
                Length::Fill
            } else {
                Length::Fixed(height)
            }
        )
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}



/// StyleFn Generators

/// Generate tokens for container style code
pub fn generate_container_style_tokens(
    text_color: Color,
    background_color: Color,
    border_color: Color,
    border_width: f32,
    border_radius_top_left: f32,
    border_radius_top_right: f32,
    border_radius_bottom_right: f32,
    border_radius_bottom_left: f32,
    shadow_enabled: bool,
    shadow_color: Color,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    shadow_blur_radius: f32,
    snap: bool,
) -> Vec<Token> {
    let mut builder = TokenBuilder::new();

    builder.add_plain("container");
    builder.add_operator("::");
    builder.add_type("Style");
    builder.add_space();
    builder.add_plain("{");
    builder.add_newline();
    builder.increase_indent();

    // text_color field
    builder.add_field("text_color", |b| {
        b.add_plain("Some(");
        b.add_color(text_color);
        b.add_plain(")");
    });

    // background field
    builder.add_field("background", |b| {
        b.add_plain("Some(");
        b.add_type("Background");
        b.add_operator("::");
        b.add_type("Color");
        b.add_plain("(");
        b.add_color(background_color);
        b.add_plain("))");
    });

    // border field
    builder.add_field("border", |b| {
        b.add_struct("Border", |b| {
            b.add_field("color", |b| b.add_color(border_color));
            b.add_field("width", |b| b.add_number(&format!("{:.1}", border_width)));
            b.add_field("radius", |b| {
                b.add_struct("Radius", |b| {
                    b.add_field("top_left", |b| b.add_number(&format!("{:.1}", border_radius_top_left)));
                    b.add_field("top_right", |b| b.add_number(&format!("{:.1}", border_radius_top_right)));
                    b.add_field("bottom_right", |b| b.add_number(&format!("{:.1}", border_radius_bottom_right)));
                    b.add_field("bottom_left", |b| b.add_number(&format!("{:.1}", border_radius_bottom_left)));
                });
            });
        });
    });

    // shadow field
    builder.add_field("shadow", |b| {
        if shadow_enabled {
            b.add_struct("Shadow", |b| {
                b.add_field("color", |b| b.add_color(shadow_color));
                b.add_field("offset", |b| {
                    b.add_struct("Vector", |b| {
                        b.add_field("x", |b| b.add_number(&format!("{:.1}", shadow_offset_x)));
                        b.add_field("y", |b| b.add_number(&format!("{:.1}", shadow_offset_y)));
                    });
                });
                b.add_field("blur_radius", |b| b.add_number(&format!("{:.1}", shadow_blur_radius)));
            });
        } else {
            b.add_type("Shadow");
            b.add_operator("::");
            b.add_function("default");
            b.add_plain("()");
        }
    });

    // snap field
    builder.add_field("snap", |b| {
        b.add_keyword(if snap { "true" } else { "false" });
    });

    builder.decrease_indent();
    builder.add_plain("}");

    builder.into_tokens()
}

struct ImportTracker {
    used_widgets: HashSet<&'static str>,
    
    uses_length: bool,
    uses_alignment: bool,
    uses_padding: bool,
    uses_color: bool,
    
    // Text properties
    uses_text_line_height: bool,
    uses_text_wrapping: bool,
    uses_text_shaping: bool,
    uses_text_alignment: bool,
    
    // Mouse
    uses_mouse: bool,
    uses_mouse_interaction: bool,
    uses_mouse_scroll_delta: bool,
    
    // Other
    uses_point: bool,
    uses_font: bool,
    uses_border: bool,
    uses_shadow: bool,
    uses_background: bool,
    uses_vector: bool,
}

impl ImportTracker {
    fn new() -> Self {
        Self {
            used_widgets: HashSet::new(),
            uses_length: false,
            uses_alignment: false,
            uses_padding: false,
            uses_color: false,
            uses_text_line_height: false,
            uses_text_wrapping: false,
            uses_text_shaping: false,
            uses_text_alignment: false,
            uses_mouse: false,
            uses_mouse_interaction: false,
            uses_mouse_scroll_delta: false,
            uses_point: false,
            uses_font: false,
            uses_border: false,
            uses_shadow: false,
            uses_background: false,
            uses_vector: false,
        }
    }
    
    fn scan_widget(&mut self, widget: &Widget) {
        let props = &widget.properties;
        
        // Track widget type
        match widget.widget_type {
            WidgetType::Container => { self.used_widgets.insert("container"); }
            WidgetType::Row => { self.used_widgets.insert("row"); }
            WidgetType::Column => { self.used_widgets.insert("column"); }
            WidgetType::Button => { self.used_widgets.insert("button"); }
            WidgetType::Text => { self.used_widgets.insert("text"); }
            WidgetType::TextInput => { self.used_widgets.insert("text_input"); }
            WidgetType::Checkbox => { self.used_widgets.insert("checkbox"); }
            WidgetType::Radio => { self.used_widgets.insert("radio"); }
            WidgetType::Slider => { self.used_widgets.insert("slider"); }
            WidgetType::VerticalSlider => { self.used_widgets.insert("vertical_slider"); }
            WidgetType::ProgressBar => { self.used_widgets.insert("progress_bar"); }
            WidgetType::Toggler => { self.used_widgets.insert("toggler"); }
            WidgetType::PickList => { self.used_widgets.insert("pick_list"); }
            WidgetType::Scrollable => { self.used_widgets.insert("scrollable"); }
            WidgetType::Space => { self.used_widgets.insert("space"); }
            WidgetType::Rule => { self.used_widgets.insert("rule"); }
            WidgetType::Image => { self.used_widgets.insert("image"); }
            WidgetType::Svg => { self.used_widgets.insert("svg"); }
            WidgetType::Tooltip => { self.used_widgets.insert("tooltip"); }
            WidgetType::ComboBox => { self.used_widgets.insert("combo_box"); }
            WidgetType::Markdown => { self.used_widgets.insert("markdown"); }
            WidgetType::MouseArea => { 
                self.used_widgets.insert("mouse_area");
                self.uses_mouse = true;
            }
            WidgetType::QRCode => { self.used_widgets.insert("qr_code"); }
            WidgetType::Stack => { self.used_widgets.insert("stack"); }
            WidgetType::Themer => { self.used_widgets.insert("themer"); }
            WidgetType::Pin => { self.used_widgets.insert("pin"); }
        }
        
        // Track if any Length is used (always true if widget exists)
        self.uses_length = true;
        
        // Track Alignment if used in Row/Column
        if matches!(widget.widget_type, WidgetType::Row | WidgetType::Column) {
            if props.align_items != Alignment::Start {
                self.uses_alignment = true;
            }
        }
        
        // Track Padding
        if props.padding_mode == PaddingMode::Individual {
            self.uses_padding = true;
        }
        
        // Track Container-specific features
        if widget.widget_type == WidgetType::Container {
            if props.border_width > 0.0 {
                self.uses_border = true;
            }
            if props.background_color.a > 0.0 {
                self.uses_background = true;
                self.uses_color = true;
            }
            if props.has_shadow {
                self.uses_shadow = true;
                self.uses_vector = true;
            }
            if props.align_x != ContainerAlignX::Left || props.align_y != ContainerAlignY::Top {
                self.uses_alignment = true;
            }
        }
        
        // Track Text properties
        if widget.widget_type == WidgetType::Text {
            if props.text_color.a > 0.0 {
                self.uses_color = true;
            }
            if props.font != FontType::Default {
                self.uses_font = true;
            }
            if props.line_height != text::LineHeight::default() {
                self.uses_text_line_height = true;
            }
            if props.wrap != text::Wrapping::default() {
                self.uses_text_wrapping = true;
            }
            if props.shaping != text::Shaping::default() {
                self.uses_text_shaping = true;
            }
            if props.text_align_x != text::Alignment::default() || 
               props.text_align_y != iced::alignment::Vertical::Top {
                self.uses_text_alignment = true;
                self.uses_alignment = true;
            }
        }
        
        // Track TextInput properties
        if widget.widget_type == WidgetType::TextInput {
            if props.text_input_font != FontType::Default {
                self.uses_font = true;
            }
            if props.text_input_line_height != text::LineHeight::default() {
                self.uses_text_line_height = true;
            }
            if props.text_input_alignment != ContainerAlignX::Left {
                self.uses_alignment = true;
            }
        }
        
        // Track MouseArea event handlers
        if widget.widget_type == WidgetType::MouseArea {
            if props.mousearea_on_scroll {
                self.uses_mouse_scroll_delta = true;
            }
            if props.mousearea_on_move {
                self.uses_point = true;
            }
            if props.mousearea_interaction.is_some() {
                self.uses_mouse_interaction = true;
            }
        }
        
        // Recursively scan children
        for child in &widget.children {
            self.scan_widget(child);
        }
    }
}