// controls.rs
use iced::{
    Alignment, Color, Element, Length, Padding,
    widget::{
        button, checkbox, column, pick_list, row, slider, text, text_input, Space, scrollable, radio
    },
};
use crate::widget_helper::{
    AlignmentOption, ContainerAlignX, ContainerAlignY, Message, PropertyChange, WidgetHierarchy,
    WidgetId, ButtonStyleType, length_to_string, parse_length, FontType, RuleOrientation
};

pub fn container_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Container Properties").size(16),

        // Widget Name
        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        // Alignment
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
            .width(Length::Fill),
            column![
                text("Vertical Align"),
                pick_list(
                    vec![ContainerAlignY::Top, ContainerAlignY::Center, ContainerAlignY::Bottom],
                    Some(props.align_y),
                    move |v| Message::PropertyChanged(widget_id, PropertyChange::AlignY(v)),
                ),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),

        text("Border").size(14),
        row![
            column![
                text("Border Width").size(12),
                slider(0.0..=15.0, props.border_width, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::BorderWidth(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.border_width)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Border Radius").size(12),
                slider(0.0..=30.0, props.border_radius, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::BorderRadius(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.border_radius)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),

        text("Padding").size(14),
        row![
            column![
                text("Top").size(12),
                slider(0.0..=50.0, props.padding.top, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.top)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Right").size(12),
                slider(0.0..=50.0, props.padding.right, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.right)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),
        row![
            column![
                text("Bottom").size(12),
                slider(0.0..=50.0, props.padding.bottom, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.bottom)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Left").size(12),
                slider(0.0..=50.0, props.padding.left, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.left)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),

        // Shadow
        column![
            checkbox("Enable Shadow", props.has_shadow)
                .on_toggle(move |v| Message::PropertyChanged(widget_id, PropertyChange::HasShadow(v))),
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
                        ],
                        column![
                            text("Offset Y").size(12),
                            slider(-20.0..=20.0, props.shadow_offset.y, move |v| {
                                Message::PropertyChanged(widget_id, PropertyChange::ShadowOffsetY(v))
                            })
                            .step(1.0),
                            text(format!("{:.0}", props.shadow_offset.y)).size(12).center(),
                        ],
                    ]
                    .spacing(15),
                    column![
                        text("Blur Radius").size(12),
                        slider(0.0..=50.0, props.shadow_blur, move |v| {
                            Message::PropertyChanged(widget_id, PropertyChange::ShadowBlur(v))
                        })
                        .step(1.0),
                        text(format!("{:.0}", props.shadow_blur)).size(12).center(),
                    ],
                ]
                .spacing(10)
            } else {
                column![]
            }
        ]
        .spacing(10),
    ]
    .spacing(15)
    .into()
}

pub fn row_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Row Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Spacing between items"),
            row![
                slider(0.0..=50.0, props.spacing, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::Spacing(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.spacing)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Vertical Alignment"),
            pick_list(
                vec![AlignmentOption::Start, AlignmentOption::Center, AlignmentOption::End],
                Some(AlignmentOption::from_alignment(props.align_items)),
                move |sel| Message::PropertyChanged(widget_id, PropertyChange::AlignItems(sel.to_alignment())),
            ),
        ]
        .spacing(5),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),

        // Padding
        text("Padding").size(14),
        row![
            column![
                text("Top").size(12),
                slider(0.0..=50.0, props.padding.top, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.top)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Right").size(12),
                slider(0.0..=50.0, props.padding.right, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.right)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),
        row![
            column![
                text("Bottom").size(12),
                slider(0.0..=50.0, props.padding.bottom, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.bottom)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Left").size(12),
                slider(0.0..=50.0, props.padding.left, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.left)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),
    ]
    .into()
}

pub fn column_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Column Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Spacing between items"),
            row![
                slider(0.0..=50.0, props.spacing, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::Spacing(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.spacing)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Horizontal Alignment"),
            pick_list(
                vec![AlignmentOption::Start, AlignmentOption::Center, AlignmentOption::End],
                Some(AlignmentOption::from_alignment(props.align_items)),
                move |sel| Message::PropertyChanged(widget_id, PropertyChange::AlignItems(sel.to_alignment())),
            ),
        ]
        .spacing(5),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),

        // Padding
        text("Padding").size(14),
        row![
            column![
                text("Top").size(12),
                slider(0.0..=50.0, props.padding.top, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.top)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Right").size(12),
                slider(0.0..=50.0, props.padding.right, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.right)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),
        row![
            column![
                text("Bottom").size(12),
                slider(0.0..=50.0, props.padding.bottom, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.bottom)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
            column![
                text("Left").size(12),
                slider(0.0..=50.0, props.padding.left, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.left)).size(12).center(),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(15),
    ]
    .spacing(15)
    .padding(20)
    .into()
}

pub fn button_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Button Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Button Text"),
            text_input("Text", &props.text_content)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::TextContent(v)))
                .width(250),
        ]
        .spacing(5),

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
            )
            .width(250),
        ]
        .spacing(5),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),

        text("Padding").size(14),
        row![
            column![
                text("Top"),
                slider(0.0..=30.0, props.padding.top, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingTop(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.top)).size(12).center(),
            ]
            .spacing(5),
            column![
                text("Right"),
                slider(0.0..=30.0, props.padding.right, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingRight(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.right)).size(12).center(),
            ]
            .spacing(5),
        ]
        .spacing(15),
        row![
            column![
                text("Bottom"),
                slider(0.0..=30.0, props.padding.bottom, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingBottom(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.bottom)).size(12).center(),
            ]
            .spacing(5),
            column![
                text("Left"),
                slider(0.0..=30.0, props.padding.left, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::PaddingLeft(v))
                })
                .step(1.0),
                text(format!("{:.0}", props.padding.left)).size(12).center(),
            ]
            .spacing(5),
        ]
        .spacing(15),
    ]
    .spacing(15)
    .into()
}

pub fn text_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Text Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Text Content"),
            text_input("Content", &props.text_content)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::TextContent(v)))
                .width(300),
        ]
        .spacing(5),

        column![
            text("Font Size"),
            row![
                slider(8.0..=72.0, props.text_size, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TextSize(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.text_size)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Font"),
            pick_list(
                vec![FontType::Default, FontType::Monospace],
                Some(props.font),
                move |v| Message::PropertyChanged(widget_id, PropertyChange::Font(v)),
            )
            .width(200),
        ]
        .spacing(5),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        )
    ]
    .spacing(15)
    .into()
}

pub fn text_input_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Text Input Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Placeholder Text"),
            text_input("Placeholder", &props.text_input_placeholder)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::TextInputPlaceholder(v)))
                .width(300),
        ]
        .spacing(5),

        column![
            text("Font Size"),
            row![
                slider(8.0..=32.0, props.text_input_size, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TextInputSize(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.text_input_size)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Internal Padding"),
            row![
                slider(0.0..=30.0, props.text_input_padding, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TextInputPadding(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.text_input_padding)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        checkbox("Secure Input (Password)", props.is_secure)
            .on_toggle(move |v| Message::PropertyChanged(widget_id, PropertyChange::IsSecure(v))),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        )
    ]
    .spacing(15)
    .into()
}

pub fn checkbox_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Checkbox Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Label Text"),
            text_input("Label", &props.checkbox_label)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::CheckboxLabel(v)))
                .width(250),
        ]
        .spacing(5),

        column![
            text("Checkbox Size"),
            row![
                slider(12.0..=40.0, props.checkbox_size, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::CheckboxSize(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.checkbox_size)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Label Spacing"),
            row![
                slider(0.0..=30.0, props.checkbox_spacing, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::CheckboxSpacing(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.checkbox_spacing)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        checkbox("Default Checked State", props.checkbox_checked)
            .on_toggle(move |v| Message::PropertyChanged(widget_id, PropertyChange::CheckboxChecked(v))),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        )
    ]
    .spacing(15)
    .into()
}

pub fn toggler_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Toggler Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Label Text"),
            text_input("Label", &props.toggler_label)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::TogglerLabel(v)))
                .width(250),
        ]
        .spacing(5),

        column![
            text("Toggler Size"),
            row![
                slider(12.0..=40.0, props.toggler_size, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TogglerSize(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.toggler_size)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        column![
            text("Label Spacing"),
            row![
                slider(0.0..=30.0, props.toggler_spacing, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TogglerSpacing(v))
                })
                .step(1.0)
                .width(200),
                text(format!("{:.0}px", props.toggler_spacing)).size(12).width(50),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(5),

        checkbox("Default Active State", props.toggler_active)
            .on_toggle(move |v| Message::PropertyChanged(widget_id, PropertyChange::TogglerActive(v))),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        )
    ]
    .spacing(15)
    .into()
}

pub fn radio_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let content = column![
        text("Radio Button Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        // Label text
        column![
            text("Label Text"),
            text_input("Label", &props.radio_label)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::RadioLabel(v)))
                .width(250),
        ]
        .spacing(5),

        // Size & spacing
        row![
            column![
                text("Radio Size"),
                row![
                    slider(12.0..=40.0, props.radio_size, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSize(v))
                    })
                    .step(1.0)
                    .width(200),
                    text(format!("{:.0}px", props.radio_size)).size(12).width(50),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(5),
            column![
                text("Label Spacing"),
                row![
                    slider(0.0..=30.0, props.radio_spacing, move |v| {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSpacing(v))
                    })
                    .step(1.0)
                    .width(200),
                    text(format!("{:.0}px", props.radio_spacing)).size(12).width(50),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(5),
        ]
        .spacing(15),

        // Options editor
        column![
            text("Options"),
            column(
                props.radio_options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let label = format!("Option {}", i + 1);
                        row![
                            text_input(&label, option)
                                .on_input({
                                    let index = i;
                                    let existing = props.radio_options.clone();
                                    move |v| {
                                        let mut next = existing.clone();
                                        if index < next.len() { next[index] = v; }
                                        Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(next))
                                    }
                                })
                                .width(220),
                            button("Remove")
                                .on_press({
                                    let index = i;
                                    let mut next = props.radio_options.clone();
                                    if index < next.len() && next.len() > 1 { next.remove(index); }
                                    Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(next))
                                }),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>()
            )
            .spacing(8),
            button("Add Option")
                .on_press({
                    let mut next = props.radio_options.clone();
                    next.push(format!("Option {}", next.len() + 1));
                    Message::PropertyChanged(widget_id, PropertyChange::RadioOptions(next))
                })
        ]
        .spacing(10),

        // Default selected
        column![
            text("Default Selection"),
            pick_list(
                props.radio_options.clone(),
                props.radio_options.get(props.radio_selected_index).cloned(),
                move |selected| {
                    let current = props.radio_options.clone();
                    if let Some(ix) = current.iter().position(|s| s == &selected) {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSelectedIndex(ix))
                    } else {
                        Message::PropertyChanged(widget_id, PropertyChange::RadioSelectedIndex(0))
                    }
                }
            )
            .width(220),
        ]
        .spacing(6),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),
    ]
    .spacing(15)
    .padding(10);

    // Make the *editor* scrollable so long forms fit nicely in the overlay
    scrollable(content)
        .height(Length::Fixed(600.0))
        .into()
}

pub fn picklist_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    column![
        text("Pick List Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        column![
            text("Placeholder Text"),
            text_input("Placeholder", &props.picklist_placeholder)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::PickListPlaceholder(v)))
                .width(250),
        ]
        .spacing(5),

        column![
            text("Options"),
            column(
                props.picklist_options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        row![
                            text_input(&format!("Option {}", i + 1), option)
                                .on_input({
                                    let index = i;
                                    let current = props.picklist_options.clone();
                                    move |v| {
                                        let mut new_options = current.clone();
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
                                    let mut new_options = props.picklist_options.clone();
                                    if index < new_options.len() {
                                        new_options.remove(index);
                                    }
                                    Message::PropertyChanged(widget_id, PropertyChange::PickListOptions(new_options))
                                })
                                .style(button::danger)
                                .padding(Padding::new(5.0)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>()
            )
            .spacing(5),

            button("Add Option")
                .on_press({
                    let mut new_options = props.picklist_options.clone();
                    new_options.push(format!("Option {}", new_options.len() + 1));
                    Message::PropertyChanged(widget_id, PropertyChange::PickListOptions(new_options))
                })
                .style(button::success)
                .padding(Padding::new(5.0)),
        ]
        .spacing(10),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        )
    ]
    .spacing(15)
    .into()
}

pub fn slider_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let min_str = format!("{:.3}", props.slider_min);
    let max_str = format!("{:.3}", props.slider_max);
    let step_str = format!("{:.3}", props.slider_step);

    column![
        text("Slider Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        // Min / Max / Step as text inputs
        row![
            column![
                text("Min"),
                text_input("min", &min_str).on_input(move |s| {
                    let v = parse_f32(&s, props.slider_min);
                    Message::PropertyChanged(widget_id, PropertyChange::SliderMin(v))
                }).width(120),
            ],
            column![
                text("Max"),
                text_input("max", &max_str).on_input(move |s| {
                    let v = parse_f32(&s, props.slider_max);
                    Message::PropertyChanged(widget_id, PropertyChange::SliderMax(v))
                }).width(120),
            ],
            column![
                text("Step"),
                text_input("step", &step_str).on_input(move |s| {
                    let v = parse_f32(&s, props.slider_step.max(0.000_001));
                    Message::PropertyChanged(widget_id, PropertyChange::SliderStep(v.max(0.000_001)))
                }).width(120),
            ],
        ]
        .spacing(15),

        // Live value slider
        column![
            text("Value"),
            row![
                slider(props.slider_min..=props.slider_max, props.slider_value, move |val| {
                    Message::PropertyChanged(widget_id, PropertyChange::SliderValue(val))
                })
                .step(props.slider_step.max(0.000_001))
                .width(300),
                text(format!("{:.3}", props.slider_value)).size(12),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(6),

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),
    ]
    .spacing(15)
    .padding(10)
    .into()
}

pub fn rule_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).unwrap();
    let p = &widget.properties;

    column![
        text("Rule").size(16),

        // Orientation
        column![
            text("Orientation"),
            pick_list(
                vec![RuleOrientation::Horizontal, RuleOrientation::Vertical],
                Some(p.rule_orientation),
                move |o| Message::PropertyChanged(widget_id, PropertyChange::RuleOrientation(o))
            )
        ]
        .spacing(6),

        // Thickness (px)
        column![
            text(format!("Thickness: {} px", p.rule_thickness)).size(12),
            slider(1.0..=20.0, p.rule_thickness as f32, move |v| {
                Message::PropertyChanged(widget_id, PropertyChange::RuleThickness(v.round()))
            })
            .step(1.0)
        ]
        .spacing(6),

        // Optional: quick presets
        row([1.0_f32,2.0,3.0,4.0,6.0,8.0,12.0].into_iter().map(|px| {
            button(text(format!("{px}px")))
                .on_press(Message::PropertyChanged(widget_id, PropertyChange::RuleThickness(px)))
                .padding(6)
                .into()
        }).collect::<Vec<_>>())
        .spacing(8)
    ]
    .spacing(12)
    .into()
}

pub fn scrollable_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    column![
        text("Scrollable Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),
    ]
    .spacing(15)
    .padding(10)
    .into()
}

pub fn space_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    // helpful quick presets
    let quick_px = [4.0_f32, 8.0, 12.0, 16.0, 24.0, 32.0];

    column![
        text("Space Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        // Width / Height editors
        size_controls(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
        ),

        // Quick size presets (sets Fixed for whichever axis is Shrink)
        row(
            quick_px.iter().map(|px| {
                let px_copy = *px;
                button(text(format!("{:.0}px", px)))
                    .on_press({
                        // choose a reasonable default: if height is Shrink, set height; else width
                        let set_height = matches!(props.height, Length::Shrink);
                        if set_height {
                            Message::PropertyChanged(widget_id, PropertyChange::Height(Length::Fixed(px_copy)))
                        } else {
                            Message::PropertyChanged(widget_id, PropertyChange::Width(Length::Fixed(px_copy)))
                        }
                    })
                    .into()
            }).collect::<Vec<_>>()
        )
        .spacing(8),
    ]
    .spacing(14)
    .padding(10)
    .into()
}

pub fn progress_controls(h: &WidgetHierarchy, widget_id: WidgetId) -> Element<Message> {
    let w = h.get_widget_by_id(widget_id).unwrap();
    let p = &w.properties;

    // convenience
    let clamp_step = ((p.progress_max - p.progress_min) / 100.0).abs().max(0.001);

    column![
        text("Progress Bar").size(16),

        // Name
        column![
            text("Name"),
            text_input("Name", &p.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v)))
        ].spacing(6),

        // Orientation
        row![
            text("Orientation").size(12).width(Length::Fixed(80.0)),
            radio("Horizontal", false, Some(p.progress_vertical), move |_|
                Message::PropertyChanged(widget_id, PropertyChange::ProgressVertical(false))
            ),
            radio("Vertical", true, Some(p.progress_vertical), move |_|
                Message::PropertyChanged(widget_id, PropertyChange::ProgressVertical(true))
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Center),

        // Length + Girth
        length_picker(
            "Length",
            p.progress_length,
            move |len| Message::PropertyChanged(widget_id, PropertyChange::ProgressLength(len))
        ),
        length_picker(
            "Girth",
            p.progress_girth,
            move |len| Message::PropertyChanged(widget_id, PropertyChange::ProgressGirth(len))
        ),

        // Range
        column![
            text("Range").size(12),
            row![
                column![
                    text("Min"),
                    text_input("min", &format!("{}", p.progress_min)).on_input(move |s| {
                        let v = s.trim().parse::<f32>().unwrap_or(p.progress_min);
                        Message::PropertyChanged(widget_id, PropertyChange::ProgressMin(v))
                    })
                    .width(120)
                ],
                column![
                    text("Max"),
                    text_input("max", &format!("{}", p.progress_max)).on_input(move |s| {
                        let v = s.trim().parse::<f32>().unwrap_or(p.progress_max);
                        Message::PropertyChanged(widget_id, PropertyChange::ProgressMax(v))
                    })
                    .width(120)
                ],
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(6),

        // Value
        column![
            text(format!("Value: {:.02}", p.progress_value)).size(12),
            slider(p.progress_min..=p.progress_max, p.progress_value, move |v| {
                Message::PropertyChanged(widget_id, PropertyChange::ProgressValue(v))
            })
            .step(clamp_step),
        ]
        .spacing(6),
    ]
    .spacing(12)
    .into()
}


/// What the user is choosing for a Length
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthChoice {
    Fill,
    FillPortion,
    Shrink,
    Fixed,
}

impl std::fmt::Display for LengthChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthChoice::Fill => write!(f, "Fill"),
            LengthChoice::FillPortion => write!(f, "FillPortion"),
            LengthChoice::Shrink => write!(f, "Shrink"),
            LengthChoice::Fixed => write!(f, "Fixed"),
        }
    }
}

impl LengthChoice {
    fn from_length(len: Length) -> Self {
        match len {
            Length::Fill => LengthChoice::Fill,
            Length::FillPortion(_) => LengthChoice::FillPortion,
            Length::Shrink => LengthChoice::Shrink,
            Length::Fixed(_) => LengthChoice::Fixed,
            _ => LengthChoice::Shrink, // fallback for any other variants
        }
    }
}

/// A single labeled Length selector: pick list + optional px textbox.
/// `on_change` should send a Message with the new `Length`.
pub fn length_picker<'a, F>(
    label: &'a str,
    current: Length,
    on_change: F,
) -> Element<'a, Message>
where
    F: Fn(Length) -> Message + 'a + Copy,
{
    const DEFAULT_PX: f32 = 120.0;
    const DEFAULT_PORTION: u16 = 1;

    let choice_now = LengthChoice::from_length(current);

    // When user changes the pick_list, update the Length immediately.
    let picker = column![
        text(label),
        pick_list(
            vec![LengthChoice::Fill, LengthChoice::FillPortion, LengthChoice::Shrink, LengthChoice::Fixed],
            Some(choice_now),
            move |choice| {
                let new_len = match choice {
                    LengthChoice::Fill => Length::Fill,
                    LengthChoice::FillPortion => match current {
                        Length::FillPortion(p) => Length::FillPortion(p),
                        _ => Length::FillPortion(DEFAULT_PORTION),
                    },
                    LengthChoice::Shrink => Length::Shrink,
                    LengthChoice::Fixed => {
                        match current {
                            Length::Fixed(px) => Length::Fixed(px),
                            _ => Length::Fixed(DEFAULT_PX),
                        }
                    }
                };
                on_change(new_len)
            }
        )
        .width(160)
    ]
    .spacing(5)
    .width(Length::Shrink);

    // Secondary control: appears only for the modes that need a value
    let extra: Element<_> = match choice_now {
        LengthChoice::Fixed => {
            let value_str = match current {
                Length::Fixed(px) => format!("{px}"),
                _ => format!("{DEFAULT_PX}"),
            };
            column![
                text("Pixels"),
                text_input("e.g. 120", &value_str)
                    .on_input(move |v| on_change(parse_length(&v))) // your existing parser
                    .width(120)
            ]
            .spacing(5)
            .into()
        }
        LengthChoice::FillPortion => {
            let portion_now = match current {
                Length::FillPortion(p) => p,
                _ => DEFAULT_PORTION,
            };
            let value_str = portion_now.to_string();
            column![
                text("Portion"),
                text_input("e.g. 1", &value_str)
                    .on_input(move |v| {
                        // parse as u16; clamp to at least 1
                        let p = v.trim().parse::<u16>().ok().map(|x| x.max(1)).unwrap_or(DEFAULT_PORTION);
                        on_change(Length::FillPortion(p))
                    })
                    .width(120)
            ]
            .spacing(5)
            .into()
        }
        _ => Space::with_width(0).into(),
    };

    row![picker, extra].spacing(15).into()
}

/// Convenience: a row with Width + Height using `length_picker`.
pub fn size_controls<'a, F, G>(
    width_now: Length,
    on_width: F,
    height_now: Length,
    on_height: G,
) -> Element<'a, Message>
where
    F: Fn(Length) -> Message + 'a + Copy,
    G: Fn(Length) -> Message + 'a + Copy,
{
    column![
        length_picker("Width", width_now, on_width),
        length_picker("Height", height_now, on_height),
    ]
    .spacing(15)
    .into()
}

fn parse_f32(s: &str, default: f32) -> f32 {
    s.trim().parse::<f32>().unwrap_or(default)
}