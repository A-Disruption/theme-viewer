use iced::{widget::{button, container, text, column, row}, Application, Task, Element, Settings, Theme};
use crate::widget::tree::{Tree, TreeNode, NodeId};    //{tree_node, DropPosition, TreeManager, TreeNodeContent};

//#[derive(Debug, Clone)]
/* pub enum Message {
    ToggleNode(String),
    SelectNode(String),
    DropNode(Vec<String>, String, DropPosition),
    ButtonPressed(String),
} */

#[derive(Debug, Clone)]
pub enum Message {
    TreeToggle(NodeId, bool),
    TreeDragStart(NodeId, iced::Point),
    TreeDragOver(NodeId, NodeId, iced::Point),
    TreeDrop(NodeId, NodeId, iced::Point),
    ButtonPressed(String),
}

pub struct App { }

impl App {

    pub fn new() -> Self {
        Self {  }
    }

    pub fn update(&mut self, message: Message) -> Action {
/*         match message {
            Message::ToggleNode(id) => {
                println!("Toggled node: {}", id);
            }
            Message::SelectNode(id) => {
                println!("Selected node: {}", id);
            }
            Message::DropNode(dragged_nodes, target, position) => {
                println!("Dropped {:?} onto {} at position {:?}", dragged_nodes, target, position);
            }
            Message::ButtonPressed(id) => {
                println!("Button pressed: {}", id);
                eprintln!("🔥 BUTTON ACTUALLY PRESSED: {}", id);
            }
        } */

        match message {
            Message::TreeToggle(node_id, expanded) => {
                println!("Node {:?} toggled to expanded: {}", node_id, expanded);
            }
            Message::TreeDragStart(node_id, position) => {
                println!("Started dragging node {:?} at {:?}", node_id, position);
            }
            Message::TreeDragOver(dragged_id, target_id, position) => {
                println!("Dragging {:?} over {:?} at {:?}", dragged_id, target_id, position);
            }
            Message::TreeDrop(dragged_id, target_id, position) => {
                println!("Dropped {:?} onto {:?} at {:?}", dragged_id, target_id, position);
            }
            Message::ButtonPressed(label) => {
                println!("Button '{}' was pressed!", label);
            }
        }

        Action::None
    }

    pub fn view(&self) -> Element<Message> {

   // Create some content elements with buttons to test event forwarding
    let root_content = row![
        text("Root Node"),
        button("Root Button").on_press(Message::ButtonPressed("root".to_string()))
    ]
    .spacing(5);

    let child1_content = row![
        text("Child 1"),
        button("Child 1 Button").on_press(Message::ButtonPressed("child1".to_string()))
    ]
    .spacing(5);

    let child2_content = row![
        text("Child 2"),
        button("Child 2 Button").on_press(Message::ButtonPressed("child2".to_string()))
    ]
    .spacing(5);

    let grandchild_content = row![
        text("Grandchild"),
        button("Grandchild Button").on_press(Message::ButtonPressed("grandchild".to_string()))
    ]
    .spacing(5);

    // Build the tree structure
    let grandchild = TreeNode::new(NodeId::new(4), grandchild_content);

    let child1 = TreeNode::new(NodeId::new(2), child1_content)
        .with_child(grandchild);

    let child2 = TreeNode::new(NodeId::new(3), child2_content);

    let root = TreeNode::new(NodeId::new(1), root_content)
        .with_children([child1, child2]);

    let tree = Tree::new()
        .with_root(root)
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink)
        .indent_size(30.0)
        .node_height(60.0)
        .on_toggle(Message::TreeToggle)
        .on_drag_start(Message::TreeDragStart)
        .on_drag_over(Message::TreeDragOver)
        .on_drop(Message::TreeDrop);

    container(
            column![
                text("Draggable Tree Example").size(24),
                tree
            ]
            .spacing(20)
        )
        .padding(20)
        .into()


/*         // Build the tree fresh each time
        let mut tree_manager = TreeManager::new();

        // Add root nodes
        tree_manager
            .add(tree_node("root1", || {
                button("Root Node 1").on_press(Message::ButtonPressed("root1".to_string())).into()
            }).accepts_drops())
            .add(tree_node("root2", || {
                container(text("Root Node 2"))
                    .padding(5)
                    .into()
            }).accepts_drops());

        // Add children
        tree_manager
            .add_child("root1", tree_node("child1", || {
                button("Child 1").on_press(Message::ButtonPressed("child1".to_string())).into()
            }))
            .add_child("root1", tree_node("child2", || {
                text("Child 2").into()
            }))
            .add_child("root2", tree_node("child3", || {
                button("Child 3").on_press(Message::ButtonPressed("child3".to_string())).into()
            }).accepts_drops());

        // Add grandchildren
        tree_manager
            .add_child("child3", tree_node("grandchild1", || {
                text("Grandchild 1").into()
            }));

        container(
            tree_manager
                .view()
                .on_toggle(Message::ToggleNode)
                .on_select(Message::SelectNode)
                .on_drop(Message::DropNode)
                .spacing(5.0)
                .indent(25.0)
        )
        .padding(20)
        .into() */
    } 
}

pub enum Action {
    Run(iced::Task<Message>),
    None,
}