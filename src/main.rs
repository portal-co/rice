fn main() -> std::io::Result<()>{
    let mut args = std::env::args();
    args.next();
    let file = args.next().unwrap();
    Ok(())
}
