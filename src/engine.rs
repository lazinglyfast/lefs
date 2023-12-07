use std::collections::VecDeque;

use crate::polyfill::Lefs;
use chrono::prelude::*;

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

        // fire transitions at the current clock cycle
        for payload in &transition.iul_payloads {
            self.lefs.transitions[payload.transition_index].constant += payload.constant;
        }

        // schedule transitions to happen at a future clock cycle
        for payload in &transition.pul_payloads {
            let event = Event {
                cycle: transition.cycle + transition.duration,
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
