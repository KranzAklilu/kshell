use std::{
    io::{self, BufRead},
    process::Command,
};

struct Cmd {
    binary: String,
    arguments: Vec<String>,
}

impl Cmd {
    fn new(command: &str) -> Self {
        let mut cmd_split = command.split(" ");
        let Some(binary) = cmd_split.next() else {
            return Cmd {
                binary: "".to_string(),
                arguments: vec![],
            };
        };
        let Some(arguments) = Some(
            cmd_split
                .map(|cmd| cmd.to_string().replace("\n", ""))
                .collect::<Vec<String>>(),
        ) else {
            return Cmd {
                binary: binary.to_string(),
                arguments: vec![],
            };
        };

        Cmd {
            binary: binary.to_string(),
            arguments,
        }
    }

    fn split_or(line: &str) -> Vec<String> {
        line.split("||").map(|s| s.trim().to_string()).collect()
    }

    fn split_and(line: &str) -> Vec<String> {
        line.split("&&").map(|s| s.trim().to_string()).collect()
    }

    fn read_line(mut reader: impl BufRead, line_stack: &mut Vec<String>) -> String {
        let mut buffer = String::new();

        if line_stack.len() > 0 {
            return line_stack.remove(0);
        }

        let _ = reader.read_line(&mut buffer);

        let lines: Vec<&str> = buffer.split(";").collect();

        if lines.len() > 1 {
            for l in lines.clone() {
                *line_stack = lines
                    .into_iter()
                    .skip(1)
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
                return l.to_string();
            }
        }
        lines.get(0).unwrap().to_string()
    }

    fn exec(&self) -> bool {
        match Command::new(&self.binary).args(&self.arguments).spawn() {
            Ok(mut child) => {
                let ecode = child.wait().expect("command wasn't running");

                return ecode.success();
            }
            Err(e) => {
                eprintln!("{:?}", e);
                false
            }
        }
    }
}

fn main() {
    let mut line_stack = vec![];

    loop {
        let mut prev_and_output = true;

        let line = Cmd::read_line(io::stdin().lock(), &mut line_stack);
        let line_and_split = Cmd::split_and(&line);

        for la in line_and_split {
            if prev_and_output {
                let mut prev_or_outputs = vec![];

                let line_or_split = Cmd::split_or(&la);
                for lo in line_or_split {
                    let command = Cmd::new(&lo);

                    let status = command.exec();

                    prev_or_outputs.push(status);
                    if status {
                        break;
                    }
                }
                if prev_or_outputs.iter().find(|o| **o).is_some() {
                    prev_and_output = true;
                } else {
                    prev_and_output = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_line() {
        let mut line_stack = vec![];
        let fake_cmd = b"ls -ls";
        let line = Cmd::read_line(&fake_cmd[..], &mut line_stack);
        assert_eq!("ls -ls", line);
        assert_eq!(line_stack, Vec::new() as Vec<String>);
    }

    #[test]
    fn read_semi_separated() {
        let mut line_stack = vec![];
        let fake_cmd = b"echo 1; echo 2";
        Cmd::read_line(&fake_cmd[..], &mut line_stack);
        assert_eq!(line_stack, vec!("echo 2"));
    }

    #[test]
    fn split_and() {
        let fake_cmd = "echo 1 && echo 2";
        let lines = Cmd::split_and(fake_cmd);
        assert_eq!(lines, vec!("echo 1", "echo 2"));
    }

    #[test]
    fn split_or() {
        let fake_cmd = "echo 1 || echo 2";
        let lines = Cmd::split_or(fake_cmd);
        assert_eq!(lines, vec!("echo 1", "echo 2"));
    }
}
