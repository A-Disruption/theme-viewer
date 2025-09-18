// controls.rs
use iced::{
    widget::{
        button, checkbox, column, pick_list, radio, row, scrollable, slider, text, text_input, vertical_space, Space, horizontal_rule
    }, Alignment, Color, Element, Length, Padding, Theme
};
use crate::widget_helper::{
    AlignmentOption, AlignmentYOption, TextWrapping, TextShaping, ContainerAlignX, ContainerAlignY, Message, PropertyChange, WidgetHierarchy,
    WidgetId, ButtonStyleType, length_to_string, parse_length, FontType, RuleOrientation, AlignText, DirChoice, AnchorChoice, ContentFit, ContentFitChoice, TooltipPosition
};
use crate::widget_helper::code_generator::{CodeGenerator, build_code_view_with_height};

pub fn container_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
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
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn row_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
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
    .padding(10)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn column_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
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
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn button_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
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
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn text_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),

        color_hex_input("Text Color", props.text_color, move |c| {
            Message::PropertyChanged(widget_id, PropertyChange::TextColor(c))
        }),

        // Wrapping
        column![
            text("Wrapping"),
            pick_list(
                vec![TextWrapping::None, TextWrapping::Word, TextWrapping::Glyph, TextWrapping::WordOrGlyph],
                Some(TextWrapping::from(props.wrap)),
                move |w| Message::PropertyChanged(widget_id, PropertyChange::TextWrap(w))
            )
        ].spacing(5),

        // Shaping
        column![
            text("Shaping"),
            pick_list(
                vec![TextShaping::Basic, TextShaping::Advanced, TextShaping::Auto],
                Some(TextShaping::from(props.shaping)),
                move |s| Message::PropertyChanged(widget_id, PropertyChange::TextShaping(s))
            )
        ].spacing(5),

        // Line height
        column![
            text("Line Height"),
            // simple slider 0.8..=2.0 -> LineHeight::Relative
            row![
                slider(0.8..=2.0, match props.line_height { text::LineHeight::Relative(v) => v, _ => 1.0 }, move |v| {
                    Message::PropertyChanged(widget_id, PropertyChange::TextLineHeight(text::LineHeight::Relative((v*100.0).round()/100.0)))
                })
                .step(0.05)
                .width(220),
                text(match props.line_height { text::LineHeight::Relative(v) => format!("{:.2}", v), _ => "1.00".into() }).size(12)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        ].spacing(5),

        // Alignment
        row![
            column![
                text("Align X"),
                pick_list(
                    vec![AlignText::Default, AlignText::Left, AlignText::Center, AlignText::Right, AlignText::Justified],
                    Some(AlignText::from(props.text_align_x)),
                    move |a| Message::PropertyChanged(widget_id, PropertyChange::TextAlignX(a))
                )
            ].spacing(5).width(Length::Fill),
            column![
                text("Align Y"),
                pick_list(
                    vec![AlignmentYOption::Top, AlignmentYOption::Center, AlignmentYOption::Bottom],
                    Some(AlignmentYOption::from(props.text_align_y)),
                    move |a| Message::PropertyChanged(widget_id, PropertyChange::TextAlignY(a))
                )
            ].spacing(5).width(Length::Fill),
        ]
        .spacing(15),
    ]
    .spacing(15)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn text_input_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(15)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn checkbox_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(15)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn toggler_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(15)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn radio_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            hierarchy,
            widget_id,
        ),
    ]
    .spacing(15)
    .padding(10)
    .into();


    scrollable(add_code_preview(content, hierarchy, widget_id, theme))
        .width(450)
        .height(Length::Fixed(600.0))
        .into()
}

pub fn picklist_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).expect("widget exists");
    let props = &widget.properties;

    let content = column![
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
            text("Default Selection"),
            pick_list(
                props.picklist_options.clone(),
                props.picklist_selected.clone(),
                move |selection| Message::PropertyChanged(widget_id, PropertyChange::PickListSelected(Some(selection)))
            ),
        ]
        .spacing(6),

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

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(15)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn slider_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let min_str = format!("{:.3}", props.slider_min);
    let max_str = format!("{:.3}", props.slider_max);
    let step_str = format!("{:.3}", props.slider_step);
    let slider_height = format!("{:.0}", props.slider_height);

    let content = column![
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

        column![
            // Horizontal slider width = the "length" axis ⇒ keep as Length
            length_picker_scrollable_aware(
                "Width (Length)",
                props.width,
                move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
                hierarchy,
                widget_id,
                false
            ),
            // Height (thickness) as f32 in px
            text_input("px", &slider_height).on_input(move |s| {
                Message::PropertyChanged(widget_id, PropertyChange::SliderHeight(parse_f32(&s, props.slider_height)))
            }).width(120)
        ]
        .spacing(12)
    ]
    .spacing(15)
    .padding(10)
    .into();

scrollable(add_code_preview(content, hierarchy, widget_id, theme)).into()
}

pub fn vertical_slider_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let min_str = format!("{:.3}", props.slider_min);
    let max_str = format!("{:.3}", props.slider_max);
    let step_str = format!("{:.3}", props.slider_step);

    let slider_width = format!("{:.0}", props.slider_width);

    let content = column![
        text("Vertical Slider Properties").size(16),

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

        column![
            // Vertical slider height = the "length" axis ⇒ keep as Length
            length_picker_scrollable_aware(
                "Height (Length)",
                props.height,
                move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
                hierarchy,
                widget_id,
                true
            ),
            // Width (thickness) as f32 in px
            text_input("px", &slider_width).on_input(move |s| {
                Message::PropertyChanged(widget_id, PropertyChange::SliderWidth(parse_f32(&s, props.slider_width)))
            }).width(120)
        ]
        .spacing(12),
    ]
    .spacing(15)
    .padding(10)
    .into();

    scrollable(add_code_preview(content, hierarchy, widget_id, theme)).into()
}

pub fn rule_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).unwrap();
    let p = &widget.properties;

    let content = column![
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
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn scrollable_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let content = column![
        text("Scrollable Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ]
        .spacing(6),

        // Size
        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            hierarchy,
            widget_id,
        ),

        // Direction
        column![
            text("Direction"),
            pick_list(
                vec![DirChoice::Vertical, DirChoice::Horizontal, DirChoice::Both],
                Some(DirChoice::to_choice(props.scroll_dir)),
                move |c| Message::PropertyChanged(widget_id, PropertyChange::ScrollableDirection(DirChoice::from_choice(c)))
            )
        ]
        .spacing(6),

        // Anchors
        row![
            column![
                text("Anchor X"),
                pick_list(
                    vec![AnchorChoice::Start, AnchorChoice::End],
                    Some(AnchorChoice::from(props.anchor_x)),
                    move |a| Message::PropertyChanged(widget_id, PropertyChange::ScrollableAnchorX(AnchorChoice::from_anchor(a)))
                )
            ].spacing(6).width(Length::Fill),
            column![
                text("Anchor Y"),
                pick_list(
                    vec![AnchorChoice::Start, AnchorChoice::End],
                    Some(AnchorChoice::from(props.anchor_y)),
                    move |a| Message::PropertyChanged(widget_id, PropertyChange::ScrollableAnchorY(AnchorChoice::from_anchor(a)))
                )
            ].spacing(6).width(Length::Fill),
        ]
        .spacing(15),
    ]
    .spacing(15)
    .padding(10)
    .into();

    scrollable(add_code_preview(content, hierarchy, widget_id, theme)).into()
}

pub fn space_controls(hierarchy: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = hierarchy.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    // helpful quick presets
    let quick_px = [4.0_f32, 8.0, 12.0, 16.0, 24.0, 32.0];

    let content = column![
        text("Space Properties").size(16),

        column![
            text("Widget Name"),
            text_input("Name", &props.widget_name)
                .on_input(move |v| Message::PropertyChanged(widget_id, PropertyChange::WidgetName(v))),
        ],

        // Width / Height editors
        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            hierarchy,
            widget_id,
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
    .into();

    scrollable(add_code_preview(content, hierarchy, widget_id, theme)).into()
}

pub fn progress_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let w = h.get_widget_by_id(widget_id).unwrap();
    let p = &w.properties;
    let girth_str = format!("{:.0}", p.progress_girth);

    // convenience
    let clamp_step = ((p.progress_max - p.progress_min) / 100.0).abs().max(0.001);

    let content = column![
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

        // Length + Girth with scrollable awareness
        // For progress bars:
        // - Horizontal: Length = width, Girth = height
        // - Vertical: Length = height, Girth = width
        if p.progress_vertical {
            // Vertical progress: length ⇒ height(px), girth ⇒ width(px)
            column![
                length_picker_scrollable_aware( 
                    "Length", 
                    p.progress_length, 
                    move |len| Message::PropertyChanged(widget_id, PropertyChange::ProgressLength(len)), 
                    h, 
                    widget_id, 
                    true // Length is height when vertical 
                ),
                text_input("px", &girth_str).on_input(move |s| {
                    Message::PropertyChanged(widget_id, PropertyChange::ProgressGirth(parse_f32(&s, p.progress_girth)))
                }).width(120)
            ]
            .spacing(15)
        } else {
            // Horizontal progress: length ⇒ width(px), girth ⇒ height(px)
            column![
                length_picker_scrollable_aware( 
                    "Length", 
                    p.progress_length, 
                    move |len| Message::PropertyChanged(widget_id, PropertyChange::ProgressLength(len)), 
                    h, 
                    widget_id, 
                    true // Length is height when vertical 
                ),
                text_input("px", &girth_str).on_input(move |s| {
                    Message::PropertyChanged(widget_id, PropertyChange::ProgressGirth(parse_f32(&s, p.progress_girth)))
                }).width(120)
            ]
            .spacing(15)
        },

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
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn image_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let w = h.get_widget_by_id(widget_id).unwrap();
    let props = &w.properties;

    let content = column![
        text("Image Properties").size(16),
        row![
            text("Path").width(Length::Fixed(80.0)),
            text_input("assets/pic.png", &props.image_path)
                .on_input(move |s| Message::PropertyChanged(widget_id, PropertyChange::ImagePath(s)))
                .width(Length::Fill),
        ]
        .spacing(10),
        row![
            text("Fit").width(Length::Fixed(80.0)),
            pick_list(
                vec![
                    ContentFitChoice::Contain,
                    ContentFitChoice::Cover,
                    ContentFitChoice::Fill,
                    ContentFitChoice::ScaleDown,
                    ContentFitChoice::None,
                ],
                Some(props.image_fit),
                move |v| Message::PropertyChanged(widget_id, PropertyChange::ImageFit(v))
            )
        ]
        .spacing(10),

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(12)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn svg_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let widget = h.get_widget_by_id(widget_id).unwrap();
    let props = &widget.properties;

    let content = column![
        text("SVG Properties").size(16),
        row![
            text("Path").width(Length::Fixed(80.0)),
            text_input("assets/icon.svg", &props.svg_path)
                .on_input(move |s| Message::PropertyChanged(widget_id, PropertyChange::SvgPath(s)))
                .width(Length::Fill),
        ]
        .spacing(10),
        row![
            text("Fit").width(Length::Fixed(80.0)),
            pick_list(
                vec![
                    ContentFitChoice::Contain,
                    ContentFitChoice::Cover,
                    ContentFitChoice::Fill,
                    ContentFitChoice::ScaleDown,
                    ContentFitChoice::None,
                ],
                Some(props.svg_fit),
                move |v| Message::PropertyChanged(widget_id, PropertyChange::SvgFit(v))
            )
        ]
        .spacing(10),

        size_controls_scrollable_aware(
            props.width,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Width(l)),
            props.height,
            move |l| Message::PropertyChanged(widget_id, PropertyChange::Height(l)),
            h,
            widget_id,
        ),
    ]
    .spacing(12)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
}

pub fn tooltip_controls(h: &WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<Message> {
    let w = h.get_widget_by_id(widget_id).unwrap();
    let p = &w.properties;

    let content = column![
        text("Tooltip Properties").size(16),
        row![
            text("Text").width(Length::Fixed(80.0)),
            text_input("Tooltip text", &p.tooltip_text)
                .on_input(move |s| Message::PropertyChanged(widget_id, PropertyChange::TooltipText(s)))
                .width(Length::Fill),
        ]
        .spacing(10),
        row![
            text("Position").width(Length::Fixed(80.0)),
            pick_list(
                vec![TooltipPosition::Top, TooltipPosition::Bottom, TooltipPosition::Left, TooltipPosition::Right],
                Some(p.tooltip_position),
                move |pos| Message::PropertyChanged(widget_id, PropertyChange::TooltipPosition(pos))
            )
        ]
        .spacing(10),
        column![
            text("Tip: Tooltip wraps two children. Add them under it in the tree.")
                .size(12).color(Color::from_rgb(0.6,0.6,0.6)),
            text("1st child is the element you hover")
                .size(12).color(Color::from_rgb(0.6,0.6,0.6)),
            text("2nd child is the tooltip content")
                .size(12).color(Color::from_rgb(0.6,0.6,0.6)),
        ],

    ]
    .spacing(12)
    .into();

    scrollable(add_code_preview(content, h, widget_id, theme)).into()
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

fn parse_f32(s: &str, default: f32) -> f32 {
    s.trim().parse::<f32>().unwrap_or(default)
}

fn parse_color_hex(s: &str, default: Color) -> Color {
    let t = s.trim().trim_start_matches('#');
    let hex = |i: usize| u8::from_str_radix(&t[i..i+2], 16).ok();
    match t.len() {
        6 => {
            if let (Some(r), Some(g), Some(b)) = (hex(0), hex(2), hex(4)) {
                return Color::from_rgba8(r, g, b, 255.0);
            }
            default
        }
        8 => {
            if let (Some(r), Some(g), Some(b), Some(a)) = (hex(0), hex(2), hex(4), hex(6)) {
                return Color::from_rgba8(r, g, b, a.into());
            }
            default
        }
        _ => default,
    }
}

fn color_to_hex(c: Color) -> String {
    let [r,g,b,a] = [
        (c.r * 255.0).round().clamp(0.0,255.0) as u8,
        (c.g * 255.0).round().clamp(0.0,255.0) as u8,
        (c.b * 255.0).round().clamp(0.0,255.0) as u8,
        (c.a * 255.0).round().clamp(0.0,255.0) as u8,
    ];
    if a == 255 { format!("#{:02X}{:02X}{:02X}", r,g,b) }
    else { format!("#{:02X}{:02X}{:02X}{:02X}", r,g,b,a) }
}

fn color_hex_input<'a, F>(label: &'a str, current: Color, on_change: F) -> Element<'a, Message>
where F: Fn(Color) -> Message + 'a + Copy {
    let cur = color_to_hex(current);
    column![
        text(label),
        text_input("#RRGGBB or #RRGGBBAA", &cur)
            .on_input(move |s| on_change(parse_color_hex(&s, current)))
            .width(160)
    ]
    .spacing(5)
    .into()
}

/// Helper for scrollable-aware size controls
pub fn size_controls_scrollable_aware<'a, F, G>(
    width_now: Length,
    on_width: F,
    height_now: Length,
    on_height: G,
    hierarchy: &WidgetHierarchy,
    widget_id: WidgetId,
) -> Element<'a, Message>
where
    F: Fn(Length) -> Message + 'a + Copy,
    G: Fn(Length) -> Message + 'a + Copy,
{
    column![
        length_picker_scrollable_aware("Width", width_now, on_width, hierarchy, widget_id, false),
        length_picker_scrollable_aware("Height", height_now, on_height, hierarchy, widget_id, true),
    ]
    .spacing(15)
    .into()
}

/// A single labeled Length selector that's aware of scrollable constraints
pub fn length_picker_scrollable_aware<'a, F>(
    label: &'a str,
    current: Length,
    on_change: F,
    hierarchy: &WidgetHierarchy,
    widget_id: WidgetId,
    is_height: bool, // true for height, false for width
) -> Element<'a, Message>
where
    F: Fn(Length) -> Message + 'a + Copy,
{
    const DEFAULT_PX: f32 = 120.0;
    const DEFAULT_PORTION: u16 = 1;

    // Check if under scrollable and what dimensions are constrained
    let (can_fill, saved_value) = if let Some((_, scroll_dir)) = hierarchy.get_scrollable_ancestor_info(widget_id) {
        let height_blocked = match scroll_dir {
            iced::widget::scrollable::Direction::Vertical(_) => true,
            iced::widget::scrollable::Direction::Both { .. } => true,
            iced::widget::scrollable::Direction::Horizontal(_) => false,
        };
        
        let width_blocked = match scroll_dir {
            iced::widget::scrollable::Direction::Horizontal(_) => true,
            iced::widget::scrollable::Direction::Both { .. } => true,
            iced::widget::scrollable::Direction::Vertical(_) => false,
        };
        
        let blocked = if is_height { height_blocked } else { width_blocked };
        
        if blocked {
            // Get the saved value if it exists
            let saved = if let Some(widget) = hierarchy.get_widget_by_id(widget_id) {
                if is_height {
                    widget.properties.saved_height_before_scrollable
                } else {
                    widget.properties.saved_width_before_scrollable
                }
            } else {
                None
            };
            (!blocked, saved)
        } else {
            (true, None)
        }
    } else {
        (true, None)
    };

    let choice_now = LengthChoice::from_length(current);

    // Build available choices based on scrollable constraints
    let mut available_choices = vec![LengthChoice::Shrink, LengthChoice::Fixed];
    if can_fill {
        available_choices.insert(0, LengthChoice::Fill);
        available_choices.insert(1, LengthChoice::FillPortion);
    }

    let picker = column![
        if !can_fill && saved_value.is_some() {
            column![
                text(label),
                text(format!("(was: {})", length_to_string(saved_value.unwrap())))
                    .size(10)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
            ]
        } else {
            column![text(label)]
        },
        pick_list(
            available_choices,
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

    // Secondary control for Fixed and FillPortion
    let extra: Element<_> = match choice_now {
        LengthChoice::Fixed => {
            let value_str = match current {
                Length::Fixed(px) => format!("{px}"),
                _ => format!("{DEFAULT_PX}"),
            };
            column![
                text("Pixels"),
                text_input("e.g. 120", &value_str)
                    .on_input(move |v| on_change(parse_length(&v)))
                    .width(120)
            ]
            .spacing(5)
            .into()
        }
        LengthChoice::FillPortion if can_fill => {
            let portion_now = match current {
                Length::FillPortion(p) => p,
                _ => DEFAULT_PORTION,
            };
            let value_str = portion_now.to_string();
            column![
                text("Portion"),
                text_input("e.g. 1", &value_str)
                    .on_input(move |v| {
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

pub fn add_code_preview<'a>(content: Element<'a, Message>, hierarchy: &'a WidgetHierarchy, widget_id: WidgetId, theme: Theme) -> Element<'a, Message> {
    let mut generator = CodeGenerator::new(hierarchy, theme.clone());
    let tokens = generator.generate_widget_code(widget_id);
    
    // Check if we have code to display
    if tokens.is_empty() {
        return content;
    }
    
    column![
        content,
        
        // Code preview section
        column![
            vertical_space().height(20),
            horizontal_rule(2),
            vertical_space().height(10),
            text("Generated Code").size(16),
            // Use a reasonable height for widget-specific code
            build_code_view_with_height(&tokens, 200.0, theme),
        ].spacing(5)
    ]
    .into()
}