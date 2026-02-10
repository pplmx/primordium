use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct MarketWidget<'a> {
    pub trade_offers: &'a [primordium_net::TradeProposal],
}

impl<'a> Widget for MarketWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let market_block = Block::default()
            .title(" 💹 Multiverse Market ")
            .borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(Color::Cyan));
        let mut lines = Vec::new();
        if self.trade_offers.is_empty() {
            lines.push(ratatui::text::Line::from(" No active trade offers. "));
        } else {
            for (i, offer) in self.trade_offers.iter().enumerate() {
                lines.push(ratatui::text::Line::from(format!(
                    " #{} Offer: {:.0} {:?}",
                    i, offer.offer_amount, offer.offer_resource
                )));
                lines.push(ratatui::text::Line::from(format!(
                    "      Request: {:.0} {:?}",
                    offer.request_amount, offer.request_resource
                )));
            }
        }
        Paragraph::new(lines).block(market_block).render(area, buf);
    }
}
