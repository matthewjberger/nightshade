pub fn run_on_thread<Task>(task: Task) -> std::io::Result<std::thread::JoinHandle<()>>
where
    Task: std::future::Future<Output = ()> + Send + 'static,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    Ok(std::thread::spawn(move || runtime.block_on(task)))
}
