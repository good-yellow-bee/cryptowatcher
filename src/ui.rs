use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::{App, CoinData};

const COLORS: [Color; 6] = [
    Color::Yellow,
    Color::Cyan,
    Color::Magenta,
    Color::Green,
    Color::Red,
    Color::Blue,
];

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let visible_coins = app.coins.len().min(area.height as usize / 8).max(1);
    let mut constraints: Vec<Constraint> = (0..visible_coins)
        .map(|_| Constraint::Ratio(1, visible_coins as u32))
        .collect();
    constraints.push(Constraint::Length(3));

    let chunks = Layout::vertical(constraints).split(area);

    for (i, coin) in app.coins.iter().skip(app.scroll_offset).take(visible_coins).enumerate() {
        render_coin_chart(frame, chunks[i], coin, COLORS[i % COLORS.len()]);
    }

    render_status_bar(frame, chunks[visible_coins], app);
}

fn render_coin_chart(frame: &mut Frame, area: Rect, coin: &CoinData, color: Color) {
    let data = coin.history_data();
    let (y_min, y_max) = coin.price_bounds();

    let change_color = if coin.change_24h >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };

    let change_sign = if coin.change_24h >= 0.0 { "+" } else { "" };

    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", coin.display_name),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("${:.2}", coin.price),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}{:.2}%", change_sign, coin.change_24h),
            Style::default().fg(change_color),
        ),
        Span::raw("  "),
        Span::styled(
            format!("H:{:.0} L:{:.0}", coin.high_24h, coin.low_24h),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Vol:{}", format_volume(coin.volume_24h)),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(&data);

    let x_max = coin.price_history.len().max(60) as f64;
    let time_labels = coin.time_labels();
    let x_labels: Vec<Span> = time_labels.iter().map(|s| Span::raw(s.clone())).collect();

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::raw(format!("{:.0}", y_min)),
                    Span::raw(format!("{:.0}", y_max)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let status = Line::from(vec![
        Span::styled(" [q]", Style::default().fg(Color::Yellow)),
        Span::raw("uit "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw("efresh "),
        Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
        Span::raw("scroll"),
        Span::raw("   "),
        Span::styled(
            format!("Updated: {}", app.last_update_str()),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("   "),
        Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
    ]);

    let paragraph = Paragraph::new(status).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}

fn format_volume(vol: f64) -> String {
    if vol >= 1_000_000_000.0 {
        format!("{:.1}B", vol / 1_000_000_000.0)
    } else if vol >= 1_000_000.0 {
        format!("{:.1}M", vol / 1_000_000.0)
    } else if vol >= 1_000.0 {
        format!("{:.1}K", vol / 1_000.0)
    } else {
        format!("{:.0}", vol)
    }
}
