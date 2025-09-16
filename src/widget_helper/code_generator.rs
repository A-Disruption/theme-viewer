use iced::{Color, Element, Length, Padding, widget::{column, container, horizontal_space, row, scrollable, text}, Background, Border, Theme};
use crate::widget_helper::{
    Widget, WidgetType, Properties, WidgetId, WidgetHierarchy, 
    ContainerAlignX, ContainerAlignY, ButtonStyleType, FontType,
    RuleOrientation, ContentFitChoice, TooltipPosition
};
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
                    TokenType::Keyword => Color::from_rgb8(86, 156, 214),
                    TokenType::Type => Color::from_rgb8(78, 201, 176),
                    TokenType::Function => Color::from_rgb8(220, 220, 170),
                    TokenType::String => Color::from_rgb8(206, 145, 120),
                    TokenType::Number => Color::from_rgb8(181, 206, 168),
                    TokenType::Comment => Color::from_rgb8(106, 153, 85),
                    TokenType::Operator => Color::from_rgb8(212, 212, 212),
                    TokenType::Identifier => Color::from_rgb8(156, 220, 254),
                    TokenType::Macro => Color::from_rgb8(197, 134, 192),
                    TokenType::Plain => Color::from_rgb8(212, 212, 212),
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

/// Code generator for creating Iced code from widget hierarchy
pub struct CodeGenerator<'a> {
    hierarchy: &'a WidgetHierarchy,
    indent_level: usize,
    tokens: Vec<Token>,
    app_name: String,
    widget_counts: HashMap<String, usize>,  // Track duplicate widgets
    used_widgets: HashSet<&'static str>,  // Track which widgets are used for the impl code gen
    widget_names: HashMap<WidgetId, String>,
    theme: Theme,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(hierarchy: &'a WidgetHierarchy, theme: Theme) -> Self {
        Self {
            hierarchy,
            indent_level: 0,
            tokens: Vec::new(),
            app_name: "App".to_string(),
            widget_counts: HashMap::new(),
            used_widgets: HashSet::new(),
            widget_names: HashMap::new(),
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

    /// Generate code for a specific widget
    pub fn generate_widget_code(&mut self, widget_id: WidgetId) -> Vec<Token> {
        self.tokens.clear();
        self.indent_level = 0;
        
        if let Some(widget) = self.hierarchy.get_widget_by_id(widget_id) {
            self.generate_widget_creation(widget, false);
        }
        
        self.tokens.clone()
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
        self.add_plain(" (Self, iced::Task<Message>) {");
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
        self.add_plain("iced::Task::none()");
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
        self.add_string(&format!("\"{}\"", self.app_name));
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

    // Updated generate_imports to only include used widgets
    fn generate_imports(&mut self) {
        self.add_keyword("use");
        self.add_plain(" ");
        self.add_type("iced");
        self.add_operator("::");
        self.add_plain("{");
        self.add_newline();
        self.indent_level += 1;
        
        // Core imports always needed
        self.add_indent();
        self.add_type("Application");
        self.add_plain(", ");
        self.add_type("Element");
        self.add_plain(", ");
        self.add_type("Settings");
        self.add_plain(", ");
        self.add_type("Theme");
        self.add_plain(",");
        self.add_newline();
        
        // Add other core types if needed
        self.add_indent();
        let mut core_types = vec![];
        if self.used_widgets.iter().any(|&w| matches!(w, "container" | "row" | "column" | "text" | "button")) {
            core_types.push("Length");
        }
        if self.used_widgets.contains(&"container") {
            core_types.push("Padding");
            core_types.push("Alignment");
        }
        if self.used_widgets.contains(&"image") || self.used_widgets.contains(&"svg") {
            core_types.push("ContentFit");
        }
        
        for (i, t) in core_types.iter().enumerate() {
            if i > 0 {
                self.add_plain(", ");
            }
            self.add_type(t);
        }
        if !core_types.is_empty() {
            self.add_plain(",");
            self.add_newline();
        }
        
        // Widget imports - only what's used
        self.add_indent();
        self.add_plain("widget::{");
        self.add_newline();
        self.indent_level += 1;
        
        let mut widgets = Vec::new();
        for &widget in &self.used_widgets {
            widgets.push(widget);
        }
        widgets.sort(); // Sort for consistent output
        
        // Group widgets into lines
        let chunks: Vec<Vec<&str>> = widgets.chunks(6).map(|c| c.to_vec()).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            self.add_indent();
            for (j, widget) in chunk.iter().enumerate() {
                if j > 0 {
                    self.add_plain(", ");
                }
                self.add_plain(widget);
            }
            if i < chunks.len() - 1 {
                self.add_plain(",");
            }
            self.add_newline();
        }
        
        self.indent_level -= 1;
        self.add_indent();
        self.add_plain("},");
        self.add_newline();
        self.indent_level -= 1;
        self.add_plain("};");
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
            WidgetType::Space => {
                self.used_widgets.insert("vertical_space")
            //    self.used_widgets.insert("horizontal_space");
            }
            WidgetType::Rule => {
                self.used_widgets.insert("horizontal_rule")
            //    self.used_widgets.insert("vertical_rule");
            }
            WidgetType::Image => self.used_widgets.insert("image"),
            WidgetType::Svg => self.used_widgets.insert("svg"),
            WidgetType::Tooltip => self.used_widgets.insert("tooltip"),
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
                self.add_indent();
                self.add_plain(&format!("{}Changed", to_pascal_case(&name)));
                self.add_plain("(");
                self.add_type("String");
                self.add_plain("),");
                self.add_newline();
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
            _ => {}
        }
        
        for child in &widget.children {
            self.generate_state_fields(child);
        }
    }

    fn generate_update_match_arms(&mut self, widget: &Widget) {
        let name = self.get_widget_name(widget.id);
        
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
        self.add_plain("(&self) -> Element<Message> {");
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

        self.add_newline();
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
                
                self.indent_level -= 1;
                self.add_indent();
                self.add_plain("]");
                self.generate_layout_properties(props, true);
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
                self.add_function("text");
                self.add_plain("(");
                self.add_string(&format!("\"{}\"", props.text_content));
                self.add_plain("))");
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
                self.generate_button_properties(props);
                self.indent_level -= 1;
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
                    self.add_string(&format!("\"{}\"", props.text_input_value));
                }
                self.add_plain(")");
                self.add_newline();
                self.indent_level += 1;
                self.add_indent();
                self.add_operator(".");
                self.add_function("on_input");
                self.add_plain("(");
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
                self.generate_text_input_properties(props);
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
                self.add_function("vertical_space");
                self.add_plain("()");
                self.generate_space_properties(props);
            }
            WidgetType::Rule => {
                self.add_indent();
                match props.rule_orientation {
                    RuleOrientation::Horizontal => {
                        self.add_function("horizontal_rule");
                    }
                    RuleOrientation::Vertical => {
                        self.add_function("vertical_rule");
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
        
        // Padding
        if props.padding != Padding::new(0.0) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("padding");
            self.add_plain("(");
            self.add_number(&format!("{}", props.padding.top));
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_layout_properties(&mut self, props: &Properties, is_row: bool) {
        // Spacing
        if props.spacing != 0.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("spacing");
            self.add_plain("(");
            self.add_number(&format!("{}", props.spacing));
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

        // Padding
        if props.padding != Padding::new(0.0) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("padding");
            self.add_plain("(");
            self.add_number(&format!("{}", props.padding.top));
            self.add_plain(")");
            self.indent_level -= 1;
        }
    }

    fn generate_button_properties(&mut self, props: &Properties) {
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
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("padding");
            self.add_plain("(");
            self.add_number(&format!("{}", props.padding.top));
            self.add_plain(")");
            self.indent_level -= 1;
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
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("size");
            self.add_plain("(");
            self.add_number(&format!("{}", props.toggler_size));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if props.toggler_spacing != iced::widget::toggler::Toggler::<Theme>::DEFAULT_SIZE / 2.0 {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("spacing");
            self.add_plain("(");
            self.add_number(&format!("{}", props.toggler_spacing));
            self.add_plain(")");
            self.indent_level -= 1;
        }
        
        if !props.toggler_label.is_empty() {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("label");
            self.add_plain("(");
            self.add_string(&format!("\"{}\"", props.toggler_label));
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
    
    fn generate_picklist_properties(&mut self, props: &Properties) {
        if !props.picklist_placeholder.is_empty() && props.picklist_placeholder != "Choose an option..." {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("placeholder");
            self.add_plain("(");
            self.add_string(&format!("\"{}\"", props.picklist_placeholder));
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

        if props.padding != (Padding { top: 5.0, bottom: 5.0, right: 10.0, left: 10.0 }) {
            self.add_newline();
            self.indent_level += 1;
            self.add_indent();
            self.add_operator(".");
            self.add_function("padding");
            self.add_plain("(");
            self.add_number(&format!("{}", props.padding.top));
            self.add_plain(")");
            self.indent_level -= 1;
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
                self.add_number(&format!("{}", px));
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
        .height(Length::Fixed(height))
    )
    .width(Length::Fill)
    .into()
}

/// Build a syntax-highlighted code view
pub fn build_code_view<'a>(tokens: &[Token], theme: Theme) -> Element<'a, crate::widget_helper::Message> {
    build_code_view_with_height(tokens, 300.0, theme)
}