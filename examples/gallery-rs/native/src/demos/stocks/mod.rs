//! Stocks: a live watchlist and a stock (as the TSX gallery's
//! `demos/Stocks.tsx`): candles, depth, the tape, a tick every 50 ms. Its
//! screen is a placeholder until the demo is ported.

use nsh_core::ui::prelude::*;

use crate::ui::IconName;

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Stocks;

impl Component for Stocks {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        super::placeholder(IconName::Stocks, "Stocks")
    }
}
