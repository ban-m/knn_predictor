extern crate dtw;
extern crate fast5wrapper;
extern crate squiggler;
extern crate rand;
extern crate csv;
extern crate rayon;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use squiggler::Squiggler;
mod utils;
use rayon::prelude::*;
fn getqueries(path:&Path)->Vec<(String,usize)>{
    BufReader::new(File::open(path).unwrap()).lines().
        filter_map(|e|e.ok()).
        map(|line|{
            let contents :Vec<_> = line.split(',').collect();
            let filename = contents[0];
            let location:usize = contents[1].parse().unwrap();
            (filename.to_string(),location)}).
        collect()
}


fn main(){
    let args:Vec<_> = std::env::args().collect();
    let queries :Vec<_> = getqueries(&Path::new(&args[1]));
    eprintln!("{:?}",&args);
    let model:Squiggler = Squiggler::new(&Path::new(&args[2])).unwrap();
    let reference :Vec<_> = match model.get_signal_from_path(&Path::new(&args[3])).
        map(|res| res.into_iter().map(|e|e.2).collect()){
        Ok(res) => dtw::normalize(&res,dtw::NormalizeType::Z),
        Err(why) => panic!("{}",why),
    };
    let querysize:usize = args[4].parse().unwrap();
    eprintln!("querysize ok");
    let refsizes:Vec<usize> = args[5].split(',').map(|e|e.parse::<usize>().unwrap()*1000).collect();
    let method = utils::Method::new(&args[6]).unwrap();
    let prep = utils::Prep::new(&args[7]).unwrap();
    let is_correct = args[8] == "CORRECT";
    let filename = format!("./result/{}{}{}{}.csv",method.name(),prep.name(),querysize,args[8]);
    let mut wtr = csv::WriterBuilder::new().from_path(&filename).unwrap();
    let header = vec!["refsize","score","type"];
    if let Err(why) = wtr.write_record(header){
        println!("{:?}",why);
    };
    let (reference,cdf) = match prep{
        utils::Prep::Normal => (reference.clone(),vec![]),
        utils::Prep::Flat => dtw::histgram_equalization(&reference),
    };
    for refsize in refsizes{
        if is_correct{
            //map to correct location
            let result :Vec<_>= queries.par_iter().
                filter_map(|&(ref filename,location)|{
                    let start = if location + refsize > reference.len(){
                        reference.len()-100-refsize
                    }else if location < 100{
                        0
                    }else{
                        location - 100
                    };
                    utils::optimal_dtw(filename,
                                       &reference,
                                       &method,querysize,start,refsize,&prep,&cdf)}).
                map(|(_,score,_)|score).collect();
            for score in result {
                let record = vec![format!("{}",refsize),
                                  format!("{}",score),
                                  format!("{}",1)];
                if let Err(why) = wtr.write_record(record){
                    eprintln!("{}",why);
                }
            }
        }else{
            let result:Vec<_> = queries.par_iter().
                filter_map(|&(ref filename,location)|{
                    let start = if location < reference.len()/2 {
                        reference.len()/2
                    }else{
                        0
                    };
                    utils::optimal_dtw(&filename,
                                       &reference,
                                       &method,querysize,start,refsize,&prep,&cdf)}).
                map(|(_,score,_)|score).collect();
            for score in result{
                let record = vec![format!("{}",refsize),
                                  format!("{}",score),
                                  format!("{}",-1)];
                if let Err(why) = wtr.write_record(record){
                    eprintln!("{}",why);
                }
            }
        }
    }
}
