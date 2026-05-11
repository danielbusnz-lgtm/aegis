pub mod claude;

pub trait Llm {
    fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}
