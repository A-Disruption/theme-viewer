use iced::{event, window, Element, Size, Subscription, Task, Theme};
use iced::widget::{button, checkbox, column, combo_box, container, space::horizontal as horizontal_space, pick_list, progress_bar, radio, row, slider, text, text_input, toggler};
use std::collections::BTreeMap;
use widget_helper::panegrid_dashboard::{PaneDock, PaneMsg};

mod icon;
mod widget;
mod widget_helper;

fn main() {
    iced::daemon(ThemeViewer::new, ThemeViewer::update, ThemeViewer::view)
        .title(ThemeViewer::title)
        .theme(ThemeViewer::theme)
        .subscription(ThemeViewer::subscription)
        .font(icon::FONT)
        .run()
        .unwrap()
}

struct ThemeViewer {
    windows: BTreeMap<window::Id, Window>,
    widget_builder: widget_helper::WidgetVisualizer,
    pane: Option<PaneDock>,
    themes: Vec<Theme>,
    theme: Option<Theme>,
    checkboxes: bool,
    text_input: String,
    password: String,
    show_password: bool,
    disabled_value: String,
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
    ShowWidgetBuilder,
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

    // Widget Builder Messages
    WidgetHelper(widget_helper::Message),
    Pane(PaneMsg),

    //window handles
    WindowClosed(iced::window::Id),
    RequestOpenWindow(WindowEnum),
    WindowOpened(iced::window::Id, WindowEnum),
}

impl ThemeViewer {
    fn new() -> (Self, Task<Message>) {
        let themes = Theme::ALL.to_vec();

        let theme_viewer = Self {
            windows: BTreeMap::new(),
            widget_builder: widget_helper::WidgetVisualizer::new(),
            pane: None,
            themes: themes,
            theme: Some(iced::theme::Theme::Dark),
            checkboxes: true,
            text_input: String::new(),
            password: String::new(),
            show_password: false,
            disabled_value: String::new(),
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
        self.theme.clone().unwrap_or(Theme::Dark)
    }

    fn title(&self, window_id: window::Id) -> String {
        self.windows.get(&window_id).map(|window| window.title.clone()).unwrap_or_default()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChooseTheme(theme) => {
                self.theme = Some(theme);
                Task::none()
            }
            Message::ShowWidgetBuilder => {
                Task::done(Message::RequestOpenWindow(WindowEnum::WidgetVisualizer))
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

            // Widget Helper
            Message::WidgetHelper(msg) => {
                match widget_helper::WidgetVisualizer::update(&mut self.widget_builder, msg) {
                    widget_helper::Action::Run(task) => {
                        return task.map(Message::WidgetHelper)
                    }
                    widget_helper::Action::None => { }
                }
                Task::none()
            }

            //window handles
            Message::WindowClosed(window_id) => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    iced::exit()
                }
                else {
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
                    WindowEnum::WidgetVisualizer => {
                        let mut windows = self.windows.iter().enumerate();
                        if let Some(id) = windows.position(|(_, w)| w.1.windowtype == WindowEnum::WidgetVisualizer ) {
                            let window_id = self.windows.iter().nth(id).unwrap().0.clone();

                            return iced::Task::batch([
                                    window::minimize(window_id, false),
                                    window::gain_focus( window_id )
                            ]);
                        }

                        let (_id, open) = iced::window::open(window::Settings {
                            size: Size::new(1920_f32, 1080_f32),
                            min_size: Some(Size::new(700_f32, 975_f32)),
                            ..window::Settings::default()
                        });
                        return open.map(|id| Message::WindowOpened(id, WindowEnum::WidgetVisualizer))
                    }
                }
            },
            Message::WindowOpened(window_id, window_type) => {
                let title = match window_type {
                    WindowEnum::Main => { String::from("Theme Viewer") }
                    WindowEnum::WidgetVisualizer => { String::from("UI Builder") }
                };

                let new_window = Window::new(window_id, title, window_type);

                self.windows.insert(window_id, new_window);

                Task::none()
            },
            Message::Pane(m) => {
                if let Some(dock) = &mut self.pane {
                    return dock.update(m).map(Message::Pane);
                }
                Task::none()
            }
        }
    }

    fn view<'a>(&'a self, window_id: window::Id) -> Element<'a, Message> {

        let open_widget_visualizer = button("Open Widget Visualizer").on_press(Message::ShowWidgetBuilder);

        let theme_pick_list = pick_list(
            self.themes.clone(), 
            self.theme.clone(), 
            Message::ChooseTheme
        );

        let theme_selection = column![
            text("Theme").size(18),
            theme_pick_list
        ].spacing(5);

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
                    text_input("Disabled Text Input", &self.disabled_value).width(650)
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
                row![
                    theme_selection,
                    horizontal_space(),
                    open_widget_visualizer,
                ],
                
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
                WindowEnum::WidgetVisualizer => {
                    if let Some(pane) = &self.pane {
                        if pane.owns_window(window_id) {
                            return pane.view(window_id).map(Message::Pane);
                        }
                    }

                    self.widget_builder.view().map(Message::WidgetHelper)
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            self.pane
                .as_ref()
                .map(|p| p.subscription().map(Message::Pane))
                .unwrap_or(iced::Subscription::none()),

            event::listen_with(handle_event),
        ])
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
    WidgetVisualizer
}

#[derive(Debug, Clone,)]
pub struct Window {
    pub title: String,
    pub windowtype: WindowEnum,
}

impl Window {
    pub fn new(_id: window::Id, title: String, window_type: WindowEnum) -> Self {
        Self {
            title: title,
            windowtype: window_type,
        }
    }
}

fn handle_event(event: event::Event, _status: event::Status, id: iced::window::Id) -> Option<Message> {
    match event {
        event::Event::Window(window::Event::Closed) => Some(Message::WindowClosed(id)),
        _ => None,
    }
}