/// Pool exit policy
#[derive(Debug, Clone, Copy, Default)]
pub enum PoolExitPolicy {
    ExitOnFirstResponse,
    #[default]
    ExitOnAllResponses,
}

impl PoolExitPolicy {
    pub fn exit_on_first_response(&self) -> bool {
        matches!(self, PoolExitPolicy::ExitOnFirstResponse)
    }

    pub fn exit_on_all_responses(&self) -> bool {
        matches!(self, PoolExitPolicy::ExitOnAllResponses)
    }
}
