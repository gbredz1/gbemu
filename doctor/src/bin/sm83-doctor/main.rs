use clap::Parser;
use colored::Colorize;
use gbemu_core::{BusIO, Cpu, InterruptBus, TestBus};
use log::{debug, error, info};
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug)]
struct Args {
    json_path: String,
    #[arg(short = 'c', long)]
    continue_on_failure: bool,
}
fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::builder().init();

    let args = Args::parse();
    debug!("{:?}", args);

    let file = File::open(args.json_path)?;
    let tests: Vec<JsonTest> = serde_json::from_reader(file)?;
    let mut all_success = true;
    let mut errors: Vec<String> = Vec::new();

    let mut cpu = Cpu::default();
    let mut bus = TestBus::default();

    for test in tests {
        cpu.reset();
        bus.set_interrupt_flag_u8(0x00);
        bus.set_interrupt_flag_u8(0x00);

        cpu.load_state(&test.initial);
        bus.load_state(&test.initial);

        cpu.fetch_instruction(&mut bus)?;
        for (pc, sp, msg) in test.cycles.iter() {
            debug!("  @cycle: {:04X} {:04X} {}", pc, sp, msg);
        }

        let mut state = State::default_with_ram(&test.r#final.ram);
        cpu.write_state(&mut state);
        bus.write_state(&mut state);

        let success = State::assert_eq(&state, &test.r#final, "Final state not equal to expected");
        all_success &= success;

        info!(
            "{} {} : {}",
            "test:".purple(),
            test.name,
            if success { "passed".green() } else { "failed".red() }
        );

        if !success {
            errors.push(test.name);

            if !args.continue_on_failure {
                break;
            }
        }
    }

    match all_success {
        true => Ok(()),
        false => Err(errors.join(", ").into()),
    }
}

trait JsonState {
    fn load_state(&mut self, state: &State);

    fn write_state(&self, state: &mut State);
}

impl JsonState for Cpu {
    fn load_state(&mut self, state: &State) {
        self.set_pc(state.pc);
        self.set_sp(state.sp);
        self.set_a(state.a);
        self.set_b(state.b);
        self.set_c(state.c);
        self.set_d(state.d);
        self.set_e(state.e);
        self.set_f(state.f);
        self.set_h(state.h);
        self.set_l(state.l);
        self.set_ime(state.ime == 1);
    }

    fn write_state(&self, state: &mut State) {
        state.pc = self.pc();
        state.sp = self.sp();
        state.a = self.a();
        state.b = self.b();
        state.c = self.c();
        state.d = self.d();
        state.e = self.e();
        state.f = self.f();
        state.h = self.h();
        state.l = self.l();
        state.ime = self.ime() as u8;
    }
}

impl JsonState for TestBus {
    fn load_state(&mut self, state: &State) {
        for ram in state.ram.iter() {
            self.write_internal_byte(ram.addr, ram.val);
        }
    }

    fn write_state(&self, state: &mut State) {
        for ram in state.ram.iter_mut() {
            ram.val = self.read_byte(ram.addr);
        }
    }
}

#[derive(Debug, Deserialize)]
struct JsonTest {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<(u16, u16, String)>,
}

#[derive(Debug, Deserialize, Default, PartialEq)]
struct State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    ime: u8,
    // ie: u8,
    ram: Vec<RamState>,
}

impl State {
    fn default_with_ram(ram: &[RamState]) -> Self {
        State {
            ram: ram
                .iter()
                .map(move |x| RamState {
                    addr: x.addr,
                    val: 0xFF,
                })
                .collect(),
            ..Default::default()
        }
    }

    fn assert_eq(actual: &State, expected: &State, message: impl Display) -> bool {
        let result = actual == expected;
        if actual != expected {
            error!("{}:", message);
            error!(" {}", actual);
            error!(" {}", expected);
        }

        result
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pc: {:04X}, sp: {:04X}, a: {:02X}, b: {:02X}, c: {:02X}, d: {:02X}, e: {:02X}, f: {:02X}, h: {:02X}, l: {:02X}, ime: {:02X}, ram:[{}]",
            self.pc,
            self.sp,
            self.a,
            self.b,
            self.c,
            self.d,
            self.e,
            self.f,
            self.h,
            self.l,
            self.ime,
            self.ram.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
        )
    }
}

#[derive(Debug, Deserialize, Default, PartialEq)]
struct RamState {
    addr: u16,
    val: u8,
}

impl Display for RamState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:04X}: {:02X}", self.addr, self.val)
    }
}
