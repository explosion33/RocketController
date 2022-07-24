use crate::igniter::Igniter;
mod igniter;

const fire: u8 = 23;
const cont: u8 = 24;

fn main() {
    let mut main_ign = Igniter::new(fire, cont);
    println!("{}", main_ign.has_continuity());
    main_ign.fire_async().join();
}
