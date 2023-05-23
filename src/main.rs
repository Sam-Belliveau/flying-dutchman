mod evaluate;
mod search;
mod tests;
mod transposition;
mod uci;

const TEST: bool = false;

fn main() {
    if TEST {
        tests::play_self::play_self();
    } else {
        uci::interpret::uci_loop();
    }
}
