use backend::auth::Auth;
use backend::{filters, handlers, models};
use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "error,backend=info");
    }

    pretty_env_logger::init();

    let mut settings_config = config::Config::default();
    settings_config
        .merge(config::File::with_name("DevSettings"))
        .unwrap()
        .merge(config::Environment::with_prefix("APP"))
        .unwrap();
    let settings = settings_config.try_into::<models::Settings>().unwrap();

    let auth = Auth::new(settings.clone());

    let api = filters::users(auth.clone())
        .or(filters::tokens(auth.clone()))
        .recover(handlers::rejection);

    let routes = api.with(warp::log("backend"));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
