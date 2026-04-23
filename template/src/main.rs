use nightshade::prelude::*;
use template_core::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template)?;
    Ok(())
}
