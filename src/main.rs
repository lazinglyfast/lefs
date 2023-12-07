mod engine;
mod error;
mod json;
mod polyfill;

use crate::{engine::Engine, polyfill::Lefs};
use error::Result;

fn main() -> Result<()> {
    // todo: clap user input
    let last_cycle = 3;
    let path = r"/home/diogo/gounizar/redes/Practica2/MiniproyectoSD/CodigoSuministradoAAlumnos/simuladores/cmd/censim/testdata/Ejemplo1ParaTests.rdp.subred0.json";

    let lefs = Lefs::new(path)?;
    let mut engine = Engine::new(lefs);
    engine.simulate(0, last_cycle);

    Ok(())
}
