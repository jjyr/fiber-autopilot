use rand::distr::{weighted::WeightedIndex, Distribution};

pub fn choice_n<T: Clone>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)> {
    // return all items if less than n
    if items.len() < n {
        return items;
    }

    let mut rng = rand::rng();
    let mut dist = WeightedIndex::new(items.iter().map(|item| item.1)).unwrap();
    let mut samples = Vec::default();
    while samples.len() < n {
        let i = dist.sample(&mut rng);
        dist.update_weights(&[(i, &0.0)]).unwrap();
        samples.push(items[i].clone());
    }
    samples
}
