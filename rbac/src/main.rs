#[macro_use] extern crate diesel;
extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate regex;
extern crate evalexpr;

extern crate log;
extern crate env_logger;

#[macro_use] extern crate serde_derive;

mod plexrbac;


fn main() {
    plexrbac::top();
}


