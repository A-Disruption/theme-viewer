use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, progress_bar, radio, row, scrollable, slider, text, text_input, toggler, vertical_space, Button, Column, Container, Radio, Row, Space, Text, TextInput
    },
    Alignment, Background, Border, Color, Element, Font, Length::{self, FillPortion}, Padding, Shadow, Task,
    Theme, Vector,
};
use std::collections::HashMap;
use crate::widget::tree::{tree, TreeNode};

#[derive(Debug, Clone)]
pub enum Message {
    TreeToggle(String),
    TreeSelect(String),
}

pub struct App {
    selected_item: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected_item: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TreeToggle(id) => {
                println!("Toggled: {}", id);
                // Tree state is now managed internally by the widget
            }
            Message::TreeSelect(id) => {
                self.selected_item = Some(id.clone());
                println!("Selected: {}", id);
            }
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        // Create sample tree data
        let nodes = vec![
            TreeNode::new("fruit".to_string(), "Fruit")
                .with_children(vec![
                    TreeNode::new("apple".to_string(), "Apple"),
                    TreeNode::new("orange".to_string(), "Orange"),
                    TreeNode::new("lemon".to_string(), "Lemon"),
                    TreeNode::new("berries".to_string(), "Berries")
                        .with_children(vec![
                            TreeNode::new("red".to_string(), "Red"),
                            TreeNode::new("blue".to_string(), "Blue"),
                            TreeNode::new("black".to_string(), "Black"),
                        ]),
                    TreeNode::new("banana".to_string(), "Banana"),
                ]),
            TreeNode::new("meals".to_string(), "Meals")
                .with_children(vec![
                    TreeNode::new("america".to_string(), "America"),
                    TreeNode::new("europe".to_string(), "Europe")
                        .with_children(vec![
                            TreeNode::new("risotto".to_string(), "Risotto"),
                            TreeNode::new("spaghetti".to_string(), "Spaghetti"),
                            TreeNode::new("pizza".to_string(), "Pizza"),
                            TreeNode::new("weisswurst".to_string(), "Weisswurst"),
                            TreeNode::new("spargel".to_string(), "Spargel"),
                        ]),
                    TreeNode::new("asia".to_string(), "Asia"),
                    TreeNode::new("australia".to_string(), "Australia"),
                ]),
            TreeNode::new("desserts".to_string(), "Desserts"),
            TreeNode::new("drinks".to_string(), "Drinks"),
        ];

        let tree_widget = tree(nodes)
            .on_toggle(Message::TreeToggle)
            .on_select(Message::TreeSelect)
            .spacing(2.0)
            .indent(20.0);

        column![
            iced::widget::text("Tree Widget Example").size(24),
            tree_widget,
            if let Some(ref selected) = self.selected_item {
                iced::widget::text(format!("Selected: {}", selected))
            } else {
                iced::widget::text("Nothing selected")
            }
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Action {
    Run(iced::Task<Message>),
    None,
}