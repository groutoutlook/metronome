use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Clone, Copy, Debug)]
pub struct UiState {
    pub bpm: u16,
    pub bar_beats: u8,
    pub denominator: u8,
    pub ticks_per_beat: u8,
    pub beat_in_bar: u8,
    pub tick_in_beat: u8,
    pub playing: bool,
    pub accent: bool,
    pub show_help: bool,
}

pub fn render_ui(frame: &mut Frame, state: UiState) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }

    render_header(frame, area, state);

    if area.height >= 3 {
        let track = Rect::new(
            area.x,
            area.y.saturating_add(2),
            area.width,
            area.height.saturating_sub(3),
        );
        render_track(frame, track, state);
        render_footer(frame, Rect::new(area.x, area.bottom() - 1, area.width, 1));
    }

    if state.show_help {
        render_help(frame, area);
    }
}

fn render_header(frame: &mut Frame, area: Rect, state: UiState) {
    let header = Rect::new(area.x, area.y, area.width, 1);
    let status = if state.playing { "RUN" } else { "PAUSE" };
    let hud = format!(
        "{:>3} BPM  |  {}/{}  |  sub {}  |  {}",
        state.bpm, state.bar_beats, state.denominator, state.ticks_per_beat, status
    );
    let hud_width = hud.len().min(u16::MAX as usize) as u16;

    if area.width > hud_width.saturating_add(11) {
        let title_area = Rect::new(area.x, area.y, area.width - hud_width, 1);
        let hud_area = Rect::new(area.right() - hud_width, area.y, hud_width, 1);
        frame.render_widget(
            Paragraph::new("Metronome")
                .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            title_area,
        );
        frame.render_widget(
            Paragraph::new(hud)
                .alignment(Alignment::Right)
                .style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            hud_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new(hud)
                .alignment(Alignment::Right)
                .style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            header,
        );
    }
}

fn render_track(frame: &mut Frame, area: Rect, state: UiState) {
    if area.is_empty() || state.bar_beats == 0 || state.ticks_per_beat == 0 {
        return;
    }

    let total_ticks = u32::from(state.bar_beats) * u32::from(state.ticks_per_beat);
    let current_tick = u32::from(state.beat_in_bar.saturating_sub(1))
        * u32::from(state.ticks_per_beat)
        + u32::from(state.tick_in_beat);
    let last_column = u32::from(area.width.saturating_sub(1));
    let current_beat = u32::from(state.beat_in_bar.saturating_sub(1));
    let buffer = frame.buffer_mut();

    for row in area.y..area.bottom() {
        for beat in 0..u32::from(state.bar_beats) {
            let left = area.x
                + ((beat * u32::from(state.ticks_per_beat) * last_column) / total_ticks) as u16;
            let right = area.x
                + (((beat + 1) * u32::from(state.ticks_per_beat) * last_column) / total_ticks)
                    as u16;
            let symbol = if beat == current_beat { "=" } else { "-" };
            for column in left.saturating_add(1)..right {
                if let Some(cell) = buffer.cell_mut((column, row)) {
                    cell.set_symbol(symbol);
                    cell.set_style(Style::new().fg(Color::DarkGray));
                }
            }
        }

        for beat in 0..=u32::from(state.bar_beats) {
            let column = area.x
                + ((beat * u32::from(state.ticks_per_beat) * last_column) / total_ticks) as u16;
            if let Some(cell) = buffer.cell_mut((column, row)) {
                cell.set_symbol("|");
                cell.set_style(Style::new().fg(Color::Blue));
            }
        }

        let tick_column = area.x + ((current_tick * last_column) / total_ticks) as u16;
        if let Some(cell) = buffer.cell_mut((tick_column.min(area.right() - 1), row)) {
            let color = if state.accent && state.tick_in_beat == 0 && state.beat_in_bar == 1 {
                Color::Yellow
            } else {
                Color::Cyan
            };
            cell.set_symbol("●");
            cell.set_style(Style::new().fg(color).add_modifier(Modifier::BOLD));
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let left = control_line(&[
        ("<Space>", ": Play/Pause   "),
        ("<q>/<Esc>", ": Quit   "),
        ("<s>", ": Subdivision   "),
        ("<Tab>", ": Signature   "),
        ("<h>", ": Help"),
    ]);
    let right = control_line(&[("<Up>/<Down>", ": +/-1   "), ("<Left>/<Right>", ": +/-5")]);
    let left_width = left.width().min(u16::MAX as usize) as u16;
    let right_width = right.width().min(u16::MAX as usize) as u16;
    let muted = Style::new().fg(Color::DarkGray);

    frame.render_widget(Paragraph::new(left).style(muted), area);
    if area.width >= left_width.saturating_add(right_width).saturating_add(2) {
        frame.render_widget(
            Paragraph::new(right)
                .alignment(Alignment::Right)
                .style(muted),
            area,
        );
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let width = area.width.min(58);
    let height = area.height.min(7);
    let popup = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    let help = vec![
        control_line(&[("<Space>", " Play/Pause   "), ("<q>/<Esc>", " Quit")]),
        control_line(&[("<s>", " Subdivision   "), ("<Tab>", " Signature")]),
        control_line(&[("<Up>/<Down>", " +/-1   "), ("<Left>/<Right>", " +/-5")]),
        control_line(&[("<h>", " Close help")]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(help)
            .block(
                Block::new()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .style(Style::new().fg(Color::Gray)),
        popup,
    );
}

fn control_line<'a>(controls: &[(&'a str, &'a str)]) -> Line<'a> {
    let mut spans = Vec::with_capacity(controls.len() * 2);
    for (key, description) in controls {
        spans.push(Span::styled(
            *key,
            Style::new().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(*description));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn state() -> UiState {
        UiState {
            bpm: 120,
            bar_beats: 4,
            denominator: 4,
            ticks_per_beat: 2,
            beat_in_bar: 1,
            tick_in_beat: 0,
            playing: true,
            accent: true,
            show_help: false,
        }
    }

    #[test]
    fn renders_normal_and_tiny_terminals() {
        for (width, height) in [(80, 24), (1, 1)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render_ui(frame, state())).unwrap();
        }
    }

    #[test]
    fn renders_help_overlay() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = state();
        state.show_help = true;
        terminal.draw(|frame| render_ui(frame, state)).unwrap();
    }
}
