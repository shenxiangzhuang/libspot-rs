use libspot::p2_quantile;

fn main() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let result = p2_quantile(0.5, &data);
    println!("P2 median result: {}", result);
    println!("Expected around: 5.5");
    println!("Difference: {}", (result - 5.5).abs());
    
    let data_100: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let q1 = p2_quantile(0.25, &data_100);
    println!("P2 Q1 result: {}", q1);
    println!("Expected around: 25.0");
    println!("Difference: {}", (q1 - 25.0).abs());
    
    let q3 = p2_quantile(0.75, &data_100);
    println!("P2 Q3 result: {}", q3);
    println!("Expected around: 75.0");
    println!("Difference: {}", (q3 - 75.0).abs());
    
    let data_1000: Vec<f64> = (1..=1000).map(|x| x as f64).collect();
    let q998 = p2_quantile(0.998, &data_1000);
    println!("P2 99.8% result: {}", q998);
    println!("Expected around: 998.0");
    println!("Difference: {}", (q998 - 998.0).abs());
}