#[allow(async_fn_in_trait)]
#[clipsync_macros::invoke]
pub trait Commands {
    async fn hello(name: String) -> String;
}
