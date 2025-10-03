use iced::{Element, Settings, Theme, Length,
    widget::{ button, column, row, rule, scrollable, text },
};
use crate::widget_helper::{WidgetType, WidgetId};

// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    SelectWidgetType(WidgetType),
}

pub fn view<'a>(
    parent_id: WidgetId,
    available_types: &[WidgetType],
) -> Element<'a, Message> {
    // Helper to create a button if the type is available
    let widget_button = |widget_type: WidgetType, label: &'a str| -> Element<'a, Message> {
        if available_types.contains(&widget_type) {
            button(text(label).center())
                .on_press(Message::SelectWidgetType(widget_type))
                .style(button::secondary)
                .width(Length::FillPortion(1))
                .into()
        } else {
            button(text(label).center())
                .style(button::secondary)
                .width(Length::FillPortion(1))
                .into()
        }
    };

    scrollable(
        column![
            if available_types.contains(&WidgetType::Container) 
                || available_types.contains(&WidgetType::Scrollable) {
                column![
                    text("Containers").size(18),
                    rule::horizontal(2),
                    row![
                        widget_button(WidgetType::Container, "Container"),
                        widget_button(WidgetType::Scrollable, "Scrollable")
                    ]
                    .spacing(10)
                    .padding(5),
                ]
            } else {
                column![]
            },
            
            if available_types.contains(&WidgetType::Row) 
                || available_types.contains(&WidgetType::Column) {
                column![
                    text("Layout").size(18),
                    rule::horizontal(2),
                    row![
                        widget_button(WidgetType::Row, "Row"),
                        widget_button(WidgetType::Column, "Column")
                    ]
                    .spacing(10)
                    .padding(5),
                ]
            } else {
                column![]
            },

            if !available_types.is_empty(){
                column![
                    text("Widgets").size(18),
                    rule::horizontal(2),
                    
                    row![
                        widget_button(WidgetType::Text, "Text"),
                        widget_button(WidgetType::TextInput, "Text Input"),
                        widget_button(WidgetType::Button, "Button")
                    ]
                    .spacing(10)
                    .padding(5),
                    
                    row![
                        widget_button(WidgetType::Checkbox, "Checkbox"),
                        widget_button(WidgetType::Radio, "Radio"),
                        widget_button(WidgetType::Toggler, "Toggler")
                    ]
                    .spacing(10)
                    .padding(5),
                    
                    row![
                        widget_button(WidgetType::Slider, "Slider"),
                        widget_button(WidgetType::VerticalSlider, "Vert. Slider"),
                        widget_button(WidgetType::ProgressBar, "Progress")
                    ]
                    .spacing(10)
                    .padding(5),
                    
                    row![
                        widget_button(WidgetType::PickList, "Pick List"),
                        widget_button(WidgetType::Space, "Space"),
                        widget_button(WidgetType::Rule, "Rule")
                    ]
                    .spacing(10)
                    .padding(5),
                    
                    row![
                        widget_button(WidgetType::Image, "Image"),
                        widget_button(WidgetType::Svg, "SVG"),
                        widget_button(WidgetType::Tooltip, "Tooltip")
                    ]
                    .spacing(10)
                    .padding(5),

                    row![
                        widget_button(WidgetType::ComboBox, "ComboBox"),
                        widget_button(WidgetType::Markdown, "Markdown"),
                        widget_button(WidgetType::MouseArea, "MouseArea")
                    ]
                    .spacing(10)
                    .padding(5),

                    row![
                        widget_button(WidgetType::Pin, "Pin"),
                        widget_button(WidgetType::QRCode, "QRCode"),
                    ]
                    .spacing(10)
                    .padding(5),
                ]
            } else {
                column![]
            }, 


            
            if available_types.is_empty() {
                column![
                    text("No widgets can be added to this parent").size(14)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .padding(10)
            } else {
                column![]
            }
        ]
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}