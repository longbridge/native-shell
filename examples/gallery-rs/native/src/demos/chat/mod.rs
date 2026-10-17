//! AI Chat: streaming replies, tools, reasoning, attachments, errors (as
//! the TSX gallery's `demos/Chat.tsx`). Its screen is a placeholder until
//! the demo is ported.

use nsh_core::ui::prelude::*;

use crate::ui::IconName;

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Chat;

impl Component for Chat {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        super::placeholder(IconName::Chat, "AI Chat")
    }
}
