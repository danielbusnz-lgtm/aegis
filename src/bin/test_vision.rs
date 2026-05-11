#[path = "../screenshot.rs"]
mod screenshot;

#[path = "../providers/mod.rs"]
mod providers;

fn main() {
    let claude = providers::claude::Claude::from_env().expect("missing ANTHROPIC_API_KEY");

    let (b64, w, h) = match screenshot::capture_active_workspace() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("capture failed: {}", e);
            return;
        }
    };
    println!("captured {}x{}, asking Claude...", w, h);

    match claude.ask_with_image("What's on this screen? Answer in one sentence.", &b64) {
        Ok(text) => println!("Claude: {}", text),
        Err(e) => eprintln!("Claude error: {}", e),
    }
}
