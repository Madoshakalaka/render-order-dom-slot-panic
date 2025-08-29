use frontend::App;
use tracing_subscriber::{filter::Targets, prelude::*};
use tracing_web::MakeWebConsoleWriter;

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .without_time()
        .with_writer(MakeWebConsoleWriter::new())
        .with_filter(Targets::new().with_default(tracing::Level::TRACE));
    let sub = tracing_subscriber::registry().with(fmt_layer);
    sub.init();

    yew::set_event_bubbling(false);
    yew::Renderer::<App>::new().hydrate();
}
