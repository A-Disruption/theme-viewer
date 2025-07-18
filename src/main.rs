use iced::{Theme, Element, Task, Size, Length};
use iced::window::Settings;
use iced::advanced::graphics::core::window;
use iced::widget::{button, checkbox, column, combo_box, container, horizontal_space, pick_list, progress_bar, radio, responsive, row, scrollable, slider, text, text_input, toggler, Action};
use std::collections::BTreeMap;

mod theme_helper;
mod widget;
// use widget::color_picker;
// use widget::new_color_picker;

fn main() {
    iced::daemon(ThemeViewer::new, ThemeViewer::update, ThemeViewer::view)
        .title(ThemeViewer::title)
        .theme(ThemeViewer::theme)
        .run()
        .unwrap()
}

struct ThemeViewer {
    windows: BTreeMap<window::Id, Window>,
    theme_builder: theme_helper::PaletteBuilder,
    themes: Vec<Theme>,
    theme: Option<Theme>,
    show_custom_theme_menu: bool,
    text_input_1: String,
    checkboxes: bool,
    text_input: String,
    password: String,
    show_password: bool,
    radio_value: Option<RadioOption>,
    slider_value: f32,
    picklist: Option<Language>,
    combobox: Option<Language>,
    combobox_state: iced::widget::combo_box::State<Language>,
    toggler: bool,
}

#[derive(Clone, Debug)]
enum Message {
    ChooseTheme(Theme),
    ShowThemeBuilder,
    ButtonPressed,
    CheckBox(bool),
    EnteringText(String),
    EnteringPassword(String),
    ShowPassword(bool),
    RadioSelected(RadioOption),
    UpdateSlider(f32),
    PickListSelection(Language),
    ComboBoxSelection(Language),
    ToggleToggler(bool),

    // Theme Helper Messages
    ThemeHelper(theme_helper::Message),

    //window handles
    WindowClosed(iced::window::Id),
    RequestOpenWindow(WindowEnum),
    WindowOpened(iced::window::Id, WindowEnum),
}

impl ThemeViewer {
    fn new() -> (Self, Task<Message>) {
        let default_custom_palette = theme_helper::CustomPalette::preset_blue();
        let custom_theme = Theme::custom("Custom".to_string(), default_custom_palette.to_iced_palette());
        let mut themes = Theme::ALL.to_vec();
        themes.push(custom_theme);

        let theme_viewer = Self {
            windows: BTreeMap::new(),
            theme_builder: theme_helper::PaletteBuilder::new(),
            themes: themes,
            theme: Some(iced::theme::Theme::Dark),
            show_custom_theme_menu: false,
            text_input_1: String::new(),
            checkboxes: true,
            text_input: String::new(),
            password: String::new(),
            show_password: false,
            radio_value: None,
            slider_value: 1_f32,
            picklist: None,
            combobox: None,
            combobox_state: iced::widget::combo_box::State::new(Language::ALL.to_vec()),
            toggler: false,
        };

        (theme_viewer, Task::done(Message::RequestOpenWindow(WindowEnum::Main)))
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        self.theme.clone().unwrap_or_default()
    }

    fn title(&self, window_id: window::Id) -> String {
        self.windows.get(&window_id).map(|window| window.title.clone()).unwrap_or_default()
        //"Theme Viewer".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChooseTheme(theme) => {
                self.theme = Some(theme);

                match self.theme.as_ref().unwrap() {
                    Theme::Custom(_) => {
                        return Task::done(Message::ShowThemeBuilder)
                    }
                    _ => {}
                }

                Task::none()
            }
            Message::ShowThemeBuilder => {
                self.show_custom_theme_menu = !self.show_custom_theme_menu;


                //Task::none()

                Task::done(Message::RequestOpenWindow(WindowEnum::CustomBuilder))
            }
            Message::ButtonPressed => {
                println!("Button pressed!");
                Task::none()
            }
            Message::CheckBox(b) => {
                self.checkboxes = b;
                Task::none()
            }
            Message::EnteringText(msg) => {
                self.text_input = msg;
                Task::none()
            }
            Message::EnteringPassword(msg) => {
                self.password = msg;
                Task::none()
            }
            Message::ShowPassword(b) => {
                self.show_password = b;
                Task::none()
            }
            Message::RadioSelected(selection) => {
                self.radio_value = Some(selection);
                Task::none()
            }
            Message::UpdateSlider(num) => {
                self.slider_value = num;
                Task::none()
            }
            Message::PickListSelection(language) => {
                self.picklist = Some(language);
                Task::none()
            }

            Message::ComboBoxSelection(language) => {
                self.combobox = Some(language);
                Task::none()
            }
            Message::ToggleToggler(b) => {
                self.toggler = b;
                Task::none()
            }

            // Theme Helper
            Message::ThemeHelper(msg) => {
                match theme_helper::PaletteBuilder::update(&mut self.theme_builder, msg) {
                    theme_helper::Action::UpdateTheme(theme) => { 
                        self.theme = Some(theme);
                        Task::none()
                    }
                    theme_helper::Action::Run(task) => { 
                        return task.map(Message::ThemeHelper)
                     }
                    theme_helper::Action::None => { Task::none() }
                }
            }

            //window handles
            Message::WindowClosed(window_id) => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            },
            Message::RequestOpenWindow(window_type) => {
                match window_type {
                    WindowEnum::Main => { 
                        let (_id, open) = iced::window::open(
                            iced::window::Settings {
                                position: window::Position::Centered,
                                size: Size::new(700_f32, 1000_f32),
                                min_size: Some(Size::new(700_f32, 975_f32)),
                                exit_on_close_request: true,
                                ..iced::window::Settings::default()
                            }
                        );
                        return open.map(|id| Message::WindowOpened(id, WindowEnum::Main))
                    }
                    WindowEnum::CustomBuilder => {
                        if self.windows.values().any(|w| w.windowtype == WindowEnum::CustomBuilder) {
                            return Task::none()
                        };
                

                        let (_id, open) = iced::window::open(window::Settings {
                            size: Size::new(700_f32, 1000_f32),
                            min_size: Some(Size::new(700_f32, 975_f32)),
                            ..window::Settings::default()
                        });
                        return open.map(|id| Message::WindowOpened(id, WindowEnum::CustomBuilder))
                    }
                }
            },
            Message::WindowOpened(window_id, window_type) => {
                let title = match window_type {
                    WindowEnum::Main => { String::from("Theme Viewer") }
                    WindowEnum::CustomBuilder => { String::from("Theme Builder") }
                };

                let new_window = Window::new(window_id, title, window_type);

                self.windows.insert(window_id, new_window);

                Task::none()
            },
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {

        let theme_pick_list = pick_list(
            self.themes.clone(), 
            self.theme.clone(), 
            Message::ChooseTheme
        );

        let theme_selection = column![
            text("Theme").size(18),
            theme_pick_list
        ].spacing(5);


/*         let section_name = if !self.show_custom_theme_menu {
            text("↓    Show Custom Theme Builder").shaping(text::Shaping::Advanced).style(text::secondary)
        } else {
            text("→    Hide Custom Theme Builder").shaping(text::Shaping::Advanced).style(text::secondary)
        };

        let conditional_theme_widgets = if !self.show_custom_theme_menu {
                column![
                    horizontal_space()
                ]                  
            } else {
                column![
                    theme_helper::PaletteBuilder::view(&self.theme_builder).map(Message::ThemeHelper)
                ]
            };

        let custom_theme_section = container(
            column![
                button(section_name).style(button::text).on_press(Message::ShowThemeBuilder).width(Length::Fill),
                conditional_theme_widgets
            ].width(Length::Fill),
        ).style(container::bordered_box); */

        let buttons = container(
            column![
                text("Buttons:").size(18),
                row![
                    column![
                        button("Primary").style(button::primary).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::primary).width(100),
                    ].spacing(5),
                    column![
                        button("Secondary").style(button::secondary).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::secondary).width(100),
                    ].spacing(5),
                    column![
                        button("Success").style(button::success).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::success).width(100)
                    ].spacing(5),
                    column![
                        button("Warning").style(button::warning).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::warning).width(100)
                    ].spacing(5),
                    column![
                        button("Danger").style(button::danger).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::danger).width(100)
                    ].spacing(5),
                    column![
                        button("Text").style(button::text).on_press(Message::ButtonPressed).width(100),
                        button("Disabled").style(button::text).width(100)
                    ].spacing(5),
                ].spacing(10),
            ]
            .spacing(10)
            .padding(10)
        )
        .style(container::bordered_box)
        .padding(
            iced::Padding {
                top: 0_f32, 
                right: 10_f32,
                bottom: 10_f32,
                left: 10_f32
            }
        )
        .width(iced::Length::Fill);

        let checkboxes = container(
            column![
                text("Checkboxes:").size(18),
                row![
                    column![
                        checkbox("Primary", self.checkboxes).style(checkbox::primary).on_toggle(Message::CheckBox).width(130),
                        checkbox("Primary", self.checkboxes).style(checkbox::primary).width(130)
                    ].spacing(5),
                    column![
                        checkbox("Secondary", self.checkboxes).style(checkbox::secondary).on_toggle(Message::CheckBox).width(130),
                        checkbox("Secondary", self.checkboxes).style(checkbox::secondary).width(130)
                    ].spacing(5),
                    column![
                        checkbox("Success", self.checkboxes).style(checkbox::success).on_toggle(Message::CheckBox).width(130),
                        checkbox("Success", self.checkboxes).style(checkbox::success).width(130)
                    ].spacing(5),
                    column![
                        checkbox("Danger", self.checkboxes).style(checkbox::danger).on_toggle(Message::CheckBox).width(130),
                        checkbox("Danger", self.checkboxes).style(checkbox::danger).width(130)
                    ].spacing(5),
                ],
            ]
            .spacing(10)
            .padding(10)
        )
        .style(container::bordered_box)
        .padding(
            iced::Padding {
                top: 0_f32, 
                right: 10_f32,
                bottom: 10_f32,
                left: 10_f32
            }
        )
        .width(iced::Length::Fill);

        let range = std::ops::RangeInclusive::new(1_f32,100_f32);

        let form_controls = container(
            column![
                text("Form Controls:").size(18),

                // Text Inputs
                text("Text Inputs: "),
                column![
                    text_input("Text input", &self.text_input).on_input(Message::EnteringText).width(650)
                ].spacing(5),
                column![
                    row![
                        text_input("Password", &self.password).on_input(Message::EnteringPassword).secure(!self.show_password),
                        checkbox("Show Password", self.show_password).on_toggle(Message::ShowPassword)
                    ].align_y(iced::Alignment::Center).spacing(10).width(640),
                ].spacing(5),
                column![

                ].spacing(5),
                column![

                ].spacing(5),

                // Radio Buttons
                text("Radio Buttons: "),
                row![
                    radio(
                        "Option 1", 
                        RadioOption::Option1, 
                        self.radio_value, 
                        Message::RadioSelected
                    ).width(150),
                    radio(
                        "Option 2", 
                        RadioOption::Option2, 
                        self.radio_value, 
                        Message::RadioSelected
                    ).width(150),
                    radio(
                        "Option 3", 
                        RadioOption::Option3, 
                        self.radio_value, 
                        Message::RadioSelected
                    ).width(150),
                ],

                
                // Slider
                text("Slider: "),
                row![
                    slider(
                        range.clone(),
                         self.slider_value,
                         Message::UpdateSlider),
                ].width(650),

                // Progress Bar
                text("Progress Bar: "),
                row![
                    progress_bar(
                        range.clone(), 
                        self.slider_value)
                ].width(650),

                
                // Pick List
                text("Pick List: "),
                row![
                    pick_list(
                        Language::ALL, 
                        self.picklist, 
                        Message::PickListSelection)
                ].width(650),

                // Combo Box
                text("Combo Box: "),
                row![
                    combo_box(
                        &self.combobox_state, 
                        "Select", 
                        self.combobox.as_ref(), 
                        Message::ComboBoxSelection)
                ].width(650),

                // Toggler
                text("Toggler: "),
                toggler(self.toggler).on_toggle(Message::ToggleToggler),
            ]
            .spacing(10)
            .padding(10)
        )
        .style(container::bordered_box)
        .padding(
            iced::Padding {
                top: 0_f32, 
                right: 10_f32,
                bottom: 10_f32,
                left: 10_f32
            }
        )
        .width(iced::Length::Fill);

        let main_window_content = container(
            column![
                theme_selection,
//                custom_theme_section, 
                buttons,
                checkboxes,
                form_controls
            ].spacing(10)
        )
        .padding(15)
        .into();

        let window_view = match self.windows.get(&window_id) {
            Some(window) => match window.windowtype {
                WindowEnum::Main => {
                    main_window_content
                }
                WindowEnum::CustomBuilder => {
                    container(
                        theme_helper::PaletteBuilder::view(&self.theme_builder).map(Message::ThemeHelper),
                    ).into()
                }
            }
            None => { 
                let content = column![
                    text(format!("Something has gone terribly wrong. Window Id: {:?}", window_id)),
                ];
                container(
                    content
                ).into() 
            }
        };

        window_view
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioOption {
    Option1,
    Option2,
    Option3,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Rust,
    Java,
    CPlusPlus,
    C,
    CSharp
}

impl Language {
    const ALL: [Language; 5] = [
        Language::Rust,
        Language::Java,
        Language::CPlusPlus,
        Language::C,
        Language::CSharp,
    ];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
            Language::CPlusPlus => write!(f, "CPlusPlus"),
            Language::C => write!(f, "C"),
            Language::CSharp => write!(f, "CSharp"),
        }
    }

}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum WindowEnum {
    #[default]
    Main,
    CustomBuilder,
}

#[derive(Debug, Clone,)]
pub struct Window {
    pub title: String,
    pub windowtype: WindowEnum,
}

impl Window {
    pub fn new(id: window::Id, title: String, window_type: WindowEnum) -> Self {
        Self {
            title: title,
            windowtype: window_type,
        }
    }
}