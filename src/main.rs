mod rendering;
use crate::rendering::App;
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let payload = info.payload();
        let string = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or(payload.downcast_ref::<&'static str>().copied())
            .unwrap_or("");
        println!(
            "Internal error occured! Please send the following info to the devs:
PANIC DUMP
T:{}
t:{}
L:{}:{}:{}
M:{}",
            env!("TARGET"),
            std::thread::current().name().unwrap_or("main"),
            location.file(),
            location.line(),
            location.column(),
            string
        );
    }));
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
