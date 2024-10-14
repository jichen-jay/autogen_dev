use autogen_dev::common_types::*;

fn main() {
    println!("Hello, world!");
}

type Fn = fn(&[u8]) -> String;

pub struct Agent {
    pub name: String,
    pub system_message: String,
    pub max_round: i8,
}


impl Agent {
    pub fn message(self) -> Vec<Message> {
        todo!()
    }

    pub fn forward(self, in_msg: Vec<Message>) -> Vec<Message> {
        todo!()
        //when message is to move to the next step, we may want the last message only, need to compress the message history, or discard the history
    }

    pub fn backprop(self, in_msg: Vec<Message>) -> Vec<Message> {
        todo!()
    }
}
