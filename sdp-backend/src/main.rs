use rocket_contrib::serve::StaticFiles;

fn main() {
    rocket::ignite()
        .mount("/static", StaticFiles::from("./static"))
        .launch();
}
