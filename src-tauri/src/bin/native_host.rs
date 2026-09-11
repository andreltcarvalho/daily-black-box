use caixa_preta::bridge::{connect_client, read_frame, write_frame, EXTENSION_ID};
use std::io::{self, Write};
fn main() {
    if let Err(error) = run() {
        eprintln!("Caixa Preta: ponte indisponível ({})", error.kind());
        std::process::exit(1);
    }
}
fn run() -> io::Result<()> {
    let origin = std::env::args().nth(1).unwrap_or_default();
    if origin != format!("chrome-extension://{}/", EXTENSION_ID.trim()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Origem não autorizada",
        ));
    }
    let mut pipe = connect_client()?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let frame = read_frame(&mut input)?;
        write_frame(&mut pipe, &frame)?;
        let response = read_frame(&mut pipe)?;
        write_frame(&mut output, &response)?;
        output.flush()?;
    }
}
