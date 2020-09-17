use std::{
    io::Read
};

use crate::db::core::{
    GameState,
    Dbref,
    Timestamp,
    Money,
    AttributeFlag,
    Attribute,
    AttributeManager,
    FunctionRestriction,
    FunctionAction,
    Function,
    FunctionManager,
    CommandFlag,
    CommandAction,
    CommandHook,
    Command,
    CommandManager,
    FlagPerm,
    Flag,
    FlagManager,
    LockFlag,
    LockType,
    Lock,
    ObjType,
    ObjAttr,
    Obj
};
use std::io::BufRead;

enum NodeValue {
    None,
    Text(String),
    Db(Dbref),
    Number(isize)
}

struct LoadNode {
    pub name: String,
    pub value: NodeValue,
    pub children: Vec<LoadNode>
}

impl Default for LoadNode {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            value: NodeValue::None,
            children: Default::default()
        }
    }
}

fn convert_nodes(node: LoadNode) -> GameState {
    let mut state = GameState::default();

    state
}

enum ParserState {

}

fn read_nodes(node: &mut LoadNode, data: &mut String, idx: usize) {
    let new_idx = idx;
    let (unused, new_data) = data.split_at(idx);
}

pub fn read_data(mut data: impl Read) -> GameState {
    let mut read_data = String::new();
    let length = data.read_to_string(&mut read_data);

    let mut root_node = LoadNode::default();

    read_nodes(&mut root_node, &mut read_data, 0);

    let mut state = convert_nodes(root_node);
    state
}