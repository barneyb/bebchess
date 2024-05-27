use std::cmp::Ordering;
use std::io;

use rand::Rng;

fn main() {
    println!("Guess the number (1-100)!");
    let secret_num = rand::thread_rng().gen_range(1..=100);
    loop {
        println!("Enter your guess:");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(e) => {
                println!("um... {e}");
                continue;
            }
        };
        println!("Guess: {guess}");

        match guess.cmp(&secret_num) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too Big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
