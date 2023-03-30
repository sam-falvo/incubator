fn main() {
    println!("Hello from the ROIL compiler.");
}

mod codegen;
mod lexer;
mod parser;
mod symtab;

#[cfg(test)]
mod tests;
