use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressPhase {
    Validating,
    Planning,
    Running,
    SavingResults,
    GeneratingReports,
    Completed,
}

impl ProgressPhase {
    pub fn label(self) -> &'static str {
        match self {
            ProgressPhase::Validating => "Validating",
            ProgressPhase::Planning => "Planning",
            ProgressPhase::Running => "Running",
            ProgressPhase::SavingResults => "Saving results",
            ProgressPhase::GeneratingReports => "Generating reports",
            ProgressPhase::Completed => "Completed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressEventKind {
    Phase,
    StepStarted,
    StepCompleted,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressUpdate {
    pub kind: ProgressEventKind,
    pub phase: ProgressPhase,
    pub message: String,
    pub completed_units: u32,
    pub total_units: u32,
    pub model_name: Option<String>,
    pub model_index: Option<usize>,
    pub total_models: Option<usize>,
    pub benchmark_id: Option<String>,
    pub benchmark_name: Option<String>,
    pub benchmark_index: Option<usize>,
    pub total_benchmarks: Option<usize>,
    pub step_index: Option<u32>,
    pub total_steps: Option<u32>,
    pub run_index: Option<u32>,
    pub prompt_name: Option<String>,
}

impl ProgressUpdate {
    pub fn percent_complete(&self) -> u32 {
        if self.total_units == 0 {
            return 100;
        }
        ((self.completed_units as f64 / self.total_units as f64) * 100.0).round() as u32
    }
}

pub trait ProgressSink {
    fn on_update(&mut self, update: ProgressUpdate);
}

#[derive(Debug, Default)]
pub struct NullProgressSink;

impl ProgressSink for NullProgressSink {
    fn on_update(&mut self, _update: ProgressUpdate) {}
}

pub struct TerminalProgressRenderer {
    interactive: bool,
    stream: Box<dyn Write + Send>,
    last_interactive_len: usize,
}

impl TerminalProgressRenderer {
    pub fn new() -> Self {
        let interactive = io::stderr().is_terminal();
        Self::with_stream(interactive, Box::new(io::stderr()))
    }

    pub fn with_stream(interactive: bool, stream: Box<dyn Write + Send>) -> Self {
        Self {
            interactive,
            stream,
            last_interactive_len: 0,
        }
    }

    fn render_bar(percent: u32) -> String {
        let width = 24usize;
        let filled = ((percent.min(100) as usize) * width) / 100;
        let mut bar = String::with_capacity(width + 2);
        bar.push('[');
        for index in 0..width {
            if index < filled {
                bar.push('=');
            } else if index == filled && percent < 100 {
                bar.push('>');
            } else {
                bar.push(' ');
            }
        }
        bar.push(']');
        bar
    }

    fn format_update(update: &ProgressUpdate) -> String {
        let percent = update.percent_complete();
        let mut line = String::new();
        let _ = write!(
            line,
            "{} {:>3}% {}",
            Self::render_bar(percent),
            percent,
            update.phase.label()
        );

        if !update.message.is_empty() {
            let _ = write!(line, " | {}", update.message);
        }

        if let (Some(model_index), Some(total_models), Some(model_name)) = (
            update.model_index,
            update.total_models,
            update.model_name.as_deref(),
        ) {
            let _ = write!(line, " | Model {model_index}/{total_models}: {model_name}");
        }

        if let (Some(benchmark_index), Some(total_benchmarks), Some(benchmark_name)) = (
            update.benchmark_index,
            update.total_benchmarks,
            update.benchmark_name.as_deref(),
        ) {
            let _ = write!(
                line,
                " | Benchmark {benchmark_index}/{total_benchmarks}: {benchmark_name}"
            );
        }

        if let (Some(step_index), Some(total_steps)) = (update.step_index, update.total_steps) {
            let _ = write!(line, " | Step {step_index}/{total_steps}");
        }

        if let Some(run_index) = update.run_index {
            let _ = write!(line, " | Run {run_index}");
        }

        if let Some(prompt_name) = update.prompt_name.as_deref() {
            let _ = write!(line, " | Prompt {prompt_name}");
        }

        line
    }

    fn write_interactive(&mut self, line: &str, finished: bool) {
        let padding = self.last_interactive_len.saturating_sub(line.len());
        let _ = write!(self.stream, "\r\x1b[2K{}{}", line, " ".repeat(padding));
        if finished {
            let _ = writeln!(self.stream);
            self.last_interactive_len = 0;
        } else {
            let _ = self.stream.flush();
            self.last_interactive_len = line.len();
        }
    }
}

impl Default for TerminalProgressRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressSink for TerminalProgressRenderer {
    fn on_update(&mut self, update: ProgressUpdate) {
        let line = Self::format_update(&update);
        if self.interactive {
            let finished = matches!(update.kind, ProgressEventKind::Finished);
            self.write_interactive(&line, finished);
            return;
        }

        match update.kind {
            ProgressEventKind::Phase
            | ProgressEventKind::StepStarted
            | ProgressEventKind::Finished => {
                let _ = writeln!(self.stream, "{line}");
            }
            ProgressEventKind::StepCompleted => {}
        }
        let _ = self.stream.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::{ProgressEventKind, ProgressPhase, ProgressUpdate, TerminalProgressRenderer};

    #[test]
    fn format_update_includes_context_fields() {
        let update = ProgressUpdate {
            kind: ProgressEventKind::StepStarted,
            phase: ProgressPhase::Running,
            message: "Issuing request".to_string(),
            completed_units: 3,
            total_units: 12,
            model_name: Some("qwen3.5:9b".to_string()),
            model_index: Some(1),
            total_models: Some(2),
            benchmark_id: Some("chat-generation".to_string()),
            benchmark_name: Some("Basic generation latency".to_string()),
            benchmark_index: Some(2),
            total_benchmarks: Some(7),
            step_index: Some(1),
            total_steps: Some(3),
            run_index: Some(1),
            prompt_name: Some("short".to_string()),
        };

        let line = TerminalProgressRenderer::format_update(&update);
        assert!(line.contains("25%"));
        assert!(line.contains("Running"));
        assert!(line.contains("Model 1/2: qwen3.5:9b"));
        assert!(line.contains("Benchmark 2/7: Basic generation latency"));
        assert!(line.contains("Step 1/3"));
        assert!(line.contains("Run 1"));
        assert!(line.contains("Prompt short"));
    }

    #[test]
    fn percent_complete_handles_zero_total_units() {
        let update = ProgressUpdate {
            kind: ProgressEventKind::Finished,
            phase: ProgressPhase::Completed,
            message: "Done".to_string(),
            completed_units: 0,
            total_units: 0,
            model_name: None,
            model_index: None,
            total_models: None,
            benchmark_id: None,
            benchmark_name: None,
            benchmark_index: None,
            total_benchmarks: None,
            step_index: None,
            total_steps: None,
            run_index: None,
            prompt_name: None,
        };

        assert_eq!(update.percent_complete(), 100);
    }
}
