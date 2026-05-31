/// Console screen state management.
///
/// Manages the G-code console: command input buffer,
/// command history recall, and printer response display.
///
const MAX_HISTORY: usize = 50;

/// State for the G-code console screen.
#[derive(Debug, Clone, Default)]
pub struct ConsoleState {
    pub command_input: String,

    /// Accumulated response text from the printer (most recent at bottom).
    pub response_text: String,

    pub command_history: Vec<String>,

    /// Current position in history recall (`None` = not recalling).
    history_index: Option<usize>,

    /// Snapshot of the in-progress command, restored when cancelling recall.
    pre_recall_buffer: String,
}

impl ConsoleState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append_char(&mut self, ch: &str) {
        self.history_index = None;
        self.command_input.push_str(ch);
    }

    pub fn backspace(&mut self) {
        self.history_index = None;
        self.command_input.pop();
    }

    pub fn clear_input(&mut self) {
        self.history_index = None;
        self.command_input.clear();
    }

    /// Finalize and return the current command for sending.
    ///
    /// Returns `None` if the input is empty. Otherwise pushes the command
    /// into history and clears the input buffer.
    pub fn finalize_command(&mut self) -> Option<String> {
        let cmd = self.command_input.trim().to_string();
        if cmd.is_empty() {
            return None;
        }

        // Deduplicate consecutive identical commands
        if self.command_history.last().map(String::as_str) != Some(&cmd) {
            self.command_history.push(cmd.clone());
            if self.command_history.len() > MAX_HISTORY {
                self.command_history.remove(0);
            }
        }

        self.history_index = None;
        self.pre_recall_buffer.clear();
        self.command_input.clear();
        Some(cmd)
    }

    pub fn recall_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.pre_recall_buffer = self.command_input.clone();
                let idx = self.command_history.len() - 1;
                self.command_input = self.command_history[idx].clone();
                self.history_index = Some(idx);
            }
            Some(0) => {}
            Some(idx) => {
                let new_idx = idx - 1;
                self.command_input = self.command_history[new_idx].clone();
                self.history_index = Some(new_idx);
            }
        }
    }

    /// Recall the next command (history down / arrow down).
    ///
    /// If the user goes past the most recent command, the pre-recall
    /// buffer is restored (effectively cancelling history recall).
    pub fn recall_down(&mut self) {
        match self.history_index {
            None => {
            }
            Some(idx) => {
                let next = idx + 1;
                if next < self.command_history.len() {
                    self.command_input = self.command_history[next].clone();
                    self.history_index = Some(next);
                } else {
                    self.command_input = self.pre_recall_buffer.clone();
                    self.history_index = None;
                }
            }
        }
    }

    pub fn push_response(&mut self, line: &str) {
        if !self.response_text.is_empty() {
            self.response_text.push('\n');
        }
        self.response_text.push_str(line);

        // Trim to last ~50 lines to prevent unbounded growth
        let line_count = self.response_text.lines().count();
        if line_count > 50 {
            let skip = line_count - 50;
            self.response_text = self
                .response_text
                .lines()
                .skip(skip)
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_backspace() {
        let mut cs = ConsoleState::new();
        cs.append_char("G");
        cs.append_char("2");
        cs.append_char("8");
        assert_eq!(cs.command_input, "G28");

        cs.backspace();
        assert_eq!(cs.command_input, "G2");
        cs.backspace();
        assert_eq!(cs.command_input, "G");
        cs.backspace();
        assert_eq!(cs.command_input, "");
        cs.backspace(); // no panic on empty
        assert_eq!(cs.command_input, "");
    }

    #[test]
    fn test_finalize_command() {
        let mut cs = ConsoleState::new();
        assert_eq!(cs.finalize_command(), None);

        cs.append_char("G28");
        let cmd = cs.finalize_command();
        assert_eq!(cmd.as_deref(), Some("G28"));
        assert!(cs.command_input.is_empty());
        assert_eq!(cs.command_history.len(), 1);
    }

    #[test]
    fn test_dedup_consecutive() {
        let mut cs = ConsoleState::new();
        cs.append_char("G28");
        cs.finalize_command();

        cs.append_char("G28");
        cs.finalize_command();

        assert_eq!(cs.command_history.len(), 1);
    }

    #[test]
    fn test_recall_up_down() {
        let mut cs = ConsoleState::new();
        cs.append_char("G28");
        cs.finalize_command();
        cs.append_char("G1 X10");
        cs.finalize_command();
        cs.append_char("M104 S200");
        cs.finalize_command();

        // Start recall
        cs.recall_up();
        assert_eq!(cs.command_input, "M104 S200");

        cs.recall_up();
        assert_eq!(cs.command_input, "G1 X10");

        cs.recall_up();
        assert_eq!(cs.command_input, "G28");

        // Can't go further back
        cs.recall_up();
        assert_eq!(cs.command_input, "G28");

        // Now go forward
        cs.recall_down();
        assert_eq!(cs.command_input, "G1 X10");

        cs.recall_down();
        assert_eq!(cs.command_input, "M104 S200");

        // Past the end: restore pre-recall buffer
        cs.recall_down();
        assert_eq!(cs.command_input, "");
    }

    #[test]
    fn test_recall_then_type_cancels() {
        let mut cs = ConsoleState::new();
        cs.append_char("G28");
        cs.finalize_command();
        cs.append_char("G1 X10");
        cs.finalize_command();

        cs.recall_up();
        assert_eq!(cs.command_input, "G1 X10");

        // Typing cancels recall
        cs.append_char(" F600");
        assert_eq!(cs.command_input, "G1 X10 F600");
        assert!(cs.history_index.is_none());
    }

    #[test]
    fn test_clear_input() {
        let mut cs = ConsoleState::new();
        cs.append_char("G28");
        cs.clear_input();
        assert!(cs.command_input.is_empty());
    }

    #[test]
    fn test_push_response() {
        let mut cs = ConsoleState::new();
        cs.push_response("ok");
        assert_eq!(cs.response_text, "ok");

        cs.push_response("T:200.0 /200.0");
        assert_eq!(cs.response_text, "ok\nT:200.0 /200.0");
    }
}
