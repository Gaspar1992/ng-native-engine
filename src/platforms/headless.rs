use super::PlatformBackend;
use crate::engine::RenderCommand;

/// Headless platform backend designed for automated unit/integration tests and CI pipelines.
/// Records every flushed frame of `RenderCommand`s in memory for assertions.
#[derive(Debug, Default)]
pub struct HeadlessBackend {
    pub frames: Vec<Vec<RenderCommand>>,
}

impl HeadlessBackend {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn total_commands(&self) -> usize {
        self.frames.iter().map(Vec::len).sum()
    }
}

impl PlatformBackend for HeadlessBackend {
    fn backend_name(&self) -> &str {
        "Headless Test Backend"
    }

    fn flush_commands(&mut self, commands: &[RenderCommand]) {
        if !commands.is_empty() {
            self.frames.push(commands.to_vec());
        }
    }
}
