use crate::Option;
use expectrl::{check, spawn, stream::stdin::Stdin};
use std::io::stdout;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Expectrl error: {0}")]
    Expectrl(#[from] expectrl::Error),
}

pub fn sshpass(option: &Option) -> Result<(), Error> {
    let command = &option.cmd;
    let password = option.get_password();

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
        .on_idle(|_state| {
            #[cfg(not(target_os = "windows"))]
            {
                let (rows, cols) = _term.size();
                _state
                    .session
                    .set_window_size(cols, rows)
                    .expect("Update window size failed");
            }
            Ok(())
        })
        .spawn()?;

    stdin.close()?;
    Ok(())
}
