use std::env;
use std::process;

const PACKAGE_NAME: &str = "sshpass";
const PACKAGE_STRING: &str = concat!("sshpass ", env!("CARGO_PKG_VERSION"));
const PASSWORD_PROMPT: &str = "password:";

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("No error (used internally)")]
    NoError,
    #[error("Invalid arguments")]
    InvalidArguments,
    #[error("Conflicting password source options")]
    ConflictingArguments,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PwType {
    Stdin,
    File(String),
    Fd(i32),
    Pass(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Option {
    pub pwtype: PwType,
    pub pwprompt: String,
    pub verbose: u32,
    pub cmd: String,
}

impl Option {
    pub fn get_password(&self) -> String {
        match &self.pwtype {
            PwType::Stdin => String::new(),
            PwType::File(_) => String::new(),
            PwType::Fd(_) => String::new(),
            PwType::Pass(pwd) => pwd.to_owned(),
        }
    }
}

fn show_help() {
    println!(
        "Usage: {PACKAGE_NAME} [-f|-d|-p|-e] [-hV] command parameters
   -f filename   Take password to use from file
   -d number     Use number as file descriptor for getting password
   -p password   Provide password as argument (security unwise)
   -e            Password is passed as env-var \"SSHPASS\"
   With no parameters - password will be taken from stdin

   -P prompt     Which string should sshpass search for to detect a password prompt
   -v            Be verbose about what you're doing
   -h            Show help (this screen)
   -V            Print version information
At most one of -f, -d, -p or -e should be used"
    );
}

pub fn parse_options(argc: usize, argv: &[String]) -> Result<Option, ParseError> {
    let mut args = Option {
        pwtype: PwType::Stdin,
        pwprompt: PASSWORD_PROMPT.to_string(),
        verbose: 0,
        cmd: String::new(),
    };
    let mut optind = 1; // Start after program name

    while optind < argc {
        let arg = &argv[optind];
        if arg.starts_with('-') {
            match arg.as_str() {
                "-f" => {
                    if args.pwtype != PwType::Stdin {
                        eprintln!("Conflicting password source");
                        return Err(ParseError::ConflictingArguments);
                    }
                    optind += 1;
                    if optind < argc {
                        args.pwtype = PwType::File(argv[optind].clone());
                    } else {
                        eprintln!("Missing filename for -f");
                        return Err(ParseError::InvalidArguments);
                    }
                }
                "-d" => {
                    if args.pwtype != PwType::Stdin {
                        eprintln!("Conflicting password source");
                        return Err(ParseError::ConflictingArguments);
                    }
                    optind += 1;
                    if optind < argc {
                        if let Ok(fd) = argv[optind].parse::<i32>() {
                            args.pwtype = PwType::Fd(fd);
                        } else {
                            eprintln!("Invalid file descriptor");
                            return Err(ParseError::InvalidArguments);
                        }
                    } else {
                        eprintln!("Missing file descriptor for -d");
                        return Err(ParseError::InvalidArguments);
                    }
                }
                "-p" => {
                    if args.pwtype != PwType::Stdin {
                        eprintln!("Conflicting password source");
                        return Err(ParseError::ConflictingArguments);
                    }
                    optind += 1;
                    if optind < argc {
                        args.pwtype = PwType::Pass(argv[optind].clone());
                        // In C, the original password in argv is hidden with 'z'.
                        // Rust can't modify argv directly, so we skip that step.
                    } else {
                        eprintln!("Missing password for -p");
                        return Err(ParseError::InvalidArguments);
                    }
                }
                "-e" => {
                    if args.pwtype != PwType::Stdin {
                        eprintln!("Conflicting password source");
                        return Err(ParseError::ConflictingArguments);
                    }
                    if let Ok(password) = env::var("SSHPASS") {
                        args.pwtype = PwType::Pass(password);
                    } else {
                        eprintln!(
                            "sshpass: -e option given but SSHPASS environment variable not set"
                        );
                        return Err(ParseError::InvalidArguments);
                    }
                }
                "-P" => {
                    optind += 1;
                    if optind < argc {
                        args.pwprompt = argv[optind].clone();
                    } else {
                        eprintln!("Missing prompt for -P");
                        return Err(ParseError::InvalidArguments);
                    }
                }
                "-v" => {
                    args.verbose += 1;
                }
                "-h" => {
                    show_help();
                    break;
                }
                "-V" => {
                    println!(
                        "{PACKAGE_STRING}
(C) 2006-2011 Lingnu Open Source Consulting Ltd.
(C) 2015-2016 Shachar Shemesh
This program is free software, and can be distributed under the terms of the GPL
See the COPYING file for more information.

Using \"{PASSWORD_PROMPT}\" as the default password prompt indicator."
                    );
                    process::exit(0);
                }
                _ => {
                    eprintln!("Invalid option: {arg}");
                    return Err(ParseError::InvalidArguments);
                }
            }
        } else {
            // Non-option argument encountered
            break;
        }
        optind += 1;
    }

    args.cmd = argv[optind..].join(" ");
    Ok(args)
}

#[cfg(test)]
mod test {
    use crate::parse_options;

    #[test]
    fn test() {
        for (argc, argv) in [(
            7,
            vec![
                "sshpass.exe",
                "-p",
                "root",
                "ssh",
                "u0_a345@192.168.0.64",
                "-p",
                "8022",
            ],
        )] {
            let v: Vec<_> = argv.iter().map(|s| s.to_string()).collect();
            let opt = parse_options(argc, &v).unwrap();
            let pwd = opt.get_password();
            assert_eq!(pwd, "root");
        }
    }
}
