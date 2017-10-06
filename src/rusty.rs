extern crate slack;

use listener::{Message, MessageListener}
use slackhandler::SlackHandler;
use slack::RtmClient;
use std::env;

pub struct Rusty {
    listeners: Vec<Box<Listener>>,
}

impl Rusty {
    pub fn new() -> Rusty {
        Rusty {
            listeners: Vec::new()
        }
    }

    pub fn connect(&self) {
        let token = match env::var("SLACK_BOT_TOKEN") {
            Ok(token) => token,
            Err(_)    => panic!("Failed to get SLACK_BOT_TOKEN from env")
        };;

        let mut handler = SlackHandler::new(|message, cli| {
            self.handle_message(message, cli)
        });

        handler.login_and_run(token);
    }

    fn handle_message(&self, message: &Message, cli: &RtmClient) {
        if message.text == "help" && message.is_addressed {
            let listener_helps = self.listeners.iter()
                .map(|x| x.help())
                .collect::<Vec<_>>()
                .join("\n")

            cli.send_message(&message.channel, &helps);
            return
        }

        for listener in self.listeners.iter() {
            if listener.can_handle(message) {
                listener.handle(message, cli);
                break;
            }
        }
    }

    pub fn add_listener<T>(&mut self, listener: T) -> &mut Rusty
        where T: MessageListener + 'static
    {
        self.listeners.push(Box::new(listener));
        self
    }
}
