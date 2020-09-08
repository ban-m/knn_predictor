extern crate csv;
extern crate rayon;
extern crate rand;
use rand::thread_rng;
use rand::Rng;
use rayon::prelude::*;
mod knn;
use knn::KNN;
const K:usize = 1000;
fn compute_k_folds(dataset:&Vec<(f64,bool)>,k:usize) ->Vec<(Vec<(f64,bool)>,Vec<(f64,bool)>)>{
    let window_size = dataset.len()/k;
    let mut result = std::vec::Vec::with_capacity(k);
    for i in 0..(k-1){
        let testset:Vec<_> = dataset[i*window_size..(i+1)*window_size].iter().map(|e|e.clone()).collect();
        let trainset:Vec<_> = dataset.iter().enumerate()
            .filter(|&(idx,_)| idx < i*window_size || idx >= (i+1)*window_size)
            .map(|(_,&e)|e.clone()).collect();
        result.push((trainset,testset));
    }
    let testset:Vec<_> = dataset[(k-1)*window_size..].iter().map(|e|e.clone()).collect();
    let trainset:Vec<_> = dataset[..(k-1)*window_size].iter().map(|e|e.clone()).collect();
    result.push((trainset,testset));
    result
}

fn varidate_by_given_k(dataset:&Vec<(f64,bool)>,k:usize) -> f64{
    // varidate k(parameter)-NN by K-Folds cross varidation;
    // return the average number of correctly distinguished data in each "fold".
    compute_k_folds(dataset,K).par_iter().map(|&(ref train,ref test)|{
        let predictor = KNN::new(&train,k);
        predictor.predict_test(&test) as f64/test.len() as f64})
        .sum::<f64>() / K as f64
}

fn cross_varidation(dataset:&Vec<(f64,bool)>) -> Vec<(f64,usize)>{
    // cross varidate k-NN for given dataset.return optimal k and its correct number.
    let len = dataset.len();
    (1..(len/3)).collect::<Vec<usize>>().par_iter().map(|i|2*i+1)
        .map(|k|(varidate_by_given_k(dataset,k),k))
        .collect()
}

type Record = (usize,f64,i64);


fn setup_csv(correct:&str,wrong:&str,refsize:usize)->csv::Result<Vec<(f64,bool)>>{
    use std::path::Path;
    let mut result = vec![];
    let mut rdr = csv::Reader::from_path(&Path::new(correct))?;
    for record in rdr.deserialize(){
        let record :Record = record?;
        if record.0 == refsize{
            result.push((record.1,record.2 == 1));
        }
    }
    let mut rdr = csv::Reader::from_path(&Path::new(wrong))?;
    for record in rdr.deserialize(){
        let record:Record = record?;
        if record.0 == refsize{
            result.push((record.1,record.2 == 1));
        }
    }
    let mut rng = thread_rng();
    rng.shuffle(&mut result);
    Ok(result)
}

fn main(){
    let args :Vec<_> = std::env::args().collect();
    eprintln!("{:?}",args);
    let refsize:usize = args[1].parse::<usize>().unwrap() * 1_000;
    let dataset = setup_csv(&args[2],&args[3],refsize).unwrap();
    let result = cross_varidation(&dataset);
    for (correct_answer_rate,k) in result{
        println!("{},{:.7},{}",refsize,correct_answer_rate,k);
    }
    // let knn = KNN::new(&dataset,10);  
    // for &(x,b) in dataset.iter(){
    //     println!("{},{}->{}",x,b,knn.predict(x));
    // }
    // println!("{}/{}",dataset.iter().map(|&(x,b)|if b==knn.predict(x){1}else{0}).sum::<i32>(),dataset.len());
    // let folds = compute_k_folds(&dataset,10);
    // let (train,test) = folds[0].clone();
    // let knn = KNN::new(&train,10);
    // println!("{}/{}",test.iter().map(|&(x,b)|if b==knn.predict(x){1}else{0}).sum::<i32>(),test.len());
}
