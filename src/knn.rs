/// k-NNの簡単な実装。というのも、一次元のk-NNなので、ものすごく雑なアルゴリズムでも、klonNの計算量で
/// クラスの判別ができるからだ。
/// データはすでに作成してあるので、じっさいは、csvを読み込んで、ひたすらにkの値を変えながらクロスバリデーションを取っていく。
///
#[derive(Debug)]
#[repr(C)]
pub struct KNN{
    // kNN. Each data consists of its value and its class
    // note that dataset is sorted by the constructor function.
    dataset:Vec<(f64,bool)>,
    k:usize,
}
impl KNN{
    pub fn predict(&self,query:f64)->bool{
        // predict query.
        // return whether of not half of the queries in k-NN is in correct class.
        2 * self.number_of_correct_in_k(query) > self.k
    }
   
    fn initialize(position:usize,len:usize) -> (Option<usize>,Option<usize>,usize){
        let left = if position == 0 { 
            None
        }else if position == len {
            Some(position-2)
        }else{
            Some(position-1)
        };
        let right = if position >= len-1 {
            None
        }else{
            Some(position+1)
        };
        let center = if position >= len-1 { len-1 } else { position };
        (left,right,center)
    }
    fn sound_expand(&self,left:usize,right:usize,query:f64)->(Option<usize>,Option<usize>,bool){
        let len = self.dataset.len();
        if (self.dataset[left].0-query).abs() > (self.dataset[right].0-query).abs(){
            let nextright = if right == len-1 { None }else{ Some(right+1)} ;
            (Some(left),nextright,self.dataset[right].1)
        }else if (self.dataset[left].0-query).abs() <= (self.dataset[right].0-query).abs(){
            let nextleft = if left == 0 { None }else{ Some(left-1)};
            (nextleft,Some(right),self.dataset[left].1)
        }else{
            eprintln!("datasetsize:{},k:{},\nleft:{},right{}",self.dataset.len(),self.k,left,right);
            eprintln!("dataset[{:?}..{:?}\nquery{}",&self.dataset[0..10],&self.dataset[len-10..len],query);
            unreachable!()
        }

    }
    fn expand(&self,left:Option<usize>,right:Option<usize>,query:f64)->(Option<usize>,Option<usize>,bool){
        // expand the range,return the class of expanded element.
        match (left,right){
            (None,Some(right)) => (None,Some(right+1),self.dataset[right].1),//already reached leftmost
            (Some(left),None) => (Some(left-1),None,self.dataset[left].1),//already reached rightmost
            (Some(left),Some(right)) => self.sound_expand(left,right,query),//expand
            (_,_) => unreachable!("no"),
        }
    }
    fn number_of_correct_in_k(&self,query:f64) -> usize{
        let position = match self.dataset.binary_search_by(|&(t,_)|t.partial_cmp(&query).unwrap()){
            Ok(position) => position,
            Err(position) if position != 0 && 
                position != self.dataset.len() && 
                (self.dataset[position].0-query).abs() > (self.dataset[position-1].0-query).abs() => position-1,
            Err(position) => position,
        };
        let len = self.dataset.len();
        let (mut left,mut right,center) = KNN::initialize(position,len);
        let mut correct = if self.dataset[center].1 { 1 }else{ 0 };
        for _ in 1..self.k{
            let next = self.expand(left,right,query);
            left = next.0;
            right = next.1;
            correct += if next.2 { 1 }else{ 0 };
        }
          correct
    }
    pub fn new(dataset:&Vec<(f64,bool)>,k:usize) -> KNN{
        let mut dataset:Vec<_> = dataset.clone();
        assert!(dataset.len() > 1);
        dataset.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        KNN{dataset,k}
    }
    pub fn predict_test(&self,testset:&Vec<(f64,bool)>) -> u64{
        // predict each data in testset, then return how good the predict was.
        testset.iter().map(|&(query,answer)| self.predict(query) == answer).
            map(|result| if result {1}else{0}).sum()
    }
}


fn compute_n_folds(dataset:&Vec<(f64,bool)>,n:usize) ->Vec<(Vec<(f64,bool)>,Vec<(f64,bool)>)>{
    let window_size = dataset.len()/n;
    use std::vec::Vec;
    let mut result = Vec::with_capacity(n);
    for i in 0..(n-1){
        let testset:Vec<_> = dataset[i*window_size..(i+1)*window_size].iter().map(|e|e.clone()).collect();
        let trainset:Vec<_> = dataset.iter().enumerate()
            .filter(|&(idx,_)| idx < i*window_size || idx >= (i+1)*window_size)
            .map(|(_,&e)|e.clone()).collect();
        result.push((trainset,testset));
    }
    let testset:Vec<_> = dataset[(n-1)*window_size..].iter().map(|e|e.clone()).collect();
    let trainset:Vec<_> = dataset[..(n-1)*window_size].iter().map(|e|e.clone()).collect();
    result.push((trainset,testset));
    result
}

pub fn varidate_by_given_k(dataset:&Vec<(f64,bool)>,k:usize) -> f64{
    // varidate k(parameter)-NN by N-Folds cross varidation;
    // return the average number of correctly distinguished data in each "fold".
    let n = dataset.len()/2;
    compute_n_folds(dataset,n).iter().map(|&(ref train,ref test)|{
        let predictor = KNN::new(&train,k);
        predictor.predict_test(&test) as f64/test.len() as f64})
        .sum::<f64>() /n as f64
}

#[cfg(test)]
mod tests{
    use super::*;
    fn mock_model()->KNN{
        let dataset = vec![(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),
                           (1.,true),(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),
                           (5.,false),(5.,false),(5.,false),(5.,false),(5.,false),(5.,false)];
        KNN::new(&dataset,2)
    }
    #[test]
    fn init(){
        let dataset = vec![(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),
                           (1.,true),(1.,true),(1.,true),(1.,true),(1.,true),(1.,true),
                           (5.,false),(5.,false),(5.,false),(5.,false),(5.,false),(5.,false)];
        KNN::new(&dataset,2);
    }
    #[test]
    fn initialize_test(){
        assert_eq!(KNN::initialize(0,2),(None,Some(1),0));
        assert_eq!(KNN::initialize(2,3),(Some(1),None,2));
        assert_eq!(KNN::initialize(4,8),(Some(3),Some(5),4));
        assert_eq!(KNN::initialize(1993,100000),(Some(1992),Some(1994),1993));
    }
    #[test]
    fn expand_test(){
        let knn = mock_model();
        assert_eq!(knn.expand(Some(0),Some(2),2.),(None,Some(2),true));
        assert_eq!(knn.expand(Some(1),Some(5),2.),(Some(0),Some(5),true));
        assert_eq!(knn.expand(Some(11),Some(14),6.),(Some(11),Some(15),false));
        assert_eq!(knn.expand(Some(6),Some(12),2.),(Some(5),Some(12),true));
    }
    fn mock_model_2()->KNN{
        let data = (0..13).map(|e| e as f64).map(|e| if e < 5.0 { (e,true)} else{ (e,false)}).collect();
        KNN::new(&data,5)
    }
    fn hard_model()->KNN{
        let data = (0..13).map(|e| (e,e % 4 == 0)).map(|(e,f)|(e as f64,f)).collect();
        KNN::new(&data,5)
    }
    #[test]
    fn number_of_correct_in_k_test(){
        let knn = mock_model();
        assert_eq!(knn.number_of_correct_in_k(1.),2);
        assert_eq!(knn.number_of_correct_in_k(7.),0);
        let knn = mock_model_2();
        assert_eq!(knn.number_of_correct_in_k(0.),5);
        assert_eq!(knn.number_of_correct_in_k(13.),0);
        assert_eq!(knn.number_of_correct_in_k(4.1),3);
        let knn = hard_model();
        assert_eq!(knn.number_of_correct_in_k(1.),2);
        assert_eq!(knn.number_of_correct_in_k(2.),2);
        assert_eq!(knn.number_of_correct_in_k(6.),2);
        assert_eq!(knn.number_of_correct_in_k(121231.),2);
    }
    #[test]
    fn predict(){
        let knn = mock_model();
        assert!(knn.predict(0.));
        assert!(knn.predict(1.));
        assert!(knn.predict(2.4));
        assert!(!knn.predict(3.,));
        assert!(!knn.predict(191929.));
        let knn = mock_model_2();
        assert!(knn.predict(1.));
        assert!(knn.predict(2.));
        assert!(knn.predict(3.));
        assert!(knn.predict(4.232));
        assert!(!knn.predict(5.));
        assert!(!knn.predict(8.));
        let knn = hard_model();
        assert!(!knn.predict(4.));
        assert!(!knn.predict(-3.));
    }
    #[test]
    fn testsert_predict(){
        let knn = mock_model();
        assert_eq!(knn.predict_test(&vec![(1.,true),(1.,true),(1.,true)]),3);
        assert_eq!(knn.predict_test(&vec![(1.,false),(1.,false),(1.,false)]),0);
        let knn = mock_model_2();
        assert_eq!(knn.predict_test(&vec![(1.,true),(-91.,true),(1.,false)]),2);
        assert_eq!(knn.predict_test(&vec![(8.,false),(9.,false),(13.,false)]),3);
        assert_eq!(knn.predict_test(&vec![(19.,true),(21.,false),(192094.,true)]),1);
    }
}
