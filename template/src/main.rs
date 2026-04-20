use nightshade::prelude::*;
use template_lib::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template)?;
    Ok(())
}
