use crate::{engine::Engine, polyfill::Lefs};
use error::Result;
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let address = &args[1];
    let server_address = &args[2];
    let last_cycle = &args[3];
    let last_cycle = last_cycle.parse::<usize>().unwrap();
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, last_cycle, address, server_address)?;
    }

    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    last_cycle: usize,
    address: &str,
    server_address: &str,
) -> Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let message: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if let Some(path) = message.get(0) {
        println!("{path}");
        let lefs = Lefs::new(path)?;
        let mut engine = Engine::new(lefs);
        engine.simulate(0, last_cycle);

        let mut stream = TcpStream::connect(server_address).unwrap();
        let message = format!("{address} processed petri network {path}\n\n");
        stream.write_all(message.as_bytes()).unwrap();
    }

    Ok(())
}

mod engine {
    use crate::polyfill::Lefs;
    use chrono::prelude::*;
    use std::collections::VecDeque;

    #[derive(Debug)]
    pub struct Engine {
        pub cycle: usize,
        pub lefs: Lefs,
        pub events: VecDeque<Event>,
        pub logs: Vec<Log>,
        pub event_count: usize,
    }

    impl Engine {
        pub fn new(lefs: Lefs) -> Self {
            Engine {
                cycle: 0,
                lefs,
                events: VecDeque::new(),
                logs: vec![],
                event_count: 0,
            }
        }

        // SimularPeriodo
        pub fn simulate(&mut self, first_cycle: usize, last_cycle: usize) {
            let start = Utc::now();
            self.cycle = first_cycle;

            for _ in first_cycle..last_cycle {
                println!("RELOJ LOCAL !!!  = {}", self.cycle);
                println!("{}", self.lefs);
                // simularUnpaso
                self.step(last_cycle);
            }

            println!("event_count: {}", self.event_count);
            let elapsed = Utc::now() - start;
            println!(
                "elapsed: {:?} microseconds",
                elapsed.num_nanoseconds().unwrap() / 1000
            );
        }

        // simularUnpaso
        fn step(&mut self, last_cycle: usize) {
            // actualizaSensibilizadas
            self.pep();

            println!("-----------Stack de transiciones sensibilizadas---------");
            println!("{:?}", self.lefs.estimulated_transition_indices);
            println!("-----------Final Stack de transiciones---------");

            while let Some(estimulated_transition_index) =
                self.lefs.estimulated_transition_indices.pop()
            {
                self.fire(estimulated_transition_index);
                self.logs.push(Log {
                    estimulated_transition_index,
                    cycle: self.cycle,
                })
            }

            println!("-----------Lista eventos después de disparos---------");
            println!("Estructura EventList");
            for (i, event) in self.events.iter().enumerate() {
                println!("  Evento -> {i}");
                println!("{event}");
            }
            println!("-----------Final lista eventos---------");

            self.cycle = if let Some(event) = self.events.front() {
                event.cycle
            } else {
                last_cycle
            };
            println!("NEXT CLOCK...... : {}", self.cycle);

            self.aftermath();
        }

        fn pep(&mut self) {
            for (i, transition) in self.lefs.transitions.iter().enumerate() {
                if transition.constant <= 0 && transition.cycle == self.cycle {
                    self.lefs.estimulated_transition_indices.push(i)
                }
            }
        }

        // dispararTransicion
        fn fire(&mut self, estimulated_transition_index: usize) {
            let transition = self.lefs.transitions[estimulated_transition_index].clone();

            for payload in &transition.iul_payloads {
                self.lefs.transitions[payload.transition_index].constant += payload.constant;
            }

            let cycle = transition.cycle + transition.duration;
            for payload in &transition.pul_payloads {
                let event = Event {
                    cycle,
                    transition_index: payload.transition_index,
                    constant: payload.constant,
                };
                self.events.push_front(event);
            }
        }

        fn aftermath(&mut self) {
            while let Some(event) = self.events.pop_front() {
                self.lefs.transitions[event.transition_index].constant += event.constant;
                self.lefs.transitions[event.transition_index].cycle = event.cycle;
                self.event_count += 1;
            }
        }
    }

    #[derive(Debug)]
    pub struct Event {
        pub cycle: usize,
        pub transition_index: usize,
        pub constant: isize,
    }

    #[derive(Debug)]
    pub struct Log {
        pub estimulated_transition_index: usize,
        pub cycle: usize,
    }
}

mod polyfill {
    use crate::{engine::Event, error::Result};
    use std::{fmt::Display, fs::File, io::BufReader};

    #[derive(Debug)]
    pub struct Lefs {
        pub transitions: Vec<Transition>,
        pub estimulated_transition_indices: Vec<usize>,
    }

    impl Lefs {
        pub fn new(path: &str) -> Result<Self> {
            let file = File::open(path)?;
            let rdr = BufReader::new(file);
            let lefs: crate::json::Lefs = serde_json::from_reader(rdr)?;

            let transitions = lefs
                .ia_red
                .into_iter()
                .map(|transition| {
                    let iul_payloads = transition
                        .ii_listactes_iul
                        .iter()
                        .map(|i| Payload {
                            transition_index: i.0,
                            constant: i.1,
                        })
                        .collect();

                    let pul_payloads = transition
                        .ii_listactes_pul
                        .iter()
                        .map(|i| Payload {
                            transition_index: i.0,
                            constant: i.1,
                        })
                        .collect();

                    Transition {
                        id: transition.ii_idglobal,
                        constant: transition.ii_valor,
                        cycle: transition.ii_tiempo,
                        duration: transition.ii_duracion_disparo,
                        iul_payloads,
                        pul_payloads,
                    }
                })
                .collect();

            let lefs = Self {
                transitions,
                estimulated_transition_indices: vec![],
            };

            Ok(lefs)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Transition {
        pub id: usize,
        pub constant: isize,
        pub cycle: usize,
        pub duration: usize,
        // I don't know what iul and pul mean, i stands for immediate
        pub iul_payloads: Vec<Payload>,
        pub pul_payloads: Vec<Payload>,
    }

    #[derive(Debug, Clone)]
    pub struct Payload {
        pub transition_index: usize,
        pub constant: isize,
    }

    impl Display for Lefs {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "STRUCT LEFS")?;
            writeln!(f, "\tNº transiciones: {}", self.transitions.len())?;
            writeln!(f, "------Lista transiciones---------")?;
            for transition in &self.transitions {
                writeln!(f, "{}", transition)?;
            }
            writeln!(f, "------Final lista transiciones---------")?;
            writeln!(f, "FINAL ESTRUCTURA LEFS")
        }
    }

    impl Display for Transition {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Dato Transicion:")?;
            writeln!(f, "IDLOCALTRANSICION: {}", self.id)?;
            writeln!(f, " VALOR LEF: {}", self.constant)?;
            writeln!(f, " TIEMPO: {}", self.cycle)?;
            writeln!(f, " DURACION DISPARO: {}", self.duration)?;
            writeln!(f, " LISTA DE CTES IUL: ")?;
            for payload in &self.iul_payloads {
                writeln!(
                    f,
                    "\tTRANSICION: {}\t\tCTE: {}",
                    payload.transition_index, payload.constant
                )?;
            }
            writeln!(f, " LISTA DE CTES PUL: ")?;
            for payload in &self.pul_payloads {
                writeln!(
                    f,
                    "\tTRANSICION: {}\t\tCTE: {}",
                    payload.transition_index, payload.constant
                )?;
            }
            Ok(())
        }
    }

    impl Display for Event {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "    TIEMPO: {}", self.cycle)?;
            writeln!(f, "    TRANSICION: {}", self.transition_index)?;
            writeln!(f, "    CONSTANTE: {}", self.constant)
        }
    }
}

mod json {

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Lefs {
        pub ia_red: Vec<Transition>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Transition {
        pub ii_idglobal: usize,
        pub ii_valor: isize,
        pub ii_tiempo: usize,
        pub ii_duracion_disparo: usize,

        #[serde(rename = "ii_listactes_IUL")]
        pub ii_listactes_iul: Vec<Payload>,

        #[serde(rename = "ii_listactes_PUL")]
        pub ii_listactes_pul: Vec<Payload>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Payload(pub usize, pub isize);
}
mod error {
    use std::{error::Error, fmt::Display};

    pub type Result<T> = std::result::Result<T, AppError>;

    #[derive(Debug)]
    pub enum AppError {
        Io(std::io::Error),
        SerdeJson(serde_json::Error),
    }

    impl Error for AppError {}

    impl Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Io(error) => write!(f, "{}", error),
                Self::SerdeJson(error) => write!(f, "{}", error),
            }
        }
    }

    impl From<std::io::Error> for AppError {
        fn from(value: std::io::Error) -> Self {
            AppError::Io(value)
        }
    }

    impl From<serde_json::Error> for AppError {
        fn from(value: serde_json::Error) -> Self {
            AppError::SerdeJson(value)
        }
    }
}
