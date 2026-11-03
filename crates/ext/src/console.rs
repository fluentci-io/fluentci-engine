//! Output-only terminal rendering. No keyboard events or kqueue dependency.
use std::{
    collections::VecDeque,
    env,
    io::{self, IsTerminal, Write},
    time::Instant,
};

use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    queue,
    terminal::{Clear, ClearType},
};
use unicode_width::UnicodeWidthChar;

pub(crate) struct Console {
    command: String,
    lines: VecDeque<String>,
    started: Instant,
    rows: u16,
    hidden: bool,
}

impl Console {
    pub(crate) fn new(command: &str) -> Option<Self> {
        enabled(
            io::stdout().is_terminal(),
            &env::var("TERM").unwrap_or_default(),
            &env::var("FLUENTCI_CONSOLE").unwrap_or_default(),
        )
        .then(|| Self {
            command: command.into(),
            lines: VecDeque::new(),
            started: Instant::now(),
            rows: 0,
            hidden: false,
        })
    }

    pub(crate) fn line(&mut self, line: &str) -> io::Result<()> {
        self.lines.push_back(line.into());
        while self.lines.len() > 20 {
            self.lines.pop_front();
        }
        self.draw(false)
    }

    pub(crate) fn finish(&mut self) -> io::Result<()> {
        self.draw(true)
    }

    fn draw(&mut self, finished: bool) -> io::Result<()> {
        let (width, height) = crossterm::terminal::size().unwrap_or((100, 30));
        let width = usize::from(width.saturating_sub(1)).max(1);
        let capacity = usize::from(height.saturating_sub(2));
        let mut frame: Vec<String> = if finished {
            Vec::new()
        } else {
            self.lines
                .iter()
                .skip(self.lines.len().saturating_sub(capacity))
                .map(|line| fit_line(line, width))
                .collect()
        };
        frame.push(fit_line(
            &format!(
                "   {} {} in {:.1}s",
                if finished { "Finished" } else { "Executing" },
                self.command,
                self.started.elapsed().as_secs_f64(),
            ),
            width,
        ));
        let mut output = io::stdout().lock();
        if !self.hidden {
            self.hidden = true;
            queue!(output, Hide)?;
        }
        redraw(&mut output, self.rows, &frame)?;
        self.rows = frame.len() as u16;
        if finished {
            queue!(output, Show)?;
            self.rows = 0;
        }
        output.flush()?;
        if finished {
            self.hidden = false;
        }
        Ok(())
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        if self.hidden {
            let mut output = io::stdout().lock();
            let _ = queue!(output, Show);
            let _ = output.flush();
        }
    }
}

fn enabled(terminal: bool, term: &str, console: &str) -> bool {
    terminal && term != "dumb" && console != "0"
}

fn redraw(output: &mut impl Write, rows: u16, frame: &[String]) -> io::Result<()> {
    if rows != 0 {
        queue!(output, MoveUp(rows))?;
    }
    queue!(output, MoveToColumn(0), Clear(ClearType::FromCursorDown))?;
    for line in frame {
        write!(output, "{line}\r\n")?;
    }
    Ok(())
}

// Keep each rendered line within one physical row, including wide Unicode.
// Child output cannot inject terminal commands or create extra physical rows.
fn fit_line(value: &str, limit: usize) -> String {
    let mut width = 0;
    value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take_while(|c| {
            width += c.width().unwrap_or(0);
            width <= limit
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tty_detection_respects_pipes_dumb_terminals_and_explicit_opt_out() {
        assert!(enabled(true, "xterm-256color", ""));
        assert!(!enabled(false, "xterm-256color", "1"));
        assert!(!enabled(true, "dumb", "1"));
        assert!(!enabled(true, "xterm-256color", "0"));
    }

    #[test]
    fn frames_fit_unicode_and_cannot_inject_extra_rows_or_commands() {
        assert_eq!(fit_line("界ab", 3), "界a");
        assert_eq!(fit_line("e\u{301}x", 1), "e\u{301}");
        assert_eq!(fit_line("hello\nworld\r\x1b[2J", 100), "hello world  [2J");
        let mut output = Vec::new();
        redraw(&mut output, 2, &[fit_line("界ab", 3)]).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\x1b[2A\x1b[1G\x1b[J界a\r\n"
        );
    }
}
