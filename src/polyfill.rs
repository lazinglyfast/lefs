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
