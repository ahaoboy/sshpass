use crate::{AppOption, ParseError, PasswordError};
use expectrl::{check, spawn, stream::stdin::Stdin};
use std::io::stdout;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("Expectrl error: {0}")]
    Expectrl(#[from] expectrl::Error),
    #[error("Expectrl error: {0}")]
    Password(#[from] PasswordError),
}

pub fn sshpass(option: &AppOption) -> Result<(), Error> {
    let command = &option.cmd;
    let password = option.get_password()?;

    let mut ssh = spawn(command).unwrap_or_else(|_| panic!("Unknown command: {command:?}"));

    loop {
        match check!(
            &mut ssh,
            _ = "(yes/no/[fingerprint])" => {
                ssh.send_line("yes")?;
            },
            _ = "password:" => {
                    ssh.send_line(password)?;
                break;
            },
        ) {
            Err(expectrl::Error::Eof) => return Err(Error::Expectrl(expectrl::Error::Eof)),
            result => result?,
        };
    }

    let mut stdin = Stdin::open()?;
    ssh.interact(&mut stdin, stdout())
        .on_idle(|_| Ok(()))
        .spawn()?;

    stdin.close()?;
    Ok(())
}
