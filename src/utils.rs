use std;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use dtw;
use rand::Rng;
use rand::thread_rng;
use fast5wrapper;
pub fn readsam(sam:&str)->std::io::Result<HashMap<String,usize>>{
    // read given (right) samfile and return HashMap that key is read id,value is its alignment location.
    Ok(BufReader::new(std::fs::File::open(Path::new(sam))?).lines().
        filter_map(|e| e.ok()).
        map(|line| {
            let contents :Vec<_>= line.split(' ').collect();
            (contents[0].to_string(),contents[1].parse().unwrap())}).
        collect())
}

#[derive(Debug,Clone,Copy)]
pub enum Method{
    ChibaNormal,
    ChibaHill,
    SubNormal,
    SubHill,
}
impl Method{
    pub fn new(method:&str)->Option<Method>{
        match method{
            "CHIBANORMAL" => Some(Method::ChibaNormal),
            "CHIBAHILL" => Some(Method::ChibaHill),
            "SUBNORMAL" => Some(Method::SubNormal),
            "SUBHILL" => Some(Method::SubHill),
            _ => {println!("{}",method);None},
        }
    }
    pub fn name(&self) -> &str{
        match self{
            &Method::ChibaHill => "ChibaHill",
            &Method::ChibaNormal => "ChibaNormal",
            &Method::SubHill => "SubHill",
            &Method::SubNormal => "SubNormal",
        }
    }
}
#[derive(Debug,Clone,Copy)]
pub enum Prep{
    Flat,
    Normal
}
impl Prep{
    pub fn new(prep:&str) -> Option<Prep>{
        match prep{
            "FLAT" => Some(Prep::Flat),
            "NORMAL" => Some(Prep::Normal),
            _ => None,
        }
    }
    pub fn name(&self) -> &str{
        match self{
            &Prep::Flat => "Flat",
            &Prep::Normal => "Normal",
        }
    }
}

pub fn set_queries(filename:&str)->std::io::Result<(HashMap<usize,Vec<(String,usize)>>,
                                                    HashMap<usize,usize>)>{
    let mut queries = HashMap::new();
    let mut startpositions = HashMap::new();
    let lines :Vec<String> = BufReader::new(std::fs::File::open(Path::new(filename))?).lines().
        filter_map(|e| e.ok()).collect();
    let mut querysize :usize = 0;
    for line in lines{
        if line.starts_with('>'){
            //start new bucket
            let line :Vec<_>= line.trim_matches('>').split(',').collect();
            querysize = line[0].parse().unwrap();
            let startposition:usize = line[1].parse().unwrap();
            startpositions.insert(querysize,startposition);
            queries.insert(querysize,vec![]);
        }else{
            let line :Vec<_> = line.split(',').collect();
            let querypath = line[0];
            let location:usize = line[1].parse().unwrap();
            if let Some(vec) = queries.get_mut(&querysize){
                vec.push((querypath.to_string(),location));
            }
        }
    }
    Ok((queries,startpositions))
}
#[inline]
fn hill(x:&f32,y:&f32) -> f32{
    let d = (x-y).powi(8);
    d/(0.01 + d)
}
#[inline]
fn normal(x:&f32,y:&f32) -> f32{
    (x-y).powi(2)
}
#[inline]
fn padding(e:&f32)->Vec<f32>{
    let mut res = vec![e.clone()];
    let continue_prob = 0.84/1.84;
    let mut rng = thread_rng();
    while rng.gen_range(0.,1.) < continue_prob{
        res.push(e.clone());
    }
    res
}
#[inline]
pub fn padding_reference(reference:&[f32]) -> Vec<f32>{
    reference.iter().map(|e| padding(e)).fold(vec![],|mut acc,mut x|{acc.append(&mut x);acc})
}
fn prepare_query(filename:&str,querysize:usize,prep:&Prep,cdf:&Vec<f32>) -> Option<Vec<f32>>{
    let query :Vec<_>= match fast5wrapper::get_event_for(filename,100,querysize){
        Ok(res) => dtw::normalize(&res.iter().map(|e|e[2]).collect(),dtw::NormalizeType::Z),
        Err(_) => return None,
    };
    match prep {
        &Prep::Flat => Some(dtw::histgram_modify(&query,cdf)),
        &Prep::Normal => Some(query),
    }
}


pub fn optimal_dtw(filename:&str,reference:&Vec<f32>,method:&Method,querysize:usize,
                   start:usize,refsize:usize,prep:&Prep,cdf:&Vec<f32>)
                   ->Option<(String,f32,usize)>{
    let id = match fast5wrapper::get_read_id(filename){
        Ok(res) => res,
        Err(_) => return None,
    };
    let query = match prepare_query(filename,querysize,prep,cdf){
        Some(res) => res,
        None => return None,
    };
    let (score,_,location) = match method{
        &Method::SubHill => dtw::dtw(&query,&reference[start..start+refsize],
                                            dtw::Mode::Sub,&hill).unwrap(),
        &Method::SubNormal => dtw::dtw(&query,&reference[start..start+refsize],
                                            dtw::Mode::Sub,&normal).unwrap(),
        &Method::ChibaHill | &Method::ChibaNormal =>{
            let mut result = None;
            let mut opt = 1000000.;
            let mut offset = 0;
            let bandwidth = if querysize / 10 % 2 == 0 {querysize/10 + 1}else{querysize/10};
            while offset+querysize < refsize {
                let subref = &reference[start+offset..start+offset+querysize];
                let subref = &padding_reference(subref)[0..querysize];
                let (score,t,sub_start) = match method {
                    &Method::ChibaHill => dtw::dtw(&query,
                                                          subref,
                                                          dtw::Mode::SakoeChiba(bandwidth),&hill).unwrap(),
                    &Method::ChibaNormal => dtw::dtw(&query,
                                                            subref,
                                                            dtw::Mode::SakoeChiba(bandwidth),&normal).unwrap(),
                    _ => unreachable!(),
                };
                if opt > score {
                    result = Some((score,t,offset+sub_start));
                    opt = score;
                    offset += bandwidth;
                }else{
                    offset += querysize/5;
                }
            };
            match result{
                Some(res) => res,
                None => unreachable!("no!!!"),
            }
        }
    };
    Some((id,score,start+location))
}

pub fn random_dtw(filename:&str,reference:&Vec<f32>,method:&Method,querysize:usize,
                  refsize:usize,times:usize,prep:&Prep,cdf:&Vec<f32>)
                  -> Option<Vec<(String,f32)>>{
    let id = match fast5wrapper::get_read_id(filename){
        Ok(res) => res,
        Err(_) => return None,
    };
    let query = match prepare_query(filename,querysize,prep,cdf){
        Some(res) => res,
        None => return None,
    };
    let bandwidth = if querysize / 10 % 2 == 0 {querysize/10 + 1}else{querysize/10};
    let mut rng = thread_rng();
    let start = rng.gen_range(0,2_000_000);
    Some((0..times).map(|_|rng.gen_range(0,refsize-querysize)+start).map(|start|{
        let subref = &reference[start..start+querysize];
        let subref = &padding_reference(subref)[0..querysize];
        match *method {
        Method::SubHill => dtw::dtw(&query,subref,dtw::Mode::Sub,&hill),
        Method::SubNormal => dtw::dtw(&query,subref,dtw::Mode::Sub,&normal),
        Method::ChibaHill => dtw::dtw(&query,subref,
                                              dtw::Mode::SakoeChiba(bandwidth),&hill),
        Method::ChibaNormal => dtw::dtw(&query,subref,
                                                dtw::Mode::SakoeChiba(bandwidth),&normal),
        }}).filter_map(|res|res.ok()).map(|(score,_,_)|(id.clone(),score)).collect())
}
pub fn correct_dtw(filename:&str,reference:&[f32],method:&Method,querysize:usize,prep:&Prep,cdf:&Vec<f32>) -> Option<(String,f32)>{
    let id = match fast5wrapper::get_read_id(filename){
        Ok(res) => res,
        Err(_) => return None,
    };
    let query = match prepare_query(filename,querysize,prep,cdf){
        Some(res) => res,
        None => return None,
    };
    let reference = &padding_reference(reference)[0..querysize];
    let bandwidth = if querysize / 10 % 2 == 0 {querysize/10 + 1}else{querysize/10};
    if let Ok((score,_,_)) =  match *method{
        Method::SubHill => dtw::dtw(&query,reference,dtw::Mode::Sub,&hill),
        Method::SubNormal => dtw::dtw(&query,reference,dtw::Mode::Sub,&normal),
        Method::ChibaHill => dtw::dtw(&query,reference,dtw::Mode::SakoeChiba(bandwidth),&hill),
        Method::ChibaNormal => dtw::dtw(&query,reference,dtw::Mode::SakoeChiba(bandwidth),&normal)}{
        Some((id,score))
    }else{
        None
    }
}
    
