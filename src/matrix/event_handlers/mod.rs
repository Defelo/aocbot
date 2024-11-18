use std::sync::Arc;

use matrix_sdk::Client;

use crate::context::Context;

mod on_invite;
mod on_member;
mod on_message;

pub fn add_event_handlers(client: &Client, context: Arc<Context>) {
    client.add_event_handler_context(context);
    client.add_event_handler(on_invite::handle);
    client.add_event_handler(on_member::handle);
    client.add_event_handler(on_message::handle);
}
