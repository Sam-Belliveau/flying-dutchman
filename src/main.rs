mod evaluate;
mod search;
mod tests;
mod transposition;
mod uci;

fn main() {
    uci::interpret::uci_loop();
}
