use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn tap_tempo_blocking(amount: u8, numerator: u8, denominator: u8) -> io::Result<Option<u16>> {
    ratatui::run(|terminal| run_tap_tempo(terminal, amount, numerator, denominator))
}

fn run_tap_tempo(
    terminal: &mut DefaultTerminal,
    amount: u8,
    numerator: u8,
    denominator: u8,
) -> io::Result<Option<u16>> {
    let mut taps = Vec::new();
    let mut flash_until = None;

    loop {
        let estimated_bpm = estimate_bpm(&taps);
        terminal.draw(|frame| {
            render_tap_ui(
                frame,
                taps.len(),
                amount,
                numerator,
                denominator,
                estimated_bpm,
                flash_until,
            );
        })?;

        if event::poll(Duration::from_millis(16))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Enter if taps.len() >= 2 => return Ok(estimate_bpm(&taps)),
                KeyCode::Char(' ') => {
                    let now = Instant::now();
                    taps.push(now);
                    flash_until = Some(now + Duration::from_millis(150));
                    if taps.len() >= usize::from(amount) {
                        return Ok(estimate_bpm(&taps));
                    }
                }
                _ => {}
            }
        }
    }
}

fn render_tap_ui(
    frame: &mut Frame,
    tap_count: usize,
    amount: u8,
    numerator: u8,
    denominator: u8,
    estimated_bpm: Option<u16>,
    flash_until: Option<Instant>,
) {
    let area = frame.area();
    let width = area.width.min(58);
    let height = area.height.min(7);
    let panel = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    let progress: String = (0..usize::from(amount))
        .map(|index| {
            if index < tap_count.min(usize::from(amount)) {
                '●'
            } else {
                '○'
            }
        })
        .collect();
    let marker_color = if flash_until.is_some_and(|until| Instant::now() < until) {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("<Space>", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(" tap   "),
            Span::styled("<Enter>", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(" accept   "),
            Span::styled("<Esc>", Style::new().add_modifier(Modifier::BOLD)),
            Span::raw(" cancel"),
        ]),
        Line::from(format!(
            "Taps: {}/{}   BPM: {}   Meter: {}/{}",
            tap_count.min(usize::from(amount)),
            amount,
            estimated_bpm.map_or_else(|| "--".to_owned(), |bpm| bpm.to_string()),
            numerator,
            denominator
        ))
        .style(Style::new().fg(Color::Yellow)),
        Line::from(progress).style(Style::new().fg(marker_color)),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(
                Block::new()
                    .title(" Tap tempo ")
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .style(Style::new().fg(Color::Gray)),
        panel,
    );
}

fn estimate_bpm(taps: &[Instant]) -> Option<u16> {
    if taps.len() < 2 {
        return None;
    }

    let total_seconds = taps
        .windows(2)
        .map(|window| (window[1] - window[0]).as_secs_f64())
        .sum::<f64>();
    let average_seconds = total_seconds / (taps.len() - 1) as f64;
    Some((60.0 / average_seconds).round().clamp(20.0, 400.0) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_tap_ui_on_tiny_terminal() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render_tap_ui(frame, 0, 10, 7, 8, None, None))
            .unwrap();
    }

    #[test]
    fn estimates_and_clamps_bpm() {
        let start = Instant::now();
        assert_eq!(estimate_bpm(&[start]), None);
        assert_eq!(
            estimate_bpm(&[start, start + Duration::from_millis(500)]),
            Some(120)
        );
        assert_eq!(
            estimate_bpm(&[start, start + Duration::from_secs(10)]),
            Some(20)
        );
    }
}
