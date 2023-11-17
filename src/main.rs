mod rendering;
use crate::rendering::App;
fn main() {
    match App::new() {
        Ok(_a) => println!("Success! Exiting."),
        Err(e) => {
            println!(
                "Failed to initialize Vulkan!
Error: {e}"
            )
        }
    }
}
