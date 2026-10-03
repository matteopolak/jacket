use jai_runtime::{Options, RunResult, Script, SourceBundle};
use std::sync::{Mutex, OnceLock};

pub(crate) const INPUT_LIMIT: usize = 4 * 1024 * 1024;
pub(crate) const ARGUMENT_LIMIT: usize = 1024 * 1024;
#[derive(Default)]
pub(crate) struct Bridge {
    pub(crate) sources: SourceBundle,
    pub(crate) source: Vec<u8>,
    pub(crate) name: Vec<u8>,
    pub(crate) argument: Vec<u8>,
    pub(crate) arguments: Vec<String>,
    pub(crate) argument_bytes: usize,
    pub(crate) diagnostic: Vec<u8>,
    pub(crate) result: Option<RunResult>,
}
pub(crate) fn state() -> &'static Mutex<Bridge> {
    static STATE: OnceLock<Mutex<Bridge>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(Bridge::default()))
}
impl Bridge {
    pub(crate) fn fail(&mut self, message: impl ToString) -> u32 {
        self.result = None;
        self.diagnostic = message.to_string().into_bytes();
        1
    }
    pub(crate) fn push(&mut self, channel: u32, byte: u32) -> u32 {
        let Ok(byte) = u8::try_from(byte) else {
            return self.fail("input is not a byte");
        };
        self.result = None;
        let admitted = match channel {
            0 => self
                .sources
                .byte_count()
                .checked_add(self.source.len())
                .is_some_and(|n| n < INPUT_LIMIT),
            1 => self
                .argument_bytes
                .checked_add(self.argument.len())
                .is_some_and(|n| n < ARGUMENT_LIMIT),
            2 => self.name.len() < 4096,
            _ => return self.fail("unknown input channel"),
        };
        if !admitted {
            return self.fail("input byte limit exceeded");
        }
        match channel {
            0 => self.source.push(byte),
            1 => self.argument.push(byte),
            2 => self.name.push(byte),
            _ => unreachable!(),
        }
        0
    }
    pub(crate) fn finish_source(&mut self) -> u32 {
        self.result = None;
        let name = match std::str::from_utf8(&self.name) {
            Ok("") => "main.jai",
            Ok(name) => name,
            Err(_) => return self.fail("source filename must be UTF-8"),
        };
        match self.sources.insert(name, std::mem::take(&mut self.source)) {
            Ok(_) => {
                self.name.clear();
                0
            }
            Err(error) => self.fail(error),
        }
    }
    pub(crate) fn finish_argument(&mut self) -> u32 {
        self.result = None;
        if self.arguments.len() >= 16_384 {
            return self.fail("argument count limit exceeded");
        }
        match String::from_utf8(std::mem::take(&mut self.argument)) {
            Ok(argument) => {
                self.argument_bytes += argument.len();
                self.arguments.push(argument);
                0
            }
            Err(_) => self.fail("arguments must be UTF-8"),
        }
    }
    pub(crate) fn run(&mut self, fuel: u32) -> u32 {
        self.result = None;
        self.diagnostic.clear();
        if !self.source.is_empty() || !self.name.is_empty() || !self.argument.is_empty() {
            return self.fail("finish every source file and argument before running");
        }
        let path = match self.sources.entry_path("main.jai") {
            Ok(path) => path,
            Err(error) => return self.fail(error),
        };
        let options = Options {
            target: jai_runtime::browser_target(),
            limits: jai_runtime::Limits {
                fuel: u64::from(fuel),
                ..Default::default()
            },
            ..Options::default()
        };
        match Script::prepare(&path, &self.sources, options)
            .and_then(|script| script.run(&self.arguments))
        {
            Ok(result) => {
                self.result = Some(result);
                0
            }
            Err(error) => self.fail(error),
        }
    }
}
