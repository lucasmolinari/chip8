mod sdl;

pub fn main() -> Result<(), String> {
    sdl::init()?;
    Ok(())
}
