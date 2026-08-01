use crate::host::NextHostBuilder;

#[derive(Default)]
pub struct PendingHost(pub Option<NextHostBuilder>);
