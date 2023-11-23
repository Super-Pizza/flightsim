mod rendering;
use crate::rendering::App;
fn main() {
    match App::new() {
        Ok(mut a) => a.run().unwrap(),
        Err(e) => {
            println!(
                "Failed to initialize Vulkan!
Error: {e}"
            )
        }
    }
}
