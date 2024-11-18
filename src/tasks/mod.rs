use std::sync::Arc;

use crate::context::Context;

mod garygrady_posts;
mod join_leave_notifications;
mod solve_notifications;
mod unlock_announcements;

pub fn start(context: Arc<Context>) {
    tokio::spawn(unlock_announcements::start(Arc::clone(&context)));
    tokio::spawn(solve_notifications::start(Arc::clone(&context)));
    tokio::spawn(join_leave_notifications::start(Arc::clone(&context)));
    tokio::spawn(garygrady_posts::start(Arc::clone(&context)));
}
