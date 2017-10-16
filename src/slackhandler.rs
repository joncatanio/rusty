extern crate slack;

use slack::{Event, RtmClient};
use karma::KarmaManager;

pub struct SlackHandler {
    pub karma_manager: KarmaManager,
}

#[allow(unused_variables)]
impl slack::EventHandler for SlackHandler {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        match event {
            Event::Message(msg) => self.karma_manager.handle_message(cli, msg),
            Event::UserChange { user }
                => self.karma_manager.update_user(cli, &user),
            Event::ReactionAdded { .. }
                => self.karma_manager.handle_reaction(cli, &event),
            _ => ()
        }
    }

    fn on_close(&mut self, cli: &RtmClient) {
        println!("on_close");
    }

    fn on_connect(&mut self, cli: &RtmClient) {
        println!("Function: `on_connect` -> updating users...");
        self.karma_manager.update_users(cli);

        // find the general channel id from the `StartResponse`
        let general_channel_id = cli.start_response()
            .channels
            .as_ref()
            .and_then(|channels| {
                channels
                .iter()
                .find(|chan| match chan.name {
                    None => false,
                    Some(ref name) => name == "public-testing",
                })
            })
            .and_then(|chan| chan.id.as_ref())
            .expect("public-testing channel not found");

        cli.sender().send_message(&general_channel_id, "waking up...").unwrap();
    }
}
