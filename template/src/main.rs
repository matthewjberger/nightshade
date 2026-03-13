use nightshade::prelude::*;
use template::Template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Template)?;
    Ok(())
}
