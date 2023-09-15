use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

struct LazyInit {
    resource: Arc<tokio::sync::OnceCell<String>>,
}

impl LazyInit {
    async fn lazy_init() -> String {
        tokio::time::sleep(Duration::from_secs(3)).await;
        "some content".to_string()
    }

    pub fn new() -> Self {
        let cell = Arc::new(tokio::sync::OnceCell::new());
        let cell_clone = cell.clone();

        tokio::spawn(async move {
            cell_clone.get_or_init(LazyInit::lazy_init).await;
        });

        Self { resource: cell }
    }

    pub async fn acquire(&self) -> &String {
        self.resource.get_or_init(LazyInit::lazy_init).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let lazy = LazyInit::new();

    // wait a bit to prove the 3-seconds initialization is concurrently happening
    println!("[{}] waiting one second", chrono::Local::now());
    tokio::time::sleep(Duration::from_secs(1)).await;

    for _ in 0..10 {
        let now = chrono::Local::now();
        println!("[{}] going to acquire", now);
        let result = lazy.acquire().await;
        println!("[{}] acquired '{}' in {}ms", chrono::Local::now(), result, (chrono::Local::now() - now).num_milliseconds());
    }

    Ok(())
}