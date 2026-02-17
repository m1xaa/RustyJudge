use rusty_judge::run_lsp_server;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    run_lsp_server().await;
}
