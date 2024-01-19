use std::collections::HashMap;
use rf_distributed::{message::Message, mailbox::{Mailbox, Messages}};

pub struct EspMailbox {
    messages: HashMap<i32, Message>,
}

impl EspMailbox {
    pub fn new() -> Self {
        Self {
            messages: Default::default(),
        }
    }
}

impl Default for EspMailbox {
    fn default() -> Self {
        EspMailbox::new()
    }
}

impl Mailbox for EspMailbox {
    fn enqueue(&mut self, msg: Message) {
        self.messages.insert(msg.source, msg);
    }

    fn messages(&mut self) -> Messages {
        self.messages.clone()
    }
}
