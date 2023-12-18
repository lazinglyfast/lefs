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
